# Open questions (index)

Status: open (2026-10-25, the final check of the lore consistency pass; the first answers are in section 1). Review
page with a reply sheet: <https://claude.ai/artifact/92EzSi9BwoKB1REyhAM2HP> (private to the user). One deduplicated list of every gap and open
question the library's writers found, plus the ones the final check found. Nothing here is decided: until the user
(or the named lane) decides, the domain docs keep the question open and [CANON.md](CANON.md) does not change. When a
question is decided, its row keeps its id and gets "Decided (date, by whom): ..." in the Rec. column; the planner
copies the decision to `games/emberwake/docs/decisions/lore.md` and updates CANON.md.

Each question has a stable id (Q-01 ...), the question, options (a / b), the recommendation, where it lives (doc and
section) and who decides (user, balance lane, design lane, data authoring). The questions are split by area into two
parts to keep each doc short:

| Part | Areas | Ids |
| --- | --- | --- |
| [World, campaign, roster, hub](questions/open_world_and_campaign.md) | Story and world; Campaign and meters; Roster and lifecycle; Hub and economy | Q-01..Q-46 |
| [Combat, corruption, items, enemies, art, scope](questions/open_systems_and_scope.md) | Combat and rules; Strain and Calcification; Items; Enemies; Art and assets; Slice scope; Process | Q-47..Q-91 |

## 1. Focus order (user 2026-10-25)

Every question carries a Focus in its row. Systems come first, split by system so one can be taken at a time (ordered
by what the slice needs first; 1.6 and 1.7 serve Acts 2-3); then art that needs a new engine type; then lore modules.
The story gaps G-21, G-23, G-25, G-26 and G-28 follow their questions (Q-29, Q-15, Q-17, Q-09, Q-21).

| Order | Focus | System | Open questions |
| --- | --- | --- | --- |
| 1.1 | `1.1 combat` | Combat rules and feel (the M1 prototype and every slice fight): armour, poise, statuses, skill conventions, board size, reach, area shapes, the Crucible winch, enemy intent | Q-49 (after balance), Q-51, Q-57, Q-58, Q-59, Q-60, Q-61, Q-70, Q-72, Q-75, Q-91 |
| 1.2 | `1.2 campaign` | Campaign meters and fail states (M0 hooks): Alert across acts, the Doom Clock budget, doctrine, lost battles, sieges, board jobs on the graph | Q-14, Q-17, Q-19, Q-20, Q-21, Q-22, Q-30 |
| 1.3 | `1.3 roster` | Roster and lifecycle: freed youths, downed and captured, keepsakes, crypts and bunks, Living Martyrs, small roster rules, the first recruit | Q-29, Q-32, Q-34, Q-36, Q-37, Q-40, Q-93 |
| 1.4 | `1.4 hub` | Hub and economy: zero Tallow, forward bases, facility levels, the Rock Salt flush, Act 3 Tallow, hub screens and meters, Quenched Cinder Salt | Q-18, Q-23, Q-42, Q-43, Q-45, Q-46, Q-69 |
| 1.5 | `1.5 slice` | Slice scope and process: the rest of Act 1, prototype order and vignettes, barks, data ids, the Bunks and Facility screens | Q-86, Q-88, Q-89, Q-90, Q-96 |
| 1.6 | `1.6 corruption` | Corruption and derived stats (Act 2 on): bell resonance on calcified operatives, Strain caps and kit, keystone cost, enemy Strain, Aline's dose, Strain sims, small Strain picks, hazard Composure checks, derived stats | Q-10, Q-62, Q-63, Q-64, Q-65, Q-66, Q-67, Q-68, Q-92; DERIVE1 DQ1-DQ5 (`docs/specs/DERIVE1.md`) |
| 1.7 | `1.7 content` | Enemy and item content design: weapon families, boss kits, Act 2-3 adversaries | Q-71, Q-74, Q-76 |
| 2 | `2 art type` | Art and assets that cannot reuse existing assets and need their own type in the engine | Q-78 (several skin ramps in the rig), Q-79 (frames larger than 48 x 48), Q-80 (palettes per world), Q-82 (camera shake) |
| 2 | `2 art` | Other art questions (process, reuse) | Q-77, Q-81 |
| 3 | `3 lore` | Lore and events, kept modular and independent: each is looked at on its own without fleshing out the broader narrative, written as a lore module ([modules/README.md](modules/README.md)); the narrative is fleshed out later by connecting the modules | Q-05, Q-08, Q-11, Q-24, Q-26, Q-27, Q-39, Q-73, Q-95, Q-97; story gaps G-03 to G-07, G-13, G-18 to G-20, G-22, G-29, G-30 |
| Separate | `core story` | The core narrative the game needs to be built at all (the spine, the slice's story beats, world logic the systems read): handled on its own track, not mixed into the lore modules | Q-02, Q-03, Q-06, Q-09, Q-28, Q-35, Q-87, Q-94; story gaps G-08, G-09, G-10, G-12, G-14 to G-17, G-31, G-32, G-33 |

### Decided first (2026-10-25)

These blocked the most other work (specs, data and the slice). Recommendations in brackets.

Answered on 2026-10-25 (user, in chat): Q-01, Q-04, Q-13, Q-16, Q-25, Q-31, Q-47, Q-47a, Q-48, Q-83, Q-84 and Q-85
are decided, with Q-07, Q-12 and Q-33 following from them (see their rows and
`games/emberwake/docs/decisions/lore.md`); Q-49 waits for a balance review. Later the same day the user also answered
Q-15, Q-38, Q-41, Q-44, Q-50, Q-52, Q-53, Q-54, Q-55 and Q-56. Story gaps G-01..G-30 (character arcs, motives and
fates) are in [STORY_GUIDE.md](STORY_GUIDE.md) section 7; they get Q-ids when decided.

| Id | Question | Blocks |
| --- | --- | --- |
| [Q-01](questions/open_world_and_campaign.md#story-and-world) | The central truth: A "the lullaby is a harvest" or B "the lullaby is real and the Clerisy is also corrupt" (A) | the endings, Othemar, Malachi, Maren's texts (Q-07), the Act 3 tremors, every Act 2-3 scene |
| [Q-04](questions/open_world_and_campaign.md#story-and-world) | Red Tithing size and recurrence, with what the soft clock does at 0 ([Q-16](questions/open_world_and_campaign.md#campaign-and-meters)) (a yearly rite of hundreds plus a Great Tithing of thousands; lesser Tithings with story costs when the soft clock runs out) | the Doom Clock spec, the vignette, the Act 3 stakes |
| [Q-13](questions/open_world_and_campaign.md#campaign-and-meters) | Inquisition Alert at 100% in Act 1 (cap at 99 in the slice) | the Alert meter spec, the Sunk Ward travel rules |
| [Q-25](questions/open_world_and_campaign.md#campaign-and-meters) | Nemeth at the Crucible: escape or defeat (escape) | the Crucible encounter, Nemeth's kit, his Cassock, Act 2's cast |
| [Q-31](questions/open_world_and_campaign.md#roster-and-lifecycle) | What "wounded" does and how it heals (-20% max HP until a rest) | the unit lifecycle hook, the Slab, the cost of losing an Act 1 fight |
| [Q-47](questions/open_systems_and_scope.md#combat-and-rules) | Composure: drop it or add a stat (drop) | Calcification I, chimes, shockwaves, Martyrs, nightmares, recruit quirks |
| [Q-48](questions/open_systems_and_scope.md#combat-and-rules) | Deck slot kinds: passive and keystone slots, or source-granted passives (slot kinds) | the deck UI, the loadout hook, every class file |
| [Q-49](questions/open_systems_and_scope.md#combat-and-rules) | Armour scale and pierce: raise armoured ratings to 25-50, raise the factor, or keep x 2 (ratings 25-50) | every enemy row, the value of the Inoculated branch, `strain_cost` tuning |
| [Q-83](questions/open_systems_and_scope.md#slice-scope), [Q-84](questions/open_systems_and_scope.md#slice-scope) | The slice's node list and mission list (01, 03, 04, 06; the roadmap's lean list) | the `sunk_ward` world data, M2's scope |
| [Q-85](questions/open_systems_and_scope.md#slice-scope) | The slice's one dilemma (a board quest with a two-way choice, e.g. Corvo's gate) | M2's dialogue and flags, Q-30 |

Also near the top: Q-09 (the bell-resonance contradiction between CANON 3.3 and 3.4, left to the user with no pick)
and Q-52 (where the ruleset switch is set), which the first combat spec needs.

## 2. Planner picks to confirm

Choices the approved brief did not make; the canon writer marked them "(planner pick)" in the files. They stand as
written until the user says otherwise.

| Id | Pick | Where |
| --- | --- | --- |
| P-01 | Prologue encounter ids ENC-PRO-0A..0C; Captain Vane gives "Order 44" in 0a and withdraws | CANON 3.1 |
| P-02 | Act 1 M1 "The Sweep" (ENC-EVT-01, Captain Vane) takes the old prologue slot | CANON 3.2 |
| P-03 | ENC-EVT-02 re-themed "The Crossroads Cages" as Maren's hook; Quell's marrow valve absorbs the old "Inquisitor Sergeant with a gland" hypocrisy beat (narrative writer) | CANON 3.2; six_founders 3 |
| P-04 | Tess's board offers the next main mission when the previous one is done; optional jobs any time | CANON 3.2 |
| P-05 | The interlude: the player picks which deployed founder takes Aline's marrow dose; that founder starts Act 2 at Calcification I | CANON 3.2 |
| P-06 | The Crucible requires four deployed founders; the Slag Gate winch is the one mechanism | CANON 3.2 |
| P-07 | The captured founder leaves an ACC heirloom keepsake, upgraded by the Act 3 Transcendent Relic | CANON 3.3, 3.5 |
| P-08 | Act 3 main path 16-17-18-21-22 (19 and 20 optional); Othemar always falls at 21; Malachi mediates at the Chorus Spire | CANON 3.5 |
| P-09 | Ending gates: A open unless Flesh-Broker, B needs Flesh-Broker, C always open | CANON 4 |
| P-10 | Max Poise = CONS x 2 + armour poise; stagger multipliers multiply; a break only cancels (no evasion or damage riders) | CANON 2.1 |
| P-11 | Active Guard: 50% damage reduction through the guard term (capped by the guard cap) and half stagger | CANON 2.1 |
| P-12 | Aberrant Edge: +50% damage and 10% recoil (the bleed and stagger extras dropped) | CANON 2.2 |
| P-13 | Stage III: -15% ranged damage beyond 3 tiles (replaces the accuracy penalty) | CANON 2.3 |
| P-14 | Engine armour rating = lore armour points x 2 (the balance lane confirms; see Q-49) | CANON 2.1 |
| P-15 | A rest costs 1 day; a battle costs none | CANON 2.6 |
| P-16 | Inquisition Alert starts at 0 | CANON 3.3 |
| P-17 | St. Lyra's is the one High Cathedral | cast_registry, places |
| P-18 | The Fourth Kiln Syndicate is the Tallow Guild's foundry arm (Bram its forge, Dunstan its bone-mill) | cast_registry |
| P-19 | Vesper's parents were rendered at the Tallow Vats; Orlov's Guild paid the informant who sold out her ring | cast_registry |
| P-20 | Maren drops the "Sister" title; her throat veins come from marrow-ink | cast_registry |
| P-21 | Lucan is a Tallow Vats vat-boy apprenticed to Bram, rescued from one culling wagon in Act 2 | cast_registry; CANON 3.4 |
| P-22 | Agnes remembers the stars through her grandmother; the Extinguishing was two centuries ago | cast_registry |
| P-23 | Pip is rescued in Act 1 (Tess's "Night of the Red Pen") | cast_registry |
| P-24 | Brother Malachi appears after the ENC-EVT-05 interlude opens his catacombs | cast_registry |
| P-25 | Madame Sallow appears in Act 1 | cast_registry |
| P-26 | Kept echoes: the Slumbering Root and the Severed Root; Captain Vane and Sister Aline Vance (always "Sister Aline" in text); the Extinguishing (event) and Extinguished (failure screen) | cast_registry, near-collisions |
| P-27 | The registry's new names: Hollis, Dunstan, Ruthven, Fitch, Quell, Nemeth, Lazar, Ulric, Jessop, Sallow, Skerrit, Rook, Ysolde, Ferrand, Clemence, Ida, Wren; places the Chorus Spire, the Silent Ossuary, the Scriptorium of St. Lyra's | cast_registry |
| P-28 | Names fixed in the final check: the 0a soldiers are Castellan Footmen (not "Wardens"); the Watch's crossbow unit is the Watch Crossbowman (the class keeps "Arbalestier"); Hollis's trait is Crag Hawkeye and his relic Hollis's Lever-Winch (the clocktower origin is retired) | bestiary; class_arbalestier; items 7 |
| P-29 | Small combat picks: Orlov's Retribution is Standard (2 blocks); Martyr's Shroud keeps 3 AP; Ocular Lock's mark is +30% crit | class_gutter_stalker; classes_and_skill_trees 6 |

## 3. Settled during the pass (not open)

- Kit extraction in M0 was approved by the user on 2026-10-25; the architecture doc's old gap 1 (defer it) is
  withdrawn ([rpg_kit_and_system_architecture.md](specs/rpg_kit_and_system_architecture.md) 7).
- The archive is exempt from the doc size check (the planner widened `tools/fc.sh docs`).
- The dialogue conditions are flag (reputation keys included), quest, items, gold and stat; there is no `status`
  condition; the effects include `turn_in` (checked against `emberwake_content` on 2026-10-25).
