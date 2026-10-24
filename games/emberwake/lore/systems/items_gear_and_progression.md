# Items, Gear & Progression Systems: Weaponry, Relics & Corruption Management

> **Category:** Gameplay Systems, Equipment Catalog & Progression Architecture  
> **Status:** Active Canonical Specification  
> **Location:** `games/emberwake/lore/systems/items_gear_and_progression.md`  
> **Reference Specs:** [classes_and_skill_trees.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/classes_and_skill_trees.md), [combat_and_timeline.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/combat_and_timeline.md), [safehouse_economy_and_triage.md](file:///c:/dev/tactical-rpg/games/emberwake/lore/systems/safehouse_economy_and_triage.md)  
> **Target Crates:** `emberwake_rules` (`gear`, `stats`, `damage`), `emberwake_data` (`models/item`, `catalogs/items.json`), `emberwake_world` (`state`)

---

## 1. Equipment Architecture & Slot Framework

Every operative in *Emberwake: The Low Sleep* equips gear across a standardized six-slot equipment framework, supplemented by an in-combat **Battle Pouch**:

```text
+---------------------------------------------------------------------------------------------------+
| OPERATIVE EQUIPMENT SLOTS (REUSING `emberwake_rules::gear::Slot`)                                 |
+-------------------+-----------------------------------+-------------------------------------------+
| SLOT TYPE         | CAPACITY                          | SYSTEM / COMBAT INFLUENCE                 |
+-------------------+-----------------------------------+-------------------------------------------+
| 1. Main Hand      | 1 Primary Weapon (1H or 2H)       | Dictates base damage, reach, & Speed Tier |
| 2. Off Hand       | Shield, Secondary Dagger, Focus   | Mitigation, parries, or off-hand jabs     |
| 3. Head           | Helm, Hood, Mask, Spectacles      | Vision/Range buffs, Blind resistance      |
| 4. Body           | Plate, Brigandine, Leathers, Habit| Base Armor Rating, Max Poise, Burden      |
| 5. Accessory 1    | Ring, Talisman, Crying Bone       | Stat bonuses, strain mitigation, passives |
| 6. Accessory 2    | Relic, Amulet, Marrow Graft Core  | Unique mechanical traits, capstone synergy|
+-------------------+-----------------------------------+-------------------------------------------+
| • Battle Pouch    | 3–4 Consumable Slots (Equipped)   | In-combat healing, strain flushes, bombs  |
+-------------------+-----------------------------------+-------------------------------------------+
```

### 1.1 Equipment Burden & Wield Penalties
The engine reuses `emberwake_rules::gear::WieldPenalty`:
* **Strength/Dexterity Deficit:** Wielding heavy military equipment without meeting statutory minimums (`STR` or `DEX`) imposes a wield penalty: increases maneuver AP cost by $+1$ and delays timeline windup by $+1$ tick.
* **Burden & Movement:** Heavy plate armor with high burden reduces baseline movement by $-1$ tile unless offset by the Vanguard's *Braced Poise* or the Brawler's *Hypertrophy*.

---

## 2. Weapon Families & Class Proficiencies

Weapons are categorized into distinct martial families. Each family is bound to specific Speed Tiers, Reach distances, damage types, and class proficiencies:

```text
+-----------------------+--------+-------+---------+--------+------------------+-----------------------------+
| WEAPON FAMILY         | HANDS  | REACH | SPEED   | DAMAGE | PRIMARY STAT     | CLASS PROFICIENCY MAPPING   |
+-----------------------+--------+-------+---------+--------+------------------+-----------------------------+
| 1. Billhooks/Poleaxes | 1H/2H  | 1–2   | Standard| Slash  | STR (Scaling)    | The Vanguard (Primary)      |
| 2. Broad/Greatswords  | 1H/2H  | 1     | Std/Hvy | Slash  | STR              | The Vanguard, The Slag-Brute|
| 3. Bodkins/Stilettos  | 1H     | 1     | Snap T2 | Pierce | DEX              | The Gutter-Stalker (Primary)|
| 4. Heavy Arbalests    | 2H     | 4–5   | Std/Hvy | Pierce | DEX / PERC       | The Arbalestier (Primary)   |
| 5. Iron Knuckles/Fists| 1H/Dual| 1     | Snap T2 | Blunt  | STR / CON        | The Slag-Brute (Primary)    |
| 6. Foundry Hammers    | 2H     | 1     | Heavy T5| Blunt  | STR              | The Slag-Brute, The Vanguard|
| 7. Bone-Saws / Censers| 1H/2H  | 1–2   | Standard| Pierce | KNOW / INT       | The Heretic Acolyte (Prim.) |
| 8. Chemical Sprayers  | 1H/2H  | 3–4   | Standard| Corros.| INT / PERC       | The Ichor Alchemist (Prim.) |
+-----------------------+--------+-------+---------+--------+------------------+-----------------------------+
```

### 2.1 Weapon Tactical Nuance & Timeline Interfacing
* **Billhooks & Poleaxes (Reach 2, Range Hold):** Allows the Vanguard to strike from behind an ally or hold ground against charging hostiles. Features high Stagger damage (+20 to +35) ideal for triggering interrupts.
* **Bodkins & Daggers (Snap T2 Affinity):** Innately bypasses windup telegraphs. Strikes land on Tick 2, allowing Gutter-Stalkers to disrupt faster enemies before they act.
* **Heavy Lever-Arbalests (Range 5, Winch Telegraph):** Tremendous piercing damage and pinning capabilities. Heavy variants require a 2-tick windup, rewarding careful overwatch positioning.
* **Foundry Knuckles & Chains (High Stagger & Knockback):** Deals massive posture damage directly to the enemy's Poise Bar; triggers Wall Slam collision damage when enemies are knocked into scenery.
* **Chemical Sprayers & Volatile Retorts (Area Hazards):** Creates persistent 3x3 ground hazard zones (Phosphorus Fire, Caustic Vitriol) rather than simple single-target damage.

---

## 3. Armor Classes & Defensive Profiles

Armor pieces govern survivability through two distinct metrics: **Base Armor Rating** (which feeds into the hyperbolic mitigation curve $r / (r + 50)$) and **Poise Value** (which expands the Stagger Bar):

```text
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| ARMOR SUITE           | ARMOR | POISE | BURDEN | CLASS ALLOCATION & TACTICAL TRAITS                          |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Heavy Iron Plate      | 4–6   | 25–40 | High   | **The Vanguard:** Maximum physical mitigation and unyielding|
|                       |       |       |        | poise; immune to micro-stagger from light Snap attacks.     |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Soot-Stained Leathers | 1–2   | 10–15 | None   | **The Gutter-Stalker:** Grants +15%–25% Evasion; costs zero |
|                       |       |       |        | movement penalty in darkness, fog, or rubble.               |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Reinforced Brigandine | 2–3   | 15–20 | Low    | **The Arbalestier:** Horn recoil shoulder plate; grants     |
|                       |       |       |        | +20% ranged defense bonus while firing from half-cover.     |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Penal Foundry Apron   | 2–4   | 30–50 | Medium | **The Slag-Brute:** Heavy boiled hide with riveted scrap;   |
|                       |       |       |        | provides 50% resistance to fire and molten slag runoff.     |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Embalmer's Habit      | 1     | 10–15 | None   | **The Heretic Acolyte:** Bleached linen; increases medical  |
|                       |       |       |        | healing potency by +20% and grants +15% Debuff Resistance.  |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
| Tanner's Rubbered Coat| 1–2   | 10–15 | Low    | **The Ichor Alchemist:** Chemically treated; grants total   |
|                       |       |       |        | immunity to self-inflicted caustic splashback and acid rot. |
+-----------------------+-------+-------+--------+-------------------------------------------------------------+
```

---

## 4. Consumables & The Corruption Management Economy

Consumables are equipped into the operative's **Battle Pouch (Max 3–4 items)** before deploying. In *Emberwake*, consumables split into standard trauma restoratives and specialized **Strain & Corruption Regulators**:

```text
+---------------------------------------------------------------------------------------------------+
| CONSUMABLES CATALOG (BATTLE POUCH SLOTS)                                                          |
+-----------------------+--------+-------+----------------------------------------------------------+
| ITEM NAME             | TYPE   | SPEED | TACTICAL EFFECT & CORRUPTION MECHANIC                    |
+-----------------------+--------+-------+----------------------------------------------------------+
| Sulfur Stitch-Poultice| Restor.| Snap  | Restores 60 HP; instantly cleanses Bleed and Poison.     |
| Tallow Burn Salve     | Restor.| Snap  | Cleanses Burn and Corrosive Blight; grants +10 HP regen. |
| Magnesium Flash-Pot   | Combat | Snap  | Thrown (Range 3); inflicts Blind and +20 Stagger in 3x3. |
| Pitch Fire-Bomb       | Combat | Std   | Thrown (Range 4); ignites 3x3 area in flames for 2 turns.|
+-----------------------+--------+-------+----------------------------------------------------------+
| Sanctified Salt Sachet| Corrupt| Snap  | Crushed in palms; **immediately vents 25% Strain**,      |
|                       |        |       | dropping the hero out of dangerous Rupture Verge brackets|
+-----------------------+--------+-------+----------------------------------------------------------+
| Brimstone Tallow Draught|Corrupt|Snap  | Bitter warded fat; grants **+30% Stagger Resistance** and|
|                       |        |       | immunity to Nightmare Whispers for 3 rounds.             |
+-----------------------+--------+-------+----------------------------------------------------------+
| Clerisy Communion Sed.|Corrupt| Snap  | Stolen church opiate; **drops Strain by 40%**, but unit  |
|                       |        |       | loses -2 AP on the subsequent round due to sedation.     |
+-----------------------+--------+-------+----------------------------------------------------------+
| Black-Bile Ampoule    | Corrupt| Snap  | Inoculation booster; violently **injects +30% Strain**   |
|                       |        |       | to intentionally launch hero into *Aberrant Edge* (+50%).|
+-----------------------+--------+-------+----------------------------------------------------------+
| Quenched Cinder Salt  | Hub    | -     | Applied at Sister Aline's slab post-battle; prevents     |
|                       |        |       | advancing a Calcification stage after an Aberration roll.|
+-----------------------+--------+-------+----------------------------------------------------------+
```

---

## 5. Monster Parts & Anatomical Harvest (Crafting & Hub Unlocks)

Defeating elite hostiles and subterranean horrors yields anatomical reagents. These are harvested and brought back to the Sluice Undercroft for surgical grafting and forge upgrades:

```text
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| HARVESTED REAGENT     | DROP SOURCE (BESTIARY)      | RECIPIENT WORKSHOP FACILITY   | UNLOCK & CRAFTING APPLICATION    |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Titan Marrow Core     | Marrow Horrors, Fissures    | Sister Aline's Apothecary Slab| Unlocks Tier 3 Inoculated skills |
|                       | (`ENC-EVT-05`)              |                               | (*Ribcage Lock*, *Aberrant Titan*)|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Stalker Optic Nerve   | Render-Beasts, Night-Stalk. | Sister Aline's Apothecary Slab| Unlocks *Ocular Lock* and night- |
|                       | (`ENC-EVT-04`)              |                               | vision targeting passives.       |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Calcified Bone Shard  | Calcified Husks, Bone-Mills | Bram Karr's Tinker Forge      | Forges armor-shredding quarrels  |
|                       | (`ENC-EVT-03`)              |                               | and reinforced iron knuckles.    |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Caustic Gall-Sac      | Ash-Crawlers, Dredgers      | Sister Aline & Elyse          | Synthesizes vitriol acid flasks  |
|                       | (`ENC-EVT-01`, `04`)        |                               | and unlocks *Caustic Vitriol*.   |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Inquisitor Marrow Vlv.| Inquisitor Proctors         | Sister Aline's Apothecary Slab| Crafts the *Apex Strain Siphon*, |
|                       | (`ENC-EVT-02`, `06`)        |                               | allowing Maren to balance strain.|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Render-Beast Tendon   | Tallow Hound Packs          | Bram Karr's Tinker Forge      | Reinforces arbalest winches:     |
|                       | (`ENC-EVT-04`)              |                               | grants +1 Range and +10 Stagger. |
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
| Warded Tallow Residue | Tallow Vats, Purifier Tanks | The Dry Hollows Brazier       | Refined into daily safehouse     |
|                       | (`ENC-EVT-04`)              |                               | fuel; prevents nightmare madness.|
+-----------------------+-----------------------------+-------------------------------+----------------------------------+
```

---

## 6. Named World Relics & Limited Unique Loot

Unlike generic iron equipment that can be bought or crafted in quantity, **Named Relics are unique, hand-crafted artifacts limited to exactly one instance per campaign world**. They are discovered through authored quests, boss executions, or hidden secret caches:

```text
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| UNIQUE RELIC NAME     | EQUIPMENT SLOT        | SOURCE ACQUISITION    | UNIQUE MECHANICAL TRAIT & LORE           |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 1. The Deserter's     | Head                  | Kaelen's Personal     | **Armor +4, Poise +15.** While Kaelen    |
|    Kettle Helm        |                       | Questline             | executes Active Guard, adjacent allies   |
|                       |                       |                       | gain +20% Physical Mitigation.           |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 2. Orlov's Flensing   | Main Hand (Dagger)    | Defeat Guild-Master   | **Power 32, Speed Snap T2.** Rear-arc    |
|    Talon              |                       | Orlov (`ENC-EVT-04`)  | attacks against bleeding targets below   |
|                       |                       |                       | 35% HP instantly execute without AP cost.|
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 3. The Clocktower     | Off Hand (Accessory)  | Bran's Personal Cache | **Arbalest Tuning.** Reduces windup of   |
|    Lever-Winch        |                       | (Seventh Ward Rafters)| *Heavy Winch Sunder* by 1 tick (Impact T4|
|                       |                       |                       | instead of T5).                          |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 4. Vane's Gilded      | Off Hand (Shield)     | Defeat Captain Vane   | **Mitigation +55%, Poise +30.** Active   |
|    Castellan Pavise   |                       | (`ENC-EVT-01`)        | Guard reflects 15 blunt damage and +20   |
|                       |                       |                       | Stagger back to melee attackers.         |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 5. Malakar's Marrow   | Main Hand (Knife)     | Defeat Proctor        | **Power 28, Speed Snap T2.** Successful  |
|    Scalpel            |                       | Malakar (`ENC-EVT-02`)| strikes vent 10% Strain from the wielder |
|                       |                       |                       | while inflicting 8 bleed damage/round.   |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 6. Grendel's Pneumatic| Main Hand (2H Hammer) | Defeat Overseer       | **Power 48, Speed Heavy T5.** Attacks    |
|    Slag-Hammer        |                       | Grendel (`ENC-EVT-03`)| send shockwaves through stone walls,     |
|                       |                       |                       | dealing 25 crushing damage to rear foes. |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 7. Sister Vance's     | Main Hand (Staff)     | Sister Aline Vance    | **Power 22, Range 2.** Increases all field|
|    Embalming Crook    |                       | Loyalty Milestone     | sutures and healing skills by +35%; Maren|
|                       |                       |                       | cleanses debuffs without Bile Phial cost.|
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 8. The Dreamer's      | Accessory             | Defeat The Calcified  | **Eldritch Relic.** Permanently locks    |
|    Crystalline Vert.  |                       | Amalgam (`ENC-EVT-05`)| Dormant Strain Floor at 20%, but all     |
|                       |                       |                       | Inoculated attacks gain +30% True Damage.|
+-----------------------+-----------------------+-----------------------+------------------------------------------+
| 9. Bastion's Warded   | Body (Armor)          | Crucible Watchpost    | **Armor +5, Poise +35.** Grants complete |
|    Cassock            |                       | Climax (`ENC-EVT-06`) | immunity to Phosphorus Flames and        |
|                       |                       |                       | subterranean Blight Miasma.              |
+-----------------------+-----------------------+-----------------------+------------------------------------------+
```

---

## 7. Engine Catalog Schema Mapping (`world/items.json`)

To bind these items into the Aegis engine data models (`emberwake_data::ItemDef`), the catalog schema expands to track speed tiers, corruption traits, and relic flags:

```json
{
  "id": "relic_orlov_flensing_talon",
  "name": "Orlov's Flensing Talon",
  "item_type": "WEAPON",
  "weapon_type": "dagger",
  "slots": ["MAIN_HAND"],
  "hands": "main_1h",
  "tier": 3,
  "unique_world_relic": true,
  "base_power": 32,
  "base_range": 1,
  "speed_tier": "SNAP",
  "stagger_power": 15,
  "armor_piercing": false,
  "stat_requirements": { "DEX": 14 },
  "granted_passive": "flensing_execute",
  "strain_modifier": 0,
  "description": "The blood-grooved boning knife of Guild-Master Orlov. Balanced for terrifying speed; automatically executes bleeding targets below 35% HP from behind."
}
```
