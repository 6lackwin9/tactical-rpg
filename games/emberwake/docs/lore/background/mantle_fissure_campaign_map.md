# The Mantle Fissure: campaign graph and Act 3 missions

Status: proposal (Act 3 "The Awakening Abyss"; not locked; follows [CANON.md](../CANON.md) sections 3.5 and 4). What
the Tithing and the titan really are is the [central truth](central_truth.md) (proposal): this file states what the
player sees. Related: [world geography](world_geography_and_hub.md),
[High Bastion map](upper_bastion_campaign_map.md), [cast registry](cast_registry.md),
[campaign structure](../narrative/campaign_narrative_structure.md),
[systems roadmap](../specs/full_game_systems_roadmap.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 3 takes place in **the Mantle Fissure and the titan catacombs** (the Sub-Basalt Abyss). With the Key of the Mantle
Chasm from the Reliquary ([High Bastion map](upper_bastion_campaign_map.md) section 4.6) the cell unseals the Grand
Spire Gate and descends into the caldera.

Far below the city lies the titan of the Low Sleep. Veins of black marrow as thick as aqueducts run through boiling
basalt canyons; the cathedral bells above ring down through the stone. The Clerisy is preparing the Red Tithing at the
High Altar (its size is open, see gaps).

The district graph has 8 nodes `[data]`. Main path 16-17-18-21-22; nodes 19 and 20 are optional.

```text
                              [ THE MANTLE FISSURE NODE GRAPH ]
                                             |
                                 [ 22. THE DREAMING CORE ]
                            (3.6, ENC-ACT3-06: the titan; endings)
                                             ^
                                             | [route: Umbilical Trench]
                                 [ 21. HIGH ALTAR ]
                         (3.5, ENC-ACT3-05: Arch-Prelate Othemar)
                                             ^
                    +------------------------+------------------------+
                    | [route: Silent Ossuary]                         | [route: Pulsing Vein Chasm]
         [ 20. SILENT OSSUARY ] (optional)                 [ 18. MARROW CONDUITS ]
     (3.4, ENC-ACT3-04: the corrupted founder)       (3.2, ENC-ACT3-02: High Castellan Ulric)
                    ^                                                 ^
                    | [route: Chasm Rift]                             | [route: Fissure Bridges]
         [ 19. CHORUS SPIRE ] (optional)                   [ 17. WEEPING BASALT CHASM ]
        (3.3, ENC-ACT3-03: Cantor Ysolde)                (3.1, ENC-ACT3-01: Chasm Gatekeepers)
                    ^                                                 ^
                    | [route: Cavern Siphon]                          | [route: Basalt Stair]
                    +------------------------+------------------------+
                                             |
                                 [ 16. CHASM INGRESS ]
                                  (entry from Act 2)
                                             |
                                             v [reinforced basalt vault]
                                 [ THE WEEPING REDOUBT ]
                      (optional forward base, proposal; bunks inherited)
```

## 2. Campaign pressure

### 2.1 The Doom Clock turns hard

- The Doom Clock is the Red Tithing calendar in days, visible since Act 1 day 1 (42 days); soft in Acts 1-2, hard in
  Act 3 `[kit hook]` campaign meter with a fail trigger, `[data]`.
- A route leg costs 1 day (stealth +1 day); a rest costs 1 day; battles cost no days `[kit hook]` travel clock
  (exists).
- If the clock reaches 0 before node 21 (the High Altar) falls, the campaign shows the **Extinguished** failure screen
  (not an ending). The Act 3 day budget is open (CANON.md section 6).

### 2.2 Continental resonance

- As the titan stirs, the cathedral bells above ring in sympathy `[kit hook]` region effect, `[data]`.
- Crossing an unwarded node gives every operative at Calcification Stage I or later +10 Strain at the start of the next
  battle.
- In battle, every 3 rounds a continental shockwave hits every unit: 10 damage that ignores armour and -15 Poise unless
  it passes a Composure check (Composure undefined, see gaps) `[kit hook]` scheduled hazard.

### 2.3 Atmosphere and Warded Tallow

- The air is thick with vaporised raw marrow (the Strain mists: [world geography](world_geography_and_hub.md)
  section 4).
- Upkeep stays the one canon rule: 1 Warded Tallow per day of travel or rest; at 0, -2 starting AP in every battle until
  restocked, and road encounters rise by the travel runtime's starved multiplier (open:
  [Q-18](../questions/open_world_and_campaign.md#campaign-and-meters)) `[kit hook]` resource purse (exists as supplies). The old Act 3 extras (2 Tallow per leg, +15 Strain on map entry while unsupplied, 60% roamer chance) are
  dropped (CANON.md section 2.5).

### 2.4 The Weeping Redoubt (optional forward base)

- Anchored in the petrified marrow arches beside node 16. One hub (the Sluice) is preferred; the Redoubt is used only
  if the story needs a forward base; bunks are inherited.
- **The Quench-Anvil:** Bram forges Black-Iron penetrators with the basalt heat: +100% Poise damage against titan bone
  carapaces `[data]` recipe and item.
- **Isolation Crypts:** iron cages where operatives at Stage IV (The Shakes) are held between battles so their fits do
  not panic the young recruits.
- **Living Martyr:** authorises one final deployment for a Stage IV operative, who fights in an aberrant trance and then
  petrifies into a relic (roadmap P2-03 in the [systems roadmap](../specs/full_game_systems_roadmap.md)) `[kit hook]`
  unit lifecycle (retired), `[data]`.

## 3. Act 3 missions

| # | Mission | Node | Encounter | Payoff |
| --- | --- | --- | --- | --- |
| 3.1 | Descent | 17 Weeping Basalt Chasm | ENC-ACT3-01, Chasm Gatekeepers | Foothold |
| 3.2 | Conduits | 18 Marrow Conduits | ENC-ACT3-02, High Castellan Ulric | Conduits cut; Doom Clock +2 days |
| 3.3 | The Choir (optional) | 19 Chorus Spire | ENC-ACT3-03, Cantor Ysolde | Flesh-Brokers may turn the Chorus against the altar; Malachi mediates |
| 3.4 | The Martyr (optional) | 20 Silent Ossuary | ENC-ACT3-04, the corrupted founder | Mercy; the Transcendent Relic |
| 3.5 | High Altar | 21 High Altar | ENC-ACT3-05, Arch-Prelate Othemar | Othemar falls; the Doom Clock stops |
| 3.6 | The Sleep | 22 Dreaming Core | ENC-ACT3-06, the titan | Endings A, B, C |

## 4. Encounters

### 4.1 Mission 3.1: the Chasm Gatekeepers

- **Story:** apex burrowing aberrations, woken by the Inquisition's deep drilling, bar the descent.
- **Encounter:** ENC-ACT3-01. 2 Bone-Chitin Colossi + 3 Ash-Crawlers `[data]`.
- **Hazard:** unstable basalt shelves: a fractured tile falls into the molten abyss 1 round after a unit steps on it
  `[kit hook]` zone hazards.
- **Play:** stagger the colossi with Black-Iron weapons; knock beasts into the crevices.

### 4.2 Mission 3.2: the Marrow Conduits

- **Story:** brass distillation siphons pump living marrow from the titan's arteries up to the cathedral reservoirs;
  cutting them takes the inquisitors' regenerative serum away.
- **Encounter:** ENC-ACT3-02. High Castellan Ulric (a reconstructed inquisitor) + 2 Inquisitorial Suture-Priests + 2
  Purifier War-Engines `[data]`.
- **Objective:** sunder four conduit valves under fire. A cut conduit floods the tiles beside it with marrow: +100%
  damage that ignores armour for anyone standing in it, and +20 Strain per round `[kit hook]` objectives, zone hazards.
- **Payoff:** enemies lose their regeneration buffs in later Act 3 battles; the Doom Clock gains 2 days `[data]`.

### 4.3 Mission 3.3: the Chorus Spire (optional)

- **Story:** Cantor Ysolde and the Awakening Chorus have raised an altar over a resonant geyser and sing to hasten the
  titan's waking.
- **Encounter:** ENC-ACT3-03. Cantor Ysolde + 4 Chorus Zealots + 2 Calcified Husks `[data]`.
- **Hazard:** sonic geysers erupt on blocks 1 and 4: Deafened (effect open) and -20 Poise within 2 tiles `[kit hook]`
  scheduled hazard.
- **Choice:** if the cell leans Flesh-Broker, Brother Malachi (the friendly hermit, never a boss) mediates and Ysolde
  turns the Chorus's song against the High Altar; otherwise the Chorus must be fought `[kit hook]` doctrine threshold
  (thresholds open), `[data]` dialogue conditions.

### 4.4 Mission 3.4: the corrupted founder (optional)

- **Only if a founder was captured:** the Crucible's (always one, unless a rescue exists; open in CANON.md section 6)
  or one captured in Act 2 `[kit hook]` unit lifecycle (captured).
- **Story:** in the deepest ossuary the cell finds the captured founder. Nemeth took them alive at the Crucible; the
  Clerisy's anatomists wired them into black-iron scaffolding and filled them with raw marrow to make a saint-aberration.
- **Encounter:** ENC-ACT3-04. The Martyr-Saint (the corrupted founder) + 2 Suture-Priests `[data]`.
- **Boss:** keeps the founder's signature skills (for example Kaelen's Shield Slam or Vesper's Shadow Leap), twisted
  with aberrant mutations `[data]` boss built from the founder's loadout, boss phases (exists).
- **Payoff:** mercy: the founder is released. The Founder's Transcendent Relic upgrades the keepsake (numbers open)
  `[data]`.

### 4.5 Mission 3.5: Othemar's last sacrament

- **Story:** on a cyclopean basalt altar over the titan's heart, Arch-Prelate Othemar prepares the captive youths for
  the Red Tithing.
- **Encounter:** ENC-ACT3-05. Arch-Prelate Othemar in a gold-and-iron reliquary war-chassis, 2 Castellan
  Inquisitors and 2 Purifier Executioners `[data]`.
- **Objective:** kill the Executioners within 4 rounds to stop the sacrifice while surviving Othemar's magnesium beams
  and holy-fire mortars `[kit hook]` objectives (slay, protect) with a round limit.
- **Milestone:** Othemar always falls here and is hurled into the chasm; the Doom Clock stops for good.

### 4.6 Mission 3.6: the Dreaming Core

- **Story:** with the conduits cut and Othemar dead, the titan stirs and begins to open its eyes; the cavern collapses
  around the cell.
- **Encounter:** ENC-ACT3-06, a three-phase boss `[kit hook]` boss phases (exists), `[data]`:
  - boss phase 1, the tendril surge: nerve-tendrils slam the arena, break cover and open lava rifts;
  - boss phase 2, the eye of the abyss: the titan's iris opens and floods the board with Strain; operatives alternate
    Inoculated bursts with Cold Steel poise defence;
  - boss phase 3, the terminal decision: the cell reaches the central synapse and chooses an ending.
- **Endings** (CANON.md section 4) `[kit hook]` campaign ending trigger gated by the doctrine meter, `[data]` text:

| Ending | What happens | Voice | Doctrine gate |
| --- | --- | --- | --- |
| A Preservation | Cold Steel: black-iron spikes sever the synapse; the titan sinks into a deep coma; humanity stays mortal and untainted | Bram | Open unless Flesh-Broker |
| B Ascendancy | Transhuman: the cell merges with the titan's marrow and rules as something new | Aline | Flesh-Broker only |
| C Red Martyrdom | The cell gives itself to break the Tithing for good | the cell | Always open |
| Extinguished | Failure screen (no living operative remains, or the Act 3 clock expires), not an ending | - | - |

What a defeat in ENC-ACT3-06 leads to is open (see gaps).

## 5. Later uses of the Act 3 encounters (proposal)

| Encounter | Act 3 role | Side or event use |
| --- | --- | --- |
| ENC-ACT3-01 | Chasm ingress | Quarry: farm black basalt ore for endgame Black-Iron armour |
| ENC-ACT3-02 | Conduit severance | Cut auxiliary lines to stop war-engine reinforcements |
| ENC-ACT3-03 | Chorus Spire | Choral pact: Chorus zealots join as guests `[kit hook]` objectives (guests) |
| ENC-ACT3-04 | The lost founder | Closure for the survivors (the old "maximum Composure" reward waits on Composure) |
| ENC-ACT3-05 | High Altar | Stops the Doom Clock |
| ENC-ACT3-06 | Finale | Endings A, B or C |

## 6. Data sketch `[data]`

Illustrative only (field names as today's `locations.json`; the logic lane owns the schema).

```json
{
  "locations": [
    { "id": "chasm_ingress", "name": "The Chasm Ingress", "kind": "gate", "region": "mantle_fissure" },
    { "id": "weeping_basalt_chasm", "name": "Weeping Basalt Chasm", "kind": "chasm", "encounters": ["enc_act3_01"] },
    { "id": "marrow_conduits", "name": "The Marrow Conduits", "kind": "conduit", "encounters": ["enc_act3_02"] },
    { "id": "chorus_spire", "name": "The Chorus Spire", "kind": "sanctum", "encounters": ["enc_act3_03"] },
    { "id": "silent_ossuary", "name": "The Silent Ossuary", "kind": "crypt", "encounters": ["enc_act3_04"] },
    { "id": "high_altar", "name": "The High Altar", "kind": "altar", "encounters": ["enc_act3_05"] },
    { "id": "dreaming_core", "name": "The Dreaming Core", "kind": "core", "encounters": ["enc_act3_06"] }
  ]
}
```

## 7. Gaps in this file

- Size of the Red Tithing at the altar (section 1): Open, see gaps.
- The Composure check and the Deafened status (sections 2.2, 4.3): Open, see gaps.
- Defeat in ENC-ACT3-06 (section 4.6): Open, see gaps.
- Act 3 day budget, doctrine thresholds, keepsake and relic numbers, rescue of a captured founder: open in CANON.md
  section 6.
