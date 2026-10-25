# CAST1 gaps and contradictions

Status: proposed (2026-10-25); part of [CAST1.md](CAST1.md). A gap is something the Act 1 cast needs that RULESET1,
ENCOUNTER2, DERIVE1 and today's systems do not provide. CAST1 designs none of them: each row names the stand-in the
data ships with now and where the hook would land; the planner decides whether and when to spec it. Q-55 asks for
every hook the slice uses to be built before the slice (forced movement and ground hazards first): when a hook lands,
a small CAST1 follow-up data task swaps the stand-ins named here for the lore effect.

## 1. Gaps

| Id | Gap | Needed by | Act 1 stand-in | Lands in | Slice |
| --- | --- | --- | --- | --- | --- |
| G1 | Class re-theme per world: a display name over a kit class, extra proficiencies and a per-world skill path (Q-53) | all six founders | character `title` = class name; drill passives grant proficiency (CQ1); the Ashlands path families stay owned | kit hook (the M0 roadmap row "base classes in the kit with a display name per world"; KIT1 leaves it to the roster or campaign spec) | yes (stand-in) |
| G2 | Teleport or free reposition on an event | Orlov's Retribution (on a kill), Void-Flicker (Act 2) | a Maneuver Disengage carried retreat on every use | kit hook | no |
| G3 | Forced movement: knockback, pull, drag, throw, push, and immunity to it | Intercepting Hook, Shackle Slam, Slag Slam, Winch Sunder, Billhook Hook, Shackle Drag, Gilded Sunder, Hammer Strike, Cataclysm, Hypertrophy, Braced Poise, Unyielding Authority | statuses (root, Staggered), LINE and SWEEP areas, `sure_break` | kit hook (Q-55's first) | yes |
| G4 | Ground hazards a skill or item leaves (smoke, fire, grease, traps) and zone rules (durations, overlap detonation) | Ash-Cloud and Fade, Tripwire Snare, Phosphorus Pot, Choke-Gas, Grease Pot, Nozzle Sweep, Volatile Reaction Catalyst | `stealth`, a thrown snare (root), area bursts with burn or poison; static hazard tiles in `rows` | kit hook (Q-55's first) | yes |
| G5 | Unit event triggers and reactions: on a break, on a kill, below an HP share, on a guarded hit, when an adjacent ally is targeted (redirect), AP refunds, battle stacks | Castellan's Penance, Kinetic Leverage, Unbroken Iron, Foundry-Wired Spite, Triage Efficiency, Dead-Eye, Tourniquet's guard, Fanatical Agony, Tenacious Chase, Osseous Shrapnel, the pavise's reflect | flat passive levers (stats, resists, `ap_regen`, `stagger_taken`) | kit hook | yes (stand-ins) |
| G6 | Barriers and line of sight | Seventh Ward Bulwark | taunt and fortify | engine or kit, after testing (Q-54) | no |
| G7 | Execute riders (dies below X% HP, double damage below X%) | Throat-Carver's Art, Orlov's Flensing Talon, the Guild Carver (Act 2) | exploit multipliers on Staggered and Rooted | kit hook | no |
| G8 | Terrain-conditional rules: range from high ground, move cost in cover, mist and rubble, no poise damage on burning tiles, warded safe tiles, tunnelling, ignoring partial cover, raised firing spots | High-Ground Traverse, Prowler's Cadence, Crag Hawkeye, Slag Tempering, Combustion Formula, Bone Bloom, Burrow Ambush, M1's crossbow perch | stats and resists; plain terrain | kit hook | no |
| G9 | A second inflicted status on an attack, and an on-hit status from an item | Billhook Sunder (root), Caustic Vomit (expose), Cataclysm (expose), the Marrow Scalpel (bleed) | the main status only; `status_potency` on the item | kit hook (skill schema) | no |
| G10 | Per-target and per-hero conditions (target faction, a named hero) | Billhook Sunder's disarm of Clerisy and Inquisition targets, Vane's counter against Kaelen, Orlov's "when Vesper attacks", the vendetta by Vesper's hand | none; key loot on Orlov's kill (CQ10) | kit hook | no |
| G11 | Conditional passive modifiers: crit immunity, crit chance, damage-type resistance, a passive exploit against statuses, halving cover | Unbroken Phalanx, Hippocratic Oath, Anatomical Flaw, Cover Breaker, Anvil Posture's blunt resist, the sprayer's fire type | `stagger_taken`, `skill_potency`, `status_potency` | kit hook (passive grammar) | no |
| G12 | Area support and auras (squad buffs, ally and enemy auras, radius bonuses, "behind the target" shape, Q-72) | Resilience Ward, Anatomical Precision, Traitor's Discipline, Vat-Master's Grease, Master Alchemist, the Slag-Hammer's shockwave | single-target grants; stats | kit hook | no |
| G13 | A join level for founders who join mid-act | Dunstan (M3), Elyse (M4) | level 1 unless RECRUIT1 supports it (CQ14) | kit hook: ROSTER1 BK5 (`recruit` `level`) covers it | yes (Dunstan) |
| G14 | A per-unit battle counter (Q-61 b), RULESET1 C4's separate spec | the Bile Phial (Copyist of St. Lyra's) | `skill_potency` heal +10% | kit hook (meter spec, Q-63) | yes (stand-in) |
| G15 | A status that bites when the unit moves (Arterial Rupture, tearing barbs) | Vivisection, Barbed Quarrel | bleed | kit hook (status row) | no |
| G16 | Terrain change by a skill (rubble, tiles set alight) | Foundry Shatter, Pulverizer, Slag Tempering | none | kit hook | no |
| G17 | New status rows and item effects: Blind as a range cap (Q-56), Flammable, mark, frenzy, AP loss, guard strip; consumables that cleanse | Flash-Bomb, Choke-Gas, Grease Pot, the Stitch-Poultice's cleanse, Ocular Lock (Act 2) | the Staggered status, poison, slow; heal only | kit hook (status table is code, RULESET1 C1) | no |
| G18 | Cheat-death: an `unkillable` that holds once, a phase trigger on that event, a phase `heal` effect (Q-74) | Quell's marrow valve | a phase at 30% HP with regen, fortify and enrage | kit hook (an ENCOUNTER2 amendment beside S1) | no (M2) |
| G19 | Capstone unlock: a phase that changes a unit's deck order, or ENCOUNTER2-X's swap brought forward | Vane, Quell, Grendel, Orlov, the Amalgam | the capstone sits last in the order from the start; the phase adds a line and enrage | kit hook (ENCOUNTER2-X or a phase `order` effect) | yes (Grendel, stand-in) |
| G20 | A per-member start buff picked by the player, and a "member deployed" dialogue condition | Aline's marrow dose (Q-66 b) | the pick is a flag only; no buff in the fight | kit hook (`start_status` with a member flag; dialogue conditions) | no |
| G21 | Multi-cell units, 96 x 96 frames (Q-79) | the Calcified Amalgam | 1 cell (CQ15) | BIGUNIT1 (idea row) | no |
| G22 | Sunk Ward zones: backdrops and tile sets for alley, rooftop, canal, foundry, vats, cavern, watchpost | every encounter | today's code zones | game data or code (zones are code, decision S12) and art | yes (placeholders) |
| G23 | A minimum deployed party for an encounter | the Crucible needs four (CANON 3.2) | none; ENCOUNTER2 canon-vs-code 6 already notes it | kit hook: ROSTER1 BK6 (`party_min`) covers it | yes |
| G24 | Scheduled arena hazards and moving terrain (vents on blocks 3 and 6, conveyor belts) | M3 | static `burning_ground` tiles and a phase line | kit hook | yes (stand-in) |
| G25 | Encounter rewards beyond Scrap and loot: purse resources (Warded Tallow) and the saved counter (Q-29) | M4's 5 Tallow, M2's freed captives | a flag set by the after-fight dialogue | kit hook (purse rewards); the counter is ROSTER1 SY1 (`youths_saved`) | no |
| G26 | Loot rows reference catalog items only (world named items carry history and grants but cannot be loot); paying a loot row from an event (the keepsake at the Act 2 transition) | the relics, the six keepsakes | relics and keepsakes as `special` catalog items; the keepsake is paid by ROSTER1 KS2 (character `keepsake`, at the capture) or by loot rows at the Act 2 opening (ROSTER1 RQ7 picks) | kit hook (loot refs); ROSTER1 KS1-KS5 | no (Act 2 opening) |
| G27 | A skill that costs HP | Martyr's Rush | none | kit hook (skill field) | yes (0b, stand-in) |
| G28 | Guard bypass and guard strip on a hit | Decapitation ("ignores Active Guard"), Heretic's Brand ("ends Active Guard") | none | kit hook | yes (M5, stand-in) |
| G29 | AI heuristics beyond the RULESET1 table: chokepoints, bracing after an ally is hit, seeking raised tiles, targeting a unit in windup, pack hunting, rushing the back line past tanks | Footmen, Crossbowmen, Flagellants, Render-Beasts, Dredgers | `ai_role`, `target_pref`, `reckless`, `flee_ratio` | kit hook (AI profiles) | yes (stand-in) |
| G30 | Mission tags the Alert meter reads (`loud`, `quiet`; Q-15) | every encounter's `tags` | free-form tags written now, read by nobody until the meter spec | the Alert meter spec (M0) | yes |

## 2. Contradictions found (lore, CANON and the specs)

| # | Where | What | CAST1's handling |
| --- | --- | --- | --- |
| C1 | `lore/questions/open_systems_and_scope.md` Q-73; CANON 3.1 | Q-73 is still marked "3 lore" (open, rec a) although the planner's brief treats it as decided; CANON 3.1 still says 0c is "3 Inquisitors + a crossbowman", while option a makes them Castellan Footmen (the Watch, not the Inquisition) guarding an Inquisition mortuary cart | CAST1 follows option a; the planner records Q-73 and updates CANON 3.1 and the bestiary encounter file |
| C2 | Q-28 (open, "core story"); the brief | The vendetta mechanic is open; the brief calls it a bonus objective | CQ10 (a: key loot on Orlov's kill) |
| C3 | ENCOUNTER2_TASKS 4.2; `sunk_ward_campaign_map.md` 5 | The Crucible's sketch id is `crucible_slag_gate`; the map sketch uses `enc_evt_01`-style ids from the lore's ENC ids | CAST1 uses `enc_evt_06` (the lore id, Q-90); ENCOUNTER2's sketch should follow |
| C4 | `class_vanguard.md` 4; onboarding 3; `bestiary_castellan_watch.md` (Vane) | Kaelen holds a two-handed billhook, yet his signature plants a tower shield; Vane carries a billhook and a heavy pavise: a two-handed polearm leaves no off-hand | Kaelen and Vane carry the two-handed bill; the pavise is Vane's drop for a one-handed build; the Bulwark's shield is open (CQ16) |
| C5 | `bestiary_castellan_watch.md` (Footman, reach 1); onboarding 3 ("billhooks"); RULESET1 K1 (polearms reach 2) | A billhook is a polearm (reach 2), but the Footman's Billhook Hook is reach 1 | CQ13 (a: a one-handed hookbill in the axe family) |
| C6 | `classes_and_skill_trees.md` 5 (data binding example) | The Bulwark example is a DEFEND with `area` LINE, `recovery_blocks` 1 and `stagger` 35: DEFEND takes no area, party units book no recovery, and `stagger` is an ATTACK field (RULESET1 1.3) | CAST1's row replaces the example; the lore example should be marked superseded |
| C7 | `classes_and_skill_trees.md` 4 | Signature skills are "given through the character's `extra_skills`", but world characters have no such field | CAST1 puts them on the founder's catalog unit (`founder_<id>.extra_skills`) |
| C8 | ENCOUNTER2_TASKS 4.1 (`kit_winch_escape` rows) | The sample's `L` glyphs at [10, 3] and [10, 5] are lava (blocked) beside the gate exits, probably meant as braziers | CAST1's Crucible uses `f` braziers; the kit sample may keep `L` (blocked props) or switch |
| C9 | `bestiary_encounters_act1.md` ENC-EVT-02; CANON 3.6 tone rule | The flagellants kill caged youths on the board every 2 rounds, while children may never be on-screen victims | CQ6 (a: the captives on the board are adults 18-24) |
| C10 | `who_is_who.md` (Orlov, G-04) | Orlov dies at M4 (slay, drops) or lives for Vesper's Act 2 feud | CAST1 lets him die (key loot on the kill), as G-04's recommendation; the Act 2 feud targets his Guild |
| C11 | `sunk_ward_campaign_map.md` 3.2 vs `bestiary_encounters_act1.md` ENC-EVT-01 | M1's roster: Vane and 2 Footmen, or Vane, 2 Footmen and a Watch Crossbowman | CAST1 follows the bestiary (four units) |
| C12 | `bestiary_aberrations.md` (the Amalgam, rating 12) vs Q-49 | The lore flags rating 12 as above CANON's band; the Q-49 test scale puts armoured units at 25-50 | CAST1 uses 50 (the boss top of the test scale) |
| C13 | `games/emberwake/schema.json` (unit `tier` help) vs GAME_BRIEF (BAL3) | The help says boss 170% HP / 120% power; the brief says boss 1.5 / 1.1 | not CAST1's to fix; the balance lane reads the code |
