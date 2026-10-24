# Magic & Corruption: The Dual-Layer Economy

> **Category:** Lore & Systems Integration  
> **Reference File:** [master_specification.md §3, §4](file:///c:/dev/tactical-rpg/games/emberwake/lore/specs/master_specification.md)

---

## 1. The Core Power Dichotomy

Combat abilities, progression choices, and party survival in *Emberwake* revolve around a brutal, asymmetric choice between mundane physical discipline and eldritch biological mutation.

```text
[ COLD STEEL / MARTIAL PATH ]            [ INOCULATED / RESONANT PATH ]
- High AP Cost (2–4 AP)                  - Low AP Cost (1–2 AP)
- Strict weapon & stance requirements    - Generates mounting Strain (15%–35%)
- Predictable, defensive, positional     - Pierces armor, inflicts corrosive blight
- Zero permanent corruption              - High burst, volatile friendly-fire risk
- Indefinite roster lifespan             - Consumable hero lifespan (Calcification Clock)
```

---

## 2. In-Combat: The Strain Gauge (0% – 100%)

Every operative possesses a real-time **Strain Gauge** reflecting the physiological and neurological toll of harboring alien marrow, using aberrant abilities, stepping on dark environmental tiles, or taking eldritch attacks.

```text
IN-COMBAT STRAIN GAUGE:
[ 0% -------- 24% | 25% -------- 49% | 50% -------- 74% | 75% -------- 99% | 100% (RUPTURE) ]
     DORMANT      | ACUTE RESONANCE  |  FLESH-TENSION   |  ABERRANT EDGE   | CRISIS ROLL
```

### 2.1 Strain Threshold Brackets

| Strain Bracket | State Name | Mechanical & Visual Effects | Tactical Reality |
| :--- | :--- | :--- | :--- |
| **0% – 24%** | **Dormant** | Standard human state. No bonuses or penalties. Operative remains fully composed. | Pure martial baseline. Safe from biological failure. |
| **25% – 49%** | **Acute Resonance** | Senses heighten unnaturally. Pupils dilate; black veins pulse slightly.<br>• **+15% Critical Hit Chance**<br>• **+1 Movement Tile**<br>• **-25% Healing Draught Effectiveness** | The "Sweet Spot" for agile strikers; healing begins to fail. |
| **50% – 74%** | **Flesh-Tension** | Subcutaneous muscle fibers harden into fibrous cordage.<br>• **+25% All Damage**<br>• **+2 Base Armor Rating**<br>• **-20 Stance Stability** (Takes double stagger from bludgeoning) | Devastating damage dealer, but extremely vulnerable to being interrupted and stunned. |
| **75% – 99%** | **Aberrant Edge** | Psychotic sensory distortion. Whispers from the Low Sleep drown out ally cries.<br>• Attacks gain bonus **Bleed** and **Stagger**<br>• **Recoil Damage:** Operative suffers 10% of damage dealt as self-harm | High-risk finishing mode; reckless use rapidly drains unit health. |
| **100%** | **Rupture** | Biological integrity collapses. The aberrant graft overrides the host's nervous system. | **Triggers an immediate Aberration Crisis Roll.** |

### 2.2 The Aberration Crisis Table (100% Strain)
When a unit hits 100% Strain, normal action resolution halts immediately and a D3 Crisis Roll resolves:

1. **Unchecked Spasm:** The operative acts instantly out of turn, delivering a wild, full-power melee or projectile strike against the nearest entity (ally or enemy alike), and then suffers 1 round of complete **Stun**.
2. **Biological Collapse:** The operative's internal organs rupture. Action Points drop to 0 for the turn, they suffer severe internal hemorrhage (**Bleed** for 3 rounds), and all active armor mitigation is stripped for the remainder of the battle.
3. **Flesh Breach:** A permanent point of **Calcification** is added immediately to the character's record. Bone spurs violently erupt through their skin, permanently mutating their body post-battle.

---

## 3. Out-of-Combat: Calcification (The Permadeath Clock)

While Strain resets or diminishes between encounters through triage and chemical vents, heretical power leaves permanent physical scars. An operative advances **1 Stage along the Calcification Clock** under the following conditions:
- Ending an encounter in the **Aberrant Edge** (75%+ Strain).
- Suffering a **Flesh Breach** during an Aberration Crisis.
- Undergoing Tier III apex surgical grafting at the Apothecary's Slab.

```text
CAMPAIGN PERMADEATH CLOCK:
Stage I: Vein-Blackening  ==>  Stage II: Calloused Chitin  ==>  Stage III: Sensory Rot  ==>  Stage IV: The Shakes  ==>  Stage V: Terminal Drift (LOST)
```

### 3.1 The Five Stages of Calcification

| Level | Physical Manifestation | Permanent Gameplay Trade-off |
| :--- | :--- | :--- |
| **I. Vein-Blackening** | Inky black streaks branch beneath the skin from the graft site; skin takes on a chalky pallor. | **-5 Max Composure** (more susceptible to panic), but gains **+5% Debuff Resistance**. |
| **II. Calloused Chitin** | Crustaceous, calcified black bone plates punch through the dermis along shoulders, forearms, or ribs. | **+1 Permanent Armor Rating**, but permanently loses **-1 Movement Tile** due to joint stiffening. |
| **III. Sensory Rot** | Pupils cloud with milky cataracts; ears weep a thick, odorless black fluid. Operative perceives reality in vibrations. | **Immune to all Fear and Morale debuffs**, but ranged accuracy beyond 3 tiles drops by **15%**. |
| **IV. The Shakes** | Violent, involuntary tremors wrack the limbs as muscles fight petrified tendons. | **Cannot equip two-handed weapons or bows**, but all bare-handed or melee attacks gain **natural Armor Piercing**. |
| **V. Terminal Drift** | Complete biological and psychological petrification. The human ego is extinguished. | **Permanent Loss.** The operative either leaves for the deep catacombs to sit among the spent elders or mutates into a hostile aberration that must be put down. |

---

## 4. Field Surgery & Triage Mechanics

In the safehouse (**The Sluice Undercroft**), the player manages corruption through specialized installations:

* **Strain Chemical Flushes:** Conducted at the **Apothecary's Slab**. Uses caustic sulfur solutions and salt-water rinses to lower baseline dormant strain floors before missions.
* **Calcification Cleansing (Late-Game/Master Tier):** Unlocked via Sister Aline Vance's master quest (*"Anatomy of a Living Saint"*). Reverses 1 stage of Calcification through brutal bone-scraping surgery, but permanently sacrifices **10% of the character's Maximum HP**.
* **Tincture of Still Blood:** An alchemical draught that temporarily locks an operative's Strain Gauge at its current value for 1 full combat round, allowing them to unload high-strain skills without triggering a Rupture.
