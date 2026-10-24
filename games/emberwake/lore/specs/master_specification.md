# EMBERWAKE: Worldbuilding, Combat Systems & Narrative Architecture

> **Document Classification:** Core Design Bible & World Specification  
> **Target Engine:** Aegis (Rust Runtime Workspace)  
> **Setting Genre:** Medieval Feudalism / Eldritch Cosmic Horror  
> **Art Target:** 2D Pixel Art (32×32 Grid, 32×48 Sprite Footprint, 640×360 Viewport)

---

## 1. Setting Overview & Cosmology

### 1.1 The High Concept
*Emberwake* takes place in a decaying medieval world where human civilization has been subjugated by the **Sovereign Clerisy**—an ancient ecclesiastical aristocracy that maintains order through brutal feudal hierarchies and symbiotic blood-pacts with an incomprehensible entity sleeping beneath the continent. The player leads **The Severed Root**, a desperate guerrilla insurrection composed exclusively of "young blood" (ages 15–24) who have not yet succumbed to the calcifying biological decay of ambient eldritch exposure.

### 1.2 The Broken Natural Order
* **The Shroud (The False Heavens):** Two centuries ago, the stars and sun were extinguished, replaced by a permanent, bruised-violet canopy of petrified cloud, freezing gloom, and drifting ash. Day and night are indistinguishable; social life and labor shifts are dictated solely by the tolling of the Hegemony’s warded cathedral bells.
* **The Low Sleep (The Subterranean Entity):** The continent rests directly atop the calcified flesh of an ancient, dreaming cosmic leviathan. Its sluggish subterranean pulse induces tremors, ground liquefaction ("tar-bogs"), and seepage of mutagenic black marrow into aquifers and mines.
* **The Stagnation of Mortality:** Natural biological decomposition is broken. Organic matter does not decompose into fertile loam; instead, uncremated flesh ossifies into brittle black bone. Corpses buried without sanctified rock salt and sulfur-cremation slowly knit back into mindless, skinless wanderers.

### 1.3 The Generational Divide
* **The Spent Generation (Ages 25+):** Decades of breathing tallow fumes, manual toil in bone-mills, and drinking church-issued communion draughts (laced with mild ossifying sedatives) leaves older citizens mentally compliant, physically stiff, and biologically dependent on cathedral warded lanterns. They actively report young dissidents to the Inquisition in exchange for salt rations and safety.
* **The Unsealed (Ages 15–24):** The youth retain supple vascular tissue, unclouded sight, and mental flexibility. Their minds can witness cosmic warping without immediate psychosis. However, entering adulthood carries a hard countdown toward either total compliance or monstrous calcification.

---

## 2. Factions & Power Structures

```
       [ THE SOVEREIGN CLERISY ]  <─── High Hegemony (Blood Pacts & Wards)
               │          │
   ┌───────────┘          └───────────┐
   ▼                                  ▼
[ THE TALLOW GUILD ]       [ THE IRON INQUISITION ]
(Resource Extraction)      (Military Enforcement & Culling)
   │                                  │
   └───────────┬──────────────────────┘
               ▼
     [ THE DOCILE MASSES ] ─── (Elders, Addicted, Spent)
               ▲
               │ (Raids & Rescues)
     [ THE SEVERED ROOT ]  <─── The Young Rebellion (Player Roster)
```

### 2.1 The Sovereign Clerisy (The Ruling Elite)
* **Role:** Ecclesiastical aristocracy acting as wardens and caretakers of the dreaming leviathan.
* **Doctrine:** Suffering is sanctified; total obedience staves off cosmic obliteration. Through periodic mass-sacrifice (**The Red Tithing**), they pacify the entity’s nightmares, preventing cataclysmic waking tremors.
* **Visual Identity:** Windowless iron masks, bleached horsehair vestments stiffened with beeswax, and swinging golden censers burning narcotic resins.

### 2.2 The Iron Inquisition & Castellan Wardens (The Military Arm)
* **Role:** Enforces martial law, maintains sector curfews, runs juvenile holding pens, and hunts unregistered mutants.
* **Combat Doctrine:** Heavy armor, poleaxes, kite shields, and blinding flash-bombs. They fight in tight formations, using Stagger-heavy attacks to beat dissidents into submission.

### 2.3 The Tallow Guild & Bone-Mills (Industrial Exploitation)
* **Role:** Renders human and animal remains into warded lighting tallow (the only oil whose flame repels mist-aberrations) and grinds calcified skeletons into structural masonry mortar.
* **Dynamic:** Universally loathed, yet structurally essential. All commerce in salt, tallow, and clean drinking water runs through their ledgers.

### 2.4 The Severed Root (The Player's Insurrection)
* **Cell Architecture:** Governed by the **Rule of Three**. Operatives operate in closed trios or quartets (matching a combat squad). No operative knows more than two external contacts.
* **Ideological Fracture:**
  * **The Purists:** Champion strict Cold Steel discipline. They believe mutating the body forfeits human will and that the regime can be broken with leverage, iron, and demolition.
  * **The Inoculated:** Argue that fighting cosmic entities with mundane steel is suicidal. They advocate harvesting aberration organs and injecting tainted marrow to gain superhuman combat reflexes, despite the risk of Calcification.

---

## 3. The Core Magic & Power Dichotomy

Combat progression is split into two mutually exclusive, yet mechanically blendable, martial philosophies.

```
[ Cold Steel / Martial Path ]             [ Inoculated / Resonant Path ]
  - High AP Cost                            - Low AP Cost
  - Strict weapon requirements              - Requires Strain generation
  - Predictable, stance-dependent           - Pierces armor, inflicts status effects
  - Zero permanent corruption               - High burst, high team liability
  - Permanent roster lifespan               - Consumable lifespan (Calcification)
```

### 3.1 Cold Steel (The Iron Discipline)
* **Nature:** Peak human conditioning, biomechanical leverage, heavy armaments, and tactical positioning.
* **Resource Profile:** Costs 2–4 AP per action. Generates **0% Strain** and zero Calcification points.
* **Primary Role:** Armor sundering, stance breaking, crowd control, and territory denial.
* **Roster Longevity:** Characters following pure Cold Steel can fight across the entire campaign without biological degradation.

### 3.2 The Bitter Inoculation (The Marrow Graft)
* **Nature:** Surgical grafting of glands, marrow, and ichor harvested from slain aberrations and inquisitors.
* **Resource Profile:** Costs 1–2 AP per action, but generates **15% to 35% Strain**.
* **Primary Role:** Armor-piercing corrosive damage, teleportation/phasing, self-sustain, and extreme burst damage.
* **Downside:** Overuse triggers in-combat **Aberration Crises** and permanently advances the character along the 5-stage **Calcification Clock**.

---

## 4. The Dual-Layer Corruption Economy

```
IN-COMBAT:    [ 0% ──── 25% ──── 50% ──── 75% ──── 99% ──── 100% (RUPTURE) ]
              [ Dormant | Resonance | Tension | Edge  | Crisis Roll        ]
                                                            │
CAMPAIGN:     Stage I ──> Stage II ──> Stage III ──> Stage IV ──> Stage V (DEATH/LOST)
```

### 4.1 In-Combat: The Strain Gauge (0% – 100%)
Every unit possesses a real-time Strain Gauge altered by heretical skill activation, dark terrain hazards, and incoming eldritch damage.

| Strain Bracket | State Name | Mechanical Modifiers |
| :--- | :--- | :--- |
| **0% – 24%** | **Dormant** | Standard performance. No bonuses or penalties. |
| **25% – 49%** | **Acute Resonance** | Senses sharpen: **+15% Critical Chance**, **+1 Movement**, but mundane healing draughts are **25% less effective**. |
| **50% – 74%** | **Flesh-Tension** | Muscle fibers harden: **+25% Damage**, **+2 Armor Rating**, but suffers **-20 Stance Stability** (takes double stagger from bludgeoning). |
| **75% – 99%** | **Aberrant Edge** | Psychosis edge: Attacks gain bonus Bleed/Stagger, but unit suffers **recoil damage** equal to 10% of damage dealt. |
| **100%** | **Rupture** | Biological integrity fails. Triggers an immediate **Aberration Crisis Roll**. |

#### The Aberration Crisis Table (100% Strain)
1. **Unchecked Spasm:** Unit acts immediately out of turn, hitting the nearest entity (ally or foe) with a wild strike, then suffers 1 round of Stun.
2. **Biological Collapse:** Unit drops to 0 AP, suffers internal hemorrhage (Bleed), and loses all active armor mitigation for the rest of the battle.
3. **Flesh Breach:** A permanent point of **Calcification** is added immediately, causing physical mutations post-battle.

### 4.2 Out-of-Combat: Calcification (The Permadeath Clock)
Ending a battle above 75% Strain or suffering an Aberration Crisis permanently advances Calcification through five stages:

| Level | Physical Manifestation | Gameplay Trade-off |
| :--- | :--- | :--- |
| **I. Vein-Blackening** | Black streaks visible under skin. | **-5 Max Composure**, **+5% Debuff Resistance**. |
| **II. Calloused Chitin** | Crustaceous skin plates erupt. | **+1 Base Armor Rating**, **-1 Movement Tile**. |
| **III. Sensory Rot** | Eyes cloud; ears leak black fluid. | Immune to Fear effects; accuracy beyond 3 tiles drops by **15%**. |
| **IV. The Shakes** | Violent involuntary muscle spasms. | Cannot equip two-handed weapons; melee attacks gain natural armor piercing. |
| **V. Terminal Drift** | Complete human collapse. | **Permanent Loss.** Unit either retreats to the catacombs as a spent elder or turns into a feral aberration. |

---

## 5. Baseline Combat System: The 6-Tick Simultaneous Timeline

Combat resolves across a unified **6-Tick Timeline Track**. All units (allies and enemies) plan orders simultaneously, resolving actions in real time upon execution.

### 5.1 Action Chunking (Maneuver Blocks)
Skills are not scheduled tick-by-tick. Players issue **Maneuver Blocks** consisting of three automated sub-phases:
1. **Windup (Ticks 1–2):** The unit draws back, telegraphing the attack. Stagger resistance drops; highly vulnerable to interrupts.
2. **Impact (Tick 3 or 4):** Damage, armor shred, and status effects resolve on the designated target or tile.
3. **Recovery (Ticks 5–6):** Balance reset. Overcommitted attacks lock the unit in vulnerable recovery; standard actions convert remaining ticks into **Active Guard** or **Strain Venting**.

### 5.2 The Speed Tiers (Fixed Anchors)
Actions snap to three standardized timeline windows:

| Speed Tier | Windup Window | Impact Tick | Tactical Purpose |
| :--- | :--- | :--- | :--- |
| **Snap (Fast)** | None / T1 | **Tick 2** | Interrupts, throwing ash, quick jabs, evasive steps. |
| **Standard (Mid)** | T1–T2 | **Tick 4** | Sword strikes, polearm thrusts, bow shots, field triage. |
| **Heavy (Slow)** | T1–T4 | **Tick 5 or 6** | Sweeps, channeled rites, heavy sunders. High power, vulnerable to interrupts. |

### 5.3 Movement Integration: "Lead or Follow"
Units are not restricted to moving on Tick 1:
* **Advance (Move-Then-Act):** Spends Ticks 1–2 advancing 1–2 tiles, then winds up and strikes on Ticks 3–5.
* **Disengage (Act-Then-Move):** Executes a Fast Snap Action on Ticks 1–2, then retreats 1–2 tiles on Ticks 3–4.
* **Auto-Chase Rules:**
  * *Reach 1 (Brawlers/Daggers):* Tenacious chase—steps forward 1 tile if target retreats during windup.
  * *Reach 2 (Spears/Halberds):* Range hold—does not step forward; hits target if still within Reach 2.
  * *Ranged (Bows/Arbalests):* Rotates facing within arc; fails only if target breaks line-of-sight behind total cover.

### 5.4 Stagger & The Interrupt Rule
* Every unit has a **Stagger Bar** derived from physical Poise and active Stance.
* If a unit receives blunt/stagger damage during an attack's **Windup Ticks** that fills their Stagger Bar:
  * The attack **collapses immediately**.
  * The remaining active ticks shatter and are replaced by a **Stagger Block**.
  * The unit drops their guard and loses their turn for that round.

---

## 6. Shipped Base Classes Adaptation & Skill Trees

The six baseline classes are adapted directly to fit the setting:

```
[ Warrior ]   ──> The Vanguard (Garrison Deserter)       [ Tank / Posture Breaker ]
[ Ranger ]    ──> The Arbalestier (Tollhouse Poacher)     [ Ranged Interrupter / Pin ]
[ Rogue ]     ──> The Gutter-Stalker (Curfew Runner)     [ Scout / Assassin / Phase ]
[ Brawler ]   ──> The Slag-Brute (Bone-Mill Laborer)      [ Brawler / Stagger Engine ]
[ Priest ]    ──> The Heretic Acolyte (Defrocked Scribe)  [ Field Surgery / Strain Siphon ]
[ Mage ]      ──> The Ichor Alchemist (Hedge Occultist)   [ Volatile Chem / Area Blight ]
```

### 6.1 Class 1: The Vanguard (Warrior)
* **Identity:** Former garrison infantry clad in salvaged plate; controls chokepoints and shatters enemy stances.

#### Branch A: Cold Steel (The Iron Discipline)
* **T1 Active — Pommel Strike (2 AP | 0% Strain):** Snap blunt strike (Impact T2) dealing moderate damage and **+25 Stagger**. Doubles stagger against winding-up targets to trigger interrupts.
* **T1 Passive — Braced Poise:** Grants **+20 Maximum Poise**. While holding Active Guard, gain total immunity to knockback/pulls from standard human enemies.
* **T2 Active — Intercepting Hook (3 AP | 0% Strain):** Standard maneuver (Impact T4). Step into an adjacent tile to take a blow meant for an ally. Reduces damage by Armor Rating and pulls attacker 1 tile closer.
* **T2 Passive — Kinetic Leverage:** Mitigating or parrying an attack during Active Guard refunds **1 AP** on the subsequent round (once per round).
* **T3 Active — Sunder & Pin (4 AP | 0% Strain):** Heavy cleave (Impact T5). Permanently strips **4 Armor Rating** and applies **Rooted** for 1 round.
* **Keystone — Unbroken Phalanx:** While at 0% Strain, incoming critical hits are downgraded to regular hits; Active Guard cannot be broken by unblockable attacks.

#### Branch B: Inoculated (The Marrow Graft)
* **T1 Active — Calcified Spike (1 AP | +15% Strain):** Bone spurs punch through skin. Pierces armor completely and inflicts **Bleed** (4 dmg/round for 2 rounds).
* **T1 Passive — Marrow Chitin:** Permanently increases Dormant Strain Floor by **+10%**. Grants **+2 Base Armor Rating**, but consumable heals are 20% less effective.
* **T2 Active — Bile Siphon (2 AP | +25% Strain):** Glands exude caustic enzymes. Deals heavy corrosive damage and restores HP equal to 50% of damage dealt.
* **T2 Passive — Spasm Counter:** Upon crossing into Flesh-Tension (50%+ Strain), immediately deliver a free bone-whip counter-attack against the last attacker.
* **T3 Active — Ribcage Lock (1 AP | +35% Strain):** Ribcage expands externally over chest. Grants **+50% Mitigation** for 1 round. When struck, shatters outward in a 4-tile AoE burst.
* **Keystone — Aberrant Core:** Sets Dormant Strain Floor to 30%. In Aberrant Edge (75%–99% Strain), all active skills cost **-1 AP** (minimum 1), but each completed battle adds **+1 Calcification Point**.

---

### 6.2 Class 2: The Gutter-Stalker (Rogue)
* **Identity:** Rooftop scout and curfew runner; eliminates high-value inquisitors from blind spots.

#### Branch A: Cold Steel (Cinder & Sinew)
* **T1 Active — Hamstring Sever (2 AP | 0% Strain):** Slicing leg cut. Reduces target Movement by **-2 tiles** for 2 rounds and disables reaction attacks. +40% damage from behind.
* **T1 Passive — Prowler's Cadence:** Moving through low cover/darkness costs 0 bonus AP. Frontal detection cone of enemies reduced by 2 tiles.
* **T2 Active — Ash-Cloud & Fade (3 AP | 0% Strain):** Smashes soot flask. Creates a **3×3 smoke zone** blocking ranged targeting for 2 rounds; disengages 2 tiles into stealth.
* **T2 Passive — Anatomical Flaw:** Attacks against Stunned, Staggered, or Rooted targets ignore Armor Rating and gain **+20% Critical Chance**.
* **T3 Active — Throat-Carver's Art (4 AP | 0% Strain):** Execution strike from the rear. Massive piercing damage; instantly executes targets below 35% HP and refunds **2 AP**.
* **Keystone — Ghost of the Undercroft:** While at 0% Strain, backstab kills refund stealth state once per encounter and inflict permanent **Fear** on adjacent standard enemies.

#### Branch B: Inoculated (The Umbral Graft)
* **T1 Active — Void-Flicker (1 AP | +15% Strain):** Teleport up to 3 tiles to an unoccupied square, ignoring obstacles, elevation, and engagement zones.
* **T1 Passive — Viscous Shadow:** Permanently increases Dormant Strain Floor by **+10%**. Silhouette blurs: gain **+15% Evasion**, but light-based attacks deal +25% damage.
* **T2 Active — Flensing Filaments (2 AP | +25% Strain):** Tendons launch from fingertips (range 4). Pulls target 2 tiles, deals corrosive damage, and applies **Exposed** (-3 Armor).
* **T2 Passive — Phase-Twitch:** In Acute Resonance (25%–49% Strain), automatically flicker 1 tile away before an incoming melee attack resolves (once per round).
* **T3 Active — Umbral Dissolution (2 AP | +35% Strain):** Dissolves physical form for 1 round; immune to physical damage. Re-solidifying grants +100% true eldritch damage on next strike, but deals 15% max HP recoil.
* **Keystone — Predatory Singularity:** Sets Dormant Strain Floor to 30%. In Aberrant Edge, critical hits vent **20% Strain** and reset Void-Flicker cooldown; battle completion in this state inflicts **+1 Calcification Point**.

---

## 7. Starting Hub: The Sluice Undercroft & Regional Map

### 7.1 Hub Architecture (Subterranean Safehouse)
The Sluice Undercroft sits three levels beneath the basalt pavement of Lower Cinderreach.

```text
                 [ TO LOWER RUNOFF GRATE / OVERWORLD ]
                                 ▲
                                 │
                   [ 3. THE WATER-LOCK & SIPHON ]
                                 │
     ┌───────────────────────────┼───────────────────────────┐
     │                           │                           │
[ 1. THE APOTHECARY SLAB ]  [ CENTRAL BASIN ]   [ 2. THE TINKER'S HEARTH ]
(Surgeries / Strain Purge)  (Meeting / Stash)   (Cold Steel Forge / Repair)
     │                           │                           │
     └───────────────────────────┼───────────────────────────┘
                                 │
                     [ 4. THE DRY HOLLOWS ]
                   (Resting Bunks / Recovery)
```

1. **The Apothecary’s Slab:** Stone dissection slab. Performs Inoculated graft installations, Calcification surgical triage, and Strain chemical flushes.
2. **The Tinker’s Hearth:** Improvised ventless charcoal forge. Repairs iron plate, adds weapon hooks/weights (Stagger tuning), and crafts throwable ash/smoke pots.
3. **The Water-Lock & Dead-Drop Basin:** Submerged portcullis. Stores communal equipment and receives coded bone-ciphers tracking patrol movements.
4. **The Dry Hollows:** Recessed brick kiln crypts. Recruits rest, clear combat fatigue, or undergo high-Strain psychological quarantine.

### 7.2 The Sunk Ward (Overworld Region Map)
```text
                              [ THE SLAG GATE ]
                       (Exit to Upper Bastion / Wastes)
                                      ▲
                                      │
                         [ THE CRUCIBLE WATCHPOST ]
                         (Inquisition Chokepoint)
                                      ▲
                         ┌────────────┴────────────┐
                         │                         │
            [ ASH-CRAWL TENEMENTS ]       [ THE TALLOW VATS ]
            (Civilian Slums / Contacts)   (Resource Node / Bone-Mill)
                         ▲                         ▲
                         │                         │
                         └────────────┬────────────┘
                                      │
                         [ DROWNED DRAINAGE GRATE ]
                                      ▲
                                      │
                         [ THE SLUICE UNDERCROFT ]
                                (Rebel Hub)
```

---

## 8. Key NPC Profiles & Personal Quest Arcs

### 8.1 Sister Aline Vance (The Apothecary)
* **Archetype:** Radical Pragmatist (Age 26). Defected from the Clerisy’s Hall of Holy Dissection after realizing commoner communion draughts were designed to induce premature ossification.
* **Philosophy:** Believes the subterranean entity will wake regardless; humanity's only survival lies in forced, directed mutation.
* **Personal Quest — *"Anatomy of a Living Saint"*:**
  * *Objective:* Infiltrate St. Lyra’s Crypt, defeat cathedral embalmers, and extract the pituitary gland-cluster of the *Reliquary Prelate* before a 3-tick decay timer expires.
  * *Master Unlock (Tier III):* Unlocks **Apex Grafts**, **Calcification Cleansing** (reverses 1 point of Calcification at the cost of 10% permanent max HP), and **Tincture of Still Blood** (freezes Strain for 1 round).

### 8.2 Bram "Iron-Lung" Karr (The Tinker)
* **Archetype:** Cold Steel Purist (Age 33). Former master smith for the Fourth Kiln Syndicate; joined the rebellion after his young apprentice was culled as flesh-stock for an Inquisitor's graft.
* **Philosophy:** Loathes the Inoculation. Believes cold steel, biomechanical leverage, and pure human grit are the only moral weapons.
* **Personal Quest — *"Cold Slag, Black Iron"*:**
  * *Objective:* Infiltrate The Crucible Kiln, disable furnace draft levers amidst periodic molten iron hazards, and slay Foundry-Master Vane.
  * *Master Unlock (Tier III):* Unlocks **Masterwork Steel Forging** (sunder-pikes, spiked pavises), **Demolition Satchels** (collapses terrain walls/floor tiles), and **Star-Slag Re-Tempering** (+3 Armor Rating, debuff immunity).

### 8.3 Tessia "Tess" Morn (The Courier)
* **Archetype:** Desperate Transactionalist (Age 17). Sluice scout with one deafened ear; operates purely on survival logic. Her 12-year-old brother, Pip, was captured during a street sweep.
* **Philosophy:** Rejects romantic revolutionary rhetoric; views the squad as a heavy battering ram to free her brother before the Red Tithing.
* **Personal Quest — *"The Night of the Red Pen"*:**
  * *Objective:* Infiltrate the Seventh Ward Toll-Pen, avoid sweeping searchlight cones, break Pip out of cage wagon #3, and escort him through a 4-round window before alarms sound.
  * *Master Unlock (Tier III):* Unlocks **Omniscient Reconnaissance** (100% accurate enemy impact ticks and trajectories revealed), **Subterranean Insertion** (deploy 2 units directly into map flanks via grates), and **Pip's Black-Market Runners**.

---

## 9. Combat UI/UX & Visual Communication

### 9.1 Trajectory & Threat Visualization
* **Tick-Beaded Paths:** Movement vectors drawn as segmented chevrons. Each segment represents **1 tick**, marked with small tick numerals (`T1`, `T2`).
* **Ground-Locked vs. Entity-Locked Decals:**
  * *Ground-Locked (Area/Heavy):* Inscribed hazard borders etched onto the floor grid with an Impact Badge (`[ ⚡ T4 ]`). Avoided by stepping off the tile before the impact tick.
  * *Entity-Locked (Homing/Light):* Pulsing reticle locked beneath target entity with a curved tether to the attacker. Mitigated via guards, parries, or fast interrupts.

### 9.2 Timeline Scrubbing & Ghost Behavior
* **Master Timeline Bar:** Gantt-style 6-tick layout displayed across the top of the HUD.
* **Scrubbing Controls:**
  * *Mouse:* Drag playhead across ticks; scroll wheel steps frame-by-frame; middle click resets to Tick 0.
  * *Keyboard:* Keys `1`–`6` jump directly to ticks; `,` and `.` step frame-by-frame; `Spacebar` plays a real-time dry run loop.
* **Ghost Projections:** Hovering or scrubbing to a tick renders translucent silhouettes of all units on their predicted tiles with weapon poses reflecting windup/impact frames.

### 9.3 Interrupt & Stagger Feedback
1. **Timeline Shatter:** Interrupted action blocks crack with a red diagonal strike-through; remaining ticks convert into a striped, amber **Stagger Block**.
2. **Grid Threat Dissolution:** Telegraphed danger decals and targeting tethers instantly dissolve into smoke/ash upon interrupt.
3. **Audio/VFX:** Anvil chime audio cue, 30–60ms hit-stop micro-pause, and floating combat text: `[ INTERRUPTED ]` or `[ STAGGERED ]`.

---

## 10. Technical Sprite & Art Specifications

### 10.1 Canvas & Resolution Anchors
* **Internal Viewport:** $640 \times 360$ (Integer scales: $2\times \to 720\text{p}$, $3\times \to 1080\text{p}$, $4\times \to 1440\text{p}$, $6\times \to 4\text{K}$).
* **Grid Tile Size:** $32 \times 32\text{ px}$.
* **Standard Map Dimensions:** $12 \times 8$ or $12 \times 9$ tiles (Chokepoints: $10 \times 6$ tiles).
* **Character Sprite Bounds:** $32 \times 48\text{ px}$ footprint centered within a $48 \times 48\text{ px}$ or $64 \times 64\text{ px}$ canvas cell to allow weapon reach and graft extrusions.

### 10.2 Palette Architecture

| Layer | Hex Color Codes | Purpose & Rules |
| :--- | :--- | :--- |
| **Environmental Base** | `#1e232d` (Slate)<br>`#2b221b` (Rotting Wood)<br>`#14171c` (Shadow) | Floors, walls, and inert background props. Luminance kept below **25%** so characters pop. |
| **Character Mid-Tones** | `#8a8f9d` (Worn Iron)<br>`#9e7144` (Leather)<br>`#d9cbb4` (Tallow Cloth) | Unit silhouettes, armor, weapons. Clear, readable contrast against floor tiles. |
| **Signal / Threat Accents** | `#e11d48` (Arterial Red)<br>`#a855f7` (Blight Violet)<br>`#fbbf24` (Candle Gold)<br>`#06b6d4` (Salt Cyan) | **Reserved strictly for mechanics:** Impact badges, Strain glows, telegraph lines, interrupts. |

### 10.3 Starter Squad Visual Briefs
1. **Kaelen (Vanguard):** Battered kettle helm, defaced breastplate with gouged-out cathedral sigil, notched billhook poleaxe. Left arm wrapped in bloody linen revealing jagged, calcified bone spurs.
2. **Vesper (Gutter-Stalker):** Shredded soot-stained cowl, slate-shingle leather jerkin, twin inverted bodkin daggers. Fingertips dissolve into shifting black mist; shadow points unnaturally toward foes.
3. **Maren (Heretic Acolyte):** Frayed cleric habit worn under a stained leather butcher's apron, cracked spectacles, bone-saw and censer crook staff. Blackened, pulsing veins creep up neck.
4. **Bran (Arbalestier):** Hunting brigandine with thick recoil horn pauldron, oversized lever-action arbalest. Grafted night-stalker vertical pupil eye that emits a thin red targeting beam when aiming.