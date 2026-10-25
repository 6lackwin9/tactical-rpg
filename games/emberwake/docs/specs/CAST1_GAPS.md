# CAST1 gaps and contradictions

Status: proposed (2026-10-25; CAST1-R the same day: the slice gaps point to HOOKS1, ROSTER1 and CAMPAIGN1); part of
[CAST1.md](CAST1.md). A gap is something the Act 1 cast needs that RULESET1, ENCOUNTER2, DERIVE1 and today's systems
do not provide. Q-55 and Q-56 ask for every hook the slice uses to be built before the slice: HOOKS1
(`docs/specs/HOOKS1.md`), ROSTER1 and CAMPAIGN1 now cover the slice's gaps, and CAST1's rows use their fields (no
stand-ins). A row marked "resolved" points to the spec and section; an "open" row names what the data does
meanwhile (the lore part is deferred, not faked) and waits for the planner to decide whether and when to spec it.

## 1. Gaps

| Id | Gap | Needed by | Status and pointer | Meanwhile (open rows) | Slice |
| --- | --- | --- | --- | --- | --- |
| G1 | Class re-theme per world: a display name, a per-world skill path (Q-53) | all six founders | resolved: HOOKS1 12 (HOOKS1-C, rows HOOKS1-A); proficiencies stay drills (CQ1 a, HQ6 a) | - | yes |
| G2 | Teleport or free reposition on an event | Orlov's Retribution (on a kill), Void-Flicker (Act 2) | open (HOOKS1 14) | a Maneuver Disengage carried retreat on every use | no |
| G3 | Forced movement and immunity to it | Intercepting Hook, Shackle Slam, Slag Slam, Winch Sunder, Billhook Hook, Shackle Drag, Buckler Jab, Gilded Sunder, Cataclysm, Hypertrophy, Braced Poise, Unyielding Authority | resolved: HOOKS1 5 (HOOKS1-F); Hammer Strike's knock-down is the Staggered status (Q-56) | - | yes |
| G4 | Ground hazards a skill or item leaves | Ash-Cloud and Fade, Tripwire Snare, Phosphorus Pot, Choke-Gas, Grease Pot, Nozzle Sweep | resolved: HOOKS1 6 (HOOKS1-G; rows HOOKS1_TASKS 2.1) | - | yes |
| G4b | Hazard rules beyond placement: zones a round longer, overlapping zones detonating | Volatile Reaction Catalyst | open (HOOKS1 14: out of scope) | the trait is INT +2 | no |
| G5 | Unit event triggers, reactions, redirects, AP refunds, battle stacks | Penance, Leverage, Phalanx, Iron, Spite, Triage, Dead-Eye, Tourniquet, Reflex, Fanatical Agony, Catalytic Munitions, the pavise's reflect | resolved: HOOKS1 8 (HOOKS1-E, -K) | - | yes |
| G5b | Events no HOOKS1 effect can answer | Tenacious Chase (follow a target moving in its windup), Osseous Shrapnel (a hit to every adjacent unit on its break), the Amalgam's phase "on a poise break" (no phase trigger reads a break) | open (no spec) | not in the data; the descriptions do not promise them | no |
| G6 | Barriers and line of sight | Seventh Ward Bulwark | open (engine or kit, after testing, Q-54) | taunt and fortify | no (level 3) |
| G7 | Execute riders (dies below X% HP, double damage below X%) and the refund tied to them | Throat-Carver's Art (and its 2 AP back), Orlov's Flensing Talon, the Guild Carver (Act 2) | open (HOOKS1 14: riders could carry them later) | exploit multipliers on Staggered and Rooted | yes (Vesper level 2) |
| G8 | Terrain-conditional rules | High-Ground Traverse, Prowler's Cadence, Crag Hawkeye, Slag Tempering, Combustion Formula, Bone Bloom, Burrow Ambush, M1's crossbow perch | open | stats and resists; plain terrain | no |
| G9 | A second inflicted status on an attack, and an on-hit status from an item | Billhook Sunder (root), Caustic Vomit (expose), Cataclysm (expose), the Marrow Scalpel (bleed) | open (HOOKS1 14: riders could carry them later) | the main status only; `status_potency` on the item | no |
| G10 | Per-target and per-hero conditions | Billhook Sunder's disarm, Vane's counter against Kaelen, Orlov's "when Vesper attacks" | open | none; key loot on Orlov's kill (CQ10 a) | no |
| G11 | Conditional passive modifiers: crit immunity, damage-type resistance, an exploit passive, halving cover | Unbroken Phalanx (crit part), Hippocratic Oath, Anatomical Flaw, Cover Breaker, Anvil Posture, the sprayer's fire type | open | `stagger_taken`, `skill_potency`, `status_potency` | no (the slice ships without these parts) |
| G12 | Area support and auras (Q-72) | Resilience Ward, Anatomical Precision, Traitor's Discipline, Vat-Master's Grease, Master Alchemist, the Slag-Hammer's shockwave | open | single-target grants; stats; M4's grease tiles | no |
| G13 | A join level for founders who join mid-act | Dunstan (M3), Elyse (M4) | resolved: ROSTER1 BK5 (`recruit` `level` `average`, CQ14 a) | - | yes |
| G14 | A per-unit battle counter (Q-61 b, Q-63) | the Bile Phial; Slag Fury | resolved: HOOKS1 9 (HOOKS1-M; HQ3 a, HQ4 a) | - | yes |
| G15 | A status that bites when the unit moves | Vivisection, Barbed Quarrel | open | bleed | no |
| G16 | Terrain change by a skill (rubble, tiles set alight) | Foundry Shatter, Pulverizer, Slag Tempering | open (HOOKS1 FM4 leaves walls whole) | none | no |
| G17 | New status rows and item effects: Blind as a range cap (Q-56), Flammable, mark, frenzy, AP loss; consumables that cleanse | Flash-Bomb, Choke-Gas, Grease Pot, the Stitch-Poultice's cleanse, Ocular Lock (Act 2) | open (guard strip is resolved by HOOKS1 `end_guard`) | the Staggered status, poison, slow; heal only | no |
| G18 | Cheat-death once, a phase trigger on it, a phase `heal` (Q-74) | Quell's marrow valve | resolved: HOOKS1 10.2 (HOOKS1-Q, after the slice, HQ13 a) | - | no (M2b) |
| G19 | Capstone unlock by a phase | Vane, Quell, Grendel, Orlov, the Amalgam | resolved: HOOKS1 10.1 (HOOKS1-U, phase effect `deck`) | - | yes (Grendel) |
| G20 | A per-member start buff picked by the player; a "member deployed" dialogue condition | Aline's marrow dose (Q-66 b) | open | the pick is a flag only; no buff in the fight | no |
| G21 | Multi-cell units, 96 x 96 frames (Q-79) | the Calcified Amalgam | open: BIGUNIT1 (idea row) | 1 cell (CQ15 a) | no |
| G22 | Sunk Ward zones and backdrops | every encounter | resolved: HOOKS1 13 (HOOKS1-Z; rows HOOKS1_TASKS 2.3); backdrop art open (CAST1_ASSETS N10, M4, asked first) | STAGE1's gradient per zone | yes |
| G23 | A minimum deployed party for an encounter | the Crucible needs four (CANON 3.2) | resolved: ROSTER1 BK6 (`party_min` 4) | - | yes |
| G24 | Scheduled arena hazards | M3's slag vents | resolved: HOOKS1 7 (HOOKS1-H); belts later as data (HQ8 a) | - | yes |
| G25 | Encounter rewards beyond Scrap and loot: purse resources (Warded Tallow) | M4's 5 Tallow | open (the saved counter is resolved: ROSTER1 SY1 `youths_saved`) | a flag set by the after-fight dialogue | no |
| G26 | Loot rows reference catalog items only (world named items cannot be loot) | the relics | open, not blocking: the keepsakes are resolved by ROSTER1 KS2 (RQ7 a, paid at the capture) | relics as `special` catalog items | no |
| G27 | A skill that costs HP | Martyr's Rush | resolved: HOOKS1 8.5 HC1 (HOOKS1-K) | - | yes (0b) |
| G28 | Guard bypass and guard strip on a hit | Decapitation, Heretic's Brand | resolved: HOOKS1 8.5 IG1 and rider `end_guard` (HOOKS1-K) | - | yes (M5) |
| G29 | AI heuristics beyond the RULESET1 table | Footmen, Flagellants, Bailiff Enforcers, Purifiers, Nemeth | resolved: HOOKS1 11 (HOOKS1-AI: `tactics`, SOFTEST, TOUGHEST) | - | yes |
| G29b | AI heuristics HOOKS1 leaves out | Render-Beasts (pack hunting), Crossbowmen (raised tiles; windup targeting not adopted, HQ2 a) | open (HOOKS1 AT4: a later tactic `pack`) | `ai_role`, `target_pref`, `reckless` | no |
| G30 | Mission tags the Alert meter reads (`loud`, `quiet`; Q-15) | every encounter's `tags` | resolved: CAMPAIGN1 M3 battle inputs (CAMPAIGN1_TASKS 4.2) | - | yes |

## 2. Contradictions found (lore, CANON and the specs)

| # | Where | What | Status and CAST1's handling |
| --- | --- | --- | --- |
| C1 | `lore/questions/open_systems_and_scope.md` Q-73; CANON 3.1 | Q-73 is marked "3 lore" (open, rec a); CANON 3.1 says 0c is "3 Inquisitors + a crossbowman", while option a makes them Castellan Footmen guarding an Inquisition mortuary cart | settled: Q-73 a (planner pick, user away); the planner records Q-73 and updates CANON 3.1 and the bestiary encounter file |
| C2 | Q-28; the brief | The vendetta mechanic; the brief calls it a bonus objective | settled: Q-28 decided ("founder hooks pay off in play"); CQ10 a (key loot on Orlov's kill) |
| C3 | ENCOUNTER2_TASKS 4.2; ROSTER1_TASKS 5.1; `sunk_ward_campaign_map.md` 5 | The Crucible's sketch id is `crucible_slag_gate`; the lore's ENC id is ENC-EVT-06 | settled: `enc_evt_06` (the lore id, Q-90; HOOKS1_TASKS 2.3 uses it); ENCOUNTER2's sketch and ROSTER1_TASKS 5.1 follow |
| C4 | `class_vanguard.md` 4; onboarding 3; `bestiary_castellan_watch.md` (Vane) | Kaelen's two-handed billhook against his signature's tower shield; Vane's billhook and pavise | settled: CQ16 a (the Bulwark plants the billhook haft and his pauldron); the pavise is Vane's drop for a one-handed build |
| C5 | `bestiary_castellan_watch.md` (Footman, reach 1); RULESET1 K1 (polearms reach 2) | A billhook is a polearm, but the Footman's Billhook Hook is reach 1 | settled: CQ13 a (a one-handed hookbill in the axe family, reach 1, with a heater shield) |
| C6 | `classes_and_skill_trees.md` 5 (data binding example) | The Bulwark example is a DEFEND with `area` LINE, `recovery_blocks` 1 and `stagger` 35 (RULESET1 1.3 forbids all three) | CAST1's row replaces the example; the lore example should be marked superseded |
| C7 | `classes_and_skill_trees.md` 4 | Signature skills "given through the character's `extra_skills`"; world characters have no such field | CAST1 puts them on the founder's catalog unit (`founder_<id>.extra_skills`) |
| C8 | ENCOUNTER2_TASKS 4.1 (`kit_winch_escape` rows) | The sample's `L` glyphs beside the gate exits, probably meant as braziers | CAST1's Crucible uses `f` braziers; the kit sample may keep `L` (blocked props) or switch |
| C9 | `bestiary_encounters_act1.md` ENC-EVT-02; CANON 3.6 tone rule | The flagellants kill caged youths on the board, while children may never be on-screen victims | settled: CQ6 a (the captives on the board are adults 18-24) |
| C10 | `who_is_who.md` (Orlov, G-04) | Orlov dies at M4 or lives for Vesper's Act 2 feud | CAST1 lets him die (key loot on the kill), as G-04's recommendation; the Act 2 feud targets his Guild |
| C11 | `sunk_ward_campaign_map.md` 3.2 vs `bestiary_encounters_act1.md` ENC-EVT-01 | M1's roster: Vane and 2 Footmen, or with a Watch Crossbowman | CAST1 follows the bestiary (four units) |
| C12 | `bestiary_aberrations.md` (the Amalgam, rating 12) vs Q-49 | The lore's rating 12 sits below the armoured band | settled: Q-49 (decided for testing: armoured ratings 25-50); the Amalgam uses 50 |
| C13 | `kits/rpg/schema.json` (unit `tier` help) vs GAME_BRIEF (BAL3) | The help says boss 170% HP / 120% power; the brief says boss 1.5 / 1.1 | not CAST1's to fix; the balance lane reads the code |
| C14 | HOOKS1 8.1 and 8.4 vs CAST1_FOUNDERS 2 | Kaelen holds two `ally_hit` redirects (Penance 35, the Hook's intercept 100); "one redirect a hit, the first holder by registration order" lists passives before a running skill's triggers, so Penance takes the first hit of a round even while the Hook runs | open: CQ18 |
| C15 | CAMPAIGN1_TASKS 4.2 ("the main missions' boss fights `loud`") vs CAST1_ENCOUNTERS (M5 and the interlude not `loud`) | A loud Crucible win could push Alert to the Act 1 Breach at the climax | open: CQ17 |
| C16 | ROSTER1_TASKS 5.1 (ROSTER1-D writes `party_min` and `then` `capture` on the Crucible) vs CAST1-A6 (writes the whole row) | Two asset tasks write the same encounter keys | CAST1-A6 writes them on `enc_evt_06`; ROSTER1-D's Crucible row becomes a check (planner) |
