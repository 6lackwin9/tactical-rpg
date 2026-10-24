# Full game systems roadmap

Status: canon for the first playable slice and milestones M0-M2; the Acts 2-3 systems (sections 4-5) are `proposal`.
Re-based on [CANON.md](../CANON.md) on 2026-10-25; carriers and hooks:
[rpg_kit_and_system_architecture.md](rpg_kit_and_system_architecture.md). Rows here are wanted work for the planner to
turn into specs and task rows; nothing here is a task yet. Layer tags: `[engine]`, `[kit hook]`, `[data]`,
`[game code]`.

## 1. The first playable slice: "Sunk Ward, first night"

One sitting from New Game to the Crucible, on the "timeline" ruleset, in the new world `sunk_ward`. Five founders:
Kaelen (Vanguard), Vesper (Gutter-Stalker), Maren (Heretic Acolyte), Hollis (Arbalestier), Dunstan "Slag-Jaw"
(Slag-Brute); Elyse joins after the slice.

| # | Beat | Content | Carried by | Tag |
| --- | --- | --- | --- | --- |
| 1 | Intro vignette | The short illustrated opening, then the rain | vignette player | `[kit hook]`, `[data]` |
| 2 | Prologue 0a The Alley | Kaelen alone, ENC-PRO-0A (2 Castellan Footmen; Captain Vane gives "Order 44" and withdraws); teaches windup, guard, the Pommel Strike interrupt | encounter, objectives | `[data]` |
| 3 | Prologue 0b The Rooftop | Vesper joins; ENC-PRO-0B ambush squad; teaches smoke zone, root, disengage | encounter, guest then recruit | `[data]` |
| 4 | Prologue 0c The Bridge | Maren and Hollis freed from the iron mortuary cart; ENC-PRO-0C (3 Inquisitors + a crossbowman); teaches ranged pins, field suture | encounter, `protect` / guests, recruit | `[data]` |
| 5 | The hearth scene | The dive through the Drowned Runoff Grate; Bram and Aline at the hearth; six bunks, four filled; the Doom Clock appears at 42 days; the Act 1 title card | event trigger, vignette player, campaign meter | `[kit hook]`, `[data]` |
| 6 | The Undercroft hub | Walkable hub with Sister Aline (the Slab; grafting opens in Act 2), Bram (the Hearth: forge, projects), Tess (the dead-drop board at the Water-Lock Basin); stash, bunks | region map location, NPCs, shops, mission board, facilities | `[data]`, `[kit hook]` |
| 7 | The district | A 4-node Sunk Ward graph (which nodes: gap 1), routes costing 1 day (stealth +1), Warded Tallow upkeep, the Alert meter | world map, travel clock, supplies as Tallow, meter | `[data]`, `[kit hook]` |
| 8 | The bone-mill raid | Act 1 M3, node 04 Fourth Kiln, ENC-EVT-03 Overseer Grendel; Dunstan joins (bunk 5) | encounter, boss phases, recruit | `[data]` |
| 9 | One dilemma | One authored choice with a lasting flag (which one: gap 2) | dialogue, flags | `[data]` |
| 10 | The Crucible | Act 1 M5, node 06 to 07, ENC-EVT-06 Inquisitor-Prelate Nemeth; deploy four; after the fight one of the four holds the Slag Gate winch and is captured | deployment, objectives, unit lifecycle "captured" | `[kit hook]`, `[data]` |
| 11 | Slice end | A card for the end of the slice (the Act 2 opening is outside it) | vignette player | `[data]` |

Rules in force: no Strain, no Calcification, no permadeath; a founder at 0 HP is downed and wounded after the battle;
Scrap is the currency; the Doom Clock is soft; no siege.

What the slice needs that does not exist: the ruleset switch and its rules, the vignette player, event triggers, the
mission board, campaign meters (Alert, Doom Clock), the unit lifecycle (wounded, captured), the bunk cap, currency and
upkeep names per world, rest time, the 0-Tallow option (-2 starting AP), the Sunk Ward art (tiles, the Undercroft,
named portraits).

## 2. Milestones

### M0 Foundation

| Deliverable | Tag |
| --- | --- |
| Kit extraction (approved by the user for M0 on 2026-10-25; the planner's record holds the scope): the genre-generic rules the lore needs, named story-free | `[kit hook]` |
| The Ashlands kept as the kit's sample and parity world: the "classic" ruleset is the default and the parity suites stay green through every later milestone | `[data]` |
| The library: CANON.md, the cast registry, this pass; the planner copies the settled rows to `games/emberwake/docs/decisions/lore.md` | docs |
| The world `sunk_ward` skeleton: manifest, regions, factions, the hub location, start party (Kaelen) | `[data]` |

Exit: the docs pass `tools/fc.sh docs`; `sunk_ward` loads through `aegis content`; the gate is clean.

### M1 Combat feel in the Dojo

| Deliverable | Tag |
| --- | --- |
| Ruleset option "timeline" / "classic" (where it is set: architecture gap 2); stances off in timeline | `[kit hook]` |
| Speed tiers through `blocks`; recovery blocks; `ap_cost` | `[kit hook]`, `[data]` |
| Poise meter, break (cancels the windup and the remaining blocks that round), stagger multipliers, repeat-break cost; Active Guard halves stagger | `[kit hook]` |
| The armour-bypass flag for Inoculated skills (used from Act 2) | `[kit hook]` |
| Impact badges on the timeline strip (strip at the top, COMBAT-UI3 layout A) | `[engine]` |
| Founder class kits for the five founders as Dojo presets | `[data]` |
| The balance sim accepts the ruleset option | `[kit hook]` |

Depends on: STAGE1 (the board) and FX1 (effects) as already approved. Exit: a Dojo duel and a 4-on-4 play the windup,
break and guard loop; the classic parity traces are unchanged.

### M2 Act 1 slice on data

| Deliverable | Tag |
| --- | --- |
| Vignette player; event triggers; mission board; campaign meters (Alert, Doom Clock, soft); unit lifecycle (downed, wounded, captured); bunk cap with a facility level | `[kit hook]` |
| Currency and upkeep names per world; the 0-stock option; a rest advances time | `[kit hook]` |
| The slice of section 1: encounters ENC-PRO-0A..0C, ENC-EVT-03, ENC-EVT-06; the hub and its NPCs, dialogue, quests on Tess's board; the 4-node district; founders as world characters | `[data]` |
| Placeholder art for the Sunk Ward (tiles by code, recoloured kit bodies, portraits only through the image API, asked first) | `[data]` |

Exit: New Game to the Crucible plays end to end; saves load at every beat.

After M2 (open, gap 3): the rest of Act 1 (M1 The Sweep, M2 The Crossroads Cages, M4 The Tallow Vats with Elyse, the
ENC-EVT-05 interlude, the optional board work) is data on the same hooks.

### M3 Corruption layer (Act 2 systems)

| Deliverable | Tag |
| --- | --- |
| Per-unit battle meter (Strain: gain on action start, decay, floor, reset), bracket effects | `[kit hook]`, `[data]` |
| The crisis table | `[game code]` or `[kit hook]` |
| Per-unit stage track (Calcification I-V) and its triggers | `[kit hook]`, `[data]` |
| Exchange recipes at facilities (Slab flushes, bone-scraping, graft surgery) | `[kit hook]`, `[data]` |
| Permadeath and capture from Act 2; the 8-bunk project; campaign fail conditions (Extinguished) | `[kit hook]` |

### M4 Balance and art

| Deliverable | Tag |
| --- | --- |
| Balance-lane runs on the timeline ruleset (targets in `games/emberwake/docs/decisions/balance.md`); the armour rescale confirmed | balance |
| The palette and skin ramp applied to Sunk Ward tiles, bodies and UI ([art_and_visual_style.md](art_and_visual_style.md)) | `[data]` |
| Named portraits, aberrations (image API, asked each time, or a future rig), the Undercroft and district backdrops | `[data]` |

## 3. Act 1 systems at a glance

| System | Act 1 | Tag |
| --- | --- | --- |
| Combat | Timeline ruleset, Cold Steel only | `[kit hook]` |
| Roster | Six founders (five in the slice), 6 bunks, up to 4 deployed; downed then wounded; the Crucible capture | `[kit hook]`, `[data]` |
| Economy | Scrap, Warded Tallow (1 per day), Rock Salt, organs in the stash | `[kit hook]`, `[data]` |
| Meters | Alert (no siege in Act 1), Doom Clock soft from day 1 | `[kit hook]` |
| Missions | Tess's board plus node travel | `[kit hook]`, `[data]` |

## 4. Act 2 systems (proposal)

| Id | System | Summary | Carrier | Tag |
| --- | --- | --- | --- | --- |
| P2-01 | Recruits from culling wagons | Rescued youths 18-24 with rolled stats, a quirk and a class; quirk list and rolls open | RECRUIT1 plus rolled characters | `[kit hook]`, `[data]` |
| P2-02 | Apex grafts and excision | 4 graft slots (Cranial, Thoracic, Forearms, Visceral); an apex graft raises the dormant Strain floor; Aline can cut one out for -15% max HP for good | second slot set, stage track | `[kit hook]`, `[data]` |
| P2-03 | Living Martyrs | A Stage IV operative may fight one last mission (immune to control, +100% true damage, 0 AP skills), then petrifies into a relic | unit lifecycle "retired" plus statuses | `[kit hook]`, `[data]` |
| P2-04 | Doctrine | Preservationist (Bram) to Flesh-Broker (Aline); gates facilities, lines and endings; inputs and thresholds open (candidate: grafts and high-Strain battles against forge upgrades and salt flushes) | campaign meter | `[kit hook]`, `[data]` |
| P2-05 | Permadeath and capture | From Act 2; whether a captured founder can be rescued is open | unit lifecycle | `[kit hook]` |
| P2-06 | Sieges | Alert Breach starts a siege of the hub; a lost siege costs facilities and forces a relocation, never a Game Over | encounter at the hub, facility levels | `[kit hook]`, `[data]` |

## 5. Act 3 systems (proposal)

| Id | System | Summary | Carrier | Tag |
| --- | --- | --- | --- | --- |
| P3-01 | Bell resonance | When the bells toll, operatives at Calcification I or more gain Strain before battle; in battle a pulse hurts units at 50+ Strain (numbers open) | meter triggers on the clock | `[data]` |
| P3-02 | Tallow blackout | Daily Tallow use doubles; at 0 the hub flame dies and sleeping operatives may gain afflictions (the d100 Composure roll is cut; the rule is open); Stage IV operatives sleep in the Isolation Crypts | upkeep option, lifecycle | `[kit hook]`, `[data]` |
| P3-03 | Hard Doom Clock | The calendar turns hard: the Extinguished screen if it reaches 0 before node 21 falls; the day budget is open | campaign meter, fail condition | `[kit hook]`, `[data]` |
| P3-04 | Optional nodes | 19 Chorus Spire and 20 Silent Ossuary (the corrupted-founder boss, only for a captured founder) | district graph, flags | `[data]` |

## 6. Endings (proposal)

| Ending | What happens | Doctrine gate |
| --- | --- | --- |
| A Preservation | Cold Steel severs the synapse; the titan sinks into a deep coma; humanity stays mortal and untainted | Open unless Flesh-Broker |
| B Ascendancy | The cell merges with the titan's marrow and rules as something new | Flesh-Broker only |
| C Red Martyrdom | The cell gives itself to break the Tithing for good | Always open |
| Extinguished | The failure screen (no operative left, or the Act 3 clock), not an ending | - |

What the endings mean depends on the central truth, a proposal for the user:
[central_truth.md](../background/central_truth.md). Carried by flags and the vignette player `[data]`.

## 7. Gaps

1. The slice's four district nodes. Options: (a) 01 Drowned Runoff Grate, 03 Ash-Crawl Crossroads, 04 Fourth Kiln,
   06 Crucible Watchpost (07 Slag Gate as the Crucible's exit map); (b) 01, 02 Alleyway Junction 7, 04, 06.
   Recommendation: (a): the entry, one street node with M2 ready for the full Act 1, the bone-mill and the Crucible;
   the full Act 1 adds 02, 05 and 07.
2. The slice's one dilemma. Options: (a) one of the Act 1 board quests given a two-way choice (for example Corvo's
   gate); (b) the ENC-EVT-05 marrow dose, which needs the interlude and only sets a flag in Act 1. Recommendation: (a); the
   interlude belongs to the full Act 1.
3. Where the rest of Act 1 lands. Options: (a) an M2b of data tasks before M3; (b) inside M4. Recommendation: (a).
4. The slice's end. Options: (a) an end card after the Crucible; (b) the Act 2 hub reveal. Recommendation: (a).
5. The founders' levels and gear in the slice and the Scrap and Tallow starting stock are not set anywhere.
   Recommendation: the balance lane proposes them after M1.
