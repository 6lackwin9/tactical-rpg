# Bestiary: Act 1 encounters

Status: canon for which encounter serves which mission ([CANON](../CANON.md) 3.1-3.2); proposal for rosters, arenas,
hazards, variants and drops. Units: [enemy bestiary](enemy_bestiary_and_ai.md). Every encounter is `[data]` (catalog or
world encounter: roster, deployment, objective, phases, arrival wave, morale, purse, `key_loot`, `objective_reward`);
new mechanics point to the index gaps.

Arenas: the battle board is 12 x 9 cells, drawn as the STAGE1 option B tilted board (32 x 24 px cells, zone backdrop,
40 px units; [STAGE1](../../specs/STAGE1.md)). The old layouts were larger; each entry gives the old size and the gap
is index gap 1.

| Id | Mission | Node | Boss | Objective `[kit hook]` exists | Joins / loses |
| --- | --- | --- | --- | --- | --- |
| ENC-PRO-0A | 0a The Alley | outside the graph | none (Captain Vane withdraws) | rout | Kaelen (solo) |
| ENC-PRO-0B | 0b The Rooftop | outside the graph | none | rout | Vesper |
| ENC-PRO-0C | 0c The Bridge | outside the graph | none | rout (proposal) | Maren, Hollis |
| ENC-EVT-01 | M1 The Sweep | 02 Alleyway Junction 7 | Captain Vane | slay (proposal) | - |
| ENC-EVT-02 | M2 The Crossroads Cages | 03 Ash-Crawl Crossroads | Proctor Quell | protect / guests (proposal) | - |
| ENC-EVT-03 | M3 The Bone-Mill Raid | 04 Fourth Kiln | Overseer Grendel | slay (proposal) | Dunstan (bunk 5) |
| ENC-EVT-04 | M4 The Tallow Vats | 05 Tallow Vats | Guild-Master Orlov | slay (proposal) | Elyse (bunk 6) |
| ENC-EVT-05 | Interlude The Awakening Fissure | 00 Sluice Undercroft | the Calcified Amalgam | slay (proposal) | - |
| ENC-EVT-06 | M5 The Crucible | 06 Crucible Watchpost to 07 Slag Gate | Inquisitor-Prelate Nemeth | escape | loses the winch founder (captured) |

## Prologue (scripted maps outside the district graph)

- ENC-PRO-0A, The Alley (Alleyway of Cinders, Seventh Ward): Kaelen alone against 2 Castellan Footmen; Captain Vane
  gives "Order 44" and withdraws. Teaches windup, guard and the Pommel Strike interrupt.
- ENC-PRO-0B, The Rooftop (Meat Market rooftops): an ambush squad; Vesper joins. Teaches the smoke zone, root and
  disengage. Roster: Q-73 (index gap 5).
- ENC-PRO-0C, The Bridge (canal bridge, iron mortuary cart): 3 Inquisitors and a crossbowman; Maren and Hollis are
  freed from the cart. Teaches ranged pins and the field suture. Unit names: Q-73 (index gap 5).

## ENC-EVT-01: The Sweep (Act 1 M1)

- Concept: rain-slick cobbled alleys between tall timber tenements; Vane's sweep is sealing the grates and the cell
  breaks it.
- Old layout 12 x 10: narrow 2-tile choke alleys, low crates (half cover), high rain gutters (raised firing spots).
- Roster: Captain Vane; 2 Castellan Footmen; 1 Watch Crossbowman on a raised tile.
- Arena rule: Rain-Slick Cobblestones (old: a DEX check or slip when moving more than 2 tiles in one block; a roll,
  index gap 3).
- Variant (side bounty, proposal): Vane at full strength, 2 more Watch Crossbowmen, an arrival wave in round 3
  (PHASEFOLD exists).
- Drops: Vane's Castellan Billhook (weapon), 25 Scrap, a City Gate Pass cipher (quest item); the relic Vane's Gilded
  Castellan Pavise ([items](items_gear_and_progression.md) 7).

## ENC-EVT-02: The Crossroads Cages (Act 1 M2)

- Concept: an open-air culling station at the Ash-Crawl Crossroads; the cell must free the caged youths before the
  flagellants kill them. Maren's hook (Quell denounced her). Freed youths are rescued townsfolk, not recruits (recruits
  start in Act 2, CANON 3.3).
- Old layout 14 x 10: an iron cage wagon as an impassable line in the centre, burning street lanterns.
- Roster: Proctor Quell; 1 Inquisitorial Purifier; 2 Mortuary Flagellants.
- Arena rule, the Culling Timer: the flagellants kill one captive every 2 rounds. Guests and protect objectives exist
  (OBJ1); a scripted kill on a round cadence is `[kit hook]` new.
- Drops: retorts for the Apothecary's Slab (CANON 3.2), Quell's bone-saws, 1 Inquisitor Marrow Valve (Aberrant Organ),
  2 Warded Tallow; the relic Quell's Marrow Scalpel.

## ENC-EVT-03: The Bone-Mill Raid (Act 1 M3)

- Concept: a roaring foundry kiln of grinding gears, bone pulverisers and rivers of molten slag; Dunstan is on the
  penal chain gang.
- Old layout 14 x 12: conveyor belts that push units 1 tile per block toward crushing rollers; open slag pools.
- Roster: Overseer Grendel; 2 Tallow Dredgers; 1 Bailiff Enforcer.
- Arena rule, Slag Vent Burst: at the end of blocks 3 and 6 steam vents scald two board columns (medium hit). Scheduled
  hazards and moving belts are index gap 2.
- Variant (side raid, proposal): wrecks the city's mortar production, Inquisition Alert -25.
- Drops: Grendel's pneumatic hammer, 50 Scrap, Slag-Tempered Plating (armour); the relic Grendel's Pneumatic
  Slag-Hammer.

## ENC-EVT-04: The Tallow Vats (Act 1 M4)

- Concept: the tallow storage vats under the tanneries; dim light, caustic fumes, narrow catwalks over boiling fat.
  Vesper's vendetta against Orlov; Elyse is cornered in her still below.
- Old layout 16 x 10: 1-tile catwalks over vat pits; falling in is a heavy hit.
- Roster: Guild-Master Orlov; 2 Render-Beasts; 2 Tallow Dredgers.
- Arena rule, Grease Miasma: a unit standing in the low vapour loses 1 AP unless it wears a mask (an item check:
  `[kit hook]` new).
- Variants (proposal): Vesper's vendetta grants her dialogue and her "Orlov's Retribution" skill; an economic sabotage
  side mission hijacks a shipment for 5 Warded Tallow.
- Drops: Orlov's flensing chain hooks, the pressurised lard sprayer, 100 Scrap, 5 Warded Tallow; the relic Orlov's
  Flensing Talon.

## ENC-EVT-05: The Awakening Fissure (Act 1 interlude)

- Concept: the floor beneath the Sluice Undercroft cracks open on a throbbing vein of the titan. Not a siege in Act 1.
  Aline gives one deployed founder her single marrow dose (the first grafting teaser); that founder starts Act 2 at
  Calcification Stage I (CANON 3.2). What the dose does inside this fight: Open, see gaps.
- Old layout 12 x 12: uneven cavern, black bone pillars (full cover), violet mist pools.
- Roster: the Calcified Amalgam; 3 Ash-Crawlers; 2 Calcified Husks.
- Arena rule, Aberrant Proximity: +5 Strain per round to every operative in the chamber (Act 2 on; idle in Act 1).
- Variant (Acts 2-3, proposal): the same chamber as a hub siege when Inquisition Alert reaches Breach (100; CANON 2.6).
  The old "at Alert 80%" trigger is retired.
- Drops: 1 Titan Marrow Core (an Apex organ, Tier III, for Act 2 grafts); the relic the Dreamer's Crystalline
  Vertebra. After this fight Brother Malachi's catacombs open (cast registry).

## ENC-EVT-06: The Crucible (Act 1 M5, the Act 1 climax)

- Concept: the fortified watchpost between the Sunk Ward and the High Bastion: iron gates, burning braziers, siege
  ballistas. The Crucible requires four deployed founders.
- Old layout 18 x 12, split by the gate; winches on the east and west flanks.
- Roster: Inquisitor-Prelate Nemeth; 2 Inquisitorial Purifiers; 2 Castellan Footmen; 2 Watch Crossbowmen.
- Scripted sacrifice: the Slag Gate winch is jammed; one founder, chosen by the player from the four deployed, holds it
  under fire so the other three escape. That founder is captured alive by the Inquisition (not killed). Escape exists
  as an objective (OBJ1); a "hold this tile, chosen hero stays behind" rule and the capture state are `[kit hook]` new
  (unit lifecycle, [roster](roster_expansion_and_lifecycle.md)).
- Whether the cell defeats Nemeth or only escapes: CANON open question.
- Payoff: the Slag Gate breach opens the High Bastion graph (node 08 on) and switches on the Act 2 systems (CANON 3.3);
  the captured founder's keepsake (an ACC heirloom, planner pick) stays with the cell; a memorial in the Dry Hollows.
  Nemeth's Warded Cassock drops only if he can be defeated (Open, see gaps).
