# The six founders and the act progression

Status: canon for sections 1-3 (Act 1); proposal for sections 4-5 (Acts 2-3) (consistency pass 2026-10-25). This doc
follows [CANON.md](../CANON.md); where they differ, CANON wins. Names follow the
[cast registry](../background/cast_registry.md). Related: [classes and skill trees](../systems/classes_and_skill_trees.md),
[campaign structure](campaign_narrative_structure.md), [recruits and prologue](recruits_and_prologue_architecture.md).
Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Summary

All six base classes are authored Act 1 founders and fill the hub's six starting bunks. The first playable slice has
five (Elyse joins later).

1. Act 1 "The Young Vanguard": learn the six classes with Cold Steel only. Inoculation appears once, as a scripted
   teaser that shows its power and its cost.
2. Act 2 "The First Rupture": the player picks an operational doctrine: spend the roster as living weapons, or keep
   Cold Steel with few grafts to preserve the veterans' minds and bodies.
3. Act 3 "The Awakening Abyss": corruption reaches hub life. The titan stirs, the bells cause resonance, Warded Tallow
   runs short, and high-Calcification veterans live in isolation.

## 2. The six founders (canon)

Four join in the prologue; two join in early Act 1 missions, completing the roster before the Crucible
`[data]` (characters, classes).

| # | Founder | Class | Age | Joins | Identity and combat niche |
| --- | --- | --- | --- | --- | --- |
| 1 | Kaelen | Vanguard (warrior) | 21 | Prologue 0a | Deserter guardsman; chokepoints, shield walls, Pommel Strike breaks |
| 2 | Vesper | Gutter-Stalker (rogue) | 18 | Prologue 0b | Curfew runner and thief; Snap strikes, smoke zones, roots |
| 3 | Maren | Heretic Acolyte (priest) | 23 | Prologue 0c | Defrocked scriptorium scribe; field sutures, poise restore, triage |
| 4 | Hollis | Arbalestier (ranger) | 20 | Prologue 0c | Crag poacher; long-range pinning quarrels, suppression |
| 5 | Dunstan "Slag-Jaw" | Slag-Brute (brawler) | 22 | Act 1 M3, The Bone-Mill Raid | Bone-mill stoker; grappling, jaw-breakers, sunder slams |
| 6 | Elyse | Ichor Alchemist (mage) | 19 | Act 1 M4, The Tallow Vats | Hedge chemist; phosphorus pots, magnesium flash, choking smoke |

The four prologue dossiers: [prologue and starter squad](prologue_and_starter_squad_lore.md) 2.

### 2.1 Dunstan "Slag-Jaw" (Slag-Brute)

- Look: a towering, soot-stained youth; iron shackles still riveted around his scarred wrists, used as knuckle guards;
  half his jaw burned by molten slag and wired with scrap.
- Backstory: a stoker at the Fourth Kiln bone-mill under Overseer Grendel, grinding calcified corpses into mortar on
  14-hour shifts; sent to the penal chain gang after cracking an overseer's skull with a shovel.
- Recruitment, M3 The Bone-Mill Raid (node 04, ENC-EVT-03, Overseer Grendel): the cell raids the bone-mill and breaks
  Dunstan off the chain gang. The fight introduces grappling and close-range knockbacks. The old version (a night cart
  ambush, Dunstan working under the Foundry-Master) is retired.

### 2.2 Elyse (Ichor Alchemist)

- Look: a slight frame in an oversized, patched leather apothecary apron stained copper-green; cracked brass goggles on
  her forehead; a wicker bandolier of sulfur and white phosphorus flasks.
- Backstory: a hedge chemist brewing counterfeit tallow and anti-sedative tonics for commoners in a still beneath the
  Tallow Vats, until an Inquisition purge cornered her.
- Recruitment, M4 The Tallow Vats (node 05, ENC-EVT-04, Guild-Master Orlov, Vesper's vendetta): the cell pulls Elyse
  out of the purge. The fight introduces thrown flash-pots and smoke that blocks line of sight. The old version ("The
  Cellar Purge" in a burning tenement) is retired.

## 3. Act 1: Cold Steel and the Inoculation teasers (canon)

Through Act 1 the player relies on discipline, AP, reach and poise; no founder carries Strain or Calcification (their
visible taints are backstory). Two scripted beats foreshadow Inoculation:

1. The regime's hypocrisy (M2, ENC-EVT-02): Proctor Quell, who has commoners executed for mutation, wears an illegal
   marrow valve in his neck. The founders see that the regime's elite carry the mutations they kill others for. (The
   old beat, an Inquisitor Sergeant with a grafted gland at Dunstan's rescue, is folded into Quell; his boss mechanics
   are `[data]`, Open.)
2. Aline's marrow dose (interlude, ENC-EVT-05, node 00): the Calcified Amalgam crawls out of the Awakening Fissure
   under the Sluice. Aline hands over one sealed dose of raw marrow and the player picks which deployed founder takes
   it `[data]` (encounter, item). The old in-battle effect (+100% damage that ignores armour for one round) waits for
   the balance lane: Open. After the battle the founder collapses coughing black bile, the veins along the neck
   blacken, and the founder starts Act 2 at Calcification Stage I `[kit hook]` (stage track set by an event). This is
   the one Act 1 exception to 0% corruption. ENC-EVT-05 is not a siege. The old variant (the dose given against an
   elite Castellan in Mission 5) is retired.

## 4. Act 2: two doctrines (proposal)

The Crucible capture leaves five founders, and Aline opens the Apothecary's Slab. The player settles into a doctrine
(the meter: [choices](narrative_choices_and_dilemmas.md) Choice 2):

| Doctrine | Flesh-Broker: "The cause demands power" | Preservationist: "Hold the human line" |
| --- | --- | --- |
| Skills | Full Inoculation | Minimal, tactical grafts |
| Strain | Fights in the Aberrant Edge (75-99) | Keeps Strain at 49 or below |
| Roster | Recruits as shock troops; units calcify, retire and are replaced | Veterans keep their minds and bodies |
| Battles | Ignores armour; short | Bram's forge; slower attrition |

Neither path is wrong. Flesh-Broker play is fast and volatile and manages a roster that is always close to collapse;
Preservationist play is high-friction positioning that keeps the founders alive through skill.

## 5. Act 3: corruption in hub life (proposal)

The titan (the Low Sleep) stirs beneath Cinderreach and corruption reaches life in the hub.

### 5.1 Bell resonance

- The bells of St. Lyra's toll on an accelerating schedule `[kit hook]` (event trigger on the campaign clock). At each
  major toll, every operative at Calcification Stage I or higher resonates with the titan.
- The old effect, "passive out-of-combat Strain", conflicts with CANON 2.2 (Strain resets to the floor after battle):
  Open.

### 5.2 Warded Tallow scarcity

- The Tallow Guild locks the city pipelines; how this reduces the Tallow on offer: Open.
- At 0 Warded Tallow every battle starts with -2 AP until restocked (CANON 2.5).
- The old nightmare layer (the flame in the Dry Hollows dies, mist seeps into the bunks, "The Leviathan's Whispers"
  cost Composure and add neuroses such as Paranoid Insomnia) is a proposal only. Composure is now defined: a derived
  stat, the corruption resist, and the nightmares are one of its readers ([CANON](../CANON.md) 2.1, Q-47a). Still
  open: the nightmare rule itself (roadmap P3-02) and the neuroses, which need a trait system.

### 5.3 The Isolation Crypts

- Operatives at Stage IV (The Shakes) give off a crystalline hum from their hardened bones; they sleep in the Isolation
  Crypts in the Dry Hollows (CANON 2.3) `[kit hook]` (hub facility).
- The old rule (left in the communal bunks they unsettle recruits and cost starting AP) only matters if the move is the
  player's choice: Open.
- Rock Salt flushes at the Slab steady them before final strikes; the flush effect is a CANON open question.

## 6. Gaps (Open)

- Quell's marrow-valve boss mechanics; the marrow dose's battle numbers.
- Bell resonance effect; how Tallow scarcity works; the nightmare rule and neuroses (Composure itself is decided, a
  derived stat).
- Whether the Crypts move is automatic; the Rock Salt flush effect.
