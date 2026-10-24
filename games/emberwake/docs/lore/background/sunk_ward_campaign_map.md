# The Sunk Ward: campaign graph and Act 1 missions

Status: canon (2026-10-25, consistent with [CANON.md](../CANON.md) section 3; Act 1 "The Young Vanguard"). World id
`sunk_ward`, the first Low Sleep world in `games/emberwake`. Related: [world geography](world_geography_and_hub.md),
[cast registry](cast_registry.md), [enemy bestiary](../systems/enemy_bestiary_and_ai.md),
[onboarding and prologue](../narrative/new_game_onboarding_and_intro.md),
[campaign structure](../narrative/campaign_narrative_structure.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 1 of Emberwake: A Scarlet Lullaby takes place in **the Sunk Ward**, the lowest industrial quarter of Cinderreach.
The district map is a node graph of street crossings, drain conduits, tenement rooftops and guarded church posts
`[kit hook]` world map (exists: `worldmap.rs` + `travel.rs`), `[data]` `locations.json` and `routes.json`. Travel is
non-linear: the cell picks routes, weighs days against risk, dodges searchlight sweeps and watches the Inquisition
Alert.

```text
                              [ THE SUNK WARD NODE GRAPH ]
                                           |
                                [ 07. THE SLAG GATE ]
                               (exit to Upper Cinderreach)
                                           ^
                                           | [route: High Gorge]
                                [ 06. CRUCIBLE WATCHPOST ]
                             (M5, ENC-EVT-06: Nemeth, the winch)
                                           ^
                    +----------------------+----------------------+
                    | [route: Flue Climb]                         | [route: Convoy Track]
         [ 04. FOURTH KILN ]                           [ 05. THE TALLOW VATS ]
      (M3, ENC-EVT-03: Grendel; Dunstan)             (M4, ENC-EVT-04: Orlov; Elyse)
                    ^                                             ^
                    | [route: Pipe Duct]                          | [route: Slum Alleys]
         [ 02. ALLEYWAY JUNCTION 7 ]                   [ 03. ASH-CRAWL CROSSROADS ]
       (M1, ENC-EVT-01: Captain Vane)                (M2, ENC-EVT-02: Proctor Quell)
                    ^                                             ^
                    | [route: Water Siphon]                       | [route: Drain Flue]
                    +----------------------+----------------------+
                                           |
                              [ 01. DROWNED RUNOFF GRATE ]
                                 (egress / checkpoint)
                                           ^
                                           | [submerged sluice gate]
                              [ 00. SLUICE UNDERCROFT ]
                     (the hub; interlude ENC-EVT-05: the Awakening Fissure)
```

The prologue maps (0a-0c) are scripted and sit outside this graph.

## 2. Travel and the Inquisition Alert

### 2.1 Route travel

- **Cost:** a route leg costs 1 day on the travel clock `[kit hook]` (exists: route `time` in `travel.rs`), and 1
  Warded Tallow per day `[kit hook]` resource purse (the runtime's supplies become Warded Tallow), `[data]`.
- **Road encounters:** each route has a road-encounter chance, 15% standard (route `encounter_pct`), raised by the
  travel runtime's starved multiplier while Warded Tallow is at 0 `[kit hook]` (exists), `[data]` (whether the x1.5 also
  applies in the Low Sleep beside the -2 AP of CANON 2.5 is open:
  [Q-18](../questions/open_world_and_campaign.md#campaign-and-meters)).
- **Stealth or forced march** (chosen per leg) `[kit hook]` route travel modes, `[data]` numbers:
  - stealth (default): under fog and chimney soot; lower encounter risk; +1 day;
  - forced march: no extra day, but +5% Alert per leg and a guaranteed roamer encounter while Alert is 50% or more.

### 2.2 Inquisition Alert and searchlight sweeps

The Inquisition Alert (0-100%) is the regime's search intensity, carried by the generic meter system (thresholds,
triggers, inputs, a warning at each threshold and Lockdown effects; user 2026-10-25, Q-15) `[engine]` if game-agnostic,
else `[kit hook]`; `[data]` bands, inputs and effects (numbers by the balance lane).

- **Searchlight sweeps:** some routes are swept by searchlights from the watchtowers on a timer `[kit hook]` timed
  route hazard on the world map. Entering a node while it is lit starts an ambush battle and adds +10% Alert.
- **Bands:**
  - 0-30% Guarded: ordinary watch patrols; routes mostly open;
  - 31-70% Suspicious: checkpoint gates locked; Castellan Footmen hold the bridge nodes; tolls (Scrap) to pass without
    a fight;
  - 71-99% Lockdown: Inquisitor-General Ruthven's Render-Beast packs roam the main roads; searchlights sweep every 2
    days;
  - 100% Breach: no siege in Act 1. An Act 1 Breach ends the run on the **Extinguished** failure screen (user
    2026-10-25, Q-13; CANON 2.6), but only a player who blatantly ignores the meter gets there: Act 1 inputs are small
    and a clear warning shows at each threshold (the warning steps are a design-lane spec with Q-15) `[kit hook]`
    campaign fail condition, `[data]` warnings and text.
- **Inputs** (Q-15 option a): loud and quiet tags on missions, bribes in Scrap or Rock Salt, and the story amounts listed
  in the docs (searchlights, forced marches); numbers by the balance lane.

## 3. Act 1 missions

Missions are taken from **Tess's dead-drop board** at the Water-Lock Basin (a hub NPC offering quests) plus node
travel `[kit hook]` mission board, `[data]` quests; the board offers the next main mission when the previous one is
done. Squad: up to 4 deployed. Founder at 0 HP: downed; after the battle a downed founder is wounded (Act 1 has no death
and no capture except the Crucible) `[kit hook]` unit lifecycle. Act 1 has no Strain, no Calcification track and no
grafts; the founders' visible taints are backstory.

| # | Mission | Node | Encounter | Joins / loses | Key choice |
| --- | --- | --- | --- | --- | --- |
| 0a | The Alley | prologue map | ENC-PRO-0A | Kaelen (solo) | - |
| 0b | The Rooftop | prologue map | ENC-PRO-0B | Vesper | - |
| 0c | The Bridge | prologue map | ENC-PRO-0C | Maren, Hollis | - |
| M1 | The Sweep | 02 Alleyway Junction 7 | ENC-EVT-01, Captain Vane | - | - |
| M2 | The Crossroads Cages | 03 Ash-Crawl Crossroads | ENC-EVT-02, Proctor Quell | - | - |
| M3 | The Bone-Mill Raid | 04 Fourth Kiln | ENC-EVT-03, Overseer Grendel | Dunstan (bunk 5) | - |
| M4 | The Tallow Vats | 05 Tallow Vats | ENC-EVT-04, Guild-Master Orlov | Elyse (bunk 6) | - |
| Interlude | The Awakening Fissure | 00 Sluice Undercroft | ENC-EVT-05, the Calcified Amalgam | - | Who takes Aline's marrow dose |
| M5 | The Crucible | 06 Crucible Watchpost to 07 Slag Gate | ENC-EVT-06, Inquisitor-Prelate Nemeth | Loses one founder (captured) | Which deployed founder holds the winch |

### 3.1 Prologue 0a-0c (summary)

Full staging: [onboarding and intro](../narrative/new_game_onboarding_and_intro.md).

- **0a The Alley** (Alleyway of Cinders, Seventh Ward): Kaelen, who deserted the Castellan Watch rather than cripple a
  girl in a Tithing sweep, fights 2 Castellan Footmen alone; Captain Vane gives "Order 44" and withdraws. Teaches
  windup, guard and the Pommel Strike interrupt.
- **0b The Rooftop** (Meat Market rooftops): Vesper joins against an ambush squad. Teaches smoke zones, root and
  disengage.
- **0c The Bridge** (canal bridge): the cell breaks open an iron mortuary cart and frees Maren and Hollis from 3
  Inquisitors and a crossbowman. Teaches ranged pins and field suture.
- Then the cell dives through the Drowned Runoff Grate (node 01) into the Sluice Undercroft: the hearth scene with
  Bram and Aline, six bunks with four filled, the Doom Clock appears at 42 days (soft), the Act 1 title card.

### 3.2 M1: The Sweep

- **Node:** 02 Alleyway Junction 7 (Seventh Ward curfew district).
- **Story:** Captain Vane, Kaelen's former commanding officer, who sold his own men to the Inquisition for a church
  commission, leads the sweep that is sealing the drain grates to trap the cell. Kaelen's personal hook.
- **Encounter:** ENC-EVT-01, Seventh Ward Alleyway Ambush. Captain Vane (boss) + 2 Castellan Footmen `[data]`.
- **Play:** windup interrupts against a boss; a Snap (1 block) Pommel Strike breaks a footman's Heavy thrust while it
  is in windup, cancelling it and the footman's remaining blocks that round `[kit hook]` ruleset "timeline".
- **Payoff:** the sweep is broken; the grates stay open.

### 3.3 M2: The Crossroads Cages

- **Node:** 03 Ash-Crawl Crossroads.
- **Story:** Proctor Quell, "the Scalpel of St. Lyra's", runs an open-air culling station at the crossroads. Quell
  denounced Maren and oversaw her mentor's autopsy (Maren's hook).
- **Encounter:** ENC-EVT-02, the Crossroads Cages. Proctor Quell + 1 Inquisitorial Purifier + 2 Mortuary Flagellants
  `[data]`.
- **Objective:** the Flagellants kill 1 caged youth every 2 rounds; the squad pushes through the narrow tenement stalls
  to break the cage locks `[kit hook]` objectives (protect, exists), `[data]` timer. What the freed youths give the cell
  is open (see gaps).
- **Play:** kill fast targets first, step out of the Purifier's phosphorus ground hazards, pin with ranged attacks.
- **Payoff:** retorts for the Apothecary's Slab `[data]` reward.

### 3.4 M3: The Bone-Mill Raid

- **Node:** 04 Fourth Kiln.
- **Story:** the Inquisition builds its watch-posts with mortar of ground human bone from the Fourth Kiln bone-mill.
  The cell breaks in to wreck the grinding machinery and free the penal chain gang.
- **Encounter:** ENC-EVT-03, the Bone-Mill Raid. Overseer Grendel + 2 Tallow Dredgers + 1 Bailiff Enforcer `[data]`.
- **Hazards:** conveyor belts push units toward the crushing rollers (schedule open); steam vents burst on blocks 3 and 6
  `[kit hook]` scheduled zone hazards, `[data]`.
- **Play:** positioning, knockback into walls, handling armoured heavies.
- **Joins:** Dunstan "Slag-Jaw" (Slag-Brute), freed from the chain gang; bunk 5 `[kit hook]` RECRUIT1 (scripted),
  `[data]`.
- **Payoff:** the Tinker's Hearth gains heavy armour reinforcement recipes paid in Scrap `[kit hook]` exchange recipes,
  `[data]`.

### 3.5 M4: The Tallow Vats

- **Node:** 05 Tallow Vats.
- **Story:** cathedral purge teams and Guild mercenaries scour the tenement cellars with scalding grease. Elyse, a hedge
  chemist brewing counterfeit tallow and anti-sedative tonics, is cornered in her still beneath the rendering cauldrons.
  Guild-Master Orlov is Vesper's enemy: his rendering pipelines took her parents, and his Guild paid the informant who
  sold out her runner ring.
- **Encounter:** ENC-EVT-04, the Tallow Vat Incursion. Guild-Master Orlov + 2 Render-Beasts + 2 Tallow Dredgers
  `[data]`.
- **Hazards:** narrow catwalks over boiling fat; grease puddles apply Flammable and slow movement `[data]`.
- **Play:** Render-Beasts chase and pounce (AI weights `[data]`); chemical fire reactions on grease. Vesper's vendetta
  against Orlov has no mechanic defined (see gaps).
- **Joins:** Elyse (Ichor Alchemist); bunk 6. The six founders are assembled. Elyse is not in the first playable slice
  (five founders).
- **Payoff:** opens Warded Tallow raids on the board `[data]` quests.

### 3.6 Interlude: The Awakening Fissure

- **Node:** 00 Sluice Undercroft (the sub-basin catacombs below the hub).
- **Story:** a deep tremor splits the stone under the Undercroft; black marrow vents and a horror of the Low Sleep
  climbs out. Not a siege: the Alert is not involved.
- **Encounter:** ENC-EVT-05, the Awakening Fissure. The Calcified Amalgam + 3 Ash-Crawlers + 2 Calcified Husks
  `[data]`. The chamber pulses with the titan's heartbeat (staging only: Act 1 has no Strain).
- **The first grafting teaser:** against the Amalgam's bone carapace, Sister Aline gives one deployed founder a single
  dose of raw marrow: for 1 round that founder deals +100% damage that ignores armour `[data]` status (old numbers and
  delivery wait for the balance lane: [Q-66](../questions/open_systems_and_scope.md#strain-and-calcification)). The player
  chooses who takes it; after the battle that founder carries Calcification Stage I into Act 2 (no effect in Act 1)
  `[kit hook]` per-unit stage track.
- **Payoff:** the first Apex organ (Aberrant Organs, Tier III) for Act 2 grafting `[data]` stash material; the fissure
  opens Brother Malachi's catacombs (he appears from here on).

### 3.7 M5: The Crucible

- **Node:** 06 Crucible Watchpost, then 07 Slag Gate.
- **Story:** the cell assaults the Crucible Watchpost to open the Slag Gate and break the cordon between the Sunk Ward
  and the cathedral district above.
- **Encounter:** ENC-EVT-06, the Crucible Watchpost. Inquisitor-Prelate Nemeth (a marrow-infused commander cold steel
  cannot finish) + 2 Inquisitorial Purifiers + 2 Castellan Footmen + 2 Watch Crossbowmen `[data]`.
- **Deployment is the decision:** the Crucible requires four deployed founders; the founder sacrificed is chosen from
  those four `[kit hook]` deployment rules, `[data]`.
- **The winch:** the Slag Gate winch is jammed with iron bolts; one founder must stay on the winch tile and work the
  lever under fire while the others get out `[kit hook]` objectives (hold, escape), `[game code]` only if the scripted
  stay-behind cannot be expressed as data. That founder is overwhelmed and taken alive by Nemeth's executioners
  (captured, not killed): the one scripted Act 1 loss `[kit hook]` unit lifecycle (captured). The cell only escapes,
  and Nemeth escapes with the captured founder (user 2026-10-25, Q-25 a; CANON 3.6): the cell sees him hand them to
  Lazar at 2.4, and at the optional Act 3 node 20 the founder, corrupted, fights at his side, then breaks and kills him.
- **Transition:** ends Act 1 (The Young Vanguard). The remaining founders go through the Slag Gate into Act 2 (The
  First Rupture). Switched on in Act 2: Inoculation, Strain, Calcification, grafts at the Apothecary's Slab, recruits,
  the doctrine split, permadeath and capture, sieges, the 8-bunk project; the captured founder leaves a keepsake (an
  ACC heirloom; its numbers are open), and their bunk takes the first procedural recruit at the start of Act 2 (Act 1
  holds only the six founders, Q-38).

### 3.8 Optional Act 1 board work

Kaelen's watchpost raid (forge bellows), Hollis's vantage points over the Crucible, Vesper's drainage locks (route
shortcuts), Tess's "Night of the Red Pen" (rescue Pip), Agnes's "Rafters of Tenement 12", Bailiff Corvo's gate and
Brother Rook on the street nodes ([NPCs](npcs_and_questlines.md)) `[data]` quests and dialogue. Which node hosts each
job is open (see gaps).

## 4. Later uses of the Act 1 encounters (proposal)

| Encounter | Act 1 role | Later use (proposal) |
| --- | --- | --- |
| ENC-EVT-01 (Vane) | M1 The Sweep | Bounty: hunt Captain Vane in the ruins for -25% Alert (only if he survives M1; open) |
| ENC-EVT-02 (Quell) | M2 The Crossroads Cages | Prison raid in Act 2: free 2 recruits (18-24) to refill lost bunks |
| ENC-EVT-03 (Grendel) | M3 The Bone-Mill Raid | Sabotage: halt bone-mortar output to weaken a fortress wall |
| ENC-EVT-04 (Orlov) | M4 The Tallow Vats | Vesper's loyalty quest: settle the feud and seize 10 Warded Tallow |
| ENC-EVT-05 (Amalgam) | Interlude, the grafting teaser | Acts 2-3 only: a candidate siege battle when Alert reaches 100% (never in Act 1) |
| ENC-EVT-06 (Nemeth) | M5 The Crucible | None: the one-time Act 1 to Act 2 trigger (Nemeth himself returns in ENC-ACT3-04) |

## 5. Data sketch `[data]`

Illustrative only: the field names follow today's world files (`locations.json`: `id`, `name`, `kind`, `region`,
`map`, `encounters`; `routes.json`: `from`, `to`, `two_way`, `time`, `encounter_pct`); the logic lane decides the real
schema, including how hub facilities attach (no `facilities` field exists today).

```json
{
  "locations": [
    { "id": "sluice_undercroft", "name": "The Sluice Undercroft", "kind": "hub", "region": "sunk_ward" },
    { "id": "alleyway_junction_7", "name": "Alleyway Junction 7", "kind": "street", "encounters": ["enc_evt_01"] },
    { "id": "ash_crawl_crossroads", "name": "Ash-Crawl Crossroads", "kind": "street", "encounters": ["enc_evt_02"] },
    { "id": "fourth_kiln", "name": "The Fourth Kiln", "kind": "industrial", "encounters": ["enc_evt_03"] },
    { "id": "tallow_vats", "name": "The Tallow Vats", "kind": "industrial", "encounters": ["enc_evt_04"] },
    { "id": "crucible_watchpost", "name": "The Crucible Watchpost", "kind": "fortress", "encounters": ["enc_evt_06"] }
  ],
  "routes": [
    { "id": "water_siphon", "from": "drowned_runoff_grate", "to": "alleyway_junction_7", "two_way": true, "time": 1, "encounter_pct": 15 }
  ]
}
```

## 6. Gaps in this file

- What the youths freed in M2 give the cell (section 3.3): Open, see gaps.
- Vesper's vendetta mechanic against Orlov (section 3.5): Open, see gaps.
- Which node hosts each optional board job, including Corvo's Third Ward gate (section 3.8): Open, see gaps.
- Alert 100% in Act 1 (decided: the run ends, Q-13), Nemeth (decided: he escapes, Q-25) and wounded effects (decided:
  -20% max HP until a rest, Q-31) are settled; the Breach story text and the warning steps are open (story guide G-23;
  Q-15 spec); keepsake numbers: open in CANON.md section 6.
