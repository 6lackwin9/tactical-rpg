# Emberwake: A Scarlet Lullaby - lore canon

Status: canon (2026-10-25, the consistency pass approved by the user). Title: **Emberwake: A Scarlet Lullaby** (user
2026-10-25; replaces the subtitle "The Low Sleep" of DEC-LORE-13; the article says the Red Tithing is one lullaby among
many, sung in blood each generation; the title need not appear in game text). The codename and paths stay `emberwake`.

This file is the single source the domain docs follow. Where a domain doc disagrees, this file wins; Acts 2-3 and
later systems are `proposal` (not locked). "(planner pick)" marks a choice the approved brief did not make (the list
to confirm is in [OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) section 2); open questions (section 6) must not be filled with
invented design. Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]` (engine = technology any game needs;
kit = a shape any story in the genre could fill; game = named by the story, mostly data). "(exists)" marks a
genre rule that today's game crates already carry (`emberwake_rules`, `emberwake_battle`, `emberwake_world`): it is a
`[kit hook]`, not `[engine]`, because the engine crates (`aegis_*`) hold no game rules.

- Names, ages, backgrounds, first appearances and place names: [cast registry](background/cast_registry.md).
- What the Tithing really does (spoiler sheet, proposal): [central truth](background/central_truth.md).

## 1. The cell in one line each

Founders (six in the full Act 1; the first playable slice has five, Elyse joins later): Kaelen (Vanguard), Vesper
(Gutter-Stalker), Maren (Heretic Acolyte), Hollis (Arbalestier), Dunstan "Slag-Jaw" (Slag-Brute), Elyse (Ichor
Alchemist). Hub cast: Sister Aline Vance (Slab), Bram "Iron-Lung" Karr (Hearth), Tess Morn (courier, mission board),
Pip (her brother). Founders' visible taints are backstory and Act 2 hooks only: Act 1 has no Strain, no Calcification
and no countdown on any founder (the hearth line "four months" becomes a non-mechanical hook).

## 2. Numbers sheet

### 2.1 Combat (ruleset option "timeline"; the Ashlands keeps "classic") `[kit hook]`

| Rule | Canon value | Layer |
| --- | --- | --- |
| Round | 6 blocks, all sides resolved simultaneously; timeline strip at the TOP of the battle screen | [kit hook] (exists: `emberwake_battle::resolver`); only the strip widget is [engine] (`aegis_ui::timeline`) |
| AP | 10 max, +4 per round; an action pays AP when it starts and lands on its last block | [kit hook] (exists) |
| AP cost | Decoupled from speed tier: skill field `ap_cost`, default = skill level. Guideline: Cold Steel 2-4 AP, Inoculated 1-2 AP plus Strain | [kit hook] (new field), [data] values |
| Speed tiers | Relative windups: Snap 1 block, Standard 2, Heavy 3-4; the impact lands at the end of the action's own blocks; a maneuver may add recovery blocks. No fixed tick anchors | [data] through the skill field `blocks` (exists); recovery blocks are a new [kit hook] |
| Same-block impacts | The game's initiative order; no mutual trades (a unit stopped earlier in the block does not land) | [kit hook] (exists) |
| Poise | Max Poise = CONS x 2 + armour poise from gear (coefficients planner pick); refills each round; break at 0. CONS runs 6-18 in the stat budget, so CONS x 2 is 12-36 before armour; the lore's stagger values (Pommel Strike 25, Jaw-Breaker 30) were set for flat bars of 30-70 and need a rescale (Q-50) | [kit hook] (new) |
| Stagger multipliers | x2 if struck in windup, x2 in Flesh-Tension, x0.5 under Active Guard; they multiply (planner pick) | [kit hook] |
| Repeated breaks | A unit's second break in the same round or in the next round costs double stagger | [kit hook] |
| Break effect | Cancels the action in windup and the unit's remaining blocks that round; nothing else (the evasion and +25% damage riders are dropped, planner pick) | [kit hook] |
| Active Guard | Base move: halves incoming stagger; 50% damage reduction through the guard term, capped by `GUARD_CAP` 0.9 (planner pick); vents 10 extra Strain at round end | [kit hook] |
| Armour | The hyperbolic curve, reduction = rating / (rating + 50). Lore armour points x 2 = engine rating (planner pick; the balance lane confirms, Q-49): Flesh-Tension +2 -> +4 rating, Chitin +1 -> +2, "shreds 4 armour" -> expose 8, boss armour 3-5 -> rating 6-10 | [kit hook] (exists: `emberwake_rules::damage`), [data] ratings |
| Pierce | Inoculated attacks ignore armour (binary, DEC-LORE-12 baseline). Today's damage type `pierce` is a type, not a bypass, so this needs an armour-bypass skill flag | [kit hook] (new flag), [data] (which skills set it) |
| Deck | 8 = 2 base moves (Move, Active Guard) + 3 class actives + 2 passives + 1 keystone. Today the basics (strike, guard, move, rest, switch_target, offhand_jab) are always active and take no slot, so the two base moves sit outside the loadout; the 5 class + 3 extra loadout slots hold skill families; passives are granted by sources (the world character, faction grants, disciplines, named items), not slotted. The 2 passive + 1 keystone slots are a new kit slot type (Q-48) | [kit hook] (new slot type) |
| Pouch, gear | Pouch 3; six gear slots MAIN, OFF, HEAD, BODY, ACC1, ACC2. Graft slots 4 (Cranial, Thoracic, Forearms, Visceral) are an Act 2 proposal | [kit hook] (exists: `emberwake_rules::gear`); graft slots new |
| Cut | Evasion and accuracy rolls, facing, backstab and stealth cones, the d100 crisis, mutual trades, the multi-track ghost scrubber (start with impact badges on the existing strip), stances (DEC-LORE-11) | - |

Board and art: STAGE1 option B, a tilted battle board of cells drawn 32x24 px (12 x 9 cells today; arena sizes:
Q-60) with zone backdrops, 40 px units and upright props sorted with the units; "32x48 sprite" means the figure
bounds inside the rig's 48x48 frames; the region map stays square top-down. Palette: the art bible's hex values, 16
master hues plus at most one dark and one light step each, plus a skin ramp.

### 2.2 Strain (Act 2 on; Act 1 has none) `[kit hook]` per-unit meter, `[data]` bracket names

| Strain | Bracket | Effect (engine scale) |
| --- | --- | --- |
| 0-24 | Dormant | none |
| 25-49 | Acute Resonance | +15% crit chance, +1 tile per move block, -25% healing received |
| 50-74 | Flesh-Tension | +25% damage, +4 armour rating, stagger taken x2 |
| 75-99 | Aberrant Edge | +50% damage, recoil 10% of damage dealt (planner pick: the bleed and stagger extras are dropped) |
| 100 | Rupture | crisis roll at round end |

- Gain: an Inoculated skill adds its `strain_cost` (guideline 15-35) when the action starts; hazards and eldritch attacks
  add more.
- Decay: at round end -10 if the unit used no Inoculated action that round, a further -10 if it held Active Guard; never
  below the dormant floor. After battle Strain resets to the floor. Grafts raise the floor (0-50).
- Crisis `[game code]` unless the kit meter's triggers can express it: 1d3, equal odds, seeded, rolled at round end:
  1 Unchecked Spasm (a full-power strike on the nearest unit, ally or foe, then stunned next round); 2 Biological
  Collapse (0 AP next round, Bleed 3 rounds, armour 0 for the rest of the battle); 3 Flesh Breach (+1 Calcification
  stage).

### 2.3 Calcification (out of combat, permanent, Act 2 on) `[kit hook]` per-unit stage track

Advances only through Flesh Breach and Tier III graft surgery (not by ending a battle in the Edge).

| Stage | Name | Effect |
| --- | --- | --- |
| I | Vein-Blackening | -5 Composure (undefined: Q-47), +5% debuff resistance |
| II | Calloused Chitin | +2 armour rating, -1 tile per move block |
| III | Sensory Rot | immune to fear and morale debuffs; -15% ranged damage beyond 3 tiles (planner pick: accuracy rolls are cut) |
| IV | The Shakes | cannot equip two-handed weapons (bows and arbalests included); melee and bare-handed attacks pierce armour; sleeps in the Isolation Crypts |
| V | Terminal Drift | lost: walks into the catacombs or turns hostile |

The only undo: Aline's bone-scraping (unlocked by her quest), -1 stage and -10% max HP for good.

### 2.4 Roster and lifecycle `[kit hook]` downed / wounded / captured / dead / retired

| Rule | Canon value |
| --- | --- |
| Ages | Playable operatives and recruits 18-24; Unsealed = everyone under 25; children are stakes, never fighters; no combat graft surgery on anyone under 18. The Spent rarely pass 45 (Agnes, 44, is one of the oldest in the Ash-Crawl; Othemar at 64 is the exception, see the central truth) |
| Squad | Up to 4 deployed (`PARTY_MAX` 4, [kit hook] exists). RECRUIT1 refuses a recruit once the roster holds `party_max`, so the bunk count needs a separate roster cap ([kit hook] new) |
| Bunks | 6 at the start of Act 1 (4 filled after the prologue); 8 through Bram's "Expanded Timber Frames" project, chosen by the player; later hubs inherit the count. Guests take no bunk until sworn in |
| Founder at 0 HP | Downed. Act 1: after the battle a downed founder is wounded (no death, no capture). From Act 2: permadeath and capture. Wounded effects and recovery: Q-31; the Act 2 downed rule: Q-32 |
| Scripted loss | The Crucible: one founder, chosen from the four deployed, holds the winch and is taken alive by the Inquisition |
| Corrupted-founder boss | Optional Act 3 node, only for a captured founder (the Crucible's, or one captured in Act 2) |

### 2.5 Resources and upkeep `[kit hook]` resource purse, `[data]` names

| Resource | Use |
| --- | --- |
| Scrap | The currency (replaces gold): trade, forge costs, bribes |
| Warded Tallow | Upkeep: 1 per day of travel or rest for the whole cell. At 0: -2 starting AP in every battle until restocked |
| Rock Salt | Stash material: Slab flushes, bribes |
| Aberrant Organs | Stash materials: graft surgery (Act 2 on) |

No other upkeep (per-operative, per-leg, Tallow-and-Salt and per-rest versions are dropped).

### 2.6 Campaign meters `[kit hook]` meters with thresholds and triggers

| Meter | Canon value |
| --- | --- |
| Inquisition Alert | 0-100%: 0-30 Guarded, 31-70 Suspicious, 71-99 Lockdown, 100 Breach. No siege in Act 1 (what 100 does in Act 1: Q-13). Acts 2-3: Breach starts a siege of the hub; a lost siege costs facilities and forces a relocation, never a Game Over |
| Doom Clock | The Red Tithing calendar in days, visible from Act 1 day 1 at 42 days ("six weeks"). Soft in Acts 1-2 (story pressure, no fail); hard in Act 3 (fail if it reaches 0 before node 21 falls) |
| Time costs | Route `time` 1 = 1 day on the travel clock ([kit hook] exists: `emberwake_world::travel`, `GameState.time`); a stealth leg takes 1 day more; a rest costs 1 day (planner pick; a rest advancing time is a [kit hook] extension); battles cost no days (planner pick) |
| Doctrine | Preservationist (Bram) to Flesh-Broker (Aline), Act 2 on; gates the endings |

Fail states: the **Extinguished** screen (a failure screen, not an ending) shows when no living operative remains in
the field or the bunks, or when the Act 3 Doom Clock expires. Not a fail: a single death, a lost siege, an Act 1-2
clock expiry.

## 3. Campaign spine

### 3.1 Prologue (canon)

Scripted maps outside the district graph; encounter ids `ENC-PRO-0A..0C` are new (planner pick).

| # | Mission | Place | Encounter | Who joins | Teaches |
| --- | --- | --- | --- | --- | --- |
| 0a | The Alley | Alleyway of Cinders, Seventh Ward | ENC-PRO-0A: 2 Castellan Footmen; Captain Vane gives "Order 44" and withdraws | Kaelen (solo) | windup, guard, Pommel Strike interrupt |
| 0b | The Rooftop | Meat Market rooftops | ENC-PRO-0B: an ambush squad | Vesper | smoke zone, root, disengage |
| 0c | The Bridge | Canal bridge, iron mortuary cart | ENC-PRO-0C: 3 Inquisitors + a crossbowman (bestiary unit names: Q-73) | Maren, Hollis (freed from the cart) | ranged pins, field suture |

Then the cell dives through the Drowned Runoff Grate (node 01) into the Sluice Undercroft: the hearth scene with Bram
and Aline, six bunks with four filled, the Doom Clock appears at 42 days, Act 1 title card.

### 3.2 Act 1 "The Young Vanguard": the Sunk Ward (canon)

Missions come from Tess's dead-drop board (a hub NPC offering quests) plus node travel; the board offers the next main
mission when the previous one is done (planner pick).

| # | Mission | Node | Encounter | Joins / loses | Key choice |
| --- | --- | --- | --- | --- | --- |
| M1 | The Sweep | 02 Alleyway Junction 7 | ENC-EVT-01, Captain Vane (boss; breaks the sweep sealing the grates) | - | - (planner pick: the old prologue slot) |
| M2 | The Crossroads Cages | 03 Ash-Crawl Crossroads | ENC-EVT-02, Proctor Quell: an open-air culling station, free the youths before the flagellants kill them; retorts for the Slab (Maren's hook) | - | - |
| M3 | The Bone-Mill Raid | 04 Fourth Kiln | ENC-EVT-03, Overseer Grendel | Dunstan (bunk 5) | - |
| M4 | The Tallow Vats | 05 Tallow Vats | ENC-EVT-04, Guild-Master Orlov (Vesper's vendetta) | Elyse (bunk 6) | - |
| Interlude | The Awakening Fissure | 00 Sluice Undercroft | ENC-EVT-05, the Calcified Amalgam; Aline's single marrow dose (first grafting teaser) | - | Which deployed founder takes the dose; that founder starts Act 2 at Calcification I (planner pick) |
| M5 | The Crucible | 06 Crucible Watchpost to 07 Slag Gate | ENC-EVT-06, Inquisitor-Prelate Nemeth; one founder holds the Slag Gate winch | Loses the chosen founder (captured) | Deploy four (the Crucible requires four, planner pick); choose who stays |

Optional Act 1 board work: Kaelen's watchpost raid (forge bellows), Hollis's vantage points over the Crucible, Vesper's
drainage locks (route shortcuts), Tess's "Night of the Red Pen" (rescue Pip), Agnes's "Rafters of Tenement 12", Corvo's
gate, Brother Rook on street nodes. ENC-EVT-05 is not a siege in Act 1.

### 3.3 Act transitions

| Transition | Hub | Alert | Doom Clock | Switches on |
| --- | --- | --- | --- | --- |
| Prologue to Act 1 | The Sluice Undercroft opens (6 bunks) | starts at 0 (planner pick) | appears at 42 days, soft | board, travel, upkeep |
| Act 1 to Act 2 | Stays the Sluice; the Slag Gate breach opens the High Bastion graph (node 08 on); the Scribe's Crypt is a proposal forward base | open (Q-14) | stays soft | Inoculation, Strain, Calcification, grafts, recruits, doctrine, permadeath and capture, sieges, the 8-bunk project; the captured founder's keepsake (an ACC heirloom, planner pick) |
| Act 2 to Act 3 | Stays the Sluice; the Weeping Redoubt is a proposal forward base | carries on | turns hard | bell resonance (but 3.4 M2.5 already silences a resonance in Act 2: Q-09), Act 3 hazards |

### 3.4 Act 2 "The First Rupture": the High Bastion (proposal)

| # | Node | Encounter | Payoff |
| --- | --- | --- | --- |
| 2.1 | 09 Gilded Boulevard | ENC-ACT2-01, Inquisitor-General Ruthven's checkpoint | Into Upper Cinderreach |
| 2.2 | 12 Culling Stockades | ENC-ACT2-02, culling wagon (repeatable) | Recruits 18-24; Lucan's rescue (planner pick: one wagon) |
| 2.3 | 10 Tallow Reservoir | ENC-ACT2-03 | +20 Warded Tallow, -30 Alert |
| 2.4 | 11 Embalming Cloisters | ENC-ACT2-04, Chief Anatomist Lazar (Aline's quest) | Apex grafting; the true Tithing date |
| 2.5 | 13 Grand Belltower | ENC-ACT2-05, Bell-Warden Colossus | Silences the bells' resonance in Upper Cinderreach (conflicts with 3.3: Q-09) |
| 2.6 | 14 Reliquary Crypts to 15 Grand Spire Gate | ENC-ACT2-06, Reliquary Guardian | Key of the Mantle Chasm; Act 3 |

Key choices: the doctrine split, Living Martyrs, the siege dilemma (defend the hub or finish the mission).

### 3.5 Act 3 "The Awakening Abyss": the Mantle Fissure (proposal)

| # | Node | Encounter | Payoff |
| --- | --- | --- | --- |
| 3.1 | 17 Weeping Basalt Chasm | ENC-ACT3-01, Chasm Gatekeepers | Foothold |
| 3.2 | 18 Marrow Conduits | ENC-ACT3-02, High Castellan Ulric | Conduits cut; Doom Clock +2 days |
| 3.3 | 19 Chorus Spire (optional) | ENC-ACT3-03, Cantor Ysolde | Flesh-Brokers may turn the Chorus against the altar; Malachi mediates (planner pick) |
| 3.4 | 20 Silent Ossuary (optional) | ENC-ACT3-04, the corrupted founder | Mercy; the Transcendent Relic (upgrades the keepsake, planner pick) |
| 3.5 | 21 High Altar | ENC-ACT3-05, Arch-Prelate Othemar | Othemar always falls here (planner pick); the Doom Clock stops |
| 3.6 | 22 Dreaming Core | ENC-ACT3-06, the titan | Endings |

Main path 16-17-18-21-22; 19 and 20 are optional (planner pick).

## 4. Endings (proposal; meaning set by the central truth)

| Ending | What happens | Voice | Doctrine gate |
| --- | --- | --- | --- |
| A Preservation | Cold Steel: black-iron spikes sever the synapse; the titan sinks into a deep coma; humanity stays mortal and untainted | Bram | Open unless Flesh-Broker (planner pick) |
| B Ascendancy | Transhuman: the cell merges with the titan's marrow and rules as something new | Aline | Flesh-Broker only |
| C Red Martyrdom | The cell gives itself to break the Tithing for good | the cell | Always open (planner pick) |
| Extinguished | Failure screen (annihilation or the Act 3 clock), not an ending | - | - |

## 5. Glossary (one name per thing)

| Canonical term | Meaning | Retired variants |
| --- | --- | --- |
| Act 1 / 2 / 3 | Campaign units; titles The Young Vanguard, The First Rupture, The Awakening Abyss | Phase 1-3, The Iron Vanguard, The Transhuman Rupture, The Blight of the Low Sleep, The Closed Four |
| The Low Sleep | The titan's slumber and, by extension, the titan | (title "Emberwake: The Low Sleep") |
| The Red Tithing | The recurring sacrifice of Unsealed youths of any age | - |
| The Unsealed / the Spent | Under 25 / 25 and over (compliant through decades of draughts and tallow fumes) | The Caged Young, The Spent Generation (fine in prose), Docile Masses |
| Sovereign Clerisy, Iron Inquisition, Castellan Watch, Tallow Guild, Fourth Kiln Syndicate, Severed Root, Awakening Chorus | Factions; the Watch is the Inquisition's city arm, the Syndicate the Guild's foundry arm | Hegemony, High Hegemony, Sub-Basalt Cults, Seventh Ward Watch, Castellan Guard |
| Preservationist / Flesh-Broker | The two doctrines | The Purists, The Inoculated (as faction names) |
| Cold Steel / Inoculated | The two skill branches | Martial path, Resonant path |
| Strain brackets | Dormant, Acute Resonance, Flesh-Tension, Aberrant Edge, Rupture | Clear, Quiescence, Twitch, Tremor, Resonance, Rupture Verge, Rupt, CRISIS |
| Calcification stages | Vein-Blackening, Calloused Chitin, Sensory Rot, The Shakes, Terminal Drift | Veins, Brittle, Gargoyle, Martyr, Death/Aberr, calcification "points" |
| Scrap | Currency | Gold, Scrap Iron, black iron slag, Salvaged Slag, salvaged timber and charcoal |
| Warded Tallow | Upkeep fuel | vials, candles, ledgers, High-Grade Warded Tallow |
| Rock Salt | Stash material | Salt Rations, Rock-Salt Rations, Sanctified (Rock) Salt (the pouch item Sanctified Salt Sachet stays an item) |
| Aberrant Organs | Harvested graft materials; Apex organs are Tier III | Harvested Aberrant Organs, Aberrant Marrow Organs |
| The Sluice Undercroft | The hub | - |
| Apothecary's Slab, Tinker's Hearth, Dry Hollows, Water-Lock Basin, Isolation Crypts | Hub facilities (the Basin is the central pool, dead-drop, launch point and guest quarters; the Crypts sit in the Dry Hollows) | Bram's Hearth/Forge, surgical rig, Water-Lock (& Siphon / Gate), Dead-Drop Basin, Central Basin, Quarantine Kiln Cell, Isolation Vaults, Quarantine Crypts |
| Mission board | Tess's dead-drop board | - |
| Doom Clock, Inquisition Alert | Campaign meters | Great Extinguishing (now the Extinguished failure screen) |
| The Extinguishing / Extinguished | The event two centuries ago that put out the sun, moon and stars / the failure screen (an intended echo, kept) | the Great Extinguishing (as a name for the loss) |
| Castellan Footman, Watch Crossbowman | The Castellan Watch's line soldier and crossbow unit | Castellan Warden (prologue), Watch Arbalestier (collided with the Arbalestier class) |

## 6. Open questions (do not fill with invented design)

Every open question, with options, a recommendation, where it lives and who decides, is in
[OPEN_QUESTIONS.md](OPEN_QUESTIONS.md) (stable ids Q-01..Q-91; the questions to decide first in its section 1, the
planner picks to confirm in its section 2). The ones this file names: the central truth (Q-01); Tithing size (Q-04);
the bell-resonance contradiction between 3.3 and 3.4 (Q-09, no pick); Act 1 Alert at 100% (Q-13); Alert across act
lines (Q-14); soft-clock expiry in Acts 1-2 and the Act 3 day budget (Q-16, Q-17); doctrine inputs and thresholds
(Q-19); whether the cell defeats Nemeth or only escapes (Q-25); a fight for Inquisitor-General Ruthven (Q-26); the act
of Bram's quest (Q-27); wounded effects and recovery (Q-31); the Act 2 downed rule (Q-32); rescuing a captured
founder in Act 2 (Q-33); keepsake numbers (Q-34); Living Martyrs' "+1 stage per battle" against the Calcification
rule of 2.3 (Q-37); costs of the 8-bunk project (Q-38); the Rock Salt flush effect (Q-43); Composure, an undefined
stat (Q-47); deck slot kinds (Q-48); the armour scale and pierce (Q-49); the stagger rescale (Q-50); post-crisis
Strain (Q-62).
