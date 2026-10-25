# RULESET1 round intent: intents, one target, follow-ups and the auto planner

Status: proposed (spec 2026-10-25 from CANON 2.1 "Round intent" and the lore decisions "Round stance and chains",
"Round stance details", "Round intent (renamed)"; renamed from RULESET1_STANCE.md 2026-10-25)
Part of [RULESET1.md](RULESET1.md) (the index; its section 4.3 points here); schema, test kit, tests and tasks:
[RULESET1_TASKS.md](RULESET1_TASKS.md). Rule ids: IN, TG, FU, AS, GD, MV, SP, RS, PL, MM, AI, UI. Replaces, under
timeline only: the first draft's chain rules T5-T9, the classic stances and their switching.

Names: "stance" stays the Ashlands' classic system (parity). The **round intent** uses rule id and skill field
`round_intent`, status column `intent_bound`, skill flag `intent_buff`, ruleset key `intents`, type `RoundIntent`,
modules `round_intent.rs` and `round_intent_plan.rs`; not `intent_kind` (the tell's `kind` is the command kind) and
not `intent.rs` (the tells, which the round intent extends: UI5).

History check (`docs/HISTORY.md`): the cut of "facing, backstab and stealth cones" (rolls and cones on a
deterministic resolver) does not hold here: the user decided flanks and Guard stealth, built without rolls or cones
(MV1, GD5). Act-then-move (Q-54) enters only as bundled after-moves (kite step, Disengage retreat). Stance damage
multipliers (DEC-LORE-11) stay cut. The half-AP maneuver move (never built) gives way to the carried move (MV4).

## 1. The model

A member plans a round with two inputs, intent and target. The planner places an opener, same-intent follow-ups in
the member's order, the moves the intent implies and Rest; manual mode builds the same by hand; enemies pick intent
and target by rule and use the same planner, with recovery and no AP. Tells and threat intel show every unit's
intent (UI5).

## 2. Intents and the skill field `[kit hook]` (rule `round_intent`)

- IN1. Intents: `assault`, `guard`, `support`, `maneuver`. `channel` is reserved for Act 2 (the meter spec): a schema
  option the content check rejects unless the ruleset row's `intents` lists it; nothing else reads it in M1.
- IN2. A unit has one round intent at a time. It persists across rounds and may be changed only at round start (while
  planning); the change costs nothing and has no damage factor, but drops the unit's intent-bound buffs (IN6). At
  battle start a unit has none; the first pick is not a change (TQ23: a temperament default). No mid-round change.
- IN3. Under timeline the classic stance machinery is inert: units stay Neutral (`unit.rs:15-48`), switch AP 0
  (`skill.rs:137-146`), matchup 1.0 (`damage.rs:196-197`); `stance` (the classic override), `ends_in_stance` and
  `disrupt_break` do nothing. Classic keeps all of it (parity).
- IN4. A skill has exactly one intent, `round_intent` (family field); "" = by kind (table below). Side effects never
  change it: Shield Bash (ATTACK) is Assault whatever else it does.
- IN5. Neutral commands, usable in any intent: a plain Move and Rest. Switch Target is never planned (TG3). Neutral is
  not an intent a member can pick. Pouch items take an intent by aim: self or ally items Support, foe-aimed items
  (`aims_at_foes`, `emberwake_data/src/lib.rs:372`) Maneuver.
- IN6. Buffs (user 2026-10-25). An **intent-bound** status instance lasts only while the intent it was applied under
  stays active: an intent change at a round start removes it whatever rounds it has left (log "`<unit>` leaves
  Guard: Riposte ends"). Every other instance (buffs, debuffs, the Focus boost) keeps its round duration across
  intent changes. Intent-bound exactly when (a) its status row has `intent_bound` true (`riposte`, the new
  `stealth`; every other row false) or (b) the skill granting it to its own user has the family flag `intent_buff`
  (a grant to another unit is never bound); `StatusState` records its intent. Classic drops only `riposte`, on any
  stance change (`unit.rs:686-691`); haste, taunt, fortify, regen, empower and `steadfast` are unbound.

| Kind | Default intent | Allowed `round_intent` |
| --- | --- | --- |
| ATTACK | assault | `assault`, `maneuver` |
| DEFEND | guard | `guard` |
| SUPPORT | support | `support`, `guard` (self-only, such as a taunt) |
| FOCUS | support | `support` |
| DISENGAGE | maneuver | `maneuver` |
| MOVE, REST, TARGET, STATUS | neutral | `neutral` |

| Intent | Round target | Skills | Implied moves (rules) | Items |
| --- | --- | --- | --- | --- |
| Assault | an enemy (an area opener: an enemy or a tile) | assault | approach before; ranged: kite before or after (AS) | none |
| Guard | self or a tile | guard | a standard move to the tile first (GD1) | none |
| Support | self or an ally | support | a standard approach into reach (SP1) | self and ally items |
| Maneuver | an enemy (Engage, Disengage); an enemy or a tile (Utility) | maneuver | each skill's carried move (MV4); out of range: MV9 | foe-aimed items (Utility) |

## 3. One target per round

- TG1. The round target is self, an ally, an enemy or a tile, as the intent table allows. Every skill and item command
  of the round acts on it: attacks hit it, area skills aim at it (its tile when it is a unit), support skills and
  items land on it. Self-only skills (DEFEND, `target_mode` SELF) act on the user and need a self or tile target.
- TG2. Moves are not a second target: the approach, kite step, carried moves and the move to a Guard tile are implied
  by intent and target; a manual plain move (IN5) is allowed in any intent and keeps the target.
- TG3. The target is the unit's lock (`active_target`, `unit.rs:99`); Switch Target is never inserted (TQ14 a; today
  Engage and the dash insert it on a change, `plan.rs:718-724,756-762`). Facing points at the target (MV1).
- TG4. When the target is downed or withdrawn before a command on it starts, that command and every later command of
  the unit this round become Rest blocks (AP not paid; TQ17 a). A tile target never fails.
- TG5. Area hits on other units and riposte counters are not commands and do not break the rule.
- TG6. The intent persists (IN2); the target does not, but the planner proposes last round's target while it is valid.

## 4. Openers and follow-ups (replaces T5-T9)

- FU1. The opener is the round's first skill or item command on the target: the player's pick, else PL3.
- FU2. Follow-ups: after the opener, block by block, the planner places the first entry of the member's order for the
  intent (FU4) that (a) has the round's intent, (b) fits wholly before any reserved after-move (no spill, T2),
  (c) is affordable without AP debt after the commands before it, (d) acts on the target from the tile the unit stands
  on then (reach band, support reach, area range; a Maneuver skill within its range, MV5) and (e) is a skill, not an
  item (the planner spends no pouch item it was not told to). An entry may repeat.
- FU3. The fill stops at the first block where no entry qualifies; every later block not reserved is Rest. Rest pays AP
  only at round end (RS2), so no later skill becomes affordable mid-round and the fill needs one pass.
- FU4. Order lists (`deck.order`, tasks 1.5): per member and intent, skill families from the deck's actives and the
  ruleset's basics of that intent, in the player's priority. Missing or empty = auto: the deck's actives of that
  intent in slot order, then those basics in the ruleset's `basics` order.
- FU5. Party-side units (heroes, guests) book no recovery (T4); AP limits heavy skills. No authored follow-up lists.
- FU6. Each command pays its own AP at its own start (`resolver.rs:103-115`); a break (P7) cancels what has not landed.

Worked examples (move 2 tiles a block; tasks 4.1 tests them):

| Unit, AP, situation | Intent, target | Plan | AP left |
| --- | --- | --- | --- |
| tl_vanguard, 10, footman 3 tiles away | Assault, footman | b1 approach 2 tiles (2), b2-b5 Heavy Cleave (4), b6 Shoulder Check (1) | 3 |
| tl_archer, 10, footman adjacent | Assault, footman | b1 kite 2 tiles (2), b2-b3 Aimed Shot (2), b4-b5 Aimed Shot (2), b6 Snap Shot (2) | 2 |
| tl_vanguard, 3, footman adjacent | Assault, footman | b1, b2, b3 Shoulder Check (1 each; Heavy Cleave unaffordable), b4-b6 Rest | 0, then Rest +1 |
| tl_spiker, 6, footman 3 tiles away, engaged with tl_vanguard | Maneuver Engage, footman | b1 Hamstring Cut carried 3 tiles to a flank tile (2), b2, b3 Hamstring Cut (2 each), b4-b6 Rest | 0, then Rest +1 |

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
  step there after the attacks (a standard move, TQ19 a). A target out of band and not in melee: approach as AS1.
- AS4. Assault never targets self or an ally.

### 5.2 Guard

- GD1. Target self: hold in place. Target a tile: a standard move there first (the picker marks cover tiles), then hold.
- GD2. Opener and follow-ups come from the Guard order (Active Guard, riposte and stealth skills, Shield Wall). A guard
  holds while a Defend runs, so the auto order repeats Active Guard (2 blocks, 1 AP): 3 AP covers six blocks.
- GD3. Riposte: today's counter (`command.rs:773-790`: melee hits at distance 1, once per attacker per round); it is
  intent-bound (IN6) and ends when the unit leaves Guard; it does not answer a flank hit (TQ12 a, MV3).
- GD4. Cover reduces damage (user 2026-10-25): today's cover term, no roll (`cover_for`, `command.rs:696-717`; summed
  with guard under `GUARD_CAP` 0.9, `damage.rs:202`). Guard adds the tile choice (PL4 ranks cover first), no factor.
- GD5. Stealth (user 2026-10-25): status `stealth` (new row: buff, not resistable, 2 rounds, refresh, `intent_bound`),
  granted by a Guard skill (`grant_status`, active from the skill's start, like riposte). A single-target attack
  locked on the unit that lands while it has `stealth` and has moved this round (a move entry of it landed earlier)
  finds nothing: no damage, stagger or status, AP spent (TQ21 a); log "`<attacker>` loses `<unit>`". Chases and
  carried moves do not step toward it; area attacks still hit its tile. It ends on leaving Guard (IN6) or with time.

### 5.3 Maneuver

- MV1. Facing (battle layer, new): a unit faces its round target, unit or tile (the sign vector from its tile to the
  target's, re-read every block); with a self target or none it keeps its last facing (battle start: party right, foes
  left, as FACE1 draws, `emberwake_gui/src/battle.rs:141-188`, which today is GUI-only). FACE1 and DIR4 draw this.
- MV2. Engaged: T is engaged with E when E is T's round target, a foe of T, within T's melee band (1 to its reach).
- MV3. Flank: A's attack on T is a flank when T is engaged with an E other than A and A's tile is outside T's front:
  `dot(A - T, facing(T)) <= 0` (beside or behind). Bonus (rule `flank`, TQ12 a): stagger x `flank_stagger_mult` (P6),
  no riposte from T, and damage only where the skill's `exploit` names the condition `flanked`.
- MV4. Carried move (user 2026-10-25; replaces the earlier half-AP maneuver move): a Maneuver skill carries its own
  movement, its `dash_tiles` (move cost in one block, `dash_fits`, `command.rs:615`; a DISENGAGE skill: `jump_tiles`),
  at no AP or block beyond the skill's own, inside its command: before the hit at its start for Engage (today's dash,
  `on_start`, `command.rs:167-176`), after the hit at its impact for Disengage. Classic counts a dash or jump as the
  round's one move (a jump first; schema help); under timeline neither does, and a jump may come last.
- MV5. Range: the target is within a Maneuver skill's range when, from the unit's tile at the skill's start, a tile
  meeting the kind's condition (Engage: outside the target's front; Disengage: the current tile) is reachable at move
  cost at most `dash_tiles` in one block (`plan.rs:736-770`) with the target in the skill's reach band (K1);
  Disengage also needs a retreat tile after the hit. Hover card: "Range: move N + reach a-b". `dash_tiles` 0: no
  carried move, range = band (content warning, tasks 1.3).
- MV6. Destination: PL4 picks the tile among those MV5 allows, stored on the step (`Step.cell`, as the planned
  approach, `plan.rs:686-694`); classic's "Already in reach" refusal (`plan.rs:740`) does not apply (an Engage in
  band still moves to a flank tile). At run time a stored tile that is taken, out of band or (Engage) in the target's
  front is re-picked by PL4; with none the move is skipped (`[Dash Blocked]`) and the hit lands only if in band.
- MV7. Engage (an enemy): the opener is a Maneuver skill whose range covers the target; its carried move goes to the
  PL4 flank tile, then the hit; follow-ups as FU2, each moving only when its tile no longer meets the condition.
  Never face-on: with no flank tile in range, Engage is not offered (or MV9).
- MV8. Disengage (an enemy in the opener's band from the unit's tile; no move before): opener and the follow-ups that
  fit; the last Maneuver skill carries the retreat after its hit, to the safest PL4 tile within its `dash_tiles`. A
  DISENGAGE skill (Backflip) in the order goes last and retreats with its jump at its own AP.
- MV9. Target out of the opener's range (TQ22, open; the canon is silent): (a) a standard plain move first (IN5: 1 AP
  a tile, at most 2 move blocks, the round's one move) to the PL4 tile from which the range covers the target,
  reason `approach: out of range`; or (b) Engage is not offered ("Wolf is beyond Flank Cut's range: 6 > 4").
- MV10. Utility (an enemy or a tile, TQ16 a): items carry no move: a standard approach (as SP1) into the item's
  `battle_reach`, then the item; follow-ups are Maneuver skills within range (MV5).
- MV11. A Maneuver round needs a kind; the picker offers the possible ones; auto: Engage, else Disengage, else Utility.

### 5.4 Support

- SP1. Target self or an ally, never an enemy. A standard approach first when the ally is out of the opener's reach
  (`support_reach`, `skill.rs:214`; items `battle_reach`).
- SP2. Opener: a Support skill or a self or ally pouch item (`item_side_problem`, `command.rs:859`); follow-ups:
  Support skills on the same target (a SELF-only skill only on a self target). Heals and buffs; Focus is Support.

## 6. Rest (rebalance for the balance lane)

Today `rest` (1 block, 0 AP, `skill.rs:151`) gives 2 AP on landing (`command.rs:347-353`) and enters Tactical; with
every leftover block a Rest that is up to 10 AP a round, and AP would stop limiting heavy skills.

- RS1. Under timeline Rest is neutral, 1 block, 0 AP and never windup (P5); the planner puts it on every unused block
  and the player can place it by hand (a wait).
- RS2. A landed Rest gives no AP at once; at round end, after `ap_regen`, the unit gains
  `min(rest_ap_cap, floor(rest_blocks x rest_ap_per_block))` AP (TQ13 a: 0.5 and 2; a full Rest round gives 2, a
  Heavy round with two Rest blocks 1). Enemy-side units have no AP and skip it.
- RS3. Each landed Rest block restores `rest_poise_per_block` poise (5), up to Max Poise.
- RS4. Classic Rest is unchanged. TQ13 b (no AP, only poise; regen 4 to 5) is the fallback if Rest farms AP.

## 7. The auto planner (RULESET1-PL)

Deterministic: the same world, plans and inputs give the same plan; it draws no random number.

- PL1. Inputs: unit, intent, target, Maneuver kind, an optional opener (skill or pouch item). Output: plan steps
  through `Plan::add_step` (`plan.rs:348`; every rule is checked in one place), each with a reason tag.
- PL2. The target must fit the intent table; else the intent is refused with the reason.
- PL3. Opener: the given one, else the first order entry (FU4) that is affordable and acts on the target after the
  intent's before-move (for Maneuver: within range, MV5).
- PL4. Tiles for every implied and carried move: the tiles reachable within the move's limit that meet the intent's
  condition (band, outside the front, item reach, the Guard tile), ranked by (1) threat, the active foes whose
  next-round reach covers the tile (distance at most their 2-block move plus their reach), fewer first; (2) cover
  toward the target's side, more first (Guard and ranged only); (3) path tiles, fewer first; (4) distance to the
  target, farther first for kite and retreat, nearer for none; (5) cell index `y x 12 + x`, lower first.
- PL5. Placement order: before-move (approach, kite, MV9 a), opener with its carried move, reserved after-move blocks
  (AS3), fill (FU2, FU3), after-move, Rest. A Disengage retreat rides in its skill's blocks (MV8).
- PL6. Reason tags (strip tooltip and the Dojo): `approach`, `approach: out of range`, `kite: in melee`,
  `kite: threatened`, `flank tile`, `cover tile`, `opener`, `follow-up k` (its place in the order), `retreat`,
  `rest: <cause>` with the first refused entry's cause (`AP 1 < 2`, `blocks 1 < 2`, `out of reach`, `out of range`).
- PL7. No valid opener: the intent (or kind) is shown disabled with its reason ("No Assault skill reaches Wolf this
  round"). The planner never plans AP debt.
- PL8. A plan is rebuilt when intent, target, opener, order or an earlier member's plan changes (approach tiles are
  claims, PLAN1), in roster order.

## 8. Manual mode

- MM1. A per-member Manual toggle; intent and target are still set first, and `Plan::add_step` refuses another
  intent's skill ("Only Assault skills this round") or another target ("One target per round: Wolf").
- MM2. Manual adds the intent's skills in any order and repeat, Rest anywhere (to time an impact into a windup), the
  bundled moves as steps (Approach, Kite), a Maneuver skill with its destination clicked among the MV5 tiles
  (default PL4), a plain move (first, as today, `plan.rs:570-578`) and pouch items as follow-ups.
- MM3. "Fill the rest" runs PL5's fill after the last manual step; Manual off rebuilds the auto plan.
- MM4. What only manual can do: a delayed opener (Rest, Rest, a Heavy landing in b6), an out-of-order follow-up, a Snap
  between two Rests, items as follow-ups, a carried move to a tile PL4 ranks lower.

## 9. Enemies and the AI (RULESET1-AI)

- AI1. Enemies plan with intents: each keeps its intent across rounds (IN2) and re-picks intent and target each round
  by the first row of the table that applies; the intent planner then fills from its `deck.order` (auto when empty).
- AI2. Enemy-side units pay no AP and have no AP bar, and book `recovery_blocks` after each impact (T1-T4, T11).
- AI3. No authored sequences in M1 (TQ18 a): a boss's pattern is its order list and its skills.
- AI4. Tells (V1, UI5) show the round intent and target with the first action; badges unchanged.
- AI5. Party autoplay (AUTO1 tactics, `emberwake_gui/src/battle.rs:43`) maps to intent picks (TQ20 a): second table.

| Order | Condition | Intent, target |
| --- | --- | --- |
| 1 | `flee_ratio` reached | today's Withdraw (a neutral move); intent unchanged |
| 2 | Support role, or someone below `heal_below`, and a Support skill or item can reach | Support, the lowest HP share |
| 3 | HP share at most `defend_below`, not reckless, a Guard skill usable | Guard, self or the best cover tile in one move if a ranged foe threatens |
| 4 | Skirmisher with a Maneuver attack whose range covers a flank tile on a ranked target engaged with another unit | Maneuver Engage, that target |
| 5 | otherwise | Assault, the first ranked target (`rank_targets`, `ai.rs:595`); Snipers kite (AS2, AS3) |
| 6 | nothing usable | Guard self, else Rest |

| AUTO1 tactic | Intent and target |
| --- | --- |
| Balanced | the AI table |
| All-out Attack | Assault, the first ranked target |
| Focus Weakest | Assault, the lowest-HP foe reachable this round |
| Guard & Counter | Guard, self |
| Kite at Range | ranged: Assault (kites); melee: Maneuver Disengage when offered, else Assault |
| Economy | Assault with the cheapest opener and no follow-ups (Rest) |

## 10. Input, tells and display (RULESET1-U, -V, -G)

- UI1. Each member's strip row: four intent buttons (keys 1-4; Channel hidden until listed), the current one lit,
  a "loses Riposte" warning on a change that drops intent-bound buffs; Engage, Disengage and Utility under Maneuver;
  a board click sets the target (valid cells lit); an opener dropdown and the Manual toggle.
- UI2. The strip shows the plan with reason tags on hover (PL6), after-moves as step glyphs, Rest as dim blocks with
  the Rest gain at the row end ("Rest: 1 AP, 10 poise"), disabled intents with their reasons.
- UI3. The board: move paths (a carried move as its skill's path), the target ring, the target's front half shaded
  under Maneuver, enemy facings, a stealth veil.
- UI4. Dojo (to judge the planner): the hero's intent, target and kind against a movable dummy, a second dummy to
  engage it (sets its facing); planned blocks with reasons, AP before and after, Rest gain, expected damage per
  command; the order editor; the plan each intent would make now. No resolver (`dojo.rs:1-27`): plans compile on a
  scratch World (hero, dummies, the 12 x 9 grid); the Arena plays them.
- UI5. Tells and threat intel (user 2026-10-25: the round intent feeds them, for both sides). The tell record
  `Intent` (`intent.rs:37`) gains `round_intent` (`Option<RoundIntent>`) and `round_target` (self, unit or cell),
  traced under timeline only (V5); `intent::intents` stays enemy-only (classic shape), a new `intent::round_intents`
  covers every active unit (party records from their plans). The tell chip leads with the intent ("Assault: Wolf",
  `intent_text.rs`); the threat intel panel (`intel`, `emberwake_gui/src/battle.rs:1784`) shows the hovered unit's
  intent, target and Maneuver kind, enemy or party; strip rows and board share the glyph. The tell `kind` stays the
  first action's command kind (`KINDS`, `intent.rs:19`), so a Defend's "guard" never stands for the Guard intent.

What the Dojo, the Arena and the battle screen show: [RULESET1.md](RULESET1.md) section 12.

## 11. Code survey (develop, 2026-10-25; under `games/emberwake/crates/`: `b/` battle, `r/` rules, `gui/` gui src)

| Piece | Where | Today |
| --- | --- | --- |
| Scheduling, start | `b/resolver.rs:68-79,97-148`, `b/command.rs:159-161` | `end = start + blocks - 1`, no overlap; lands at `end` by initiative; AP paid at start (debt allowed); only the downed cancel |
| Guard, armour | `b/resolver.rs:84-95`, `r/damage.rs:11,122-138,202` | a running Defend raises `guard`; `min(0.9, guard + cover)`; `r / (r + 50)` |
| Classic stances | `r/unit.rs:15-48,686-691`, `damage.rs:196-204`, `b/skill.rs:43-52,83-146`, `b/plan.rs:246-250,1085-1087`, `b/resolver.rs:151-177` | four stances, matchup 1.25 / 0.75; by kind or `stance`; switch 1 AP; `disrupt_break`; a change removes `riposte` |
| AP | `b/skill.rs:149-156`, `b/session.rs:90` | `clamp(level, 1, 5) + modifier`; Move 1 a tile; Rest, Target, Status 0 |
| Basics | `emberwake_data/src/lib.rs:480-505` | `strike`, `offhand_jab`, `guard`, `move`, `rest`, `switch_target` |
| Move, Engage | `b/plan.rs:21,333-339,570-578,670-734` | once a round, first, at most 2 blocks, 1 AP a tile; approach, Switch Target, attack, chase |
| Dash, jump | `b/plan.rs:736-770`, `b/command.rs:167-176,472-497,579-617` | gap-close in `dash_tiles` (one block, nearest band tile, refused in reach); DISENGAGE jumps `jump_tiles` |
| Reach | `b/skill.rs:214-236`, `b/command.rs:244,260,538-549` | ranged (`attack_range > 1`): band (2, max(3, range)); melee (1, 1); support 0-3 |
| Statuses, tells | `r/status.rs:37-53`, `b/intent.rs:19-48,135-146` | fifteen rows; `Intent` per enemy: first action's kind and target, reveal |
| Threat intel | `gui/battle.rs:39,1784`, `gui/intent_text.rs:53-61` | the `intel` panel (hovered enemy); tell text |
| Loadout, skill row | `r/unit.rs:279-400`, `emberwake_data/src/lib.rs:155-245` | 5 class + 3 extra, passives always on; `blocks` 1-6, reach, `exploit` |
| Encounter, world | `b/session.rs:44-66`, `emberwake_world/src/data.rs:166` | no ruleset; `Session::start` 17 call sites; lineage fields |
| Dojo, Arena, sim | `b/dojo.rs:1-27,631`, `gui/runner.rs:124-127`, `emberwake_sim/src/args.rs:36-55` | no resolver, stance shift; any encounter; args outside the help |
| Facing, widget | `gui/battle.rs:141-188`; `engine/crates/aegis_ui/src/timeline.rs:20-58` | GUI-only left or right; segments, no marks or parts |
