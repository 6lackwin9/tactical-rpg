# Campaign narrative structure: the three acts

Status: canon for the prologue and Act 1; proposal for Acts 2 and 3 (consistency pass 2026-10-25). This doc follows
[CANON.md](../CANON.md); where they differ, CANON wins. Names follow the [cast registry](../background/cast_registry.md).
Related: [choices and dilemmas](narrative_choices_and_dilemmas.md), [agency and fail states](narrative_agency_and_fail_states.md),
[roster lifecycle](../systems/roster_expansion_and_lifecycle.md).

Layer tags: `[engine]` technology any game needs; `[kit hook]` a shape any story in the genre could fill; `[data]`
content that fills a kit hook; `[game code]` a mechanic only this story uses.

## 1. The arc in brief

- Act 1 is Cold Steel only. The player learns human limits (weapon weight, armour, AP, poise breaks) before any eldritch
  power, so Inoculation in Act 2 arrives as a cost the cell accepts, not as a new skill tree.
- The early roster is authored: six named founders with personal hooks, no procedural recruits.
- The Crucible's scripted loss turns the argument between Bram (Preservationist) and Aline (Flesh-Broker) into the
  cell's question for Act 2: what it will spend to keep fighting. The squad's outlook moves from "we will survive this"
  to "the mission continues, whatever it costs".

## 2. The acts at a glance

| Aspect | Act 1 The Young Vanguard | Act 2 The First Rupture | Act 3 The Awakening Abyss |
| --- | --- | --- | --- |
| Region | The Sunk Ward | The High Bastion | The Mantle Fissure |
| Status | canon | proposal | proposal |
| Length target | ~4-5 h | ~7-9 h | ~5-6 h |
| Battles | 9 main (0a-0c, M1-M5, the interlude) plus optional board work | ~14-18 | ~10-12 |
| Rules on | Cold Steel; downed founders end battles wounded | Inoculation, Strain, Calcification, grafts, recruits, permadeath and capture, sieges | Doom Clock turns hard; bell resonance and Act 3 hazards |
| Roster | six founders (five in the first playable slice) | five founders plus recruits aged 18-24 | veterans across Calcification stages |
| Tone | gritty, cautious | desperate, transhuman | apocalyptic |

## 3. Prologue (canon)

Three scripted missions outside the district graph `[data]` (encounters ENC-PRO-0A..0C), chained by a `[kit hook]`
scripted mission sequence with join events. Detail: [recruits and prologue](recruits_and_prologue_architecture.md);
dossiers and the hearth scene: [prologue and starter squad](prologue_and_starter_squad_lore.md); cold open:
[onboarding](new_game_onboarding_and_intro.md).

| # | Mission | Who joins |
| --- | --- | --- |
| 0a | The Alley (Alleyway of Cinders, Seventh Ward) | Kaelen, alone |
| 0b | The Rooftop (Meat Market rooftops) | Vesper |
| 0c | The Bridge (canal bridge, iron mortuary cart) | Maren and Hollis, freed from the cart |

The cell dives through the Drowned Runoff Grate (node 01) into the Sluice Undercroft: the hearth scene with Bram and
Aline, six bunks with four filled, the Doom Clock appears at 42 days, the Act 1 title card.

## 4. Act 1 "The Young Vanguard" (canon)

Theme: desperate idealists in the sewers who believe steel, stealth and coordination can break the Inquisition.

- Cold Steel only: grafts, Strain and the Inoculated branches are locked `[kit hook]` (ruleset switches per act). The
  player learns AP budgeting, windups, Active Guard, poise breaks and weapon reach. Stances are not part of the game
  (DEC-LORE-11).
- The founders' visible taints (Kaelen's spurs, Hollis's eye, Maren's veins, Vesper's smoke) are backstory and Act 2
  hooks: 0% corruption and no countdown on anyone in Act 1.
- A founder at 0 HP is downed; after the battle a downed founder is wounded `[kit hook]` (unit lifecycle). No death and
  no capture in Act 1 except the Crucible. A wounded founder deploys at -20% max HP until a rest (user 2026-10-25,
  Q-31; a baseline the balance lane revisits). Act 1 holds only the six founders (Q-38).
- Missions come from Tess's dead-drop board `[kit hook]` (mission board: a hub NPC offering quests; today quests plus
  the dialogue effect `start_quest`) `[data]`, and from node travel on the district map (`worldmap.rs`, `travel.rs`).
  The board offers the next main mission when the previous one is done.
- Doom Clock `[kit hook]` campaign meter: visible from day 1 at 42 days, soft (story pressure, no fail; at 0 a lesser
  tithing, then it restarts). Inquisition Alert runs 0-100% with no siege in Act 1; an Act 1 Breach ends the run on
  the Extinguished screen, reachable only by blatantly ignoring the meter (small inputs, a warning at each threshold;
  user 2026-10-25, Q-13).
- Upkeep `[kit hook]` resource purse: 1 Warded Tallow per day of travel or rest for the whole cell.

### 4.1 Main missions

| # | Mission | Node | Encounter `[data]` | Joins / loses |
| --- | --- | --- | --- | --- |
| M1 | The Sweep | 02 Alleyway Junction 7 | ENC-EVT-01, Captain Vane | - |
| M2 | The Crossroads Cages | 03 Ash-Crawl Crossroads | ENC-EVT-02, Proctor Quell; retorts for the Slab (Maren's hook) | - |
| M3 | The Bone-Mill Raid | 04 Fourth Kiln | ENC-EVT-03, Overseer Grendel | Dunstan joins (bunk 5) |
| M4 | The Tallow Vats | 05 Tallow Vats | ENC-EVT-04, Guild-Master Orlov | Elyse joins (bunk 6) |
| Interlude | The Awakening Fissure | 00 Sluice Undercroft | ENC-EVT-05, the Calcified Amalgam; Aline's marrow dose | - |
| M5 | The Crucible | 06 Crucible Watchpost to 07 Slag Gate | ENC-EVT-06, Inquisitor-Prelate Nemeth | one founder captured |

### 4.2 Founder hooks (optional board work unless noted)

| Founder | Hook | Reward |
| --- | --- | --- |
| Kaelen | Raid a watchpost for forge bellows | Bellows for the Tinker's Hearth (effect: Open) |
| Maren | M2 (main mission): the crossroads cages | Embalming retorts for the Apothecary's Slab |
| Vesper | Map the canal drainage locks | Route shortcuts on the district map |
| Hollis | Secure vantage points over the Crucible | Open: what the vantage points change in M5 |
| Dunstan | Open | Open |
| Elyse | Open | Open |

Other Act 1 board work: Tess's "Night of the Red Pen" (rescue Pip), Agnes's "Rafters of Tenement 12", Corvo's gate,
Brother Rook on street nodes.

### 4.3 The climax: the Crucible (M5)

- The cell assaults the Crucible Watchpost (node 06) to break through the Slag Gate (node 07), the cordon between the
  Sunk Ward and the High Bastion.
- Inquisitor-Prelate Nemeth, infused with prime aberrant marrow, is a foe that cold steel cannot finish. The cell only
  escapes him, and he escapes with the captured founder (user 2026-10-25, Q-25 a; his arc: CANON 3.6).
- The Slag Gate's portcullis winch is jammed. One founder must stay on the winch tile under fire so the other three
  escape. The player picks who stays from the four deployed; the Crucible requires four, so deployment is the decision.
- Nemeth takes that founder alive `[kit hook]` (unit lifecycle: scripted capture). The survivors fall back to the
  Sluice, outmatched. Per-founder beats and keepsakes: [choices](narrative_choices_and_dilemmas.md) Choice 1.
- Encounter, winch tile and escape objective: `[data]` over the `[kit hook]` objectives (escape; hold).

## 5. Act 2 "The First Rupture" (proposal)

Theme: grief turns into pragmatism; cold steel alone cannot pierce what is coming.

- Inoculation: Aline opens the Apothecary's Slab for graft surgery with Aberrant Organs harvested from aberrations
  `[kit hook]` (exchange recipes) `[data]`.
- Strain (in battle, 0-100) and Calcification (out of battle, stages I-V) switch on `[kit hook]` (per-unit meter, stage
  track); numbers in CANON 2.2-2.3. Players learn to keep Strain in a useful bracket without crossing into a crisis.
- Recruits: Unsealed youths aged 18-24 freed from culling wagons and stockades (ENC-ACT2-02, repeatable), offered on
  Tess's board `[kit hook]` (RECRUIT1, dialogue effect `recruit`) `[data]` (the recruit pool). The first takes the bunk
  the Crucible emptied, at the start of Act 2 (Q-38).
- Permadeath and capture switch on `[kit hook]`. Which downed operatives die, are captured or are wounded: Open.
- The captured founder's keepsake: an ACC heirloom any operative can equip `[data]`; numbers Open.
- Doctrine `[kit hook]` campaign meter from Preservationist (Bram) to Flesh-Broker (Aline); inputs Open.
- Sieges: Alert at 100% (Breach) starts a siege of the hub; a lost siege costs facilities and forces a relocation,
  never a Game Over.
- Eight bunks through Bram's "Expanded Timber Frames" project `[kit hook]` (hub facility levels); a Scrap cost from
  the balance lane (Q-38).
- Enemy rosters add witch-hunters with flash-bombs, shielded castellans with heavy knockback and mist-aberrations whose
  attacks add Strain `[data]`.
- Main beats (CANON 3.4): Ruthven's checkpoint (node 09), the culling stockades (12), the Tallow Reservoir (10), the
  Embalming Cloisters (11; Chief Anatomist Lazar, Aline's quest: Apex grafting; Lazar's records give the Great
  Tithing's date and scale and show the cell's raids caused it, and the clock is relabelled "the Great Tithing"; the
  cell sees Nemeth deliver the captured founder to Lazar and arrives too late; CANON 3.6), the Grand Belltower (13),
  and the act climax at the Reliquary Crypts to the Grand Spire Gate (14-15; the tomb and record of the first
  Extinguishers, the Clerisy's founders; the Key of the Mantle Chasm). The old "Reliquary heist in St. Lyra's crypts"
  is split between nodes 11 and 14.

## 6. Act 3 "The Awakening Abyss" (proposal)

Theme: the bells of St. Lyra's toll without pause; the titan shifts in its sleep; the Inquisition gathers for the
Great Tithing.

- The Doom Clock turns hard: if it reaches 0 before node 21 falls, the Great Tithing completes and the campaign shows
  the Extinguished failure screen. Route legs and rests cost days. What the Tithing does:
  [central truth](../background/central_truth.md) (Option A, canon; CANON 3.6).
- Alert and sieges continue as in Act 2.
- Veterans reach Calcification stages III and IV (fear immunity and natural armour, but no two-handed weapons at IV);
  the player rotates mutated shock troops with clean Cold Steel anchors.
- Bell resonance and Act 3 hazards switch on; detail Open (see [six founders](six_founders_and_phase_progression.md) 5).
- The corrupted founder (optional, node 20 Silent Ossuary; CANON 3.6): the captured founder, made a saint-weapon by
  Lazar's anatomists, fights at Nemeth's side; fighting their old teammates breaks them, and they go insane and kill
  everyone in reach, Nemeth and his guard included; the cell fights the monstrosity, grants mercy and gains the
  Transcendent Relic, which upgrades the keepsake.
- Main path 16-17-18-21-22: the Weeping Basalt Chasm, the Marrow Conduits (High Castellan Ulric; cutting them adds 2
  days to the clock), the High Altar (Arch-Prelate Othemar falls; the clock stops), the Dreaming Core (the endings,
  gated by doctrine; see [choices](narrative_choices_and_dilemmas.md) Choice 5).

## 7. Scope targets (proposal)

| Item | Target |
| --- | --- |
| Playtime | 16-20 h |
| Battles | 32-40 |
| Bunks | 6; 8 through the Act 2 project |
| Replay | a different sacrificed founder, different recruit pools, a Cold Steel-only run |

## 8. Gaps (Open)

- The outcome of a lost Act 1 battle (all four downed). Wounded effects: decided (Q-31).
- Act 1 Alert at 100%: decided (the run ends, Q-13); the warning steps and the Breach story text stay open.
- Founder hooks for Dunstan and Elyse; the effect of Kaelen's bellows and Hollis's vantage points.
- Nemeth at the Crucible: decided (the cell escapes, Q-25; his arc: CANON 3.6).
- The Act 2 rule for which downed operatives die, are captured or are wounded.
- Keepsake numbers; doctrine inputs; the 8-bunk project cost (Scrap, balance lane); where the cell relocates after a
  lost siege.
- The mission list of the first playable slice: decided (Q-84: 0a-0c, the hearth, M3, one dilemma, M5), from the
  prologue at level 1 (Q-41).
