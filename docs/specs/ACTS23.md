# ACTS23: the systems Acts 2 and 3 need beyond the Act 1 slice

Status: **answered** (user 2026-11-10: every recommendation in section 5 accepted except **Q-A9**, reopened as the Channel rework in section 5.1 and decided the same day: **Option B, the fuel row**, with derived marrow versions plus overrides and all-marrow Channel rounds). Nothing is queued yet: each work package (section 4) gets a `.1` design session in `docs/sessions/`, then a Build `feasibility-check`, as `MASTERPLAN1` did.
Depends on: R1-R4 (shipped); the Act 1 slice data in `games/emberwake/data/worlds/sunk_ward/`
Layers: kit (most hooks), game (data, screens), engine (only the 2 x 2 unit footprint, if chosen)
Sources: `games/emberwake/docs/lore/CANON.md` sections 2-4 (Acts 2-3 are `proposal` there), `lore/specs/full_game_systems_roadmap.md` sections 4-6 (`P2-01..P3-05`), `lore/background/{upper_bastion,mantle_fissure}_campaign_map.md`, `lore/narrative/campaign_narrative_structure.md` sections 5-8, `lore/questions/open_world_and_campaign.md` (Q-09..Q-46, mostly decided 2026-10-25). Code status: a read-only audit of `develop @4093aed3` (2026-11-10), file:line evidence kept in section 2.

## Goal

List every system Acts 2 ("The First Rupture", the High Bastion, nodes 08-15) and 3 ("The Awakening Abyss", the Mantle Fissure, nodes 16-22) switch on, check each against what the engine and kit already carry, and turn the gaps into a small number of work packages with an order and the questions only the user can answer.

**Verdict.** The kit is further along than the lore docs assume: the per-unit battle meter, campaign gauges and countdowns, checkpoints, act switching, roster departures with per-act rules, stay-behind retreats, bunks, encounter phases with a `kill` effect, scheduled arena hazards and `fail` / `bad_end` cards all exist (built as M0 hooks before `MASTERPLAN1`). The real gaps are **8 new kit hooks** (persistent per-unit stage track, graft slots, a seeded random-table effect, bleed-out and revive, a procedural recruit generator, facilities with levels, sieges as connected rooms, and endings), **4 small battle extensions** (Channel's behaviour, a side switch, a unit swap, meter deltas from hazards) and **one engine-or-kit choice** (2 x 2 units). Everything else is data: two act regions, about 12 main and 10+ side encounters, the Act 2-3 bestiary, Strain and Calcification rows, doctrine numbers, loyalty missions and endings text.

## 1. What Act 1 already switches on (reused, not rebuilt)

Timeline ruleset, round intents (Assault, Guard, Support, Maneuver, Field), Poise pips, statuses, the `sunk_ward` world (hub `sluice_undercroft`, nodes 01, 03, 04, 06), Scrap and Warded Tallow purse with daily upkeep and the 0-Tallow `-2` start AP, the Alert gauge (Act 1 Breach ends the run on `extinguished_breach`; `floors.from_act 2 = 30` already authored) and the soft Doom Clock (`d_zero` restarts at 14 days with `lesser_tithings` in acts 1-2), downed then wounded (`-20%` max HP until a rest), the Crucible stay-behind capture, bunks (6), and `world_stage = campaign.act` scaling foe levels.

Act 1 remainder (outside the slice: M1 The Sweep, M2 The Crossroads Cages, M4 The Tallow Vats with Elyse, the ENC-EVT-05 interlude, the optional board work, Act 1 checkpoints) is data on these hooks and is not in this spec, except where Act 2 reads it (Q-A3).

## 2. System inventory: Acts 2-3 against the code

Status: **E** exists and is usable as-is, **P** partial, **N** none. "Data" means an existing hook only needs rows.

| # | System (canon) | Act | Status | What exists / what is missing | Package |
| --- | --- | --- | --- | --- | --- |
| 1 | Strain: per-unit battle meter 0-100, brackets, gain on Inoculated action start, decay, floor, reset after battle (CANON 2.2) | 2 | **E**, data missing | `rpgkit_battle/src/meter.rs` (`battle_start` :131, `pay` :359, `round_end` decay with `unless_gained` / `if_guarded` :410, `bands`, `rules`); floor from a passive `meter` grant (`rpgkit_rules/src/grants.rs:144`). `unit_meters.json` has no `strain` row | PK1 (data) |
| 2 | Crisis at 100: seeded 1d3 table at round end (CANON 2.2) | 2 | **N** | Meter `rules` fire effects on a crossing, but no effect picks 1 of N (`rpgkit_data/src/lib.rs:1840`) | PK1 |
| 3 | Inoculated skills and the `Channel` intent | 2 | **P** | Meter-cost skills (`skills.meters`, `unit_meters.skills` without a deck slot) and armour `pierce` exist; `RoundIntent::Channel` exists but is inert (accepts any target, `round_intent.rs:243`) and the game ruleset does not list it | PK1 |
| 4 | Aberrant enemies carry Strain; hazards and eldritch attacks add Strain | 2-3 | **P** | Enemies can hold meters (`unit_pools`); no `meter` delta in arena / ground hazard effects | PK1 |
| 5 | Calcification: permanent per-unit stage track I-V, advanced by Flesh Breach and Tier III surgery; V = lost (CANON 2.3) | 2 | **N** | Only `MEMBER_CONDITIONS = ["wounded"]` (`lib.rs:1612`); unit-scope campaign meters refused as "CAMPAIGN1-DOC (not built yet)" (`check/campaign.rs:288`) | PK2 |
| 6 | Grafts: 4 body slots (Cranial, Thoracic, Forearms, Visceral) raising the Strain floor; excision; bone-scraping | 2 | **N** | `LOADOUT_SLOTS` has the 6 gear slots only (`lib.rs:26`); worn gear may not grant a meter (`grants.rs:153`) | PK2 |
| 7 | Exchange recipes at facilities (Slab flush, surgery, bone-scraping); Rock Salt and Aberrant Organs | 2 | **N** | Only `material`-family stash items (for example `inquisitor_marrow_valve`); no recipe service | PK2 |
| 8 | Downed units bleed out after N rounds unless revived (Q-32; revive is a key on heal skills and items, ROSTER1 RQ) | 2 | **P** | `roster.rules.bleed_out_rounds` is parsed and checked (`lib.rs:1350`, `check/world.rs:802`) but no battle code reads it; no revive | PK3 |
| 9 | Permadeath and capture from Act 2; leave-a-unit-behind | 2 | **E** | Departures captured / dead / retired / dismissed / lost and per-act `roster.rules` (`rpgkit_world/src/roster.rs:292`, `:389`); `stay_behind` hold / stand then none / capture / kill (`encounter.rs:151-160`). `sunk_ward` authors no `roster` block yet | PK3 (data) |
| 10 | Procedural recruits 18-24 (rolled stats, quirk, class from the encounter); first recruit at the act line (Q-93) | 2 | **P** | Bunks (`bunk_cap`, `bunks_flag`, `roster.rs:437-466`) and the `recruit` dialogue effect for authored characters; `roster.pool` is 0 "until ROSTER1-P" (`roster.rs:523`), no generator | PK3 |
| 11 | Keepsakes from captured or fallen units, named, procedural ones too (Q-34) | 2 | **P** | Authored `keepsake` item goes to the stash on departure (`roster.rs:409-415`); `keepsakes.by_class` / `name` parsed but unused (`lib.rs:1404`) | PK3 |
| 12 | Living Martyrs: fixed counter per deployment and per N days in stasis, retire or last fight at Stage V (Q-37, Q-37a); Isolation Crypts housing (Q-36) | 2-3 | **N** | Crypt places named only in comments (`roster.rs:544`) | PK3 (on PK2's track) |
| 13 | Doctrine: cell gauge Preservationist to Flesh-Broker plus a per-character lean, moved by play and choices, gating lines and endings (Q-19, Q-19a) | 2 | **N** (cell gauge is data) | A campaign gauge with `meter_add` from dialogue can carry the cell value today; the per-character lean needs unit-scope meters (the PK2 track) and play inputs (grafts, high-Strain battles, forge projects, flushes) need triggers on those events | PK4 |
| 14 | Checkpoint missions with deadlines; Act 2 bad ends go on worse (Q-20a) | 2-3 | **E**, 0 authored | `campaign/checkpoint.rs:102` (`warn_days`, `end_run` -> `bad_end`, `go_on` effects); `sunk_ward/checkpoints.json` is empty | PK4 (data) |
| 15 | Great Tithing reveal at 2.4: relabel the clock, correct it once, lock some side quests (Q-16, Q-17) | 2 | **P** | `meter_set` and flags exist; no relabel of a meter's display name | PK4 |
| 16 | Hard Doom Clock in Act 3 (Extinguished if 0 before node 21); node 18 adds 2 days; node 21 stops it | 3 | **E**, data | Countdown rules with `acts` and the `end` effect exist; Act 3 rows not authored | PK4 (data) |
| 17 | Hub facilities with a level tree each from Act 2 (Q-42), screens from the keeper NPC (Q-46), Bram's 8-bunk project (Q-38) | 2 | **N** | NPC services are turn_in, offer, board, shop, recruit (`explore.rs:315`); the bunk count is an int flag only | PK5 |
| 18 | Sieges: Alert Breach starts a siege fought in the hub as connected rooms (floors to climb, waves, rooms to clear); a lost siege costs facility levels and relocates the cell (Q-22, Q-23) | 2-3 | **N** | No multi-room battle; Breach has only the Act 1 `end` | PK5 |
| 19 | Encounter phases for bosses (Nemeth at node 20: `hp_below` break, kill Nemeth and his guard, founder turns on everyone, unit swap to the monstrosity) | 3 | **P** | Triggers `hp_below` + `or_round`, `round`, `arrival`, `downed`, `broken`; effects line, wave, status, ai, enrage, deck, shake, talk, `kill` (`encounter.rs:310`, `:396-446`); no side switch, no unit swap | PK6 |
| 20 | Large units on 2 x 2 cells (the Amalgam, the titan, the node 20 monstrosity; Q-79) | 1-3 | **N** | One cell per unit (`grid.positions`); footprints exist for props only | PK6 |
| 21 | Bell resonance: Calcified operatives start battles at +X Strain (Q-10); an in-battle pulse hurts units at 50+ Strain; Act 2 in Upper Cinderreach, silenced at 2.5, a continental wave in Act 3 (Q-09) | 2-3 | **P** | Scheduled `arena_hazards` (`arena.rs`) can toll on rounds; no meter condition or meter delta; battle-start Strain from campaign state needs PK2's track | PK1 + PK6 |
| 22 | Act 3 Tallow scarcity: Tallow costs more Scrap (Q-45); Stage IV sleep in the crypts | 3 | **P** | Shop prices exist; no per-act price change | PK4 (data or a small price rule) |
| 23 | Final battle retry inside the dream: each attempt costs Act 3 days, the fallen return as dream-echoes (Q-21, Q-21a) | 3 | **P** | Story fight `on_lost: "retry"` rests and repeats (`story.rs:440`); no day cost, no echoes | PK6 |
| 24 | Endings A Preservation / B Ascendancy / C Red Martyrdom gated by doctrine; the Extinguished screen | 3 | **P** | `end` kinds are `fail` and `bad_end` (`lib.rs:1065`), run-over screen `emberwake_gui/src/runover.rs`, Extinguished cards wired; no ending kind, no choice by doctrine or flags | PK7 |
| 25 | Act 2 and Act 3 maps, missions, bestiary, bosses, recruits pool, keepsake names, loyalty missions per surviving founder (G-16) | 2-3 | **N** (docs only) | Only `sunk_ward` (Act 1) plus the sample worlds `ashlands`, `iron_reaches`, `tidewrack` | PK8 (data) |

## 3. Already decided (not reopened)

From `open_world_and_campaign.md`, user 2026-10-25: Alert carries over with a floor of 30 (Q-14); Alert inputs are mission tags plus bribes (Q-15); lesser Tithings at 0 in Acts 1-2 (Q-16); the found date corrects the clock once and becomes the Act 3 budget (Q-17); Act 1 bad end = reload, Act 2 on = go on worse (Q-20a); dream retry with echoes (Q-21a); sieges in the hub as connected rooms (Q-22); forward bases are story only (Q-23); the captured founder is never rescued (Q-33); keepsakes about 50% of a trait, named stronger (Q-34); crypt place keeps the bunk, Martyrs take a crypt place (Q-36); Martyr counter ticks in stasis (Q-37a); 8-bunk project Act 2 on (Q-38); recruit class set by the encounter, temperament passive plus default intent (Q-40, Q-40a); facility level trees from Act 2 (Q-42); the Rock Salt flush lowers the floor by 10 (Q-43); per-graft floor step +10 by default (Q-44); Act 3 Tallow costs more Scrap (Q-45); facility screens from the keeper (Q-46); first recruit joins at the act line (Q-93); only the Crucible's founder fights at node 20 (Q-94).

## 4. Work packages (proposed order)

Each package extends an existing `MASTERPLAN1` block rather than adding a block; the kit stays story-free (names such as Strain and Calcification live in game data).

| Package | Extends | Scope | New kit hooks | Data (game) |
| --- | --- | --- | --- | --- |
| **PK1 Corruption in battle** | `S2` pools, `A1` costs, `F` intents | Strain row, crisis, Inoculated skills, Channel, meter deltas from hazards and attacks | (a) a seeded **random-table effect** (`roll_table`: weighted rows of effects on the battle RNG stream), usable by meter rules and triggers; (b) **Channel as a fuel row** (section 5.1 Option B: a version row targeted as the skill's primary intent, all-marrow rounds, derived marrow package with `channel` overrides, live only for Strain carriers, behind a ruleset flag); (c) a **`meter` effect** on arena and ground hazards and on skill hits | `strain` in `unit_meters.json` with 5 bands, crisis rows, Inoculated skill costs per class, aberrant enemy meters |
| **PK2 Calcification and grafts** | `E3` roster, `U3` gear, `U4` grants | Persistent per-unit stage track, stage passives, graft slots, recipes | (d) **unit-scope campaign track** (the refused CAMPAIGN1-DOC: per-member value, stages with passives, advance and set effects, a "lost at stage N" departure); (e) a **second slot set** (`body_slots`, kit-generic) whose items may grant a meter floor; (f) **recipes**, an NPC service turning purse and stash inputs into effects on a member (surgery, flush, excision, bone-scraping) | Calcification stages I-V, 4 graft slots, organ and salt items, the recipe rows at the Slab |
| **PK3 Roster lifecycle** | `E3` roster, `F` fight flow | Bleed-out, revive, recruits, procedural keepsakes, Martyrs, crypts | (g) **bleed-out and revive** in battle (read `bleed_out_rounds`; a `revive` key on heal skills and items); (h) **recruit generator** (ROSTER1-P: a pool with rolled stats inside a role template, a quirk list, class from the encounter, ages 18-24 as flavour); procedural keepsakes from the fields already parsed; Martyr counter on PK2's track | `sunk_ward` `roster` rules per act, recruit pools and quirks, keepsake names |
| **PK4 Campaign pressure** | `W3` clock, `T1` story | Doctrine, checkpoints, the Great Tithing reveal, Act 3 clock and prices | (i) **meter display relabel** effect; doctrine **play inputs** as triggers on surgery, flush, forge project and battle-end Strain; per-character lean on PK2's track; a per-act price rule if data cannot express it | Doctrine gauge and thresholds, Act 2-3 checkpoints, hard-clock rows, side-quest locks, the saved-youths gauge |
| **PK5 Hub and sieges** | `W1` services, `E2` purse, `T2` sequences | Facilities with levels and projects, facility screens, sieges | (j) **facilities**: a catalog kind (id, keeper NPC, levels with costs and unlock effects) and a `facility` service and screen; (k) **siege**: a Breach trigger starting a chain of room encounters in the hub with carried HP and downed state, room outcomes that remove facility levels, relocation as story | Facilities (Hearth, Slab, Board, Bunks, Crypts) with levels, the siege rooms and waves per act |
| **PK6 Boss and Act 3 tech** | `F` encounters, `P` space, `T2` story fights | Side switch, unit swap, 2 x 2 units, dream retry, resonance | (l) **side switch** effect (`hostile_all` or a named faction) and (m) **unit swap** effect (replace a unit by another row, keeping position); (n) **2 x 2 footprint** (Q-A6); (o) story fight retry **day cost** and **echo** spawns of the fallen; resonance as a hazard with a meter condition | Node 20, the titan, the Amalgam, resonance numbers |
| **PK7 Endings** | `T2` sequences, X1 screens | Endings A, B, C, the ending screen | (p) an **`ending` end kind** chosen by gauge and flag conditions, with a closing sequence and a credits / epilogue screen beside the run-over screen | Ending sequences and text, Bram's and Aline's fates (G-17), the saved-youths lines |
| **PK8 Act 2 and Act 3 content** | data on PK1-PK7 | Maps, missions, encounters, bestiary, bosses, loyalty missions | none (any gap goes back to a PK) | Two act regions, 6 + 6 main missions, wagons, loyalty missions, enemies, items, dialogue |

Proposed releases (names provisional): **R5** = PK1 + PK2 + PK3 (the corruption and roster layer, testable in the Dojo and on Act 1 data with the switches on); **R6** = PK4 + PK5 (campaign and hub); **R7** = PK6 + PK7; PK8 content waves follow each release (Act 2 after R6, Act 3 after R7). `WEBREL1` and the game UI polish run alongside.

## 5. Questions for the user (each with a recommendation)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| Q-A1 | Lock the Acts 2-3 outline (CANON 3.4, 3.5, sections 4 and 6 of the systems roadmap) as canon so packages can be designed against it? | (a) lock as is; (b) lock the systems, keep the mission list open; (c) review the story first | **(b)**: the systems are well settled by Q-09..Q-46; missions can still move in the story track |
| Q-A2 | World layout for Acts 2-3 | (a) one campaign world (`sunk_ward` grows act-gated regions for the High Bastion and the Mantle Fissure; hub, meters and roster carry over for free); (b) one world per act with a hand-over | **(a)**: CANON keeps one hub (Q-23) and carries Alert, the clock and the roster across acts |
| Q-A3 | Order against the rest of Act 1 | (a) finish the Act 1 content (M2b) first, then R5; (b) R5 kit work in parallel with Act 1 data | **(b)**: different lanes; Act 1 data also gives R5 a test bed |
| Q-A4 | Crisis as a kit hook or game code | (a) kit `roll_table` effect (any game can use random outcome tables); (b) game-only code | **(a)** |
| Q-A5 | Doctrine's shape | (a) one cell gauge (-100 Preservationist to +100 Flesh-Broker) plus a per-character lean on the PK2 unit track; (b) cell gauge only | **(a)**, as Q-19a decided |
| Q-A6 | 2 x 2 units | (a) a real footprint in the battle grid (pathing, reach, forced moves, AoE count 4 cells); (b) a big sprite on 1 cell | **(a)**: the Amalgam (Act 1 interlude), the titan and the monstrosity need to block and be flanked as big bodies |
| Q-A7 | Siege form | (a) a chain of room encounters (one map per room, HP and downed carry over, a room lost costs its facility level); (b) one large connected map | **(a)**: reuses story fight sequences and the region map; (b) needs multi-map battles in the engine |
| Q-A8 | Facilities and levels | (a) 5 facilities (Hearth, Slab, Board, Bunks, Crypts) x 3 levels; (b) fewer, deeper trees | **(a)**, numbers by the balance lane |
| Q-A9 | What Channel is | (a) the intent of Inoculated and aberrant skills (a skill's intent stays single: Channel skills cost Strain); (b) a stance that turns any skill Inoculated | **(a)**: keeps "a skill has one intent" (CANON 2.1) |
| Q-A10 | Scope target | (a) keep 16-20 h, 32-40 battles (campaign structure 7); (b) a shorter first release of Act 2 only | **(a)** for the plan; content waves can still ship Act 2 first |

Answers (user 2026-11-10): Q-A1 (b), Q-A2 (a), Q-A3 (b), Q-A4 (a), Q-A5 (a), Q-A6 (a), Q-A7 (a), Q-A8 (a), Q-A10 (a); **Q-A9 reopened**: "now we can have a skill with two intents, so see what Channel contributes and whether it can be reworked" (section 5.1).

### 5.1 Channel rework (Q-A9 reopened, analysis for the user)

**What Channel contributes today: nothing that runs.** It is an enum slot (`rpgkit_battle/src/round_intent.rs:25`), a term (`intent_channel`, "Channel"), a tip label "FOCUS" (`emberwake_gui/src/battle/tips.rs:31`) and a name in the ruleset's intent list (`rpgkit_rules/src/ruleset.rs:115`). The planner refuses Channel rounds ("Channel rounds are not planned yet", `round_intent_plan.rs:497`), `target_problem` accepts any target (`round_intent.rs:243`), `IR21`'s `intent::derive` never returns it, and the game ruleset does not list it. On paper: CANON 2.1 "Channel from Act 2"; F3.1 "hidden Channel when a unit owns a `grant_pool` meter"; INTENT2 IR13: an intent names the round's target (IR1, IR2) and Channel names none, so it fails the test ("most likely a Stand round with a meter rule").

**What a round intent gives that a plain skill tag does not:** (1) a round commitment: follow-ups come from the same intent (FU2); (2) the intent tell: enemies' plans and threat intel show it, so a round is readable at a glance; (3) intent-bound buffs; (4) per-intent deck order and AI rows; (5) round-end rules can read it.

**What the Act 2 Inoculation needs** (CANON 2.2, `magic_and_corruption.md`, `classes_and_skill_trees.md`): a Strain cost on use (exists: `skills.meters`), decay that skips a round of marrow use (exists: `unless_gained`), armour pierce (exists: `pierce`), and two things that are missing: **a readable sign that a unit is burning marrow this round**, and **a real choice between Cold Steel and marrow** (today a skill is one or the other, so the Inoculated branch is a second skill list to author).

**Dual intents change the question.** Since `IR21` a skill shows in up to two rows and the row it is picked from sets its version (Field vs Assault). So Channel no longer has to be a sixth target kind or a separate skill list: it can be a **version row**.

| Option | What Channel becomes | Gains | Costs |
| --- | --- | --- | --- |
| **A** Retire | Removed; "Inoculated" is only a skill tag (a meter cost) usable in any intent, shown by a marrow mark on the plan badge | Simplest; no new rule | No round commitment, no tell beyond a badge, Inoculated skills stay a separate list |
| **B** Fuel row (**recommended**) | A row that never names a target: a skill with a marrow version also shows in the Channel row, anchored and targeted as its primary intent (IR2 unchanged). Picked from that row, the round is a **Channel round**: every follow-up must come from the Channel row, each pays its `strain_cost` at action start, and the marrow package applies (armour pierce, a power bonus, any authored extra effect). The row is live only for a unit that carries the `strain` meter (grafted, Act 2 on), as F3.1 already says | One catalogue serves both branches: a Cold Steel skill and its marrow version are one row each, so the Act 2 skill pass is mostly versions, not new skills; "this round I burn marrow" is one fact on the plan strip and in enemy tells (aberrants telegraph Strain attacks the same way); the Cold Steel vs Inoculation choice becomes per round, which is the risk-and-reward loop the lore wants; decay already reads the round | A sixth row slot for Strain carriers; `IR21` gains a rule that the fuel row does not count toward the 2 target rows; planner, tells and AI learn Channel rounds; a skill field for the marrow version |
| **C** Stand charge (IR13's idea) | A self-target Stand round that builds Strain or a charge and empowers the next round | Small | Overlaps Stand and `brace`; does not give marrow versions or a per-round choice |
| **D** Round mode toggle | A Channel switch beside the intent, like the old stances | Flexible | Stances were cut (DEC-LORE-11); a second axis on every round |

**Decided (user 2026-11-10): Option B**, with sub-question 2 as recommended (derived package plus per-skill overrides) and sub-question 1 as recommended (a Channel round is all marrow); sub-questions 3-5 take their recommendations as defaults for the PK1 `.1` session.

Sub-questions under **B** (recommendations in bold):

1. Can a Channel round mix in Cold Steel follow-ups? **No**: a Channel round is all marrow, so the tell is true for the whole round (Rest and Move stay neutral).
2. How is a marrow version defined? **A derived default package** (armour pierce, power x 1.25, `strain_cost` from the skill's AP: 1 AP -> 15, 2 -> 25, 3+ -> 35, the CANON guideline) **plus optional per-skill overrides** (a `channel` block on the skill: `strain_cost`, extra effects through `EffectSpec`, or `only: true` for marrow-only skills such as Calcified Spike), the same "derived, not authored per skill" rule as the Field bonuses.
3. Which skills get the row? **Every attack, support and Stand skill of a Strain carrier by default**, with `channel: {none: true}` to exclude one; balance can narrow it.
4. Intent-bound buffs in a Channel round: **only buffs bound to Channel** (none at first); the primary row's intent buffs do not apply.
5. Enemies: aberrants that carry Strain plan Channel rounds like players (AI row), and their tell shows Channel.

**Follow-up rulings (user 2026-11-10), replacing sub-question 3:**

- **Cap: only single-intent skills get a Channel row.** A skill with two intents (`IR21` dual, 26 of 129 today) has no marrow version, so no skill ever shows in more than two rows; marrow-only skills are authored with a single intent (a content check enforces both).
- **The Channel kit (user 2026-11-10, second follow-up; replaces the draft bullet above it):**
  1. **Grafts carry the marrow-exclusive skills**: each installed graft adds its exclusive skill to the Channel row, with no deck slot.
  2. **Two Channel basics**, free like the other basics: one **universal marrow basic** (a new basic skill every Strain carrier has), and the **class role basic's marrow version**. The other universal basics (Strike, Off-hand Jab, Active Guard) get no marrow version.
  3. **The 3 actives: the player selects which are marrow** in the loadout; only the selected ones (single-intent, by the cap) show in the Channel row.
  4. **What you can pick depends on the round's first pick** (the intent rule): open from the Channel row and the round is all marrow (graft skills, the two Channel basics, the selected actives); open from any other row and the round is Cold Steel.
- **The Channel row and how a Channel round opens (user 2026-11-10, third follow-up; refines items 1-4):**
  1. **The Channel row holds at most 5 skills**: up to **4 graft skills** (one per graft slot; an Inoculated keystone, installed by surgery, counts as one of them) plus the **universal marrow basic**. It never grows past 5, whatever the deck holds.
  2. **A Channel round opens only from the Channel row** (a graft skill or the marrow basic).
  3. **Once it is open, the marrow versions show in the other rows**: the round's follow-ups may also be the marrow versions of the actives selected as marrow (single-intent) and the class basic's marrow version, picked from their own rows (Assault, Stand, Support, Maneuver, Field), always on the round's one target (TG1: same target kind as the opener). Cold Steel versions stay out of a Channel round.
  4. Opened from any other row, the round is Cold Steel and no marrow version is offered.
- **The Channel opener (user 2026-11-10, fourth follow-up; replaces the separate Channel row of the third):**
  1. **Channel opener**: a neutral command beside Move, Rest and Fill, **0 AP and 0 blocks**, open to Strain carriers only. It must be the round's first pick; it names no target and turns the round into a **Channel round** (the plan strip and the enemy tell show Channel from then on).
  2. **No dedicated Channel row** (so no oversized bar): in a Channel round every row (Assault, Stand, Support, Maneuver, Field) offers only marrow versions: the graft skills (up to 4, marrow-exclusive, each shown in its own single intent's row), the universal marrow basic, the class basic's marrow version, the actives selected as marrow, and the marrow version of an Inoculated keystone. Cold Steel versions are greyed for that round.
  3. **The first skill after the opener fixes the round's one target** (TG1, confirmed by the user); the follow-ups stay on that target, as in any round.
  4. **Keystone**: it keeps its own deck slot and is not one of the 4 graft slots (the user left this open; with no Channel row the row size no longer matters, so the deck shape decides).
  5. A round that does not start with the opener is Cold Steel; enemies that carry Strain use the opener the same way (AI row).
- **Answers to the open points (user 2026-11-10, fifth follow-up):**
  1. **A marrow-enabled skill keeps its Cold Steel version**: entering a Channel round switches it from Cold Steel to marrow, leaving it switches back. The theme by intent count (user's correction: "1 in 10 / 2 in 10" meant one intent / two intents): **single-intent skills switch** between Cold Steel and marrow; **dual-intent skills are the Cold Steel heavies** with no marrow version (the cap above); the **dedicated marrow skills** (the graft skills) are usually stronger versions of regular skills.
  2. **How many actives may be marrow depends on the graft level** (more or higher-tier grafts allow more marrow actives); numbers to settle in the PK1 `.1` session.
  3. **No -1 AP**: a marrow version keeps the Cold Steel AP cost and gets enough power instead (balance tuning).
- **Still open for the PK1 `.1` session:** the marrow-active limit per graft level; the default package numbers (power x 1.25, armour pierce, Strain 15 / 25 / 35); whether keystones have marrow versions.

B lands in **PK1** (corruption in battle) as kit hook (b) in place of "Channel behaviour", behind a ruleset flag (off in `classic`, off in `timeline` until the Act 2 data turns it on), so no golden moves until data uses it; INTENT2 IR13 and CANON 2.1 ("a skill has one intent") get a note.

## 6. Next steps after review

1. Apply the answers here and in `CANON.md` (the canon pass is Design's), add the decisions, and open `.1` sessions `PK1`..`PK7` in `docs/sessions/` in release order.
2. Queue a Build `feasibility-check` per package in `docs/READY.md` Section 2 (R5 first), then slice into Section 1 as before.
3. The story track (interactive with the user) drafts the Act 2 mission list and loyalty missions against PK8 while R5 is built.
