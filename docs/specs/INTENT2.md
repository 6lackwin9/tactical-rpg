# INTENT2: what a round intent means, and how every kind of skill and item behaves under it

Status: answered by the user 2026-11-05; only section 7 is open. Depends on: ATTACKLOCK1. Lane: logic, asset, balance. Touches: section 8.

Section 2 (the behaviour matrix, AI rows, tips) is in [INTENT2_MATRIX.md](INTENT2_MATRIX.md). Sections 3-5 (archetypes, worked rounds, winners and losers, the thin-rounds test), item values and feel measures are in `games/emberwake/docs/specs/INTENT2_CASES.md`, as they name catalog skills and units. Every skill and consumable: `games/emberwake/docs/specs/INTENT2_SKILLS.md`.

Decided by the user: the intents below, the second shown as **Stand** (id stays `guard`) and the new one called **Field**; rings go to Stand, aimed swings to Field; one source tile a Field round; a shape with a direction expands away from the caster through the source tile, so a Move changes it and single-target attacks are untouched; a skill or item lists at most 2 intents (capstone 3) and shows in each row; a fixed set of Field bonuses, not per skill; a shaped Assault row follows the ranged rule as it is; Utility retires; a pouch has 3 slots, each a stack limited by the item's `max_stack`, no refill in battle, automatic refill from the stash after it; items cost blocks, not AP; Rest is the fill and AP management; the planning menu draws every row in fixed slots.

## Goal

"The intent tells what your target is for the round and then all your skills work around that target." This spec makes the target the only meaning of an intent and adds a tile intent.

SWEEP does not work today: the planner refuses it on every target (section 9), so its users never swing. K2 fixes it.

## 1. The framework

Rule ids IR. New behaviour is a rule option; classic picks the legacy option (section 8).

- IR1. Three separate things. The **intent** names the round's target. The **shape** names which cells relative to that target. The **effect** (damage, heal, status, hazard, forced move) is independent of both. Area is not an intent.
- IR2. One test: **what is it aimed at, and what follows if that thing moves?** An enemy, the skill follows it: Assault. A tile, nothing follows: Field. You, the skill moves with you: Stand. Another friend, the skill follows them: Support. A place you end up, re-read around its reference: Maneuver.
- IR3. Tie-breaks: (a) when the point of the skill is where you end up, it is Maneuver, even with a tile aimed at; (b) a target of you is always Stand, whatever the effect; (c) Move and Rest are neutral. Side effects never change the answer.
- IR4. One target a round (TG1 stays). A Field round has one source tile for all its blocks, so "this region is dangerous this round" is one fact to read. Moves are not a target (TG2). Stand's tile target (GD1) is a plain Move here, then a self target.
- IR5. Shapes are allowed in any intent; the intent picks the anchor: the target's tile at impact (Assault, Support), the source tile fixed at planning (Field), your tile at impact (Stand). Field names two things: the **source tile** is the target tile the player clicked; the **impact tiles** are the shape drawn from the source.
- IR6. Shapes (`area`). No direction: `PLUS`, `BURST`, `RING` (new: the 8 neighbours of the anchor; Stand). With a direction, each starting on the source: `LINE` (3 tiles from the source onward); `ARC` (new, replaces `SWEEP`: the source and its two neighbours across the direction; on a diagonal, today's L: the source as the corner, its arms back toward the caster's row and column); `CONE` (defined now, built when a skill wants it: the source, then the 3 tiles across one step on; on a diagonal, the source and its 3 neighbours away from the caster). Only these shapes read a direction; nothing here touches single-target attacks or facing (TG3, MV1 stay).
- IR7. **Direction.** It expands away from the caster, through the source. d = source - the caster's tile, read from the planned end tile, so a planned Move changes it and the marks follow on each pick.
  - Field row: fixed at planning. A shaped Assault row: read at impact from the target's tile then, foes only.
  - Snap: |dx| > |dy| gives left or right; |dy| > |dx| up or down; |dx| = |dy| the exact diagonal, in the shape's diagonal form. The straight cases read as the sprite's `Facing::toward`; a diagonal is the one case the 4-way sprite cannot show, so the marks say it. MV3's flank and its 8-way sign vector do not change.
  - Source adjacent to the caster: any of the 8 neighbours; for these shapes a diagonal neighbour counts as distance 1 in the band test, whatever `melee_diagonal` says (this and the diagonal form fix SWEEP). Source on the caster's own tile: a shape without a direction is fine (a reach 0 burst); a shape with one is refused, with the reason on its grey icon.
  - Drawn: a direction shape with reach 1, from your planned tile. If you are not on it at impact (shoved, or your Move cut short) the swing misses (`[Area Miss]`, AP spent). Thrown: any shape with reach above 1, and any PLUS or BURST: it lands anyway. So a marked tile is never a surprise (decision "Attack lock at wind-up"), and shoving is counterplay.
- IR8. Commit: as the matrix; melee and ranged Assault are ATTACKLOCK1 unchanged.
- IR9. Friendly fire. Field: the skill's new `affects` (all, foes, friends; default all, as today), since its tiles are marked. An Assault shape moves and cannot be marked, so it hits foes only, as does a Stand ring. Support shape: friends only. Test: an ally beside the target is unhurt by the Assault row and hit by the Field row when it stands in the marks.
- IR10. Follow-ups: FU2 unchanged (same intent, same target, no debt). In Field the auto-fill places each family once a round (`field_fill_once`): a repeat on tiles the foes have left is the player's call. What cannot fill is Rest.
- IR11. Items follow the same test. The first pick fixes the round's intent, skill or item, so an intent needs no skill. Items cost no AP (`item_ap` 0; today 1), only blocks: a strong item takes longer or stacks smaller. The planner never spends an item it was not told to (FU2 e stays, for the ghost, Fill, auto-fill and tactics): the stack is real. An item lists Maneuver only when it is about positioning or mobility (section 6).
- IR12. Stack spend. A foe or tile use spends one at its impact whether or not it finds anyone (the foe left reach or hid, the tile is empty). Nothing is spent when the command never starts (TG4: target downed or withdrawn first, it became Rest), when the user is down, when its wind-up is broken (P7), or for a friend use whose friend is down or out of reach at impact (nothing was poured). A foe downed after the throw started counts as a miss. Code today: the first three agree (`into_rest`, resolver); a foe out of reach at impact does not, it logs "not used" and keeps the item (`command.rs:605-612`), and a hidden foe is hit (no GD5 check on items): both change.
- IR13. Channel is not a target kind, so the test finds nothing for it. It stays the hidden slot (`target_problem` accepts anything, the planner refuses it); the meter spec must answer the test, most likely a self target: a Stand round with a meter rule rather than a sixth meaning.

Several intents (IN4's "exactly one" gives way):

- IR14. *(Superseded by IR21: allowed intents are derived by the kit from baseline fields rather than authored in `also_intents`.)*
- IR15. Rows. The row a skill is picked from sets its anchor and version. Support's target narrows to another friend, and a self target is always Stand. Fill and order: `deck.order` and `follow_ups` may hold a skill in both lists (`deck_errors` reads the derived allowed list). The auto order holds only skills whose primary is that intent, then basics: a second row is by pick or by the player's order, or a healer's auto Stand round would heal forever. AI: row 2 picks its target first; itself gives a Stand round with the heal as opener, an ally a Support round.
- IR16. Stand rounds. A self-heal or self-buff now keeps intent-bound buffs: today Support on yourself drops Riposte (IN6), under this it does not. A guard holds only while a Defend runs, so heal blocks are unguarded. The risk is stalemates, already behind `tl_skirmish`'s 24-25 % capped fights. Knob: lower the Stand row's heal if needed.
- IR17. **A thrown area skill in both rows.** The Assault row uses the ranged rule as it is, no new rule: aimed at an enemy, chasing 1 tile only when out of range at its start (MR16), hitting while the enemy is in range at impact, shape centred where it stands then. It keeps the authored shape and takes `splash_mult` 0.5 off the target (rule value), status on the target only, no hazard, foes only, and single-target follow-ups may join. The Field row: authored shape, full power and status on every impact tile, its Field bonus (IR21), hits hidden units, keeps landing when the target dies, may centre on empty ground; it can hit friends and only Field follow-ups may join. **Longer wind-up (user 2026-11-05):** a shaped Assault row, skill or item, takes one block more than its Field row (rule value `assault_shape_blocks` 1). The chase runs only at the command's start, so a longer wind-up gives more room to step out of range, and fewer casts a round mean fewer chases.

| Walks up to 2 tiles, open board: tiles that escape | Field BURST, centre | Assault band 2-3, command began before the move | began after the move (chases) | band 2-4, before | after |
| --- | --- | --- | --- | --- | --- |
| Target at distance 3 in line (of 12) | 4 | 9 | 5 | 6 | 0 |
| Target at (2, 1) (of 12) | 4 | 7 | 3 | 5 | 0 |
| Target at distance 2 in line (of 11) | 3 | 6 | 0 | 1 | 0 |

By script. A Move lands at the end of block 1 and comes first, so a burst opening in block 1 cannot chase it (the "before" columns), and any later burst can (the "after" columns: 0-5 of 12). What keeps Field worth picking is output and reach: per cast, Assault gives p(1 + (n - 1) / 2) and Field p n, so 1.5 against 2 at two units, 2.5 against 4 at four, with status on all and the bonus; at one unit Assault is better. So Field wins on clusters that hold still and with hazards; Assault is the default on one or two.

- IR18. *(Superseded by IR21: per-intent differences come from ruleset values and derived Field bonuses; `intent_versions` is not built.)*
- IR19. **Target first; every row drawn.** IN2's picked, persisting intent ends; the first pick of the round sets it, as the code does today (`pick_intent`, `Session::pick`). Every intent row is drawn always, in fixed slots, pre-rendered, so no pane is regenerated (the approved VISCLAR1-D mock). Rows the target does not allow are grey; once the opener fixes the intent, rows other than its own fade; the neutral row (Fill, Rest, Move) is always live.

| Click | Live rows | Notes |
| --- | --- | --- |
| Enemy | Assault, Maneuver; Field when the unit holds a Field skill or item that can use that tile | Both aim at that enemy, so the first pick decides |
| Empty tile | Field; Maneuver for a tile-aimed jump | Move here is the neutral row's. After it the Stand row is live, target yourself, for what today's combined row does |
| Ally | Support; Field as for an enemy | A click on a unit means the unit; the Field row is how to aim at its tile |
| Yourself (Self button, SL4) | Stand | A self-heal shows here |

What follows from it. **Persisting intent**: kept only as "last round's intent", read for IN6 (a first pick of another intent drops bound buffs, warned on the row) and for the ring before any pick. **TG6**: stays for the ghost and hotkeys. **Party cards and the ring**: the planned intent (pick, else ghost), dim while a ghost. **Ghost, tactics**: a member with no pick shows its tactic's plan; a tactic names target and intent. **Fill on a target with two live rows**: last round's intent when live, else the row order (`intents_for`: Assault, Maneuver). **AI and tells**: they pick both and the plan carries the intent. **Keys**: 1-4 party; Shift+1-9 stay fixed per hero (HK1); a key whose row is grey is refused; self-only keys target self (HK3). **An intent button**: not needed; it left with IF8.

The user's aim: smooth input while planning. You plan a round with a target and intent in mind; once a direction is picked, only moves within that scope are offered; the decision is made each round. Target and intent stay sticky into the next round, and choosing a different target takes one click on it, with no clearing step. A new target that the plan can already reach keeps the plan: an archer switching to a second enemy in range keeps its skills and tile and only the aim changes, with no new moves; only what the new target rules out is rebuilt. Test: `tl_` archer, round 2, click another foe in band: same steps, new target.

- IR20. **Field bonus.** A Field row names one of a fixed set, `field_bonus`, with values in the ruleset row. The content check enforces each one's condition, and warns on a Field row with none. Assignments: the listing.
- IR21. **Skills hold baseline data; the rule engine does the rest (user 2026-11-05; overrides the authored parts of IR14, IR18 and IR20).** A skill's allowed intents, its per-intent changes and its Field bonus are derived by one kit function from `kind`, `target_mode`, shape, reach, `hazard` and any dash, so a new skill behaves predictably once defined, and the content check prints what it derived. Rows: DEFEND and FOCUS, or SUPPORT with SELF: Stand. SUPPORT able to land on yourself: Support, Stand; on allies only: Support. DISENGAGE, or an ATTACK with a dash or position: Maneuver. RING: Stand. ARC, LINE, any `hazard` placement: Field. Thrown PLUS or BURST with reach above 1 and no hazard: Field, Assault. No shape: Assault. Variants come from ruleset values, not skill rows: a shaped Assault takes one block more and half splash; the Field bonus is lingering with a hazard, area for ARC and LINE, reach for a throw above reach 1, else power. So `intent_versions`, `also_intents` and an authored `field_bonus` are not built; `round_intent` stays only as a rare override the check reports. A boss's telegraph is an AI rule (row Z prefers Field), not skill data. Items follow the same function. Checked by script against the 129 skills: 112 match the listing as they stand, 17 do not: 7 SWEEPs need `area` set to RING or ARC (data, no intent), 7 throws gain an Assault row (Volley, which gives the ranger back its mix, and 6 enemy bursts), 3 gap-close skills are the open question below. 3 Field bonuses change (Shatter to area; Pulse and Pulverizer to reach).
- IR22. **What Stand is (user 2026-11-05; replaces GD2's "a guard holds only while a Defend runs" and narrows IN6).** A guard skill is a self buff: it gives a mitigation status for the round, or several rounds for a stronger guard, and the unit's sprite shows the guard pose while the status holds. A stronger guard's status outlives the round: the unit may use other intents in later rounds and keep the mitigation, so that status is not intent-bound. Stand also holds other self buffs, and area debuffs in the 3 x 3 around the unit (RING): mostly attack or defence down, taunt or root; foes only (IR9). Open: section 7.
- IR23. **Move and Stand (user 2026-11-05).** On an empty tile the Stand row is live at once: a pick there means Move here, then that skill, and is grey if the tile is out of move range. The row picked sets the meaning: neutral Move, Field aims at the tile, Stand moves there. Self in place: the Self click. Stand then Move: the skill, then Move, then only Rest. Hover previews the plate and any ring at the destination. If the unit's last intent was Stand, Fill on a tile means move here and fill with Stand. Replaces IR19's second step on the tile row.

| `field_bonus` | Effect (first pass) | Allowed on |
| --- | --- | --- |
| `reach` | `reach_max` + 1 | thrown shapes only, never a melee swing |
| `area` | one size up: one tile to PLUS, PLUS to BURST; LINE 3 to 4; ARC 3 to 5 across | not BURST (5 x 5 is too much of a 9-tall board) |
| `power` | `power_mult` x 1.25 | any; avoid above power 1.6 |
| `lingering` | the skill's hazard + 1 round | needs a `hazard` |

The list settles: self-or-ally heal and buff; a heal around you or an ally; a guard on an ally; items; splash and the mage's mix (IR17). Not settled by it: sweeps are split instead (ring and arc are different shapes); Focus is self-only, so it pays next round.

The test is not one clear answer for 10 of 129 (flagged in the listing); SKILLPASS1 settles those rows, and none blocks the framework.

## 6. Consumables

An item row behaves as a thrown skill's, so similar throws behave alike: a bomb has one authored shape, PLUS or BURST per item, in both rows (a new `area` on items); Assault under the ranged rule with half splash, Field at full power with its bonus. No item needs `intent_versions` at the start.

**Maneuver items.** Utility retires as a kind (MV10, MV11, `ManeuverKind::Utility`, the skill menu's tile row for a Maneuver area skill). An item lists Maneuver when it is about positioning or mobility. None of the 6 qualifies, so no row uses it yet; the coverage line "Maneuver: fire bomb" for founders goes, and `flank_cut` covers them. Planned archetype, not built here: a grapnel or vault item, `jump_tiles` 3, `use_blocks` 1, no damage, stack 2, aimed at the tile you land on (IR3 a); not the round's regular move (ET5), as a dash is. It needs an item that moves its user: its own task when wanted.

First-pass blocks, stacks and shapes per item, with the code checks behind them: `INTENT2_CASES.md` section 6.

**Stacks.** Limit per item in `max_stack` (decision "Material stacks"); a second slot of the same item is not allowed; no class gate, as the pouch is the kit's way past the tree. Keep `pouch` as a list of repeated ids, so saves and the battle code (`pouch_ids`, `pouch_count`, `items_left`, carry weight) read it as they do; "3" becomes 3 distinct ids, each at most its `max_stack` (`gear.rs:309`, `state.rs:394`, `party.rs:1316,2052`, `dojo.rs:512`, sim `args.rs:687`). Old saves stay valid ([a, a, b] is 2 slots). Add `pouch_packed` (the counts to refill to; missing reads the current pouch; no save bump). Refill is automatic in `settle_party` (`roster.rs:192`), with a Results line "Stash short: `<name>` 1 of 3 `<item>`". Limited shop `count`s and Scrap prices: the balance lane, after the purse task.

**Menu** (VISCLAR1-D slice 2). An item is an icon in each allowed intent's row, after the skills, with the same stack count on each; hover says "0 AP, 2 blocks". A round has one intent, so planned uses never race across rows. A fifth row appears when the ruleset's `intents` lists Field. Fill and the ghost never list an item.

## 7. Open questions

Each came out of the answers above.

1. **The Assault row's price: answered (user 2026-11-05).** One block more than the Field row (IR17), with the authored shape in both rows; half splash stays as the first value, and the feel run may raise it once the longer wind-up is measured; `splash_mult` 0.33 stays the fallback.
2. **The bonus table's first pass: answered (user 2026-11-05).** As listed: the aimed boss swings take `area`, 5 wide.
3. **Gap-close skills** Answered (user 2026-11-05): decision "Stand answers".
4. **When a guard starts.** Answered (user 2026-11-05): decision "Stand answers".
5. **How long and at what price.** Answered (user 2026-11-05): decision "Stand answers".
6. **Other Stand buffs across rounds.** Answered (user 2026-11-05): decision "Stand answers".

## 8. What it takes to build

Classic stays unchanged: each is a rule row or value classic does not list (decision "INPUT1-M rule choices"). No `tl_` deck holds an area skill, so the timeline golden should not move until a seed touches it. Size: S under a day of builder time, M about a slice, L two slices.

Data shape (**updated per `IR21`, 2026-11-07**): skills hold **baseline fields only** (`round_intent`, `kind`, `target_mode`, `area`, `reach_min`, `reach_max`, `hazard`, `dash_tiles`). Per `IR21` (section 1), `also_intents`, `intent_versions`, and authored `field_bonus` are **not** added to `SkillDef` or `schema.json`; instead, one kit function derives allowed intents, upgraded/downgraded intent versions (such as Assault +1 block / half splash vs Field full power), and the Field bonus from the skill's baseline fields and the ruleset's `field_bonus_values` / `splash_mult`. Item rows gain `area` and `max_stack`.

This narrows: IN2 (no picked persisting intent), IN4 (one intent), IF2 (rows by target; Support off the self click), MV10-MV11 (Utility), decisions "Combat input feedback" (the tile's combined row) and, for an Assault row only, "Attack lock at wind-up" and "INPUT1-M rule choices" (its shape moves with its target and it chases); Field is what those decisions describe.

| Task | Title | Layer | Lane | After | Size | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| INTENT2-K1 | Rule `field_intent` (`A4.2`): `RoundIntent::Field`, targets (IR2-IR4), kit derivation of allowed intents, versions and Field bonus from baseline fields (`IR21`, superseding `IR14`/`IR18`/`IR20` schema fields), `field_fill_once`, Stand's tile as Move here, Utility retired, `kind_intents` | kit | logic | ATTACKLOCK1 | M | `round_intent*.rs`, `ruleset.rs`, `plan.rs`, `check/skill.rs`, schema |
| INTENT2-K2 | Rule `shape_anchors`: IR7's direction and snap (extends `ground.rs` `heading`), preview on the plan, RING, ARC with its L (fixes SWEEP), LINE at any source, source tile, drawn miss, Stand area commands, Assault shape on the ranged rule, `affects` | kit | logic | K1 | M | `skill.rs`, `command.rs`, `plan.rs`, `ground.rs`, `rpgkit_data`, schema |
| INTENT2-K3 | Items: lists, `area`, `item_ap`, tile command, IR12 spend (`thrown_miss_spends`), stacks (`pouch_stacks`), `pouch_packed`, refill, Brimstone stat | kit | logic | K1 | M | `command.rs`, `plan.rs`, `gear.rs`, `state.rs`, `roster.rs` |
| INTENT2-AI | Row Z, ring row, row 2's self target, reckless and taunt rules | kit | logic | K1, K2 | S | `ai.rs` |
| INTENT2-A | Seed through `aegis content`: the listing's 28 intents, 12 second intents, 7 shapes, 23 Field bonuses, order lists, skill text, `strike` for two bosses, Brand recovery, item table, `intents`, terms (Stand, Field); a `tl_` fight whose units hold area skills (a BURST caster with both rows, a melee ARC, a ring) | game | asset | K1-3 | S | `data/catalog/` |
| INTENT2-U | All rows drawn and greyed (IR19), Field ring colour, source and impact tiles, dual skills and items with counts and tips, pouch stacks on the party screen, Results line; mocks first; a Field icon | game | logic | K1-3, VISCLAR1-D | M | `emberwake_gui/src/battle/`, `party.rs` |
| INTENT2-B | Feel run, the measures of INTENT2_CASES section 7, on the new `tl_` area fight, the other `tl_` fights and CAST1's first fights, rule on and off | game | balance | all above, SIMCOUNT1 | M | `BALANCE_REPORT.md` |

ATTACKLOCK1 lands first: approved, and it owns the commit rule. Its "planning makes plain an area opener" becomes INTENT2-U's marks; if not started, merge that item here. TACTIC2 comes after K1: tactics name intents, so Guard & Counter gains rings and a Field tactic becomes possible. SKILLPASS1 takes the flagged rows; STATUSPASS1 is untouched. SWEEP's fix should reach its users before CAST1-A6.

## 9. Evidence, out of scope

**Verified in code or data**: classification and counts by script; SWEEP refused: `plan.rs:1300` needs a diagonal aim, while `attack_opener` approaches to band (1, 1) in Manhattan distance, so the aim is never diagonal, and one seed each on a scratch copy shows no sweep firing while a burst does; area cells fixed at planning, hits any occupant but the caster, riposte on area hits (`command.rs`); item costs (INTENT2_CASES section 6); facing: an 8-way sign vector in the kit (`update_facing`), 4-way on the sprites (DIR4-G `Facing::toward`), `facing_point` turning to a tile; engagement needs a unit target (`engaged_with`); basics in the ruleset row; spend paths; `kind_intents`; a trap refuses an occupied tile (`ground.rs:231`). IR17's dodge table is by script on an open board. **Reasoning**: every verdict and feel judgment, the worked rounds' outcomes (no build run), the fill powers, all first-pass values.

Out of scope: balance numbers beyond the first pass; new skills; the mobility item; status rules (STATUSPASS1); party tactics using items; enemy items; meters and Channel beyond IR13; art beyond a Field icon; the cycle menu.
