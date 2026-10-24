# Open questions: combat, corruption, items, enemies, art, scope

Status: open (2026-10-25, the lore consistency pass, final check). Part of [OPEN_QUESTIONS.md](../OPEN_QUESTIONS.md),
which explains the columns and lists the questions to decide first. Ids are stable: a decided question keeps its id
and gets "Decided (date, by whom): ..." in its Rec. column.

Columns: Id; Question; Options; Rec.; Where (paths from `games/emberwake/docs/lore/`); Decides.

## Combat and rules

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-47 | Composure is not one of the six stats (STR DEX INT CONS PERC KNOW), yet Calcification I, the curfew chimes, the continental shockwave, the Martyr mercy reward, the nightmares and recruit quirks use it | (a) drop it and restate each use on existing levers (debuff resistance; a flat poise loss halved by Active Guard); (b) a new kit stat, or derived from KNOW | Decided (2026-10-25, user): drop Composure as a stat; its uses go to the resist system or a separate corruption resist (Q-47a) | CANON 2.3; systems/magic_and_corruption.md gaps; rpg_kit gap 5; world_geography 4; upper_bastion 4.5; mantle 2.2 | user |
| Q-47a | Which resist takes Composure's uses (Calcification I, the curfew chimes, the shockwave, nightmares, the Martyr mercy reward, recruit quirks) | (a) the existing debuff resistance; (b) a separate corruption resist (a new kit stat derived per unit, raised by Preservationist gear and salt) | b (corruption becomes something to build against, and the Act 2 Inoculation trade-off reads on one number) | CANON 2.3; magic_and_corruption gaps | user |
| Q-48 | Deck slot kinds: the canon deck has 2 passives and 1 keystone, but today passives come only from sources and the 5 class + 3 extra slots hold skill families | (a) passive and keystone loadout slot kinds for the "timeline" ruleset (a new kit slot type; actives in class slots); (b) passives stay source-granted; the deck is 2 base moves + 3 actives + 1 keystone | Decided (2026-10-25, user): a; works like today's skill slots, bucketed by kind | CANON 2.1; skill_acquisition 2.2, gaps; rpg_kit gap 3; screen_architecture 4 | user |
| Q-49 | Armour scale and pierce: at lore points x 2 the Inquisition's armour is rating 6-10 (11-17% reduction), so binary pierce gains little | (a) give armoured units ratings of 25-50 in data; (b) raise the lore-to-engine factor; (c) keep x 2 | Deferred (2026-10-25, user): set after a balance review; intent: from Act 2 fights are hard without armour pierce and the other Inoculation benefits | CANON 2.1; risk_reward 3, 8; bestiary_aberrations | user, balance lane |
| Q-50 | Stagger rescale and the interrupt lock: a 25-stagger Snap in windup (x2 = 50) breaks any 12-36 bar, and enemies plan first and never re-plan | (a) rescale lore stagger to a fraction of a typical Max Poise; heavy enemy windups get more poise or an unbreakable flag; then a sim; (b) keep the numbers, tune poise only | a | combat_and_timeline 10; risk_reward 8; rules_engine 8 | balance lane |
| Q-51 | Poise details: armour poise per gear piece; flat poise bonuses on passives (Braced Poise +20) | armour poise: (a) a flat value per weight class, (b) the item's mitigation x a factor; passives: allowed as a third term | b; passives allowed | combat_and_timeline 10; classes_and_skill_trees 6 | balance lane |
| Q-52 | The ruleset switch: where it is set and when it is on | (a) a world field (Ashlands "classic", `sunk_ward` "timeline") with a Dojo toggle; (b) per encounter or in the game manifest. Also: the first playable on "timeline" from day one; the sim accepts the option in M1 | a; yes; yes | rpg_kit gap 2; rules_engine 8; gameplay_loop gap 2 | user |
| Q-53 | Class names per world (Vanguard for `warrior`) | (a) new class rows for the new worlds; (b) a display name per world over the shared classes | b | rpg_kit gap 4 | user |
| Q-54 | Movement, sight and cover: act-then-move maneuvers, line of sight (smoke, ranged chase, the Bulwark), cover, default recovery | act-then-move: only the existing first-in-round jump in Act 1; line of sight: smoke gives cover guard in Act 1, a kit line-of-sight check in Act 2; cover: only the existing cover guard (the brigandine's half-cover trait and "full cover" pillars dropped); recovery: 0 by default, set per skill | as listed | combat_and_timeline 6, 10; items gap 2; class_vanguard 5 | user |
| Q-55 | Combat hooks the maneuvers need that do not exist: forced movement, ground hazards a skill leaves, auras, poise-break triggers, destructible terrain, fixed-block arena hazards, HP-threshold traits, execute riders, cadence kills, item checks | (a) the first playable uses existing statuses and phases only; kit tasks start with forced movement and ground hazards; (b) build them all before Act 1 data | a | enemy_bestiary gap 2; combat_and_timeline 6 | user (the planner orders the tasks) |
| Q-56 | Effects that leaned on cut rolls: Blind, flash-bombs, searchlights, Deafened, smoke cover, ambush openings, the 50% hallucination, slip checks | (a) rebuild each on existing levers (a range cap, poise damage, AP loss), numbers by the balance lane; (b) drop them | a | world_geography 4; upper_bastion 2.2, 4.1; mantle 4.3; factions 2.2; enemy_bestiary gap 3 | user |
| Q-57 | New statuses: corrosion, blind, mark, frenzy, mutilated, disarmed, Flammable, Arterial Rupture, the Pinned release rule; Kaelen's "disarmed" -50% exceeds the `weaken` cap of 25% | each becomes a new row or maps onto one of the fifteen, decided with the data | - | classes_and_skill_trees 3; enemy_bestiary 3; class_vanguard 4 | data authoring |
| Q-58 | Skills whose identity was a cut rider (Ghost of the Undercroft, Viscous Shadow, Cinder-Child's Reflex) and the Crag Hawkeye reveal | (a) rebuild on `exploit` bonuses, the attack chase and intent foresight (Hawkeye: the foresight reveal of enemies in his range); (b) replace the skills. Either way no Act 1 trait may use a founder's taint mechanically (Hollis's eye is backstory) | a | classes_and_skill_trees 6; class_gutter_stalker 5; class_arbalestier 4 | design lane |
| Q-59 | Skill conventions: flat damage and heal values; skill points and drills; Inoculated skills in family lists; graft slot limits; the `riposte` kit skill under "timeline" | flat values convert to `power_mult` at the asset pass; no skill points (tiers by level, drills a small Hearth bonus); Inoculated skills are `learned_only`, taught by the graft recipe; one graft slot holds one organ and teaches one skill; `riposte` reviewed with the data | as listed | classes_and_skill_trees 6; skill_acquisition 3, 5 | data authoring |
| Q-60 | Board size: the board is 12 x 9 cells; the old encounter layouts ask for 12 x 10 up to 18 x 12 | (a) fit Act 1 arenas to 12 x 9; (b) a larger board option | a | enemy_bestiary gap 1; bestiary_encounters_act1 | user |
| Q-91 | What the timeline shows of enemy plans without Tess's reconnaissance (today's intent tells show each enemy's first action to everyone) | (a) the existing tells plus impact badges by default, the full plan (speed tier, impact block, movement) only with the reconnaissance unlock; (b) the full plan always | a | background/npcs_and_questlines.md 2.3, 7 | user |
| Q-61 | Where Maren's Bile Phial is stored | (a) a pouch slot; (b) a trait counter | b (keeps the pouch for consumables) | class_heretic_acolyte 4-5 | design lane |

## Strain and Calcification

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-62 | Strain above 100 within a round, and Strain after a crisis | (a) cap at 100; set to 75 after the crisis; (b) no cap; reset to the floor | a | CANON 2.2; combat_and_timeline 10; magic gaps | user |
| Q-63 | The Strain meter as kit or game code | (a) a generic per-unit meter with thresholds, triggers and decay (kit); the crisis table game code if triggers cannot roll a table; (b) all game code | a | rules_engine 8; rpg_kit 5 | user |
| Q-64 | Lasting cost of the Inoculated keystones (their old Calcification-per-battle cost is retired) | (a) the floor lock at 30 is the whole cost; (b) installing one counts as Tier III surgery (+1 stage) | b | classes_and_skill_trees 2, 6 | user |
| Q-65 | Enemy Strain, and enemies with Strain riders in Act 1 (Quell, the Marrow Horror, the Amalgam) | (a) aberrant enemies carry the meter from Act 2, humans ignore it; Act 1 versions without the riders; (b) no enemy Strain, swap the riders | a | classes_and_skill_trees 6; enemy_bestiary gap 4 | user |
| Q-66 | Aline's marrow dose at ENC-EVT-05: how it is given and what it does in the fight | delivery: (a) a one-use pouch item at deployment, (b) a one-fight buff through the encounter's start status; effect: the old "+100% damage that ignores armour for 1 round" waits for the balance lane | b | six_founders 3; sunk_ward 3.6; bestiary_encounters_act1 ENC-EVT-05 | user, balance lane |
| Q-67 | Balance risks to measure once the "timeline" ruleset runs in the sim: Inoculated dominance in short fights, a tight Strain economy, Flesh-Tension as a trap, Heavy Winch Sunder's forced break | sims per class and per encounter | - | risk_reward 8; class_arbalestier 5 | balance lane |
| Q-68 | Small Strain picks: Stage III "-15% ranged damage beyond 3 tiles" (replaces an accuracy roll); Spore-Bloom's "floor -15" read as for the rest of the battle | keep both | keep | magic gaps; class_ichor_alchemist 5 | user |

## Items

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-69 | Quenched Cinder Salt overlaps the Rock Salt flush | (a) fold it into the flush; (b) keep it as the one-charge "prevent one Calcification advance" item | follows Q-43: b if the flush lowers the floor, a if the flush cancels an advance | items 4, gap 3 | user |
| Q-70 | Reach 2 melee (billhooks, the Flagellant's scourge); the engine's melee reach is 1 | (a) reach 1 plus range-2 skills; (b) a reach-2 rule | a | items gap 1; combat_and_timeline 6 | user |
| Q-71 | Missing weapon families: billhook or polearm, chemical sprayer, bone-saw, censer | (a) stand-in families; (b) new families as data | b | items 2, gap 6 | data authoring |
| Q-72 | Area shapes for "behind the target" and lines | checked against the engine's `Area` shapes when the data is written | - | items gap 4 | data authoring |

## Enemies

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-73 | Prologue rosters: 0a names Castellan Footmen (fixed in this pass); 0b's "ambush squad" and 0c's "3 Inquisitors + a crossbowman" name no bestiary unit | (a) 0b Mortuary Flagellants; 0c Castellan Footmen and 1 Watch Crossbowman; (b) prologue-only units | a | enemy_bestiary gap 5; CANON 3.1; bestiary_encounters_act1 | data authoring |
| Q-74 | Boss gaps: Quell's marrow valve (a Strain trigger in Act 1, a cheat-death trigger); Nemeth's trait, maneuver and capstone (after Q-25) | written by the design lane | - | bestiary_iron_inquisition; six_founders 6 | design lane |
| Q-75 | The Crucible winch: how the stay-behind is played | (a) scripted in dialogue after the fight; (b) a kit "hold this tile, the chosen hero stays" objective | a for the slice | bestiary_encounters_act1 ENC-EVT-06; sunk_ward 3.7 | user |
| Q-76 | Act 2-3 adversaries and Awakening Chorus troops have no maneuvers; the factions of Lazar, the Bell-Warden Colossus, the Reliquary Guardian and the Chasm Gatekeepers are open | written by the design lane before Act 2 data | - | enemy_bestiary 4, gap 6 | design lane |

## Art and assets

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-77 | The palette step rule | (a) computed (dark 65% value, light 35% toward Bleached Linen), hand-picked where it reads badly; (b) hand-picked per hue | a | specs/art_and_visual_style.md gap 1 | user (asset lane applies) |
| Q-78 | One skin ramp for a varied cast | (a) add a light and a dark ramp; (b) one ramp, vary by recolour | a | art gap 2 | user |
| Q-79 | Units larger than 48 x 48 (the Amalgam, the titan) | (a) 96 x 96 frames on the same scale, through the image API (asked first); (b) scaled 48 x 48 bodies | a | art gap 3 | user |
| Q-80 | The Ashlands kit bodies' palette | (a) the Low Sleep palette applies per world through palette files; (b) it replaces the kit palette | a | art gap 4 | user |
| Q-81 | The preference "per-world art waits until the Ashlands are complete" against M2's Sunk Ward placeholders | the user confirms the placeholders may start | - | art gap 5 | user |
| Q-82 | A camera shake for the bell tolls | checked with FX1; a new `[engine]` feature if missing | - | onboarding 4.1, 6 | logic lane |

## Slice scope

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-83 | The slice's district nodes | (a) 01, 03, 04, 06 with 07 as the Crucible's exit map; (b) 01, 02, 04, 06 | Decided (2026-10-25, user): a; the slice is for playtesting the feel of the UI and gameplay, planned around the whole game | specs/full_game_systems_roadmap.md gap 1 | user |
| Q-84 | The slice's mission list | (a) the roadmap's: 0a-0c, the hearth, M3, one dilemma, M5; (b) the narrative writer's: 0a-0c, M1-M3, the interlude, M5 (M4 cut) | Decided (2026-10-25, user): a | roadmap 1; campaign_narrative 8 | user |
| Q-85 | The slice's one dilemma | (a) one Act 1 board quest given a two-way choice (for example Corvo's gate); (b) the ENC-EVT-05 marrow dose | Decided (2026-10-25, user): a | roadmap gap 2 | user |
| Q-86 | Where the rest of Act 1 lands | (a) an M2b of data tasks before M3; (b) inside M4 | a | roadmap gap 3 | user |
| Q-87 | The slice's end | (a) an end card after the Crucible; (b) the Act 2 hub reveal | a | roadmap gap 4 | user |
| Q-88 | Prototype order, mission sites, vignette scope | arena first on ENC-PRO-0A in the Scene Runner (rec) or the vertical slice; main missions on region maps, road fights from nodes; vignettes as still cards, typewriter text and fades | as listed | gameplay_loop gaps; screen_architecture gap 3 | user |
| Q-89 | Battle barks | (a) in the slice; (b) out of scope until after the slice | b | narrative_agency 1.1 | user |

## Process

| Id | Question | Options | Rec. | Where | Decides |
| --- | --- | --- | --- | --- | --- |
| Q-90 | Ids for the `sunk_ward` factions, locations and units | set by the data authoring task from the cast registry (ids never follow a rename) | - | specs/worldbuilding_decisions.md 4; naming_and_asset_guide 1 | data authoring |
