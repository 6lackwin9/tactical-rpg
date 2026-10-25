# RULESET1: the "timeline" combat ruleset (M1 feel prototype)

Status: proposed (spec 2026-10-25 from CANON 2.1 and Q-48..Q-61, Q-70, Q-72, Q-91 of `decisions/lore.md`; revised
2026-10-25 to the user's round-intent model, first called the round stance; open questions in section 14)
Depends on: DERIVE1-E/S/R (Max Poise is a derived row, TQ5); the M0 `sunk_ward` world skeleton (the world field's
data); KIT1 if it lands first (same modules, kit crate paths); STAGE1 and FX1 as the roadmap's M1 says
Lane: logic (R, S, T, IN, PL, AI, P, K, V, D, E, U, DU, G, X), asset (A), balance (B)
Touches: per task in [RULESET1_TASKS.md](RULESET1_TASKS.md) section 6
Companions: [RULESET1_INTENT.md](RULESET1_INTENT.md) (round intent, planner, AI, input, tells, code survey);
[RULESET1_TASKS.md](RULESET1_TASKS.md) (schema, assets, tests, parity, tasks). ENCOUNTER2 is separate. Id: was
TIMELINE1 (renamed 2026-10-25; that names the done widget task, `docs/archive/TASKS_DONE.md:247`).
History (`docs/HISTORY.md`), nothing re-proposed: tick anchors (not adopted: windups stay relative); the 2026-10-25 cut
list (no roll returns; 5.5's break chance is a poise roll; facing and stealth return without cones or rolls by the
user's decision, see the intent companion); the AP-cost unlock (deprecated): `ap_cost` gates nothing.

## 1. Goal

A second ruleset, **timeline**, on the existing block resolver: a round intent per member that limits the round's
skills, one target per round, auto-planned combined commands with same-intent follow-ups, relative windups, enemy
recovery, `ap_cost` apart from level, poise and breaks, Active Guard as a basic, binary armour pierce, reach-2 melee,
flanks, a bucketed deck and impact badges. The Ashlands keep **classic**, byte-identical: every timeline rule sits
behind a flag classic never sets. M1 exit: a Dojo duel and a 4-on-4 in the Combat Arena play the intent, windup,
break and guard loop at level 1; the planner's choices are readable in the Dojo; golden traces recorded; a sim report;
classic parity unchanged.

## 2. Survey

File and line for every piece the rules change: [RULESET1_INTENT.md](RULESET1_INTENT.md) section 11.

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
| `round_intent` | [kit hook] | option | intent companion: round intent, one target, follow-ups, combined commands, carried moves, Rest, facing, stealth, intent-bound buffs; classic stances, switch AP, matchup, `disrupt_break`, `ends_in_stance` inert | today's stances, Rest and planning |
| `enemy_moves` | [kit hook] | option | enemy-side units pay no AP and book `recovery_blocks` (4.2, T11) | AP for all, field ignored |
| `deck` | [kit hook] | option | section 9: 3 actives, 2 passives, 1 keystone, basics open to all, order lists | 5 class + 3 extra, all basics |
| `poise` | [kit hook] | extension | section 5 (with Active Guard's halving, 6) | no poise, no stagger, no breaks |
| `ap_cost` | [kit hook] | extension | the skill field replaces level as the AP base (4.4) | AP = level rule |
| `pierce` | [kit hook] | extension | `armor_piercing` skips the armour term (7) | field ignored |
| `reach` | [kit hook] | extension | family rule `reach`: melee band 1-2 (8) | reach 1 |
| `flank` | [kit hook] | extension | the flank bonus (intent MV3, TQ12 a) | no bonus (flanks still drawn) |
| `plan_badges` | [kit hook] | extension | impact badges, the round intent in tells and threat intel, the reveal flag (10) | today's tells only |

The first draft's `no_stances`, `recovery`, `chains` folded into `round_intent` (was `round_stance`), `enemy_moves`.

### 3.3 Classic stays byte-identical

- Every new branch reads the flag first; classic builds no poise, draws no extra random number, writes no new trace,
  log, metrics or report key, and adds no plan entry.
- New data fields default to today's behaviour (`ap_cost` -1, `recovery_blocks` 0, `round_intent` "", the rest
  0/false/empty); classic ignores `round_intent` and the new status column `intent_bound`.
- Catalog rows are only appended (skills are addressed by index: `skill.rs:9`); no Ashlands row changes.
- Bug fixes go to the core for both rulesets; one that moves a classic trace stops the task (CLAUDE.md parity rule).

## 4. Timing: windups, recovery, the round intent, AP `[kit hook]` (RULESET1-T, RULESET1-IN)

### 4.1 Relative windups `[data]`

Speed tiers are data through the existing `blocks`: Snap 1, Standard 2, Heavy 3-4. No new field. The **windup** of
an action is every block from its start to its **impact block** (its last windup block), where it lands as today.

### 4.2 Recovery blocks (enemy-side units, rule `enemy_moves`)

- T1. For an enemy-side unit a skill's `recovery_blocks` (0-2, default 0) are booked after the impact block: the action
  occupies `start .. impact + recovery`. `ScheduledAction` gains `impact`; the resolver lands actions on
  `impact == block` (classic: `impact == end`, so the same set lands in the same order).
- T2. The whole action, recovery included, fits the round (`impact + recovery <= total_blocks`; TQ4: no spill).
- T3. Recovery is not windup (stagger x1, 5.2). A unit in recovery takes no other action.
- T4. Party-side units (heroes, guests) never book recovery (CANON: AP limits their heavy skills); the same skill row
  carries recovery for enemy use. A move after a recovery starts after it.

### 4.3 Round intent, one target, follow-ups (replaces T5-T9)

[RULESET1_INTENT.md](RULESET1_INTENT.md): one persisting intent a member, its skills only, one target a round,
follow-ups, Rest, combined commands, Maneuver skills carrying their own move, the intent in tells and threat intel.

### 4.4 AP cost apart from level

- T10. Skill field `ap_cost` (-1 = unset, else 0-10; per rank). Unset: today's rule exactly (`skill.rs:149`). Set:
  `max(0, ap_cost + ap_cost_modifier)`, at most `BASE_AP`; Move, Rest, Target and Status keep their rules. Guideline
  (CANON): Cold Steel 2-4, Inoculated 1-2 plus Strain (Act 2). `level` keeps gating unlocks and the AI `skill_cap`.
- T11. Under `enemy_moves` enemy-side units pay nothing at a start and show no AP; AP debt statuses (`status.rs`
  `DEBT_STAGGER`, `DEBT_STUN`) never reach them. Party-side AP is as T10, with Rest per the intent companion's RS2.

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
  blocks restore some within the round (intent RS3).

### 5.2 Stagger and multipliers

- P4. An ATTACK skill's `stagger` (0-100, default 0, per rank) is dealt on impact to each unit it damages (single or
  area), after the damage and its status roll. Riposte counters, items, round-end ticks and hazards deal none in M1.
- P5. A unit is **in windup** when it has a started, not cancelled, not yet landed action of kind Attack, Area,
  Support, Focus or Item. Defend, Walk (bundled moves too), Disengage, Target, Rest and lost-block entries are never
  windup (revised: Rest left the list because the planner fills every spare block with it).
- P6. Stagger taken = `roundi(stagger x windup x guard x repeat x flank x passive x hook)`, floor 0, the factors
  multiply: windup = `stagger_windup_mult` (2.0) in windup; guard = `stagger_guard_mult` (0.5) while a Defend command
  of the unit runs (6); repeat = `stagger_repeat_mult` (0.5) per P10; flank = `flank_stagger_mult` on a flank hit
  (intent MV3; 1.0 when the `flank` rule is off; TQ12 a); passive = the product of the unit's `stagger_taken` effects
  that match the action's `blocks` (a Snap-proof trait: mult 0, max_blocks 1); hook = 1.0 (the Act 2 Flesh-Tension x2
  plugs in here through the meter spec; nothing in M1 sets it).

### 5.3 The break

- P7. When stagger takes poise to 0 or below, the unit **breaks** (in any state; TQ11): poise stays 0 until the next
  refill; every scheduled action of the unit that has not landed is cancelled, including one later in the same block
  (no mutual trades), its Rest blocks and its running Defend (its `guard` drops at once); AP already paid stays
  spent; actions not started pay nothing. Recovery of a landed action ends. Nothing else (no evasion or damage rider).
- P8. A unit breaks at most once per round; further stagger that round is not recorded as a break.
- P9. Log line `[Break] <unit>'s <skill> is broken`; metrics (timeline-only keys): breaks, windup breaks, chances.
- P10. A unit broken in the previous round takes stagger x `stagger_repeat_mult` this round: its second break costs
  double stagger (CANON 2.1). The same-round case cannot occur (P8).

### 5.4 Heavy enemy windups

- P11. `windup_poise` (0-100, per rank): poise added when the action starts and removed when it lands or is
  cancelled (never below 0 on removal: the unit keeps `max(0, poise - bonus)`).
- P12. `unbreakable` (family flag): while this action is in windup the unit's poise cannot drop below 1. The status
  row `steadfast` (new, a buff: "cannot be broken") does the same for every action while active.

### 5.5 Stagger intent: break chance and guaranteed break (Q-50)

A break is a tactical choice: windups double stagger, so the player reads a tell or badge and times a hit into it
(manual mode can place a Snap in the impact block, intent MM4).

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
  intent, the data row `active_guard`: DEFEND, blocks 2, `ap_cost` 1, mitigation 0.5, `basic` false so classic units
  never own it. It runs through the existing guard term: `min(GUARD_CAP 0.9, guard + cover)` (`damage.rs:202`).
- G2. While any Defend command of the unit runs (not broken), stagger taken is x0.5 (P6). Under timeline the classic
  `guard` basic is not offered (the ruleset's `basics` replace it).
- G3. Act 2 hook only: Active Guard vents 10 extra Strain at round end (the meter spec reads the round record).

## 7. Armour and pierce `[kit hook]`, `[data]`

- A1. The curve stays `r / (r + 50)` (`damage.rs:11`); expose, fortify and the unit `armor` override unchanged.
- A2. Skill family flag `armor_piercing`: `HitInput` gains `ignore_armor`; the hit's armour term is 1.0 and the
  breakdown says "armour pierced". Guard and cover still apply. Classic data never sets it.
- A3. Test data (Q-49): `armor` 25, 35, 50 (cut 33%, 41%, 50%), so pierce gains 33-100% over a plain hit; the
  founders' Act 1 kits have no pierce (Inoculation is Act 2). Strain is out of scope (`strain_cost`, the meter spec).

## 8. Reach-2 melee `[kit hook]` (RULESET1-K, Q-70)

- K1. A weapon family with the new engine rule `reach` (melee only: not with `always_ranged`, base range 1) gives its
  wielder a **melee reach** of 2. Families (TQ8): `polearm` (spears, billhooks, halberds) and `scourge`.
- K2. Melee attacks without their own `reach_max` use the band (1, melee reach). `attack_range` stays 1, so the unit
  stays melee for cover (none), riposte, AI role, kiting (none) and deployment; off-hand jabs, LINE and SWEEP stay
  (1, 1). The melee band also defines "engaged" (intent MV2).
- K3. Distance is the grid's Manhattan distance as today; no line of sight (Q-54): a reach-2 strike passes over a unit
  or an obstacle between.
- K4. An approach and a dash stop at the nearest band tile (a polearm 2 tiles away when it can); a carried move takes
  the PL4 tile in the band (intent MV6); the chase (`follow_tiles`) steps only while the target is out of band.
- K5. Riposte answers only at distance 1 (`command.rs:782`): a reach-2 strike from distance 2 is not countered.
- K6. Guard: Active Guard and cover apply as to any melee hit. Previews: the battle screen's reach highlight and the
  attack forecast use the band; the hover card names "Reach 1-2".

## 9. The deck `[kit hook]` (RULESET1-D, Q-48)

- D1. Slot kinds: active, passive, keystone; counts from the ruleset row (`deck_slots`: active 3, passive 2, keystone
  1). No base-move slots (user 2026-10-25): the ruleset's `basics` (`move`, `rest`, `strike`, `offhand_jab`,
  `active_guard`, `flank_cut`: TQ15 a) are open to every unit, take no slot and cannot be removed. No other
  basic is usable; `switch_target` is never planned (intent TG3).
- D2. Active slots hold skill families the unit owns (class path or learned), as today's slots; the class and extra
  split does not apply.
- D3. Passive rows gain `slot`: `""` (source-granted, always on: today), `"passive"` or `"keystone"`. Under the
  `deck` rule a slottable passive applies only while slotted; classic ignores `slot` (applies as today).
- D4. A unit owns a slottable passive when a source grants it (world character, discipline, faction grant, named
  item) or, for a catalog unit, when its `deck` names it.
- D5. Data shape: catalog unit and party member field `deck` with keys `active` (families), `passive` (ids),
  `keystone` (ids) and `order` (intent -> families), companion 1.5; empty or missing keys = auto (actives: the first
  families the class path reaches, as today's default; no passives; order: intent FU4). Members save it (a new key;
  save migration step, companion 1.8).
- D6. UI: under a timeline world the party screen's Skills tab shows three buckets instead of 5 + 3: Actives (3),
  Passives (2), Keystone (1), filled by click or drag like today's slots; a read-only Basics row; and an order editor:
  one short list per intent (its actives and basics, drag to reorder). The Dojo shows the same for its heroes.
- D7. Open item OI1 (user 2026-10-25, not redesigned here): 3 actives spread over four intents may give thin variety
  per intent; the slot counts and the command UI are revisited after the M1 feel run (the sim reports the share of
  rounds where an intent had no active, tasks 3).

## 10. Enemy plan visibility `[kit hook]` (RULESET1-V, Q-91)

- V1. Default (`plan_badges` on): today's tell (each enemy's first action, `intent.rs:136`) plus its round intent and
  round target (intent UI5, AI4), and an **impact badge** on that enemy's strip row at the impact block of every live
  action but a move, with a coarse kind (hit for Attack and Area, else other) and no skill, target or start (TQ7).
- V2. Full plan: when the battle's `plan_reveal` is set, every enemy is revealed as foresight reveals it today: each
  action with its windup, impact, recovery, target and moves (the strip and the board path).
- V3. `plan_reveal` is set by data: the world manifest field `plan_reveal_flag` names a game-state flag (Tess's
  reconnaissance, set by her quest in M2 data); the bridge sets `plan_reveal` when the flag is set. The Arena and
  Dojo have a toggle; the sim ignores it (the AI never reads tells).
- V4. Foresight gains an optional `range` (Q-58, the Crag Hawkeye reveal): with `range` N the holder reveals every
  enemy within N tiles at planning time; without it, today's rule.
- V5. `Intent` (`intent.rs:37`) gains `round_intent`, `round_target` and `impacts` (block, kind); the trace writes
  them only under timeline (the classic shape stays). `intent::round_intents` adds the party side for the threat
  intel panel, strip rows and board glyphs (intent UI5); `intent::intents` stays enemy-only.

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
  the data. Each lore skill gets one `round_intent` at the asset pass.
- C4. Q-61: Maren's Bile Phial is a per-unit battle meter, not built here (section 13).
- C5. Q-72: area shapes come from the engine's `Area` list (PLUS, BURST, LINE, SWEEP); a shape the data needs and
  the list lacks ("behind the target") becomes a kit task when that data is written.
- C6. No taint mechanics in Act 1: no Act 1 row may name Strain, Calcification or a founder's taint; `strain_cost`,
  the meter and the Channel intent wait for Act 2.

## 12. What the Dojo and the Arena show (RULESET1-G, -U, -X)

- Gear Dojo: a Classic / Timeline toggle; under timeline the intent controls and planner view (intent UI4) replace
  the stance shift (`dojo.rs:631`); Max Poise with its DERIVE1 breakdown; poise bars (dummy `poise` 40); "Dummy in
  windup" beside "Dummy guards"; hit breakdowns add `stagger 12 x windup 2 x guard 0.5 = 12 -> poise 40 -> 28`,
  "flank", "armour pierced"; AP by `ap_cost`; "Next round" refills poise and applies the Rest gain; deck buckets (D6).
- Combat Arena: a World / Classic / Timeline select, the extension toggles (3.2), "Reveal enemy plans", a ruleset chip.
- Battle screen: intent UI1-UI3 and UI5; windup shading, impact markers, enemy recovery hatching, after-move glyphs,
  BREAK markers; badges or full plans (V1, V2); poise bars; forecasts ("in windup: stagger x2, breaks"; "may break:
  40%"; "flank"); the reach band (K6); a round summary (breaks, windups broken of hit, follow-ups, flanks, Rest gain).
- Exit evidence: golden traces (tasks 4.2), Arena recordings of `tl_duel_windup`, `tl_flank`, `tl_skirmish` in
  `games/emberwake/playtests/`, the sim report (RULESET1-B).

## 13. Out of scope

- Strain, the crisis, Calcification, grafts; Act 2 hooks beyond their slots (P6 hook, G3, P14, the Channel intent).
- The per-unit battle meter (Q-61 Bile Phial, Strain Q-63): a separate design spec.
- Forced movement, ground hazards from skills, auras, arena hazards, line of sight (own M1 items, Q-54, Q-55).
- ENCOUNTER2 (swaps recompute Max Poise, P3; its stand may grant `steadfast`; hold AI H6: Guard intent on a tile).
- The founders' class kits and the bestiary as data (a later M1 asset task; Q-73, Q-74).
- A free move after acting; a re-planning AI (Q-50 kept INTENT1); authored enemy move sets (TQ18 b); the slot and
  command-UI revisit (OI1); any Ashlands row, label or help text change.

## 14. Open questions for the user

Decided (user 2026-10-25): TQ1-TQ21 as recommended, option a each (TQ9's `follow_ups` gave way to intent follow-ups);
comments: TQ15's basic Flank Cut carries its own move like every Maneuver skill (intent MV4); TQ19's kite step stays
a standard move (AS3). Answers in: TQ12 MV3, P6; TQ13 RS2-RS3; TQ14 TG3; TQ15 D1; TQ16 MV10; TQ17 TG4; TQ18 AI3;
TQ20 AI5; TQ21 GD5. Chains apply to any skill type. Open item OI1: D7.

Revised 2026-10-25 (round model, CANON 2.1): follow-ups replace T5-T9; no base slots (D1); enemy-only recovery, no
enemy AP (T1, T4, T11); Rest left the windup list (P5); cover, stealth, a persisting free choice (GD4, GD5, IN2).

Revised 2026-10-25 (intent rename; CANON 2.1 "Round intent", decision "Round intent (renamed)"): the round stance is
the round intent (names: intent companion header; task RULESET1-IN; file RULESET1_INTENT.md); the classic stance
keeps its name and fields. It extends the tells and threat intel for both sides (V1, V5, UI5). Intent-bound buffs
drop on a change, all others keep their round durations (IN6). A Maneuver skill carries its own move, free with the
target in its range (MV4-MV10), replacing the half-AP maneuver move; `tl_flank_cut` is now `tl_hamstring_cut`.

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| TQ22 | A Maneuver opener whose range does not cover the target (intent MV9; the canon is silent) | (a) a standard plain move first (1 AP a tile, at most 2 blocks, the round's one move), then the skill carries the rest; (b) Engage is not offered: the player picks Assault to close in | a: Maneuver stays usable from afar at the plain move's price; being in range still saves that AP |
| TQ23 | A unit's intent at battle start (Q-40a c: temperament gives a default intent) | (a) a unit and member field `default_intent` ("" = none) now, filled later by the temperament spec; (b) none until the temperament spec adds it | a: one field; the first round needs one input less |
