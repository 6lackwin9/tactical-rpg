# RULESET1 round stance: stances, one target, follow-ups and the auto planner

Status: proposed (spec 2026-10-25, the user's round-stance model: CANON 2.1 "Round stance (intent)", decisions
"Round stance and chains" and "Round stance details" in `games/emberwake/docs/decisions/lore.md`, and the user's
cover, stealth and stance-persistence answers of 2026-10-25)
Part of [RULESET1.md](RULESET1.md) (the index; its section 4.3 points here). Schema, test kit, tests and tasks:
[RULESET1_TASKS.md](RULESET1_TASKS.md). Rule ids here: ST, TG, FU, AS, GD, MV, SP, RS, PL, MM, AI, UI.
Replaces, under timeline only: the chain rules T5-T9 of the first draft, the classic stances and their switching.

History check (`docs/HISTORY.md`):

- The 2026-10-25 row cut "facing, backstab and stealth cones" (reason: scope; rolls and cones on a deterministic
  resolver). Flanking and Guard stealth return because the user decided them ("Round stance details", the stealth
  answer); both are built without rolls or cones: facing is derived from the round target (MV1), stealth is a status
  that makes locked attacks lose a unit that moved (GD5). The original reason does not apply to these forms.
- Act-then-move (Q-54, out of scope in the first draft): only the two bundled after-moves come in (the kite step
  after an attack, the Disengage retreat); a free move after acting stays out.
- Stance damage multipliers (DEC-LORE-11) stay cut: the round stance limits skills and adds no damage factor.

## 1. The model

A member plans a round with two inputs, stance and target. The planner places an opener, same-stance follow-ups in
the member's order, the moves the stance implies and Rest; manual mode builds the same by hand; enemies pick stance
and target by rule and use the same planner, with recovery and no AP.

## 2. Stances and the skill field `[kit hook]` (rule `round_stance`)

- ST1. Stances: `assault`, `guard`, `support`, `maneuver`. `channel` is reserved for Act 2 (the meter spec): a schema
  option the content check rejects unless the ruleset row's `stances` lists it; nothing else reads it in M1.
- ST2. A unit has one round stance at a time. It persists across rounds and may be changed only at round start (while
  planning); the change costs nothing and has no damage factor, but the unit loses its stance buffs (ST6). At battle
  start a unit has none; the first pick is not a change. Mid-round changes do not exist.
- ST3. Under timeline the classic stance machinery is inert: units stay Neutral (`unit.rs:15-48`), switch AP 0
  (`skill.rs:137-146`), matchup 1.0 (`damage.rs:196-197`); `stance` (the classic override), `ends_in_stance` and
  `disrupt_break` do nothing. Classic keeps all of it (parity).
- ST4. A skill has exactly one stance, `round_stance` (family field); "" = by kind (table below). Side effects never
  change it: Shield Bash (ATTACK) is Assault whatever else it does.
- ST5. Neutral commands, usable in any stance: a plain Move and Rest. Switch Target is never planned (TG3). Neutral is
  not a stance a member can pick. Pouch items take a stance by aim: self or ally items Support, foe-aimed items
  (`aims_at_foes`, `emberwake_data/src/lib.rs:372`) Maneuver.
- ST6. Stance buffs: a status instance tagged with the holder's round stance when it is applied; when the holder's
  stance changes at a round start, every tagged instance whose tag differs is removed (log "`<unit>` leaves Guard:
  Riposte ends"). A status is tagged when its status-table row has `stance_bound` (code: `riposte`, the new
  `stealth`) or when the skill that grants it to its own user has the family flag `stance_buff` (data). Today's
  basics carry none (Strike, Off-hand Jab, Move, Rest, Switch Target, Guard grant no status; Active Guard's
  mitigation is the running Defend, not a status). Of today's statuses only `riposte` behaves so in classic (removed
  on any stance change, `unit.rs:686-691`); haste, taunt, fortify, regen and empower stay unbound, and the Focus boost
  (`attack_boost`, a unit field) is unbound so Support-then-Assault keeps it.

| Kind | Default stance | Allowed `round_stance` |
| --- | --- | --- |
| ATTACK | assault | `assault`, `maneuver` |
| DEFEND | guard | `guard` |
| SUPPORT | support | `support`, `guard` (self-only, such as a taunt) |
| FOCUS | support | `support` |
| DISENGAGE | maneuver | `maneuver` |
| MOVE, REST, TARGET, STATUS | neutral | `neutral` |

| Stance | Round target | Skills | Implied moves (rules) | Items |
| --- | --- | --- | --- | --- |
| Assault | an enemy (an area opener: an enemy or a tile) | assault | approach before; ranged: kite before or after (AS) | none |
| Guard | self or a tile | guard | a standard move to the tile first (GD1) | none |
| Support | self or an ally | support | a standard approach into reach (SP1) | self and ally items |
| Maneuver | an enemy (Engage, Disengage); an enemy or a tile (Utility) | maneuver | the bundled maneuver move, before or after (MV) | foe-aimed items (Utility) |

## 3. One target per round

- TG1. The round target is self, an ally, an enemy or a tile, as the stance table allows. Every skill and item command
  of the round acts on it: attacks hit it, area skills aim at it (its tile when it is a unit), support skills and
  items land on it. Self-only skills (DEFEND, `target_mode` SELF) act on the user and need a self or tile target.
- TG2. Moves are not a second target: the approach, kite step, flank route, retreat and the move to a Guard tile are
  implied by stance and target; a manual plain move (ST5) is allowed in any stance and keeps the target.
- TG3. The target is the unit's lock (`active_target`, `unit.rs:99`); Switch Target is never inserted (TQ14 a; today
  Engage inserts it on a change, `plan.rs:718-724`). Facing points at the target (MV1).
- TG4. When the target is downed or withdrawn before a command on it starts, that command and every later command of
  the unit this round become Rest blocks (AP not paid; TQ17 a). A tile target never fails.
- TG5. Area hits on other units and riposte counters are not commands and do not break the rule.
- TG6. The stance persists (ST2); the target does not, but the planner proposes last round's target while it is valid.

## 4. Openers and follow-ups (replaces T5-T9)

- FU1. The opener is the round's first skill or item command on the target: the player's pick, else PL3.
- FU2. Follow-ups: after the opener, block by block, the planner places the first entry of the member's order for the
  stance (FU4) that (a) has the round's stance, (b) fits wholly before any reserved after-move (no spill, T2),
  (c) is affordable without AP debt after the commands before it, (d) acts on the target from the tile the unit stands
  on then (reach band, support reach, area range) and (e) is a skill, not an item (the planner spends no pouch item
  it was not told to). An entry may repeat.
- FU3. The fill stops at the first block where no entry qualifies; every later block not reserved is Rest. Rest pays AP
  only at round end (RS2), so no later skill becomes affordable mid-round and the fill needs one pass.
- FU4. Order lists (`deck.order`, tasks 1.5): per member and stance, skill families from the deck's actives and the
  ruleset's basics of that stance, in the player's priority. Missing or empty = auto: the deck's actives of that
  stance in slot order, then those basics in the ruleset's `basics` order.
- FU5. Party-side units (heroes, guests) book no recovery: `recovery_blocks` is read for enemy-side units only (T4).
  AP limits heavy skills. Players author no follow-up lists (the first draft's `follow_ups` was never built: dropped).
- FU6. Each command pays its own AP at its own start (`resolver.rs:103-115`); a break (P7) cancels what has not landed.

Worked examples (move 2 tiles a block; tasks 4.1 tests them):

| Unit, AP, situation | Stance, target | Plan | AP left |
| --- | --- | --- | --- |
| tl_vanguard, 10, footman 3 tiles away | Assault, footman | b1 approach 2 tiles (2), b2-b5 Heavy Cleave (4), b6 Shoulder Check (1) | 3 |
| tl_archer, 10, footman adjacent | Assault, footman | b1 kite 2 tiles (2), b2-b3 Aimed Shot (2), b4-b5 Aimed Shot (2), b6 Snap Shot (2) | 2 |
| tl_vanguard, 3, footman adjacent | Assault, footman | b1, b2, b3 Shoulder Check (1 each; Heavy Cleave unaffordable), b4-b6 Rest | 0, then Rest +1 |

## 5. Combined commands

### 5.1 Assault

- AS1. Melee: when the target is outside the opener's band, a standard approach comes first (today's Engage,
  `plan.rs:670-734`: 1 AP a tile, at most 2 move blocks, once a round) to the PL4 tile; then opener and follow-ups.
  The chase (`follow_tiles`) keeps the target in the band as today.
- AS2. Kite before: a ranged unit (`attack_range > 1`, `skill.rs:232`) whose target stands within the target's own
  melee band of it at planning spends block 1 on a kite step: one standard move block to the PL4 tile in the
  opener's band farthest from the target; then the attacks.
- AS3. Kite after: otherwise, when the end tile is threatened by the target (distance at most the target's 2-block
  move plus its reach) and a farther tile in band is reachable in one move block, the last block is reserved for a
  step there after the attacks (standard move cost, TQ19 a). A target out of band and not in melee: approach as AS1.
- AS4. Assault never targets self or an ally.

### 5.2 Guard

- GD1. Target self: hold in place. Target a tile: a standard move there first (the picker marks cover tiles), then hold.
- GD2. Opener and follow-ups come from the Guard order (Active Guard, riposte and stealth skills, Shield Wall). A guard
  holds while a Defend runs, so the auto order repeats Active Guard (2 blocks, 1 AP): 3 AP covers six blocks.
- GD3. Riposte: today's counter (`command.rs:773-790`: melee hits at distance 1, once per attacker per round); it is a
  stance buff (ST6) and ends when the unit leaves Guard; it does not answer a flank hit when TQ12 a holds.
- GD4. Cover reduces damage (user 2026-10-25): today's cover term, no roll (`cover_for`, `command.rs:696-717`: the
  tile's or a blocking neighbour's `cover_guard` against ranged hits, summed with guard under `GUARD_CAP` 0.9,
  `damage.rs:202`). Guard adds the tile choice (PL4 ranks cover first for Guard), no extra factor.
- GD5. Stealth (user 2026-10-25): status `stealth` (new row: buff, not resistable, 2 rounds, refresh, `stance_bound`),
  granted by a Guard skill (`grant_status`, active from the block the skill starts, like riposte). An attack locked on
  a unit (a single-target attack whose step targets it) that lands while the unit has `stealth` and has moved this
  round (a move entry of it landed earlier) finds nothing: no damage, stagger or status; AP spent stays spent (TQ21 a);
  the log says "`<attacker>` loses `<unit>`". A chase (`follow_tiles`) does not step toward it. Area attacks still hit
  its tile. Stealth ends when the unit leaves Guard (ST6) or when its rounds run out.

### 5.3 Maneuver

- MV1. Facing (battle layer, new): a unit faces its round target, unit or tile (the sign vector from its tile to the
  target's, re-read every block); with a self target or none it keeps its last facing (battle start: party right, foes
  left, as FACE1 draws, `emberwake_gui/src/battle.rs:141-188`, which today is GUI-only). FACE1 and DIR4 draw this.
- MV2. Engaged: T is engaged with E when E is T's round target, a foe of T, within T's melee band (1 to its reach).
- MV3. Flank: A's attack on T is a flank when T is engaged with an E other than A and A's tile is outside T's front:
  `dot(A - T, facing(T)) <= 0` (beside or behind). Its bonus is TQ12 (rule `flank`).
- MV4. Maneuver move (the movement bundled in a Maneuver command): `ceil(tiles x maneuver_move_ap)` AP (0.5) and
  `ceil(tiles / (move_per_block x maneuver_move_speed))` blocks (2.0); the same tile limit as a standard move
  (`plan.rs:333-339`); it is the round's one move. Four tiles at 2 a block: standard 4 AP and 2 blocks, maneuver 2 AP
  and 1 block.
- MV5. Engage (an enemy): a maneuver move to a PL4 tile outside the target's front within the opener's band, then
  opener and follow-ups. Never face-on: with no such tile, Engage is not offered.
- MV6. Disengage (an enemy already in the opener's band): opener and the follow-ups that fit, then a maneuver move to
  the safest reachable PL4 tile in the last blocks it needs. A DISENGAGE skill (Backflip) first in the Maneuver order
  replaces the walk with its jump at its own AP. Not offered when the target is out of band.
- MV7. Utility (an enemy or a tile, TQ16 a): a maneuver move until the target is within the item's `battle_reach`,
  then the chosen pouch item; follow-ups are Maneuver skills on the same target.
- MV8. A Maneuver round needs a kind; the picker offers the possible ones; auto takes Engage, else Disengage, else
  Utility.

### 5.4 Support

- SP1. Target self or an ally, never an enemy. A standard approach first when the ally is out of the opener's reach
  (`support_reach`, `skill.rs:214`; items `battle_reach`).
- SP2. Opener: a Support skill or a self or ally pouch item (`item_side_problem`, `command.rs:859`); follow-ups:
  Support skills on the same target (a SELF-only skill only on a self target). Heals and buffs; Focus is Support.

## 6. Rest (rebalance for the balance lane)

Today: the system basic `rest` is REST, 1 block, 0 AP (`skill.rs:151`), and gives 2 AP when it lands (the schema
default `recover_ap` 2; the row sets none; `command.rs:347-353`, planner `plan.rs:1107-1109`); it enters Tactical
for free (`skill.rs:48,139`; its text says "return to Neutral", a mismatch outside this spec). The AI rests at most
twice a round when short of AP (`ai.rs:195-200`); COMBAT-UI3's pickers fill empty blocks with a strike, else Rest.
With free stance changes and every leftover block a Rest, 2 AP a block would return up to 10 AP a round (a Snap and
five Rests), and AP would stop limiting heavy skills.

- RS1. Under timeline Rest is neutral, 1 block, 0 AP and never windup (P5); the planner puts it on every unused block
  and the player can place it by hand (a wait).
- RS2. A landed Rest gives no AP at once; at round end, after `ap_regen`, the unit gains
  `min(rest_ap_cap, floor(rest_blocks x rest_ap_per_block))` AP (first sim 0.5 and 2: a full Rest round gives 2, a
  Heavy round with two Rest blocks 1). Enemy-side units have no AP and skip it.
- RS3. Each landed Rest block restores `rest_poise_per_block` poise (5), up to Max Poise.
- RS4. Classic Rest is unchanged. TQ13 holds the alternative.

## 7. The auto planner (RULESET1-PL)

Deterministic: the same world, plans and inputs give the same plan; it draws no random number.

- PL1. Inputs: unit, stance, target, Maneuver kind, an optional opener (skill or pouch item). Output: plan steps
  through `Plan::add_step` (`plan.rs:348`; every rule is checked in one place), each with a reason tag.
- PL2. The target must fit the stance table; else the stance is refused with the reason.
- PL3. Opener: the given one, else the first order entry (FU4) that is affordable and acts on the target after the
  stance's before-move.
- PL4. Tiles for every implied move: the tiles reachable within the move's limit that meet the stance's condition
  (band, outside the front, item reach, the Guard tile), ranked by (1) threat, the active foes whose next-round reach
  covers the tile (distance at most their 2-block move plus their reach), fewer first; (2) cover toward the target's
  side, more first (Guard and ranged only); (3) path tiles, fewer first; (4) distance to the target, farther first for
  kite and retreat, nearer for none; (5) cell index `y x 12 + x`, lower first.
- PL5. Placement order: before-move, opener, reserved after-move blocks (AS3, MV6), fill (FU2, FU3), after-move, Rest.
- PL6. Reason tags (strip tooltip and the Dojo): `approach`, `kite: in melee`, `kite: threatened`, `flank tile`,
  `cover tile`, `opener`, `follow-up k` (its place in the order), `retreat`, `rest: <cause>` with the first refused
  entry's cause (`AP 1 < 2`, `blocks 1 < 2`, `out of reach`).
- PL7. No valid opener: the stance (or kind) is shown disabled with its reason ("No Assault skill reaches Wolf this
  round"). The planner never plans AP debt.
- PL8. A plan is rebuilt when stance, target, opener, order or an earlier member's plan changes (approach tiles are
  claims, PLAN1), in roster order.

## 8. Manual mode

- MM1. A per-member Manual toggle; stance and target are still set first, and `Plan::add_step` refuses another
  stance's skill ("Only Assault skills this round") or another target ("One target per round: Wolf").
- MM2. Manual adds the stance's skills in any order and repeat, Rest anywhere (to time an impact into a windup), the
  stance's bundled moves as steps (Approach, Kite, Flank move, Retreat), a plain move (first, as today,
  `plan.rs:570-578`) and pouch items as follow-ups.
- MM3. "Fill the rest" runs PL5's fill after the last manual step; Manual off rebuilds the auto plan.
- MM4. What only manual can do: a delayed opener (Rest, Rest, a Heavy landing in b6), an out-of-order follow-up, a Snap
  between two Rests, items as follow-ups.

## 9. Enemies and the AI (RULESET1-AI)

- AI1. Enemies plan with stances: each keeps its stance across rounds (ST2) and re-picks stance and target each round
  by the first row of the table that applies; the stance planner then fills from its `deck.order` (auto when empty).
- AI2. Enemy-side units pay no AP and have no AP bar, and book `recovery_blocks` after each impact (T1-T4, T11).
- AI3. No authored sequences in M1 (TQ18 a): a boss's pattern is its order list and its skills.
- AI4. Tells (V1) show the stance glyph and the round target with the first action; badges unchanged.
- AI5. Party autoplay (AUTO1 tactics, `emberwake_gui/src/battle.rs:43`) maps to stance picks (TQ20 a): second table.

| Order | Condition | Stance, target |
| --- | --- | --- |
| 1 | `flee_ratio` reached | today's Withdraw (a neutral move); stance unchanged |
| 2 | Support role, or someone below `heal_below`, and a Support skill or item can reach | Support, the lowest HP share |
| 3 | HP share at most `defend_below`, not reckless, a Guard skill usable | Guard, self or the best cover tile in one move if a ranged foe threatens |
| 4 | Skirmisher with a Maneuver attack and a flank tile on a ranked target engaged with another unit | Maneuver Engage, that target |
| 5 | otherwise | Assault, the first ranked target (`rank_targets`, `ai.rs:595`); Snipers kite (AS2, AS3) |
| 6 | nothing usable | Guard self, else Rest |

| AUTO1 tactic | Stance and target |
| --- | --- |
| Balanced | the AI table |
| All-out Attack | Assault, the first ranked target |
| Focus Weakest | Assault, the lowest-HP foe reachable this round |
| Guard & Counter | Guard, self |
| Kite at Range | ranged: Assault (kites); melee: Maneuver Disengage when offered, else Assault |
| Economy | Assault with the cheapest opener and no follow-ups (Rest) |

## 10. Input and display (RULESET1-U, -G)

- UI1. Each member's strip row: four stance buttons (keys 1-4; Channel hidden until the ruleset lists it), the
  current stance lit (it persists) and a "loses Riposte" warning on a change that drops buffs; Engage, Disengage and
  Utility under Maneuver; then a click on the board sets the target (valid cells lit). Two inputs plan the round. An
  opener dropdown and the Manual toggle sit on the row.
- UI2. The strip shows the plan with reason tags on hover (PL6), after-moves as step glyphs, Rest as dim blocks with
  the Rest gain at the row end ("Rest: 1 AP, 10 poise"), disabled stances with their reasons.
- UI3. The board shows the move paths, the target ring, the target's front half shaded while Maneuver is picked,
  enemy facings from their tells, and a stealth veil on stealthed units.
- UI4. Dojo (to judge the planner): the hero's stance, target and kind against the dummy; a movable dummy and a second
  dummy to engage it (sets its facing); the planned blocks with reasons, AP before and after, Rest gain, expected
  damage per command; the order editor; a table of the plan each stance would make now. The Dojo has no resolver
  (`dojo.rs:1-27`): it compiles plans on a scratch World (hero, dummies, the 12 x 9 grid); the Arena plays them.

## 11. What the Dojo and the Arena show (RULESET1-G, -U, -X; was RULESET1.md section 12)

- Gear Dojo: a Classic / Timeline toggle. Under timeline: the round stance controls and the planner view (UI4)
  instead of the stance shift (`dojo.rs:631`); Max Poise with its DERIVE1 breakdown; poise bars for hero and dummy
  (dummy `poise` 40); "Dummy in windup" beside "Dummy guards"; hit breakdowns add `stagger 12 x windup 2 x guard 0.5
  = 12 -> poise 40 -> 28`, "flank" and "armour pierced"; AP by the `ap_cost` rule; "Next round" refills poise and
  applies the Rest gain; the deck buckets and order editor (RULESET1 D6).
- Combat Arena: a World / Classic / Timeline select, the extension toggles (RULESET1 3.2), "Reveal enemy plans", a
  ruleset chip.
- Battle screen: UI1-UI3; windup shading, impact markers, enemy recovery hatching, after-move glyphs and BREAK markers
  on the strip; badges or full plans (RULESET1 V1, V2); poise bars on unit and hover cards; a target forecast ("in
  windup: stagger x2, breaks"; "may break: 40%"; "flank"); the reach band (K6); a round summary line: breaks, windups
  broken of windups hit, follow-ups, flanks, guards, Rest gain.
- Exit evidence: golden traces (tasks 4.2), Arena recordings of `tl_duel_windup`, `tl_flank` and `tl_skirmish` copied
  to `games/emberwake/playtests/`, and the sim report (RULESET1-B).

## 12. Code survey (develop, 2026-10-25; paths under `games/emberwake/crates/`, `b/` = `emberwake_battle/src/`)

| Piece | Where | Today |
| --- | --- | --- |
| Scheduling, landing | `b/resolver.rs:68-79,97-99` | `end = start + blocks - 1`, refused past the round or on overlap; `end == block` lands by initiative |
| Start, cancel | `b/resolver.rs:103-148`, `b/command.rs:159-161` | AP and switch paid at start (debt allowed), stance entered, `on_start`; only a downed unit's actions cancel; initiative `priority x 1000 + speed` |
| Guard, armour | `b/resolver.rs:84-95`, `emberwake_rules/src/damage.rs:11,122-138,202` | a running Defend raises `guard`; `min(0.9, guard + cover)`; rating gear + STR/4 (or `armor`) + fortify - expose, `r / (r + 50)` |
| Damage stance term | `damage.rs:4,159-160,196-204` | `stance.matchup(target.stance)` in every hit |
| Stances | `emberwake_rules/src/unit.rs:15-48,686-691` | Neutral, Offensive, Defensive, Tactical; 1.25 / 0.75; a change removes `riposte` |
| Skill stance rules | `b/skill.rs:43-52,83-146` | by kind; the row's `stance` override (AUTO, `emberwake_data/src/lib.rs:166,218`); switch 1 AP |
| Stance in plans | `b/plan.rs:246-250,1085-1087`, `b/resolver.rs:151-177` | switch AP, stance on start and end, `disrupt_break` |
| AP | `b/skill.rs:149-156`, `b/session.rs:90` | `clamp(level, 1, 5) + modifier`; Move 1 a tile; Rest, Target, Status 0; pool `unit_ap` (12) |
| Basics | `emberwake_data/src/lib.rs:480-505`; `data/catalog/skills.json` | `strike`, `offhand_jab`, `guard`, `move`, `rest`, `switch_target` (last three `system`) |
| Move, Engage | `b/plan.rs:21,333-339,570-578,670-734,1037` | once a round, first, at most 2 blocks, 1 AP a tile; approach, Switch Target, attack, chase |
| Reach | `b/skill.rs:214-236`, `b/command.rs:244,260,538-549` | `attack_range > 1` = ranged: band (2, max(3, range)), cover, no riposte; melee (1, 1); support 0-3 |
| Items | `b/plan.rs:515-540`, `b/command.rs:839-873`, `b/skill.rs:188-203` | pouch steps, foe-aimed or friendly, `battle_reach` |
| Riposte, cover | `b/command.rs:773-790`, `b/command.rs:696-717` | melee at 1, once per attacker; ranged cover term |
| Statuses, tells | `emberwake_rules/src/status.rs:37-53`, `b/intent.rs:20,136-146` | fifteen rows, no stance flag; each enemy's first action, whole plans for foresight |
| Loadout, passives | `emberwake_rules/src/{stats.rs:10-11,unit.rs:279-400}` | 5 class + 3 extra, basics free; passives always on |
| Families, skill row | `emberwake_content/src/check/item.rs:26`, `emberwake_data/src/lib.rs:155-245` | closed `engine_rules`; `blocks` 1-6, `level` 1-5, reach, `exploit` |
| Encounter, world | `b/session.rs:44-66`, `emberwake_world/src/data.rs:166` | no ruleset; `Session::start` 17 call sites; lineage fields like `party_max` |
| AI | `b/ai.rs:46,195-200,385,522,595,621,695` | draft, rests, items, kite, rank, retreat, disengage |
| Dojo, Arena, sim | `b/dojo.rs:1-27,631`, `emberwake_gui/src/runner.rs:124-127`, `b/sim.rs:20-31`, `emberwake_sim/src/args.rs:36-55` | no resolver, stance shift; any encounter; `FightSpec`, args outside the help |
| Facing, widget | `emberwake_gui/src/battle.rs:141-188`; `engine/crates/aegis_ui/src/timeline.rs:20-58` | GUI-only left or right; segments, no marks or parts |
