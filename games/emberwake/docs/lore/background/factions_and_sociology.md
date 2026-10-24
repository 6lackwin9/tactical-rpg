# Factions and sociology: power in Cinderreach

Status: canon (2026-10-25, consistent with [CANON.md](../CANON.md)); the doctrine split (section 4) is Act 2 and its
numbers are `proposal`. Names follow the [cast registry](cast_registry.md); the Clerisy's real motive is the
[central truth](central_truth.md) (proposal). Layer tags: `[engine]`, `[kit hook]`, `[data]`, `[game code]`.

## 1. Hierarchy

Cinderreach is an ecclesiastical police state built on scarcity, the industrial rendering of the dead and fear of the
titan. Power runs down from the cathedral through the Inquisition and the guilds to the labourers.

```text
                    [ THE SOVEREIGN CLERISY ]   blood rites, cathedral wards
                       /                 \
                      /                   \
    [ THE TALLOW GUILD ]               [ THE IRON INQUISITION ]
    rendering, tallow, salt            culling, curfews, pens
    + Fourth Kiln Syndicate            + Castellan Watch
      (foundry and bone-mill arm)        (city arm)
                      \                   /
                       \                 /
                     [ THE SPENT ]   adults 25+, compliant
                            |
                            |  raids, rescues, resistance
                            v
                  [ THE SEVERED ROOT ]   the Unsealed cell (player roster)

    Outside the order: [ THE AWAKENING CHORUS ]   cult that sings the titan awake (Act 3)
```

## 2. Faction dossiers

### 2.1 The Sovereign Clerisy (the ruling elite)

- **Role:** the high clergy, wardens and ritual keepers of the titan; headed by Arch-Prelate Othemar.
- **Doctrine:** suffering is holy; obedience keeps the continent from sinking into the black tar-sea; humanity's
  original sin disturbed the titan's sleep, and only tithes of blood, labour and silence keep it asleep.
- **The Red Tithing:** bells summon the city to the plazas; Unsealed youths of any age are chosen by the hierarchs and
  led in chains into the catacombs as the lullaby that keeps the titan asleep (what the Clerisy preaches). The date is
  the Doom Clock `[kit hook]` campaign meter; size and frequency are open (see gaps).
- **Look:** featureless windowless iron masks with breath slits; bleached horsehair vestments stiffened with warded
  beeswax; swinging gilded censers burning narcotic resin that dulls a crowd into reverent stupor.

### 2.2 The Iron Inquisition and the Castellan Watch (the armed arm)

- **Role:** the Clerisy's enforcers, led by Inquisitor-General Ruthven. The Castellan Watch is its city arm: it walks
  the ramparts, enforces curfews, runs the juvenile pens and roots out unregistered mutants and cells.
- **Fighting style** `[data]` (enemy units, skills and AI weights):
  - heavy foot in fluted iron plate and kettle helms;
  - billhooks, halberds, notched sunder-pikes, spiked kite shields;
  - they break poise: heavy stagger attacks wear a target's poise down, then a heavy blow lands on the broken unit
    (poise and breaks are `[kit hook]` ruleset "timeline");
  - magnesium flash-bombs in narrow alleys (the effect on the target is open: accuracy rolls are cut, see gaps).
- **The culling pens:** walled stockades at every ward gate where the young are held without trial and inspected for
  bone growths under the skin or black blood.

### 2.3 The Tallow Guild and the Fourth Kiln Syndicate (industry)

- **Role:** the city's industrial backbone: rendering vats and crushing mills on the edge of the slums. The Fourth
  Kiln Syndicate is the Guild's foundry and bone-mill arm (Bram worked its upper forge, Dunstan its bone-mill).
- **Monopolies:**
  - **Warded Tallow:** rendered from carcasses, livestock and the unredeemed human dead; mandatory for house lamps,
    because wood or raw coal draws shadow-aberrations from the vents;
  - **Bone-mortar:** ground ossified remains, cement for the crumbling foundations of Upper Cinderreach;
  - **Salt ledgers:** the Rock Salt trade, rationed to keep ward bosses compliant.
- **Standing:** despised as butcher-merchants, yet indispensable: insult a tallow-master and a household's lamp ration
  is revoked, which in the dark means death.
- **Figures:** Guild-Master Orlov (the Tallow Vats), Overseer Grendel (the bone-mill), Foundry-Master Jessop (the
  Fourth Kiln casting floor); Madame Sallow trades on the Guild's black margin.

### 2.4 The Spent (the subjugated majority)

- **Who:** the labourers, mill-hands, seamstresses and masons aged 25 and over.
- **State:** bound by exhaustion, the sedative communion draughts and lung-rot from tallow smoke. They see rebellion
  as suicide; most parents will bind a rebellious child and hand them to the inquisitors to keep the household's salt
  and oil.

### 2.5 The Awakening Chorus (Act 3, proposal)

A cult of the Sub-Basalt Abyss led by Cantor Ysolde that sings to wake the titan from the Chorus Spire (Act 3 node
19, ENC-ACT3-03). Brother Malachi is not of the Chorus: he is a friendly hermit who can mediate with it (see
[NPCs](npcs_and_questlines.md)).

## 3. The Severed Root (the insurrection)

The cell is founded and manned by the Unsealed; every operative is 18-24. It works out of the drowned drains and
forgotten crypts and fights to survive.

### 3.1 Rule of three

To withstand torture and scrying, the rebellion keeps strict compartments.

- **Cells:** operatives work in cells of three or four, the size of a deployed squad (up to 4, `PARTY_MAX`)
  `[kit hook]` (exists).
- **The blind chain:** no operative knows more than two outside contacts; a captured cell can expose only its
  immediate handler.
- **Bone-ciphers:** messages carved on fragments of black bone, dropped into submerged grates and collected at the
  Water-Lock Basin, where Tess runs the dead-drop board (the mission board: a hub NPC offering quests) `[kit hook]`
  mission board, `[data]` quests.

### 3.2 Founders

Six in the full Act 1 (the first playable slice has five; Elyse joins later): Kaelen (Vanguard), Vesper
(Gutter-Stalker), Maren (Heretic Acolyte), Hollis (Arbalestier), Dunstan "Slag-Jaw" (Slag-Brute), Elyse (Ichor
Alchemist). Backgrounds are in the [cast registry](cast_registry.md).

## 4. The doctrine split (Act 2 on)

From Act 2 the cell splits over one question: graft the titan's flesh, or refuse it. Doctrine is a campaign meter
from Preservationist (Bram's voice) to Flesh-Broker (Aline's voice) `[kit hook]` campaign meter with thresholds,
`[data]` names; it gates the endings (CANON.md section 4). Its inputs and thresholds are open (CANON.md section 6).

```text
                       [ THE SEVERED ROOT ]
                              /  \
                             /    \
         [ THE PRESERVATIONISTS ]  [ THE FLESH-BROKERS ]
         - Cold Steel              - Inoculated grafts
         - human will              - adaptation
         - demolition              - armour-piercing eldritch bursts
         - no grafts               - Strain and Calcification
```

### 4.1 The Preservationists (Cold Steel loyalists)

- **Belief:** led by smiths and old line soldiers (Bram "Iron-Lung" Karr). Grafting aberration flesh is surrender: it
  turns rebels into the monsters they fight.
- **Methods:** discipline, leverage, counterweights, armour plate, ambush and demolition charges.
- **What it means in play:** Cold Steel skills add no Strain `[data]`. A unit still gains Strain from hazards and
  eldritch attacks, so a Rupture (and with it a Flesh Breach) remains possible but rare; a unit with no Tier III
  grafts advances in Calcification only through a Flesh Breach (CANON.md sections 2.2-2.3).

### 4.2 The Flesh-Brokers (resonant flesh-crafters)

- **Belief:** led by heretic apothecaries (Sister Aline, a non-combat surgeon) and the desperate fighters who follow
  her. Meeting cosmic horrors and armoured inquisitors with plain iron is romantic suicide.
- **Methods:** harvest glands, marrow, nerve clusters and ichor from slain aberrations and officers (Aberrant Organs,
  a stash material `[data]`) and graft them into operatives at the Apothecary's Slab: speed, phase steps, acid bile.
- **Cost in play:** each Inoculated skill adds its `strain_cost` when the action starts `[kit hook]` per-unit meter,
  `[data]` costs; at 100 a crisis is rolled at round end, and a Flesh Breach or Tier III graft surgery advances
  Calcification one stage toward Stage V, Terminal Drift (the unit is lost). Ending a battle at high Strain does not
  by itself calcify (CANON.md section 2.3).

## 5. Gaps in this file

- Flash-bomb effect without accuracy rolls (section 2.2): Open, see gaps.
- Red Tithing size and frequency (section 2.1): Open, see gaps.
- Doctrine inputs and thresholds (section 4): open in CANON.md section 6.
