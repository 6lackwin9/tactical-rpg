# The High Bastion: campaign graph and Act 2 missions

Status: proposal (Act 2 "The First Rupture"; not locked; follows [CANON.md](../CANON.md) section 3.4). Related:
[world geography](world_geography_and_hub.md), [Sunk Ward map](sunk_ward_campaign_map.md),
[cast registry](cast_registry.md), [enemy bestiary](../systems/enemy_bestiary_and_ai.md),
[campaign structure](../narrative/campaign_narrative_structure.md),
[systems roadmap](../specs/full_game_systems_roadmap.md). Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. Overview

Act 2 takes place in **the High Bastion and St. Lyra's Cathedral district** (Upper Cinderreach). After the Crucible
([Sunk Ward map](sunk_ward_campaign_map.md) section 3.7) the remaining founders come through the Slag Gate into the
Clerisy's wealthy heart: gothic spires, polished basalt boulevards and magnesium lamps over embalming laboratories,
gallows and iron culling stockades.

The district graph has 8 nodes `[data]` and adds rooftop routes, timed searchlight sweeps and moving culling convoys
`[kit hook]` world map extensions.

```text
                             [ THE HIGH BASTION NODE GRAPH ]
                                           |
                                [ 15. GRAND SPIRE GATE ]
                               (overlook to the Mantle Fissure)
                                           ^
                                           | [route: Sacred Chancel]
                                [ 14. RELIQUARY CRYPTS ]
                         (2.6, ENC-ACT2-06: Reliquary Guardian)
                                           ^
                    +----------------------+----------------------+
                    | [route: Spire Buttress]                     | [route: Catacomb Flue]
         [ 13. GRAND BELLTOWER ]                       [ 11. EMBALMING CLOISTERS ]
     (2.5, ENC-ACT2-05: Bell-Warden Colossus)      (2.4, ENC-ACT2-04: Chief Anatomist Lazar)
                    ^                                             ^
                    | [route: Gargoyle Rafters]                   | [route: Scriptorium Duct]
         [ 12. CULLING STOCKADES ]                     [ 10. TALLOW RESERVOIR ]
        (2.2, ENC-ACT2-02: culling wagon)              (2.3, ENC-ACT2-03: silo sabotage)
                    ^                                             ^
                    | [route: Gallows Avenue]                     | [route: Viaduct Aqueduct]
                    +----------------------+----------------------+
                                           |
                                [ 09. GILDED BOULEVARD ]
                       (2.1, ENC-ACT2-01: Ruthven's checkpoint)
                                           ^
                                           | [breached Slag portcullis]
                                [ 08. SLAG GATE BREACH ]
                                   (entry from Act 1)
                                           |
                                           v [concealed siphon runoff]
                                [ THE SCRIBE'S CRYPT ]
                     (optional forward base, proposal; bunks inherited)
```

## 2. Travel

### 2.1 Street or rooftop

Each leg offers two routes `[kit hook]` route variants, `[data]` numbers.

1. **Street cobbles:** 1 day; +10% Alert per leg. Checkpoints held by Castellan Sergeants and Heavy Shield-Bearers;
   Inquisitorial Purifiers likely.
2. **Rooftop rafters and flying buttresses:** 2 days; +2% Alert. Avoids the checkpoints, but risks a structural
   collapse event (minor stagger or fall damage) and Cathedral Crossbow Snipers.

Both cost 1 Warded Tallow per day, as everywhere (CANON.md section 2.5).

### 2.2 Magnesium searchlights

- Upper Cinderreach is lit by tower-mounted magnesium arc-lamps.
- On the district map the cones sweep across the boulevards every 2 days `[kit hook]` timed route hazard.
- Entering a swept node while the beam is on forces an ambush battle and adds +15% Alert. The old "+25% enemy accuracy"
  rider is retired with accuracy rolls; what an ambush does in battle is open (see gaps).

### 2.3 Culling convoys and recruits

- **Convoys:** armoured culling wagons move 09 to 12 to 11 every 3 days, carrying captive youths to the dissection
  slabs `[kit hook]` moving world-map event, `[data]`.
- **Interception:** engaging a convoy starts ENC-ACT2-02; a freed wagon yields 1-2 recruits aged 18-24 for empty bunks
  `[kit hook]` RECRUIT1 (exists), `[data]` recruit pools. One wagon carries Lucan (Bram's former apprentice, 14, a
  non-combat NPC). What happens to freed youths under 18 is open (see gaps). The first procedural recruit takes the
  bunk the Crucible emptied, at the start of Act 2 (user 2026-10-25, Q-38; whether that recruit comes from the first
  wagon or joins at the act transition: see gaps).

### 2.4 The Scribe's Crypt (optional forward base)

- Beneath St. Lyra's scriptorium, reached from node 08. One hub (the Sluice) is preferred; the Crypt is used only if
  the story needs a forward base.
- **Bunks:** inherited (6, or 8 after Bram's "Expanded Timber Frames" project); the Crypt adds none.
- **Apex grafting** (four graft slots: Cranial, Thoracic, Forearms, Visceral) opens at the Apothecary's Slab after
  M2.4 `[kit hook]` graft slots (Act 2 proposal), `[data]`; Rock Salt flushes (effect open, CANON.md section 6).
- **The Courier Blind:** see [world geography](world_geography_and_hub.md) section 2.2.

## 3. Act 2 missions

| # | Mission | Node | Encounter | Payoff |
| --- | --- | --- | --- | --- |
| 2.1 | Ingress | 09 Gilded Boulevard | ENC-ACT2-01, Inquisitor-General Ruthven's checkpoint | Into Upper Cinderreach |
| 2.2 | Jailbreak | 12 Culling Stockades | ENC-ACT2-02, culling wagon (repeatable) | Recruits 18-24; Lucan's rescue (one wagon) |
| 2.3 | Sabotage | 10 Tallow Reservoir | ENC-ACT2-03, silo sabotage | +20 Warded Tallow, -30% Alert |
| 2.4 | The Cloisters | 11 Embalming Cloisters | ENC-ACT2-04, Chief Anatomist Lazar (Aline's quest) | Apex grafting; the Great Tithing's date, scale and cause; Nemeth seen delivering the captured founder |
| 2.5 | The Chimes | 13 Grand Belltower | ENC-ACT2-05, Bell-Warden Colossus | Silences the bells' resonance |
| 2.6 | The Reliquary | 14 Reliquary Crypts to 15 Grand Spire Gate | ENC-ACT2-06, Reliquary Guardian | Key of the Mantle Chasm; Act 3 |

Key choices: the doctrine split, Living Martyrs, the siege dilemma (defend the hub or finish the mission). From Act 2:
founders at 0 HP are downed and may die or be captured `[kit hook]` unit lifecycle; Alert 100% starts a siege of the
hub, and a lost siege costs facilities and forces a relocation, never a Game Over `[kit hook]` campaign meter triggers.

## 4. Encounters

### 4.1 Mission 2.1: the Gilded Boulevard breach

- **Story:** out of the smoking Slag Gate, the founders must break the outer curfew barrier held by Inquisitor-General
  Ruthven's iron vanguard. Ruthven is on stage here; whether he fights is open (CANON.md section 6).
- **Encounter:** ENC-ACT2-01, Gilded Avenue Checkpoint. Castellan Captain + 2 Inquisitorial Purifiers + 2 Greatshield
  Castellans `[data]`.
- **Play:** break the Greatshields with stagger and wall slams, or pierce them with the first Inoculated strikes;
  stay clear of magnesium flash-bombs (Blinded: effect open, accuracy rolls are cut).
- **The captured founder's keepsake** (an ACC heirloom) is in the stash from the start of Act 2; its effect numbers are
  open `[data]`.

### 4.2 Mission 2.2: the culling wagon

- **Node:** 12, the Inquisitorial Penitentiary and Culling Stockades.
- **Story:** a fresh levy of young factory apprentices waits in iron transit pens for the dissection slabs.
- **Encounter:** ENC-ACT2-02, Culling Wagon Interception. Proctor Jailer + 2 Bailiff Enforcers + 2 War Hounds
  (Render-Beast variants) `[data]`.
- **Objective:** the wagon leaves on round 4; breach the pen gate or kill the draught beasts before it exits the map
  `[kit hook]` objectives with a round limit, `[data]`.
- **Payoff:** recruits through RECRUIT1 `[kit hook]`.

### 4.3 Mission 2.3: the Tallow Reservoir

- **Node:** 10, the Merchant Guild Tallow Reservoir.
- **Story:** three huge tallow silos light and power Upper Cinderreach; wrecking the pumps blacks out the district and
  blinds the church searchlights.
- **Encounter:** ENC-ACT2-03, Tallow Silo Sabotage. Guild-Master's Provost + 3 Tallow Dredgers + 2 Crossbow Watchmen
  `[data]`.
- **Hazard:** smashing a silo valve floods lanes with scalding grease: burning or slick tiles `[kit hook]` zone hazards.
- **Payoff:** -30% Alert and +20 Warded Tallow `[data]`.

### 4.4 Mission 2.4: the Embalming Cloisters (Sister Aline's quest)

- **Node:** 11, St. Lyra's Embalming Cloisters and Dispensary.
- **Story:** Aline's personal quest, "Anatomy of a Living Saint" ([NPCs](npcs_and_questlines.md) section 2.1): her
  former order's laboratory, to retrieve her confiscated notes and an Apex gland from an embalmed prelate before it
  decays. Aline is non-combat: she directs, the squad fights.
- **Encounter:** ENC-ACT2-04, the Embalming Cloister Infiltration. Chief Anatomist Lazar + 2 Calcified Husks + 2
  Inquisitorial Suture-Priests `[data]`.
- **Play:** the Suture-Priests' channelled heal regenerates the husks; break it while it is in windup (it lands at the
  end of its own blocks) `[kit hook]` ruleset "timeline".
- **The Great Tithing reveal** (user 2026-10-25, [CANON.md](../CANON.md) 3.6): Lazar's records give the Great
  Tithing's date and scale (thousands) and show that the cell's own raids caused it: every tithe it freed and every
  station it broke starved the dream, the titan stirred, and the Clerisy turned this year's rite into a Great Tithing to
  drug it back to sleep. The visible Doom Clock is relabelled "the Great Tithing" `[data]` (the meter's label).
- **Nemeth's sighting** (refinement, CANON 3.6): the cell sees Inquisitor-Prelate Nemeth deliver the founder captured
  at the Crucible to Lazar's anatomists, to be made into a saint-weapon against the cell, and arrives too late; this
  closes a rescue in practice (Q-33 option a). Staging `[data]` (a scene; no battle with Nemeth here).
- **Payoff:** Apex grafting at the Apothecary's Slab and Aline's bone-scraping; the records above (whether the true
  date also corrects the day count once and sets the Act 3 budget is open: Q-17).

### 4.5 Mission 2.5: silencing the chimes

- **Node:** 13, the Grand Belltower of the Slagged Saint.
- **Story:** the great bells ring every hour of the bell-day; their resonance agitates the sleeping titan and rattles
  calcified bone.
- **Encounter:** ENC-ACT2-05, the Grand Belltower. The Bell-Warden Colossus (heavy automaton) + 2 Watch Snipers
  `[data]`.
- **Hazard:** the chimes sound every 2 rounds; every unit loses 15 Poise unless it passes a Composure check (Composure
  is the derived corruption resist, CANON 2.1; how the check resolves: see gaps) `[kit hook]` scheduled hazard.
- **Payoff:** ends the acoustic resonance penalty in Upper Cinderreach (operatives no longer start battles there with
  extra Strain) `[data]` flag.

### 4.6 Mission 2.6: the Reliquary (Act 2 climax)

- **Node:** 14, the Reliquary Crypts of the First Apostles, then 15 Grand Spire Gate.
- **Story:** under the cathedral floor lies the tomb of the first Extinguishers and the **Key of the Mantle Chasm**, the
  iron seal to the volcanic vents beneath the city. The first Extinguishers are the Clerisy's founders (Q-12 option a,
  CANON 3.6): they founded the order two centuries ago, in the dark of the Extinguishing, with the first Great Tithing,
  recorded as the titan's wrath, and were the first to drink the marrow. Their tomb holds the record of that night,
  which the cell finds here `[data]` (a scene or a found document).
- **Encounter:** ENC-ACT2-06, the Reliquary. The Reliquary Guardian (a gilded saint's sarcophagus animated by Apex
  marrow) + 2 Inquisitor Castellans + 2 Choral Acolytes `[data]`.
- **Play:** the Guardian ignores Cold Steel damage until its four reliquary marrow-vials are smashed; Inoculated
  attacks at high Strain get through its wards `[data]` boss phases (exists).
- **Transition:** the Key opens node 15; the cell looks down into the Sub-Basalt Abyss. Act 3 (The Awakening Abyss)
  begins: the Doom Clock (visible since Act 1 day 1) turns hard, and the tremors start.

## 5. Later uses of the Act 2 encounters (proposal)

| Encounter | Act 2 role | Repeatable or event use |
| --- | --- | --- |
| ENC-ACT2-01 | Boulevard ingress | Roadblock: re-appears on random avenue nodes while Alert is above 60% |
| ENC-ACT2-02 | Recruit source | A convoy appears every 5 days with new recruits |
| ENC-ACT2-03 | Reservoir sabotage | Fuel bounty: side silos keep the Warded Tallow stock up |
| ENC-ACT2-04 | Aline's quest | Organ hunt: raid side dispensaries for rare Aberrant Organs |
| ENC-ACT2-05 | Belltower | Alert suppression: wreck auxiliary searchlight mirrors to clear fog of war |
| ENC-ACT2-06 | Mantle Key | None: the one-time Act 2 to Act 3 trigger (the Doom Clock turns hard) |

## 6. Data sketch `[data]`

Illustrative only (field names as today's `locations.json`; the logic lane owns the schema).

```json
{
  "locations": [
    { "id": "slag_gate_breach", "name": "Slag Gate Breach", "kind": "gate", "region": "high_bastion" },
    { "id": "gilded_boulevard", "name": "Gilded Boulevard", "kind": "street", "encounters": ["enc_act2_01"] },
    { "id": "tallow_reservoir", "name": "Tallow Reservoir", "kind": "industrial", "encounters": ["enc_act2_03"] },
    { "id": "embalming_cloisters", "name": "St. Lyra's Embalming Cloisters", "kind": "cloister", "encounters": ["enc_act2_04"] },
    { "id": "culling_stockades", "name": "Culling Stockades", "kind": "prison", "encounters": ["enc_act2_02"] },
    { "id": "grand_belltower", "name": "Grand Belltower of the Slagged Saint", "kind": "spire", "encounters": ["enc_act2_05"] },
    { "id": "reliquary_crypts", "name": "Reliquary Crypts of the First Apostles", "kind": "crypt", "encounters": ["enc_act2_06"] },
    { "id": "grand_spire_gate", "name": "Grand Spire Gate", "kind": "gate" }
  ]
}
```

## 7. Gaps in this file

- What an ambush does in battle now that accuracy is cut; the Blinded status (sections 2.2, 4.1): Open, see gaps.
- What happens to freed youths under 18 (section 2.3): Open, see gaps.
- How the first procedural recruit arrives at the start of Act 2 (section 2.3): from the first culling wagon (2.2) or
  as a join at the act transition; not written.
- How the "true Tithing date" changes the visible Doom Clock (section 4.4): the relabel to "the Great Tithing" is
  decided (CANON 2.6); a one-time correction of the day count and the Act 3 budget stay open (Q-17).
- The Composure check (section 4.5): Composure is decided (a derived stat, CANON 2.1); how a hazard's check resolves
  (a threshold, a scaled poise loss or a seeded roll) is open for the derived-stat spec and the balance lane.
- Whether "the First Apostles" of the crypts' name are the founders' public title (section 4.6): not written.
- A fight for Ruthven; Alert across act lines; the siege battle content: open in CANON.md section 6 and gaps.
