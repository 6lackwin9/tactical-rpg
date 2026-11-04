# INTENT2: what a round intent means, and how every kind of skill and item behaves under it

Status: proposed; open points are in section 7. Written 2026-11-05 (design lane). Depends on: ATTACKLOCK1. Lane: logic, asset, balance. Touches: section 8. Listing of 129 skills and 6 consumables: `games/emberwake/docs/specs/INTENT2_SKILLS.md`. Feeds SKILLPASS1.

Already decided by the user (2026-11-05), taken as given here: a pouch has 3 slots, each a stack, no refill in battle, restocked only from the stash; items cost blocks, not AP, for burst rounds; Rest is the fill and AP management, not a rival; one catalog item shows in each allowed intent's row with one stack, and the row sets its behaviour; skills may list several intents too (heal on yourself defensive, on an ally supportive), at most 2, capstones 3, each possibly a version of its own; the new intent is called Field; Field versions earn their place with more range or a larger impact area; the intent comes from the target, and planning loads the neutral row and that intent's row.

## Goal

The user's idea: "the intent tells what your target is for the round and then all your skills work around that target". Today 88 of 129 skills are Assault, and area skills sit there with a tile or a unit as target. This spec makes the target the only meaning of an intent, adds a tile intent, and tests it against every skill and item. Where the idea gives a bad result it says so:

- **A heavy sweep under Guard is counterintuitive**: a 2.2 blow behind a marker that reads "safe". Rings go to Guard, aimed swings to Field (Q2).
- **SWEEP does not work today**: the planner refuses it on every target (section 9), so its bosses never swing. A live bug whatever is decided.
- **"The intent comes from the target" holds for three clicks of four** (section 1, IR19): yourself, an ally and an empty tile each load one row, but an enemy is a target for both Assault and Maneuver, and an occupied tile can also be a Field anchor. Show those rows; nothing breaks, and the change is small because the menu is already target first (IF1, IF2, `pick_options`).
- **Several intents make purpose harder to read in three places**: (1) "Guard" now spans Riposte, Focus, Taunt, rings and self-heals, so the ring says "this round is about itself", not "safe to approach" (Q1); (2) one icon in two rows can look like two skills, so the tip must name both rows; (3) a healer who self-heals beside Riposte no longer pays IN6's price, today's reason to choose (IR16).

## 1. The framework

Rule ids IR. New behaviour is a rule option; classic picks the legacy option (section 8).

- IR1. Three separate things. The **intent** names the round's target. The **shape** names which cells relative to that target. The **effect** (damage, heal, status, hazard, forced move) is independent of both. Area is not an intent.
- IR2. One test: **what is it aimed at, and what follows if that thing moves?** An enemy, the skill follows it: Assault. A tile, nothing follows: Field. You, the skill moves with you: Guard. Another friend, the skill follows them: Support. A place you end up, re-read around its reference: Maneuver.
- IR3. Tie-breaks: (a) when the point of the skill is where you end up, it is Maneuver, even with a tile aimed at; (b) a target of you is always Guard, whatever the effect; (c) Move and Rest are neutral. Side effects never change the answer: Shield Bash is Assault.
- IR4. One target a round (TG1 stays). A Field round has one anchor tile for all its blocks, so "this region is dangerous this round" is one fact to read. Moves are not a target (TG2). Guard's tile target (GD1) is a plain Move here, then a self target.
- IR5. Shapes are allowed in any intent; the intent picks the anchor: the target's tile at impact (Assault, Support), the anchor tile fixed at planning (Field), your tile at impact (Guard). A shape needing a direction takes it from you toward the anchor at planning, so Guard shapes have no direction.
- IR6. Shapes (`area`): `PLUS`, `BURST`, `LINE` as today; `RING` (new: the 8 neighbours of the anchor); `ARC` (new, replaces `SWEEP`: 3 tiles aimed at any of the 8 neighbours: an orthogonal aim gives the row across the heading, as the hazard shape already does, a diagonal aim gives today's L). A LINE takes its direction from any anchor in a straight line (today: only an adjacent tile).
- IR7. Drawn shapes. LINE and ARC are drawn from you, fixed at planning; if you are not on the planned tile when the impact lands, the swing misses (`[Area Miss]`, AP spent). Thrown shapes (PLUS, BURST at any reach) land anyway. So a marked tile is never a surprise (decision "Attack lock at wind-up"), and shoving a boss cancels its wall.
- IR8. Commit: as the matrix; melee and ranged Assault are ATTACKLOCK1 unchanged.
- IR9. Friendly fire. Field: the skill's new `affects` (all, foes, friends; default all, as today). Assault splash and Guard ring: foes only, since they are aimed at an enemy or at you. Support shape: friends only.
- IR10. Follow-ups: FU2 unchanged (same intent, same target, no debt). In Field the auto-fill places each family once a round (`field_fill_once`): a repeat on tiles the foes have left is the player's call. What cannot fill is Rest.
- IR11. Items follow the same test. The first pick fixes the round's intent, skill or item, so an intent needs no skill. Items cost no AP (`item_ap` 0; today 1), only blocks: a strong item takes longer or stacks smaller. The planner never spends an item it was not told to (FU2 e stays, for the ghost, Fill, auto-fill and tactics): the stack is real.
- IR12. Stack spend. A foe or tile use spends one at its impact whether or not it finds anyone (the foe left reach or hid, the tile is empty). Nothing is spent when the command never starts (TG4: target downed or withdrawn first, it became Rest), when the user is down, when its wind-up is broken (P7), or for a friend use whose friend is down or out of reach at impact (nothing was poured). A foe downed after the throw started counts as a miss. Code today: the first three agree (`into_rest`, resolver); a foe out of reach at impact does not, it logs "not used" and keeps the item (`command.rs:605-612`), and a hidden foe is hit (no GD5 check on items): both change.
- IR13. Channel is not a target kind, so the test finds nothing for it. It stays the hidden slot (`target_problem` accepts anything, the planner refuses it); the meter spec must answer the test, most likely a self target: a Guard round with a meter rule rather than a sixth meaning.

Several intents (user 2026-11-05; IN4's "exactly one" gives way):

- IR14. **A skill or item has an authored list of allowed intents. It lists a second only when all three hold**: (a) the effect is the same, only the anchor changes (a version may adjust numbers, IR18, never the effect); (b) its fields already aim there (`target_mode` ALLY_OR_SELF, an item's reach, a thrown shape); (c) neither row dominates: a row that follows a unit beats a row fixed on a tile, so an Assault + Field pair needs IR17. Against 129 skills: 12 list two: 8 SUPPORT with ALLY_OR_SELF (Mend, Greater Mend, Purify, Aegis, Field Dressing, Cauterize, Tonic, Resilience Ward), 4 thrown area skills (Firebomb, Inferno, Glacial Burst, Flash Bomb); 117 list one, 93 of the 97 ATTACKs included.
- IR15. Rows. The row a skill is picked from sets its anchor and version. So Support's target narrows to another friend, and a self target is always Guard. Fill and order: `deck.order` and `follow_ups` may hold a skill in both lists (`deck_errors` reads the allowed list). The auto order holds only skills whose primary is that intent, then basics: a listed second row is by pick or by the player's order, or a healer's auto Guard round would heal forever instead of guarding. AI: row 2 picks its target first; itself gives a Guard round with the heal as opener, an ally a Support round. Same heals as today; the tell reads Guard.
- IR16. Guard rounds. A self-heal or self-buff now keeps intent-bound buffs: today Support on yourself drops Riposte (IN6), under this it does not. A guard holds only while a Defend runs, so heal blocks are unguarded. Could it be too strong: Riposte + self-heal + Fortify. No shipped class path holds both (warrior has Riposte, priest heals); Maren can do Tonic on herself + Active Guard (guarded 2 blocks of 6). The risk is stalemates, already behind `tl_skirmish`'s 24-25 % capped fights. Knob: an override lowers the Guard row's heal. SKILLPASS1 and the feel run watch it.

- IR17. **Thrown area in both rows: what makes each worth choosing.** Under the ranged rule an Assault burst follows its target; Field is dodged by walking. Unequal values fix that, from rule values so that these rows need no authoring (`assault_splash` {`area` PLUS, `splash_mult` 0.5}, `field_reach_bonus` 1; classic 0). **Assault row**: PLUS centred where the target stands at impact, splash x 0.5, status on the target only, no hazard, foes only, authored reach, no chase (decision "INPUT1-M rule choices": area skills do not chase), single-target follow-ups allowed. **Field row**: the authored shape at full power and status on every cell, `reach_max` + 1 (every thrown Field row, Field-only too), marked, hits hidden units, keeps landing when the target dies, may centre on empty ground; it can hit friends and only Field follow-ups can join it. Of 26 area skills: both rows 4; Field only 19 (hazards: Phosphorus Pot, Choke Gas, Nozzle Sweep, Grease Pot; the trap Tripwire Snare; drawn: 3 LINE, 4 ARC; Volley, whose Assault row would only be a weak Aimed Shot; enemy bursts: Lard Spray, Transfusion Pulse, Pulverizer, Cataclysm, Spine Volley, Bone Bloom, because a boss's area must telegraph); Guard only 3 (rings).
- Friendly fire with a moving shape: an Assault row's cells cannot be marked at planning, so hitting a friend would be unfair: foes only (IR9). That is a real gain over Field, paid for by the smaller shape and half splash. Test: Kaelen beside the target: Assault row, Kaelen unhurt; Field row with him in the marked cells, hit.

| Walks up to 2 tiles on an open board; tiles that escape | Field BURST, centre | Assault band 2-3, no chase | same, with chase | band 2-4, no chase | same, with chase |
| --- | --- | --- | --- | --- | --- |
| Target at distance 3 in line (of 12 tiles) | 4 | 9 | 5 | 6 | 0 |
| Target at (2, 1) (of 12) | 4 | 7 | 3 | 5 | 0 |
| Target at distance 2 in line (of 11) | 3 | 6 | 0 | 1 | 0 |

By script. The fear that it cannot be dodged holds with the chase or a wide band, so the Assault row has neither (reach stays authored, at most 3): then leaving the band is as easy as leaving the cells, though a mover must know the caster's band. Round 9 gives the output.

- IR18. Limits and versions. A skill lists at most 2 intents; a capstone 3. Capstone: a new family flag `capstone` (default false, content check: at most one in a deck's `active`), as `grade` means class-path progress and a boss's capstone is unlocked by a phase. No row needs 3 now. Per-intent version: `intent_versions`, a map from intent to differences, only from `area`, `reach_min`, `reach_max`, `power_mult`, `splash_mult`, `blocks` (`use_blocks` on items), `status_splash` and `hazard` (off only). The check allows at most 3 keys a version and warns when `power_mult` differs from the primary by more than 50 %. Name, icon, kind, AP, status and riders are shared. A version needing more is two skills. Balance tunes one row of numbers; rule values cover most rows, so authored versions should stay rare: the bomb's Assault row (one target) is the first.
- IR19. **Target first; the target loads the rows.** IN2's picked, persisting intent ends; the first pick of the round sets it, as the code does today (`pick_intent`, `Session::pick`).

| Click | Rows beside the neutral row (Fill, Rest, Move) | Notes |
| --- | --- | --- |
| Enemy | Assault, Maneuver; Field if the unit holds a Field skill or item | Both aim at that enemy, so the first pick decides. A position pick (clicking a flank tile) would clash with Move and Field on the same tile: not proposed |
| Empty tile | Field | Move here is the neutral row's. After it the menu stays open with the Guard row, target yourself, for what today's combined row does |
| Ally | Support; Field if held, anchored on that tile | A click on a unit means the unit; the Field row is how to aim at its tile |
| Yourself (Self button, SL4) | Guard | Support's self rows are gone: a heal shows here |

What follows from it. **Persisting intent**: kept only as "last round's intent", read for IN6 (a first pick of another intent drops bound buffs, warned on the row) and for the ring before any pick. **TG6**: stays for the ghost and hotkeys. **Party cards and the ring**: the planned intent (pick, else ghost), dim while a ghost. **Ghost, tactics**: a member with no pick shows its tactic's plan, as now; a tactic names target and intent. **Fill on a target with two rows**: last round's intent when live, else the row order (`intents_for`: Assault, Maneuver). **AI and tells**: unchanged, they pick both and the plan carries the intent. **Keys**: 1-4 party; Shift+1-9 stay fixed per hero (HK1), so memory holds; a key whose row is not loaded is refused; self-only keys target self (HK3). **An intent button**: not needed; it left with IF8, and the ring and rows say the rest. **VISCLAR1-D slice 2**: reusable: same icon rows in fixed slots, nothing shifts; only the loaded rows draw, then others fade once the opener fixes the intent as approved. Tell lane 2 before it starts that row: a row count from the ruleset's `intents`, lit by `pick_options`.

How the list settles the hard cases: **self-or-ally heal and buff: resolved. Heal around you or an ally: resolved (one RING, two anchors). Guard on an ally: resolved (a Defend listing Support). Items: resolved. Splash and the mage's mix: resolved by IR17. Sweep: not resolved: ring and arc are different shapes, so it fails IR14 a; Q2 stands. Focus: not resolved: self-only, so it never joins an attack round; it pays next round.** Tile-aimed teleport: unchanged, Maneuver.

Where the test is not one clear answer (10 of 129, flagged): gap-close Assault (Lunge, Pounce, Charge In, kept by decision "Gap-close"); Intercepting Hook (guards any friend within 1, aimed at none); Orlov's Retribution (a retreat after the hit); reach 1 or 0 bursts that read as "around me" (Transfusion Pulse, Pulverizer, Cataclysm, Spine Volley, Bone Bloom). 31 more need a named rule, 9 feel wrong until tuned. SKILLPASS1 settles those rows; none blocks the framework.

## 2. Behaviour matrix

Table A: target and timing.

| Intent | Valid round target | Shape anchored on | Moves before, after | Chase | Commits | The other side avoids it by | Target gone or moved |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Assault | an enemy | its tile at impact | approach, kite (AS1-3), after-move | 1 tile (MR16); area rows and items none | melee: its tile at wind-up start, after the chase; ranged and thrown: in band at impact, shape read then | leaving the locked tile or the band, hiding, breaking the wind-up | down first: Rest (TG4); left: miss, AP and stack spent |
| Field | a tile in the arena | it, fixed at planning; drawn shapes from your planned tile | one regular move first, or Step here after | none | planning, every shape | walking out of the marked tiles; shoving the caster off a drawn shape | never fails; empty cells `[Area Miss]`; hazards skip unwalkable tiles, traps occupied ones (GH2) |
| Guard | you | your tile at impact | Move here first, or Step here after | none | impact | leaving your neighbours before impact | cannot fail |
| Support | another friend | its tile at impact | approach into reach (SP1) | 1 tile (MR16); items none | in reach at impact | killing or pulling the friend away | down first: Rest; out of reach at impact: not landed, AP spent, item kept |
| Maneuver | a position by an enemy, or away from it, or a tile (a jump) | none | the dash or jump rides in the skill; regular move only beyond range (MR7) | inside the dash (MR8) | destination at the wind-up tick; then the melee lock | filling the position tiles; leaving after the dash | `[Flank Protected]`, `[Dash Blocked]` as MR8; down first: Rest |

Table B: interplay and display.

| Intent | Friendly fire | Riposte | Stealth | Flank | Follow-up fill | Items | Player sees in planning | AI picks it when |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Assault | none (IR9) | melee at distance 1 answered | hidden target: the whole command misses | hits flank (MV3); engaged, so it can be flanked | FU2 on the unit | a bomb on a foe | intent ring, target arrow, locked tile (ATTACKLOCK1) | row 5, as today |
| Field | `affects` | a melee area hit is answered as today (`command.rs:494`) | hits hidden units | not engaged, cannot be flanked; its hits flank as today | same anchor, each family once, then Rest | a bomb on a tile | marked cells from the first pick, line to the anchor; enemy Field rounds marked too | new row Z |
| Guard | none (IR9) | your own Riposte holds beside a ring or heal (IN6) | a ring does not end it | not engaged, so never flanked and Riposte always answers | Guard order; a guard only while a Defend runs | elixirs; a potion on yourself | ring preview on neighbours while hovered | rows 3 and brace; row 2 on itself; new ring row |
| Support | friends only | none | a hidden friend can still be helped | none | same friend (SP2) | a potion on an ally | arrow to the friend | row 2 on another |
| Maneuver | none | a flank hit gets none (MV3) | no dash at a hidden target (GD5) | the point of it | MR6, decision "Auto-fill keeps a flank" | none (Utility retires, Q5) | position plates (MR13) | row 4 |

AI rows. Row Z, before row 5: the unit has a Field skill; its best anchor (extends `flush_pick`) covers at least `area_threshold` foes, for an elite or boss at least the first ranked one, passes `friendly_fire_ok`, and the unit did not use Field last round (so it alternates; AI3's order lists stay). Ring row, before row 4: two or more foes adjacent and a ring usable. Reckless drops Defend skills, not the Guard intent, or a reckless boss never flails. A taunted unit picks only a Field anchor covering the taunter. Enemies pay no AP: recovery stops repetition, as a recovery past the round is refused. Enemies use no items (`pouch` ["none"], `use_items` off, sim only).

Tips and forecasts. A dual tip names both rows: "On an ally: Support. On yourself: Guard, keeps Riposte." IF3's "loses Riposte" reads the row, so Mend on an ally warns and Mend on yourself does not. Forecasts read the planned command and do not change; the strip colours by the round's intent. Skill text that opens "Support, Standard:" drops the intent word on dual rows.

## 3. Archetypes

Verdict: C fits cleanly, R fits with a rule, X counterintuitive. "Test" is a builder's test; "plausible" rows have no catalog skill yet.

| # | Archetype (example) | Intent | In a round | Test | V |
| --- | --- | --- | --- | --- | --- |
| 1 | Melee single (Strike) | Assault | locks the tile at wind-up | target steps off before impact: miss, "Dodged", AP spent | C |
| 2 | Ranged single (Aimed Shot) | Assault | no tile lock | target stays in band on a new tile: hit; leaves band: miss | C |
| 3 | Reach 1-2 (Hook Thrust) | Assault | melee lock, band 2 | target steps inside the band but off the tile: miss | R |
| 4 | Gap-close (Lunge) | Assault | dash only when out of band | dash, then lock; plain Move dodges after the dash | R |
| 5 | Position dash (Flank Cut) | Maneuver | MR1-MR8 | positions full at start: `[Flank Protected]`, AP spent | C |
| 6 | Hit then retreat (Orlov's Retribution) | Maneuver | hits, then a dash away | ends farther from the target than it began | R |
| 7 | Jump (Backflip) | Maneuver | aims at a foe, lands farthest | rooted: refused; follow-ups none, Rest | C |
| 8 | Teleport to a tile (plausible) | Maneuver (IR3 a) | tile target, no follow-up can act | Field skill refused after it; 5 Rest blocks | X |
| 9 | Burst, Assault row (Firebomb) | Assault | PLUS around the target at impact, splash x 0.5, foes only, no chase | target moves 1 in band: cells move; ally beside it unhurt; target leaves band: miss | R |
| 10 | Burst, Field row (Firebomb) | Field | BURST, fixed, reach + 1 | target walks out: hits nobody, AP spent, cells unchanged; friend in cells hit | R |
| 11 | Enemy-only burst aimed at a foe (Lard Spray) | Field | a click on a foe, the Field row: the foe's tile is the anchor; it will not follow | target is a tile; a later Strike is refused | X |
| 12 | Burst at reach 0 (Bone Bloom) | Field | may centre on the caster, stays if it is pushed | push caster 1: cells unchanged, caster unhurt | X |
| 13 | Reach 1 burst with pull (Cataclysm) | Field | fixed cells, pulls those hit | each unit hit pulled 1 toward the caster | R |
| 14 | Line (Execution Wall) | Field | drawn from the cast tile | caster pushed before impact: `[Area Miss]` | R |
| 15 | Heavy aimed swing (Gilded Sunder) | Field | ARC, marked, 3 blocks | aim (6, 4) from (5, 4): (6, 3-5); aim (6, 5): (6, 4), (6, 5), (5, 5) | R |
| 16 | Light ring (Sweep Kick) | Guard | RING read at impact, foes only | foe and ally adjacent: foe hit, ally not; caster pushed: ring moves | R |
| 17 | Heavy ring (Judgment, if ringed) | Guard | 2.2 power under a Guard marker | not built: Q2's option b | X |
| 18 | Ring beside Riposte | Guard | both run, no guard while the ring runs | round 3: 2.6 power from a Guard round | X |
| 19 | Defend, Riposte, Stealth (Active Guard, Brace Counter, Fade) | Guard | hold, counter, hide | side melee hit answered; moved and stealthed: single-target and thrown miss, Field hits | C |
| 20 | Hazard under you (Ash-Cloud Fade) | Guard | placed at start, stays when you walk | walk out: hazard stays, stealth stays with you | R |
| 21 | Taunt (Taunt Shout) | Guard (was Support) | self-only | taunted enemy anchors Field only over the taunter | R |
| 22 | Focus | Guard (was Support) | boost this round, attack next | no attack in the same round; boost on in next round's first block | X |
| 23 | Guard on an ally (plausible; T4 Bodyguard deferred) | Guard, Support | the row picks the anchor | ally steps 1: chases; on yourself: no chase | R |
| 24 | Intercept (Intercepting Hook) | Guard | self target, redirects a hit within 1 | no friend near: plain guard | R |
| 25 | Heal one (Mend) | Support, Guard | ally row or self row | self row keeps Riposte; ally row warns "loses Riposte" | R |
| 26 | Heal beside Riposte (Brace + Mend, learned) | Guard | keeps both, heal blocks unguarded | fight at cap with both units Guarding: counted by the feel run | X |
| 27 | Heal around you or an ally (plausible) | Guard, Support | RING anchored by the row | self row: ring moves with you; ally row: follows the ally | R |
| 28 | Heal on a tile (plausible) | Field, `affects` friends | ground helps friends | foe on a cell unhealed; needs Area on non-attacks (M) | R |
| 29 | Trap (Tripwire Snare) | Field | one tile, unoccupied | aim at an occupied tile: refused at the pick; then Rest | X |
| 30 | Hazard burst (Phosphorus Pot) | Field | hit, then ground | burn ground stays after the caster leaves | C |
| 31 | Forced move (Slag Slam) | Assault | push leaves the band | next follow-up chases 1 | C |
| 32 | Leech, HP cost, rider, debuff (Martyr's Rush, Brand, Intimidate) | as the aim | effects never change intent | HP cost refused at low HP; a no-damage hit still misses if the target left | C |
| 33 | One-skill enemy (Husk); snap with no recovery (Brand) | Assault | Slam 2 + 1 twice; Brand 6 a round | fill: 2 Slams, no third; Brand: data adds recovery 1 | X |
| 34 | Potion, either row | Support, Guard | item opener, 0 AP | ally out of reach at impact: not used, stack kept; self row beside Active Guard | R |
| 35 | Bomb, either row | Assault, Field | one foe, no chase, band at impact; or PLUS on a tile | foe leaves reach: miss, spent; foe down first: Rest, kept; tile empty: spent; Kaelen's Field row holds only the bomb | R |
| 36 | Pouch as kit (a healer's bombs) | any | the pouch reaches intents the tree lacks | Maren with bombs opens Field; fill finds no skill: Rest | C |
| 37 | Burst round | as the opener | AP on skills, then items | 0 AP left: a bomb is still offered; no AP debt | C |

## 4. Worked rounds

Six blocks, AP 10, move 2 tiles a block, ATTACKLOCK1 on; item values are section 6's first pass.

1. **Mixed party.** Kaelen, Assault, Footman A: Pommel b1-b5, Rest b6. Maren, Support, Kaelen: Tonic b1-b2, Cauterize b3, Rest b4-b6. Vesper, Maneuver, Footman B: Sever (dash) b1, Sever b2-b5, Rest b6. Hollis, Field, tile (8, 3): Wire b1, Rest b2-b6. Four rings, nothing mixes inside a unit. Hollis is the cost: a wire and five Rest blocks, where he shoots too today. Round 5 is the fix, with no new skill.
2. **A boss's telegraphed Field round.** Orlov anchors (6, 4); Lard Spray (BURST, 2 blocks, 2 recovery) marks (5-7, 3-5) at planning. b1-b2 Spray lands b2; b3-b4 recovery; b5-b6 Rest (a second Spray's recovery would pass the round). Kaelen on (6, 4) walks to (6, 6) in b1 (2 AP): safe. Vesper on (7, 5) picks Sever: its dash takes her out free. Maren on (5, 4) keeps healing and takes 0.9 power and burn, her choice with the marks in view. A friend of Orlov's in the cells is hit too. Next round row Z does not repeat: Hook Drag on the first ranked. With `field_reach_bonus` its reach is 2-5.
3. **A Guard round with a ring.** Deck [Brace Counter, Sweep Kick], two footmen and an ally adjacent. b1-b2 Brace Counter (guard 0.3, Riposte on); b3-b4 Sweep Kick lands b4 on both footmen, not the ally, no guard meanwhile; b5-b6 Active Guard. Footman A's hit lands b2: guarded, countered 0.5. Footman B's lands b4: full damage, countered 0.5. Output 0.8 x 2 + 0.5 x 2 = 2.6 power, against 3 Strikes = 3.0 on one foe in Assault, guarded in 4 of 6 blocks and impossible to flank. Strong when surrounded, weak otherwise.
4. **Dodges.** Footman A starts Thrust (3 blocks) on Kaelen in b1: the tile locks at b1's start. Kaelen's Move in b1 lands at b1's end: b3 misses, "Dodged", and he attacks from b2 (a Move must come first, so this is his only time). Had the Thrust started in b2 the chase would re-lock and hit. Footman B starts Thrust on Vesper in b1; she picks Rest, Rest, Sever: her dash at b3's tick leaves the tile, her hit lands, B's misses. Same dodge, no Move and no AP beyond the skill.
5. **Items outside the skill tree.** Kaelen, Field, tile (8, 4), no Field skill: Bomb b1, Bomb b2, Rest b3-b6; 0 AP. Vesper, Guard, self: Fade b1, Draught b2 (0 AP), Active Guard b3-b4, Rest b5-b6. Hollis, Support, Vesper: Move b1, Poultice b2-b3, Poultice b4-b5, Rest b6; 120 HP for 2 AP. Maren, hurt, Guard, self: Tonic on herself b1-b2, Active Guard b3-b4, Rest: the skill that was Support on Kaelen in round 1. Hollis's Wire then a Bomb in one Field round mends round 1.
6. **Burst round.** Hollis, AP 3, Assault Footman: Quarrel b1-b2 (2 AP), Strike b3-b4 (1 AP), Bomb b5, Bomb b6 (0 AP): 0.95 and 1.0 power plus 40 and 40 flat, no armour. The Footman leaves reach in b6: the second bomb misses, spent, no refill this fight. That is the balance: the cap, not the cost.
7. **Mid-round failures.** Kaelen's target dies in b2: b3-b6 become Rest (TG4). Elyse's Field on the same tile keeps landing on empty cells. A bomb at a foe downed before it starts: Rest, stack kept; downed after: spent. A stealthed foe that moved: Strike and bomb miss, Spray hits its tile.
8. **A boss across rounds.** The Amalgam (reckless, no AP). R1 Field Spine Volley on the party's centre: b1-b2, recovery b3-b4, Rest. R2 two adjacent: Guard, Flail (ring) b1-b2 + 1 recovery, Flail b4-b5 + 1: 0.9 x 2 foes twice. R3 Field. Row Z and the ring row make the pattern with no authored sequence (AI3).
9. **Why a mage picks each row.** Firebomb (0.6), mage at (7, 2). Cluster F1 (6, 4), F2 (7, 4), F3 (6, 5), F4 (7, 5), engaged with Kaelen on (5, 4). Field, anchor (7, 4): 3 x 3 hits all four, Kaelen clear: 0.6 x 4 = 2.4 with status on each, then a second Field skill on the same anchor (Glacial Burst, 0.6 x 4): 4.8 and all rooted, if they stay. Assault, F2: 0.6 + 0.3 (F1) + 0.3 (F4) = 1.2; F3 is diagonal; then Arcane Bolt on F2: 1.1. Total 2.3, of which 1.7 on F2. One foe alone: if he steps off the tile but stays in band, Field 0, Assault 0.6 + 1.1 = 1.7; if he stays, Field 0.6 alone. So Field wins a cluster that stays, on range too (it reaches F1 from a tile Assault cannot), and Assault wins pursuit and focus: both are worth a pick. An enemy approaching from far is what Assault is for: Field must guess its tile.

## 5. Winners and losers

Power in one six-block round from today's order lists, by script: AP 10 for founders, no AP and recovery for enemies, no reach or targets counted (Field and ring numbers multiply by those hit). "Today" drops SWEEP (it never fires).

| Unit | Today | Assault | Field | Guard ring | Judgment and fix |
| --- | --- | --- | --- | --- | --- |
| Vane | nothing: order holds only Sunder | none | Sunder 2.0 | - | wins; add `strike` to his Assault list |
| Bailiff Enforcer | Drag x2, 1.0 | Drag 1.0 | Judgment 2.2 | - | wins |
| Grendel | Jab x3, 1.8 | Jab 1.8 | Hammer 2.2 | - | wins |
| Orlov | Spray 0.9 | Drag x2, 1.8 | Spray 0.9 | - | alternates; even |
| Amalgam | Volley 1.0 | none | Volley 1.0 | Flail x2, 1.8 | wins with the reckless rule; add `strike` |
| Quell, Nemeth | 1.4, 2.4 | same | capstone only | - | no change |
| Purifier | Nozzle + Brand x2, 1.7 | Brand x6, 3.0 | Nozzle 0.7 | - | loses its mix; Brand needs recovery 1 |
| Dredger | Grease x2, 0.8 | Strike x3, 3.0 | Grease 0.4 | - | its Strike out-hits its kit; keep `strike` out of its order and let the feel run decide |
| Hollis | Wire x3 (wasted on a unit target), Strike | Quarrel x3, 2.85 | Wire, then Rest | - | clearer, thin Field; pouch bombs fill |
| Elyse | Pot x3, 1.5 | Strike x3, 3.0 | Pot + Flash, 0.7 | - | loses repeats in fill, picks can repeat; ground stays |
| Kaelen, Vesper, Dunstan, Maren | 3.0-3.6 | same | none (Shatter at level 2) | - | no change; Field through the pouch |

By class: the **mage** keeps burst then bolt in the Assault row at a smaller burst, and gains a real Field round (IR17, round 9): a winner, not a loser. The **brawler** wins most: rings give it a real Guard round. The **ranger** loses Volley then shots (Volley stays Field only, reach 2-4). **Priest** gains a self-heal Guard round; **rogue, warrior** barely move (Taunt and Focus to Guard). Every unit keeps Assault and Guard through basics (`strike`, `active_guard`; `flank_cut` for Maneuver), so no class has a dead intent it needs: Support and Field may be out of reach of a unit with an empty pouch, and its menu just shows fewer icons. Enemy fixes are data. **New skills needed before the slice: none.**

## 6. Consumables: the thin-rounds test

**Verified.** 6 consumables. Each: `use_blocks` 1, `ITEM_AP` 1 (`command.rs:1390`, read in `plan.rs:723`, `round_intent_plan.rs:1674`, host `lib.rs:550`), `max_stack` 5 (no kit code reads it). `item_intent` reads damage-only as Maneuver, else Support; `item_opener` needs a unit, `pick_options` lists no item on a tile; `CmdKind::Item` holds a unit. `offered_in_battle` hides STAMINA_BREW and ANTIDOTE types (no rows). `aims_at_foes` makes a damage-less foe item a friend item: the authored list replaces it. No class, drill or `can_wear` gate: `pouch_put` and `UnitState` validation check only the CONSUMABLE type and "offered". Side bug: `buff_pct` always boosts attack, so Brimstone Draught ("+30 % CONS") gives +30 % attack (`command.rs:628-631`).

**Does the pouch answer thin rounds?** Access: yes. With a stack any unit can open an intent its tree lacks: no Field or Support basic is needed for the slice. Filling: no. Auto-fill spends no item, so after an item opener the ghost is the intent's skills or Rest; the player can pick the item again. Whole fight: no, and that is right: 3 slots of 2-3 uses carry a missing intent for 1-3 rounds of a fight capped at 8-10 (`max_rounds`), so a pouch is a spike, not an identity. Units that carry none lose nothing (basics). Enemies are not helped. Cost: stash stock, 1 bomb and 2 poultices at the hearth (CAST1_GEAR 6): the balance lane raises it for the slice; the shop's `count` limits stock already.

**AP and blocks.** Items pay blocks only; AP limits skills, stacks limit items. An item may replace a Rest block; the feel run counts Rest blocks as a plain measure, not a risk to tune.

First-pass values (the user tunes; `use_blocks` and `max_stack` per item through `aegis content`):

| Item | Effect | Allowed | `use_blocks` (today 1) | `max_stack` (today 5) | Why |
| --- | --- | --- | --- | --- | --- |
| `con_heal_potion` | heal 90 | Support, Guard | 3 | 2 | the strongest; time is the price |
| `sulfur_poultice` | heal 60 | Support, Guard | 2 | 3 | at most 180 a round with nothing else |
| `con_focus_elixir` | attack +50 %, 2 rounds | Guard | 1 | 2 | must outlast its round |
| `brimstone_draught` | +30 %, 3 rounds | Guard | 1 | 2 | fix the stat bug first |
| `pitch_fire_bomb` | 40 flat | Assault, Field (PLUS) | 1 | 2 | burst: 80 in 2 blocks, then gone |
| `con_fire_bomb` | 45 flat | Assault, Field (PLUS) | 2 | 3 | slower, more of them |

**Stacks.** Keep `pouch` as a list of repeated ids, so saves and the battle code (`pouch_ids`, `pouch_count`, `items_left`, carry weight) read it as they do; "3" becomes 3 distinct ids, each at most its `max_stack` (`gear.rs:309`, `state.rs:394`, `party.rs:1316,2052`, `dojo.rs:512`, sim `args.rs:687`). Old saves stay valid ([a, a, b] is 2 slots). Add `pouch_packed` (the counts to refill to; missing reads the current pouch; no save bump). Refill runs in `settle_party` (`roster.rs:192`).

**Menu** (VISCLAR1-D slice 2). An item is an icon in each allowed intent's row, after the skills, with the same stack count on each; hover says "0 AP, 2 blocks". The clicked target leaves one row live, as for a dual skill. A round has one intent, so planned uses never race across rows. A fifth row appears when the ruleset's `intents` lists Field. Fill and the ghost never list an item. The listing keeps one row per item, with an Allowed column.

**Feel run measures**: Field impacts per fight and the share that hit nobody, by side; dodges by Move and by Maneuver; boss damage per round against today; ring rounds and their output; capped fights (`tl_skirmish` 24-25 % today); self-heal rounds and rounds with Riposte and a self-heal together; share of rounds per intent; items used per fight by type, thrown misses, stacks empty by round; stash spent per fight; rounds where an intent had no active (OI1); Rest blocks per round.

## 7. Questions for the user

Framework first. Items, stacks, the 2-intent limit, and Field's bonus are decided; these are what is left.

0. **An enemy click loads Assault and Maneuver, plus Field if held.** (a) Show them; the first pick decides. (b) Maneuver as a pick of a position tile: clashes with Move and Field on that tile. (c) A modifier key for Field on an occupied tile: hidden. Recommend a.
1. **Is "Guard" still the word?** A self target is always Guard: Riposte, Focus, Taunt, rings, self-heals. (a) Keep the id `guard`, show "Stand" (a data edit in the terms table; "Hold" clashes with hold tiles). (b) Keep the word. (c) Move Focus and Taunt back to Support, so the target no longer defines the intent. Recommend a.
2. **Sweeps.** (a) Split: rings to Guard (Sweep Kick, Cyclone Kick, Tendril Flail), aimed ARC to Field (Sunder, Judgment, Hammer, Shatter). (b) All ring in Guard, the user's first thought: a 2.2 blow under a marker that reads safe, and AI rows that Guard only when hurt would hide the bosses' main attack. (c) All ARC in Field: a brawler's Guard stays thin. Recommend a.
3. **One tile for a whole Field round?** (a) Yes (IR4). (b) One per skill: more output, but two regions to read and the target no longer names the round. Recommend a.
4. **Starting values for both rows (IR17).** (a) Assault PLUS, splash x 0.5, no chase, authored reach; Field authored shape, reach + 1. (b) Field gets a larger shape, not range: needs a 5 x 5 for BURST, which is a quarter of the board's height: too large. (c) Same power in both: Field dies. Recommend a; the feel run tunes it. Also: `field_reach_bonus` on every thrown Field row, enemy ones too (Lard Spray 2-5), or only on rows with an Assault twin? Recommend every row, one knob.
5. **Maneuver as "a place".** (a) Position, retreat or jump tile; the Utility kind retires, since only two bombs used it. (b) Keep Utility. Recommend a.
6. **Open details of the shared item.** (a) A bomb on a foe hits only the foe, on a tile a PLUS. This departs from "the shape stays the same": a PLUS that follows would beat the tile throw on every target. (b) PLUS in both, with a lower-power override on Assault. Recommend a. Tuning: an optional per-intent version on any row (`intent_versions`, IR18), used only when one row is too strong.
7. **Stack and refill.** Limit per item in `max_stack` (decision "Material stacks"), not per kind; a second slot of the same item not allowed; refill automatic to `pouch_packed`, with a Results line "Stash short: Kaelen 1 of 3 Stitch-Poultice" (a player action would be forgotten); limited shop `count`s and Scrap prices set by the balance lane after the purse task; no class gate, as the pouch is the kit's way past the tree. Recommend all five; say which differ.

## 8. What it takes to build

Classic stays unchanged: each is a rule row or value classic does not list (decision "INPUT1-M rule choices"). No `tl_` deck holds an area skill, so the timeline golden should not move until a seed touches it. Size: S under a day of builder time, M about a slice.

Data shape: keep `round_intent` (string, the primary) and add `also_intents` (list, default empty) for skills and items, rather than turning `round_intent` into a list: the check caps it (IR18), the writer leaves the default out, so no re-save of 72 rows and no change to `checks_golden`'s lines, and the 22 uses of the primary keep working. Allowed = primary + `also_intents`. `kind_intents` becomes: ATTACK assault, maneuver, field, guard; DEFEND guard, support; SUPPORT support, guard; FOCUS guard, support; the check tests every listed entry. Item default when unset: damage: field, assault; heal: support, guard; reach 0: guard. Plus `intent_versions` and `capstone` (IR18).

This narrows: IN2 (no picked persisting intent), IN4 (one intent), IF2 (rows by target; Support off the self click), decisions "Combat input feedback" (the tile's combined row) and "Attack lock at wind-up" and "INPUT1-M rule choices" only for an Assault row, whose shape moves with its target; Field is what those decisions describe.

| Task | Title | Layer | Lane | After | Size | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| INTENT2-K1 | Rule `field_intent`: `RoundIntent::Field`, targets (IR2-IR4), `also_intents`, `intent_versions`, `capstone` and rows (IR14-IR19), `assault_splash`, `field_reach_bonus`, `field_fill_once`, Guard's tile as Move here, Utility retired, `kind_intents` | kit | logic | ATTACKLOCK1, Q1-5 | M | `round_intent*.rs`, `ruleset.rs`, `plan.rs`, `check/skill.rs`, schema |
| INTENT2-K2 | Rule `shape_anchors`: RING, ARC (fixes SWEEP), LINE by direction, drawn miss, Guard area commands, `affects` | kit | logic | K1 | M | `skill.rs`, `command.rs`, `rpgkit_data`, schema |
| INTENT2-K3 | Items: lists, `item_ap`, tile command, IR12 spend (`thrown_miss_spends`), stacks (`pouch_stacks`), `pouch_packed`, refill, Brimstone stat | kit | logic | K1 | M | `command.rs`, `plan.rs`, `gear.rs`, `state.rs`, `roster.rs` |
| INTENT2-AI | Row Z, ring row, row 2's self target, reckless and taunt rules | kit | logic | K1, K2 | S | `ai.rs` |
| INTENT2-A | Seed through `aegis content`: the 28 intents, 12 second intents and 7 shapes of the listing, order lists, skill text, `strike` for Vane and the Amalgam, Brand recovery, item table, `intents`, terms | game | asset | K1-3 | S | `data/catalog/` |
| INTENT2-U | Field row and ring colour, marked cells, rows by target (IR19), items and dual skills in two rows with counts and tips, pouch stacks on the party screen, Results line; mocks first; a Field icon | game | logic | K1-3, VISCLAR1-D | M | `emberwake_gui/src/battle/`, `party.rs` |
| INTENT2-B | Feel run, section 6's measures, `tl_` and CAST1's first fights, rule on and off | game | balance | all above, SIMCOUNT1 | S | `BALANCE_REPORT.md` |

ATTACKLOCK1 lands first: approved, and it owns the commit rule. Its "planning makes plain an area opener" becomes INTENT2-U's marked cells; if not started, merge that item here. TACTIC2 comes after K1: tactics name intents, so Guard & Counter gains rings and a Field tactic becomes possible. SKILLPASS1 takes the flagged rows; STATUSPASS1 is untouched. SWEEP's fix should reach the bosses before CAST1-A6: if this spec is deferred, K2's ARC alone (S) under today's Assault. Cost to name: the mix ranger and Purifier lose stays until SKILLPASS1, and K1-K3 are three kit slices before the slice's feel is known; nothing here needs a mass of new skills.

## 9. Evidence, out of scope

**Verified in code or data**: classification and counts by script; SWEEP refused: `plan.rs:1300` needs a diagonal aim, while `attack_opener` approaches to band (1, 1) in Manhattan distance, so the aim is never diagonal, and one seed each on a scratch copy (Bailiff, Grendel, Amalgam) shows no sweep firing while Orlov's burst does; area cells fixed at planning, hits any occupant but the caster, riposte on area hits (`command.rs`); engagement needs a unit target (`engaged_with`); basics in the ruleset row; item costs and spend paths; `kind_intents`; a trap refuses an occupied tile (`ground.rs:231`), so Hollis's Wire on a unit target today spends 3 AP for nothing (read, not run). IR17's dodge table is by script on an open board. **Reasoning**: every verdict and feel judgment, section 4's outcomes (no build run), section 5's powers (a fill model, not a sim), first-pass values.

Out of scope: balance numbers beyond the first pass; new skills; status rules (STATUSPASS1); party tactics using items; enemy items; meters and Channel beyond IR13; art beyond one Field icon and ring colour.
