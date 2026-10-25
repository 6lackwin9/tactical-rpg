# RULESET1: the "timeline" combat ruleset (M1 feel prototype)

Status: proposed (spec 2026-10-25 from CANON 2.1 and Q-48..Q-61, Q-70, Q-72, Q-91 of `decisions/lore.md`; revised
2026-10-25 to the user's round-stance model; open questions in section 14)
Depends on: DERIVE1-E/S/R (Max Poise is a derived row, TQ5); the M0 `sunk_ward` world skeleton (the world field's
data); KIT1 if it lands first (same modules, kit crate paths); STAGE1 and FX1 as the roadmap's M1 says
Lane: logic (R, S, T, ST, PL, AI, P, K, V, D, E, U, DU, G, X), asset (A), balance (B)
Touches: per task in [RULESET1_TASKS.md](RULESET1_TASKS.md) section 6
Companions: [RULESET1_STANCE.md](RULESET1_STANCE.md) (round stance, planner, AI, input, Dojo);
[RULESET1_TASKS.md](RULESET1_TASKS.md) (schema, assets, tests, parity, tasks, code survey). ENCOUNTER2 is separate.
Id note: renamed from TIMELINE1 by the planner (2026-10-25), which names the done timeline widget task (`docs/archive/TASKS_DONE.md:247`).
History (`docs/HISTORY.md`), nothing re-proposed: tick anchors (not adopted: windups stay relative); the 2026-10-25 cut
list (no roll returns; 5.5's break chance is a poise roll; facing and stealth return without cones or rolls by the
user's decision, see the stance companion); the AP-cost unlock (deprecated): `ap_cost` gates nothing.

## 1. Goal

A second ruleset, **timeline**, on the existing block resolver: a round stance per member that limits the round's
skills, one target per round, auto-planned combined commands with same-stance follow-ups, relative windups, enemy
recovery, `ap_cost` apart from level, poise and breaks, Active Guard as a basic, binary armour pierce, reach-2 melee,
flanks, a bucketed deck and impact badges. The Ashlands keep **classic**, byte-identical: every timeline rule sits
behind a flag classic never sets. M1 exit: a Dojo duel and a 4-on-4 in the Combat Arena play the stance, windup,
break and guard loop at level 1; the planner's choices are readable in the Dojo; golden traces recorded; a sim report;
classic parity unchanged.

## 2. Survey

File and line for every piece the rules change: [RULESET1_STANCE.md](RULESET1_STANCE.md) section 12.

## 3. The ruleset switch and the rule register

### 3.1 The switch `[kit hook]` (RULESET1-R)

1. World manifest field `ruleset`: `""` (inherit), `"classic"` or `"timeline"`. Resolution walks the lineage like
   `party_max`; nothing set anywhere means classic. Ashlands, Iron Reaches and Tidewrack resolve to classic with no
   data change; `sunk_ward` sets `"timeline"`.
2. A battle's ruleset is the ruleset of the world that owns the encounter (`d.resolve(world, "encounters", ...)`); a
   catalog encounter is classic unless a caller picks one. ENCOUNTER2's linked rooms all run their world's ruleset.
3. `EncounterDef` gains a runtime `ruleset: Ruleset` (not read from the encounter row; default classic) set by its
   caller: the battle bridge (world), the Combat Arena select, the Dojo toggle, the sim (`ruleset=`), the host
   `battle` command. `Session::start` keeps its signature.
4. `Ruleset { id, rules, numbers }`: classic is the empty set; timeline reads the row `rulesets.timeline` (companion
   1.2; a timeline world without it is a content error). The Arena and the sim may switch single extensions off.

### 3.2 Rule register

Option = incompatible with classic, on only under timeline. Extension = compatible and inert without data, still off
in classic so classic cannot drift when someone adds data; the Arena and the sim can disable each one.

| Rule id | Layer | Kind | Timeline effect | Off (and classic) |
| --- | --- | --- | --- | --- |
| `round_stance` | [kit hook] | option | stance companion: round stance, one target, follow-ups, combined commands, Rest, facing, stealth, stance buffs; classic stances, switch AP, matchup, `disrupt_break`, `ends_in_stance` inert | today's stances, Rest and planning |
| `enemy_moves` | [kit hook] | option | enemy-side units pay no AP and book `recovery_blocks` (4.2, T11) | AP for all, field ignored |
| `deck` | [kit hook] | option | section 9: 3 actives, 2 passives, 1 keystone, basics open to all, order lists | 5 class + 3 extra, all basics |
| `poise` | [kit hook] | extension | section 5 (with Active Guard's halving, 6) | no poise, no stagger, no breaks |
| `ap_cost` | [kit hook] | extension | the skill field replaces level as the AP base (4.4) | AP = level rule |
| `pierce` | [kit hook] | extension | `armor_piercing` skips the armour term (7) | field ignored |
| `reach` | [kit hook] | extension | family rule `reach`: melee band 1-2 (8) | reach 1 |
| `flank` | [kit hook] | extension | the flank bonus (stance MV3, TQ12) | no bonus (flanks still drawn) |
| `plan_badges` | [kit hook] | extension | impact badges, the stance in tells and the reveal flag (10) | today's tells only |

The first draft's `no_stances`, `recovery` and `chains` are folded into `round_stance` and `enemy_moves`.

### 3.3 Classic stays byte-identical

- Every new branch reads the flag first; classic builds no poise, draws no extra random number, writes no new trace,
  log, metrics or report key, and adds no plan entry.
- New data fields default to today's behaviour (`ap_cost` -1, `recovery_blocks` 0, `round_stance` "", the rest
  0/false/empty); classic ignores `round_stance` and the new status column.
- Catalog rows are only appended (skills are addressed by index: `skill.rs:9`); no Ashlands row changes.
- Bug fixes go to the core for both rulesets; one that moves a classic trace stops the task (CLAUDE.md parity rule).

## 4. Timing: windups, recovery, the round stance, AP `[kit hook]` (RULESET1-T, RULESET1-ST)

### 4.1 Relative windups `[data]`

Speed tiers are data through the existing `blocks`: Snap 1, Standard 2, Heavy 3-4. No new field. The **windup** of
an action is every block from its start to its **impact block** (its last windup block), where it lands as today.

### 4.2 Recovery blocks (enemy-side units, rule `enemy_moves`)

- T1. For an enemy-side unit a skill's `recovery_blocks` (0-2, default 0) are booked after the impact block: the action
  occupies `start .. impact + recovery`. `ScheduledAction` gains `impact`; the resolver lands actions on
  `impact == block` (classic: `impact == end`, so the same set lands in the same order).
- T2. The whole action, recovery included, must fit the round (`impact + recovery <= total_blocks`), as `schedule`
  checks today (TQ4: no spill into the next round).
- T3. Recovery is not windup (stagger x1, 5.2). A unit in recovery takes no other action.
- T4. Party-side units (heroes, guests) never book recovery (CANON: AP limits their heavy skills); the same skill row
  carries recovery for enemy use. A move after a recovery starts after it.

### 4.3 Round stance, one target, follow-ups (replaces T5-T9)

[RULESET1_STANCE.md](RULESET1_STANCE.md): one persisting stance a member (Assault, Guard, Support, Maneuver),
its skills only, one target a round, follow-ups in the member's order, leftover blocks Rest, combined commands.

### 4.4 AP cost apart from level

- T10. Skill field `ap_cost` (-1 = unset, else 0-10; per rank). Unset: today's rule exactly (`skill.rs:149`). Set:
  `max(0, ap_cost + ap_cost_modifier)`, at most `BASE_AP`; Move, Rest, Target and Status keep their rules. Guideline
  (CANON): Cold Steel 2-4, Inoculated 1-2 plus Strain (Act 2). `level` keeps gating unlocks and the AI `skill_cap`.
- T11. Under `enemy_moves` enemy-side units pay nothing at a start and show no AP; AP debt statuses (`status.rs`
  `DEBT_STAGGER`, `DEBT_STUN`) never reach them. Party-side AP is as T10, with Rest per the stance companion's RS2.

## 5. Poise and breaks `[kit hook]` (RULESET1-P)

### 5.1 Max Poise

- P1. Max Poise = the derived row named by the ruleset's `poise_stat` (DERIVE1; TQ5). Data: `max_poise` =
  `CONS * 2 + gear_mitigation * 1.0`, where `gear_mitigation` is a new formula input (sum of the unit's worn
  `effective_mitigation`, `gear.rs:156,298`; Q-51 b: the item's mitigation x a factor, factor 1.0 a planner pick).
  Passives add poise through DERIVE1's `stat` effects on `max_poise` (Q-51: "Braced Poise +20").
- P2. Unit field `poise` (-1 default = the derived value; 0 = no poise bar, never breaks; above 0 = a fixed Max Poise,
  for bosses and the Dojo dummy).
- P3. Poise refills to Max Poise at the start of every round (`begin_round`, before planning) and when a unit enters
  the battle (waves, ENCOUNTER2 room moves keep the current value). A unit swap (phase effect) recomputes it. Rest
  blocks restore some within the round (stance RS3).

### 5.2 Stagger and multipliers

- P4. An ATTACK skill's `stagger` (0-100, default 0, per rank) is dealt on impact to each unit it damages (single or
  area), after the damage and its status roll. Riposte counters, items, round-end ticks and hazards deal none in M1.
- P5. A unit is **in windup** when it has a started, not cancelled, not yet landed action of kind Attack, Area,
  Support, Focus or Item. Defend, Walk (bundled moves too), Disengage, Target, Rest and lost-block entries are never
  windup (revised: Rest left the list because the planner fills every spare block with it).
- P6. Stagger taken = `roundi(stagger x windup x guard x repeat x flank x passive x hook)`, floor 0, the factors
  multiply: windup = `stagger_windup_mult` (2.0) in windup; guard = `stagger_guard_mult` (0.5) while a Defend command
  of the unit runs (6); repeat = `stagger_repeat_mult` (0.5) per P10; flank = `flank_stagger_mult` on a flank hit
  (stance MV3; 1.0 when the `flank` rule is off; TQ12); passive = the product of the unit's `stagger_taken` effects
  that match the action's `blocks` (a Snap-proof trait: mult 0, max_blocks 1); hook = 1.0 (the Act 2 Flesh-Tension x2
  plugs in here through the meter spec; nothing in M1 sets it).

### 5.3 The break

- P7. When stagger takes poise to 0 or below, the unit **breaks** (in any state; TQ11): poise stays 0 until the next
  refill; every scheduled action of the unit that has not landed is cancelled, including one later in the same block
  (no mutual trades), its Rest blocks and its running Defend (its `guard` drops at once); AP already paid stays
  spent; actions not started pay nothing. Recovery of a landed action ends. Nothing else (no evasion or damage rider).
- P8. A unit breaks at most once per round; further stagger that round is not recorded as a break.
- P9. The log gets a break line (`[Break] <unit>'s <skill> is broken`); metrics count breaks, windup breaks and
  chances rolled per unit (timeline-only keys).
- P10. A unit broken in the previous round takes stagger x `stagger_repeat_mult` this round: its second break costs
  double stagger (CANON 2.1). The same-round case cannot occur (P8).

### 5.4 Heavy enemy windups

- P11. `windup_poise` (0-100, per rank): poise added when the action starts and removed when it lands or is
  cancelled (never below 0 on removal: the unit keeps `max(0, poise - bonus)`).
- P12. `unbreakable` (family flag): while this action is in windup the unit's poise cannot drop below 1. The status
  row `steadfast` (new, a buff: "cannot be broken") does the same for every action while active.

### 5.5 Stagger intent: break chance and guaranteed break (Q-50)

A break is a tactical choice: windups double stagger, so the player reads a tell or badge and times a hit into it
(manual mode can place a Snap in the impact block, stance MM4).

- P13. Break chance (ruleset `break_chance_scale`, 0 = off; `break_chance_cap`): a hit on a unit in windup whose
  stagger taken leaves poise above 0 rolls `chance = min(cap, floor(100 x taken / poise_before x scale))` with one
  `randi_range(1, 100) <= chance` draw on the resolver RNG, after the hit's damage, status and riposte draws; a
  success breaks the unit. No roll for unbreakable or steadfast units or when `taken` is 0.
- P14. Guaranteed break (TQ1 a, decided): a family flag `sure_break` on ATTACK skills: a hit that lands in a windup
  breaks the target whatever its poise, unless unbreakable or steadfast. The cost is the skill itself: its AP (author
  it 1 above a plain skill of its tier), a deck slot and its own stagger. A meter cost (Strain, Act 2) is a later
  field on the same flag (`sure_break_cost`), not built now.

### 5.6 Stagger rescale `[data]`

Bars run about 20-60 at P1 (lore stagger assumed 30-70). Engine stagger = lore x 0.5, rounded (TQ6): Pommel Strike
25 -> 12, Jaw-Breaker 30 -> 15, wall slam 20 -> 10; tiers Snap 8-12, Standard 6-12, Heavy 15-25. A Snap of 12 in
windup (24) breaks light units; heavies need a second hit, the chance roll or `sure_break`. Balance sets the factor.

## 6. Active Guard `[kit hook]`, `[data]`

- G1. Active Guard is a timeline basic (the ruleset's `basics`, no slot, open to every unit; section 9) of the Guard
  stance, the data row `active_guard`: DEFEND, blocks 2, `ap_cost` 1, mitigation 0.5, `basic` false so classic units
  never own it. It runs through the existing guard term: `min(GUARD_CAP 0.9, guard + cover)` (`damage.rs:202`).
- G2. While any Defend command of the unit runs (not broken), stagger taken is x0.5 (P6). Under timeline the classic
  `guard` basic is not offered (the ruleset's `basics` replace it).
- G3. Act 2 hook only: Active Guard vents 10 extra Strain at round end (the meter spec reads "held a Defend this
  round" from the round record; nothing here).

## 7. Armour and pierce `[kit hook]`, `[data]`

- A1. The curve stays `r / (r + 50)` (`damage.rs:11`); expose, fortify and the unit `armor` override unchanged.
- A2. Skill family flag `armor_piercing`: `HitInput` gains `ignore_armor`; the hit's armour term is 1.0 and the
  breakdown says "armour pierced". Guard and cover still apply. Classic data never sets it.
- A3. Test data (Q-49): armoured enemies carry `armor` 25, 35 and 50 (cut 33%, 41%, 50%) so the pierce skill gains
  33-100% over a plain hit; the founders' Act 1 kits have no pierce (Inoculation is Act 2), so the Dojo shows the
  trade without Strain. Strain itself is out of scope (its cost hook: `strain_cost`, the meter spec).

## 8. Reach-2 melee `[kit hook]` (RULESET1-K, Q-70)

- K1. A weapon family with the new engine rule `reach` (melee only: not with `always_ranged`, base range 1) gives its
  wielder a **melee reach** of 2. Families (TQ8): `polearm` (spears, billhooks, halberds) and `scourge`.
- K2. Melee attacks without their own `reach_max` use the band (1, melee reach). `attack_range` stays 1, so the unit
  stays melee for cover (none), riposte, AI role, kiting (none) and deployment; off-hand jabs, LINE and SWEEP stay
  (1, 1). The melee band also defines "engaged" (stance MV2).
- K3. Distance is the grid's Manhattan distance as today; no line of sight (Q-54): a reach-2 strike passes over a unit
  or an obstacle between.
- K4. Movement: an approach, a dash (`dash_tiles`) and a Maneuver move stop at the nearest tile in the band, so a
  polearm stops 2 tiles away when it can; the chase (`follow_tiles`) steps only while the target is outside the band.
- K5. Riposte answers only at distance 1 (`command.rs:782`): a reach-2 strike from distance 2 is not countered.
- K6. Guard: Active Guard and cover apply as to any melee hit. Previews: the battle screen's reach highlight and the
  attack forecast use the band; the hover card names "Reach 1-2".

## 9. The deck `[kit hook]` (RULESET1-D, Q-48)

- D1. Slot kinds: active, passive, keystone; counts from the ruleset row (`deck_slots`: active 3, passive 2, keystone
  1). No base-move slots (user 2026-10-25): the ruleset's `basics` (`move`, `rest`, `strike`, `offhand_jab`,
  `active_guard`; TQ15 may add a Maneuver basic) are open to every unit, take no slot and cannot be removed. No other
  basic is usable; `switch_target` is never planned (stance TG3).
- D2. Active slots hold skill families the unit owns (class path or learned), as today's slots; the class and extra
  split does not apply.
- D3. Passive rows gain `slot`: `""` (source-granted, always on: today), `"passive"` or `"keystone"`. Under the
  `deck` rule a slottable passive applies only while slotted; classic ignores `slot` (applies as today).
- D4. A unit owns a slottable passive when a source grants it (world character, discipline, faction grant, named
  item) or, for a catalog unit, when its `deck` names it.
- D5. Data shape: catalog unit and party member field `deck` with keys `active` (families), `passive` (ids),
  `keystone` (ids) and `order` (stance -> families), companion 1.5; empty or missing keys = auto (actives: the first
  families the class path reaches, as today's default; no passives; order: stance FU4). Members save it (a new key;
  save migration step, companion 1.8).
- D6. UI: under a timeline world the party screen's Skills tab shows three buckets instead of 5 + 3: Actives (3),
  Passives (2), Keystone (1), filled by click or drag like today's slots; a read-only Basics row; and an order editor:
  one short list per stance (its actives and basics, drag to reorder). The Dojo shows the same for its heroes.
- D7. Open item OI1 (user 2026-10-25, not redesigned here): 3 actives spread over four stances may give thin variety
  per stance; the slot counts and the command UI are revisited after the M1 feel run (the sim reports the share of
  rounds where a stance had no active, companion 3).

## 10. Enemy plan visibility `[kit hook]` (RULESET1-V, Q-91)

- V1. Default (`plan_badges` on): today's tell (each enemy's first action, `intent.rs:136`) plus its round stance and
  target (stance AI4), and an **impact badge** on that enemy's strip row at the impact block of every live action
  other than a move, with a coarse kind (hit for Attack and Area, else other) and no skill, target or start (TQ7).
- V2. Full plan: when the battle's `plan_reveal` is set, every enemy is revealed as foresight reveals it today: each
  action with its windup, impact, recovery, target and moves (the strip and the board path).
- V3. `plan_reveal` is set by data: the world manifest field `plan_reveal_flag` names a game-state flag (Tess's
  reconnaissance, set by her quest in M2 data); the bridge sets `plan_reveal` when the flag is set. The Arena and
  Dojo have a toggle; the sim ignores it (the AI never reads tells).
- V4. Foresight gains an optional `range` (Q-58, the Crag Hawkeye reveal): with `range` N the holder reveals every
  enemy within N tiles at planning time; without it, today's rule.
- V5. `Intent` gains `stance` and `impacts` (block, kind); the trace writes them only under timeline (the classic shape
  stays).

## 11. Statuses and skill conventions `[data]` (Q-57, Q-58, Q-59, Q-61, Q-72)

- C1. A new status maps onto one of the fifteen when that row carries the effect within its caps; else it is a new
  row through a logic task (the table is code, `status.rs:37`; this spec adds `steadfast` and `stealth`). Suggested
  for the data pass (Q-57 decides with the data): corrosion, "shreds armour" -> `expose`; disarmed, mutilated ->
  `weaken` (cap 25 stays; Kaelen's -50% rescaled); frenzy -> `empower`; Arterial Rupture -> `bleed`; Pinned -> `root`;
  mark, blind, Flammable: no row fits.
- C2. Cut-rider skills (Q-58: Ghost of the Undercroft, Viscous Shadow, Cinder-Child's Reflex) are rebuilt on four
  levers: `exploit` power against a status, the attack chase (`follow_tiles`), foresight (V4) and Guard stealth.
- C3. Q-59: lore flat values become `power_mult` (and `heal_base`/`heal_mult`) at the asset pass; no skill points;
  Inoculated skills are `learned_only`; `riposte` stays a kit skill under timeline (Guard), its numbers reviewed with
  the data. Each lore skill gets one `round_stance` at the asset pass.
- C4. Q-61: Maren's Bile Phial is a per-unit battle meter, not built here (section 13). C5. Q-72: area shapes come from the engine's `Area` list (PLUS, BURST, LINE, SWEEP); a shape the data needs and
  the list lacks ("behind the target") becomes a kit task when that data is written.
- C6. No taint mechanics in Act 1: no Act 1 row may name Strain, Calcification or a founder's taint; `strain_cost`,
  the meter and the Channel stance wait for Act 2.

## 12. What the Dojo and the Arena show (RULESET1-G, -U, -X)

Moved to the stance companion, section 11.

## 13. Out of scope

- Strain, the crisis, Calcification, grafts; Act 2 hooks beyond their named slots (P6 hook, G3, P14, the Channel
  stance).
- The per-unit battle meter (Q-61 Bile Phial, Strain Q-63): a separate design spec.
- Forced movement, ground hazards from skills, auras, arena hazards, line of sight (own M1 items, Q-54, Q-55).
- ENCOUNTER2 (its swaps recompute Max Poise, P3; its stand may grant `steadfast`; its hold AI, H6, is a Guard round).
- The founders' class kits and the bestiary as data (a later M1 asset task; Q-73, Q-74).
- A free move after acting; a re-planning AI (Q-50 kept INTENT1); authored enemy move sets (TQ18 b); the slot and
  command-UI revisit (OI1); any Ashlands row, label or help text change.

## 14. Open questions for the user

Decided (user 2026-10-25): every TQ question as recommended. Chains apply to any skill type, not only heavy skills: a
chain adds a quick follow-up on the target and cuts the selecting and clicking per character; each member sets an intent
for the round and acts only within it. The intent model is being clarified with the user; section 6 (chains) will be
revised after.

Revised 2026-10-25 (the user's round-stance model, CANON 2.1 "Round stance (intent)"): T5-T9 and TQ9 (authored
`follow_ups`) are replaced by stance follow-ups (stance 4); the deck's base slots (D1) are gone and TQ3 holds as
Strike being a basic open to all; recovery is enemy-only (T1, T4) and enemies pay no AP (T11); Rest left the windup
list (P5); cover reduces damage, stealth makes locked attacks lose a unit that moved and ends on leaving Guard, and a
stance persists across rounds with free changes that drop stance buffs (user answers, stance GD4, GD5, ST2, ST6).
TQ1-TQ11 stay decided (TQ1 a, TQ2 a, TQ3 a, TQ4 a, TQ5 a, TQ6 a, TQ7 a, TQ8 a, TQ10 a, TQ11 a). Open item OI1: D7.

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| TQ12 | The flank bonus (rule `flank`) | (a) stagger x1.5 on every flank, no riposte from the flanked unit, damage only where a Maneuver skill's `exploit` names the condition `flanked`; (b) damage x1.25 and stagger x1.5 on every flank | a: flanks feed the break loop, damage stays authored per skill |
| TQ13 | Rest under timeline | (a) AP at round end 0.5 a Rest block, cap 2, plus 5 poise a block (RS2, RS3); (b) Rest gives no AP, only poise, and the round regen rises from 4 to 5 | a for the first sim; b if the sim shows Rest-heavy rounds farming AP |
| TQ14 | Changing the round target | (a) free: the target is the lock, Switch Target never planned; (b) one block as today when it differs from last round's | a: two inputs a round; a block tax on every change punishes the model |
| TQ15 | A Maneuver basic | (a) a basic open to all (Flank Cut: Snap, power 0.6, Maneuver) so every unit can Engage and Disengage; (b) Maneuver attacks only from deck actives | a: with 3 actives most units would have no Maneuver attack |
| TQ16 | Utility targets | (a) foe-aimed and tile items only (ally items stay Support); (b) any item the target allows | a: keeps Support the friendly-item stance |
| TQ17 | Round target downed mid-round | (a) the rest of the round becomes Rest; (b) the planner retargets the nearest foe | a: keeps the one-target rule and the plan readable |
| TQ18 | Enemy move sets | (a) the stance planner with each enemy's order list and recovery; (b) authored `move_sets` (skill sequences) per unit | a for M1; b only if boss patterns read badly in the Arena |
| TQ19 | Cost of the kite step | (a) a standard move (1 AP a tile, one block); (b) a Maneuver move | a: Maneuver stays the cheap mover; the sim watches ranged AP |
| TQ20 | AUTO1 tactics under timeline | (a) each tactic picks stance and target by the stance companion's table; (b) tactics off under timeline | a: autoplay and sims keep working |
| TQ21 | A locked attack that loses a stealthed unit | (a) lands on nothing at its impact, AP spent; (b) cancelled at its start when the unit has already moved, AP kept | a: committing to a target is the risk stealth punishes |
