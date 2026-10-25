# RULESET1: the "timeline" combat ruleset (M1 feel prototype)

Status: proposed (spec 2026-10-25, from CANON 2.1 and the decided rows Q-48..Q-61, Q-70, Q-72, Q-91 of
`games/emberwake/docs/decisions/lore.md`; open questions in section 14)
Depends on: DERIVE1-E/S/R (Max Poise is a derived row, TQ5); the M0 `sunk_ward` world skeleton (the world field's
data); KIT1 if it lands first (same modules, kit crate paths); STAGE1 and FX1 as the roadmap's M1 says
Lane: logic (R, S, T, P, K, V, D, E, U, G, X), asset (A), balance (B)
Touches: per task in [RULESET1_TASKS.md](RULESET1_TASKS.md) section 6
Companion: [RULESET1_TASKS.md](RULESET1_TASKS.md) (schema, assets, tests, parity, tasks, docs delta, code survey).
ENCOUNTER2 (linked rooms, stay-behind objectives, boss phase effects) is a separate spec, referenced where it touches.
Id note: renamed from TIMELINE1 by the planner (2026-10-25), which names the done timeline widget task (`docs/archive/TASKS_DONE.md:247`).
History (`docs/HISTORY.md`), nothing re-proposed: tick anchors (not adopted: windups stay relative); the 2026-10-25 cut
list (no roll returns; 5.6's break chance is a poise roll); the AP-cost unlock (deprecated): `ap_cost` gates nothing.

## 1. Goal

A second ruleset, **timeline**, on the existing block resolver: relative windups with optional recovery and chained
follow-ups, `ap_cost` apart from level, poise and breaks, Active Guard as a base move, binary armour pierce, reach-2
melee, a bucketed deck and impact badges. The Ashlands keep **classic**, byte-identical: every timeline rule sits
behind a flag classic never sets. M1 exit: a Dojo duel and a 4-on-4 in the Combat Arena play the windup, break, chain
and guard loop at level 1; golden traces recorded; a sim report; classic parity unchanged.

## 2. Survey

File and line for every piece the rules change: [RULESET1_TASKS.md](RULESET1_TASKS.md) section 8. In short: the
resolver lands an action at the end of its own blocks in initiative order and cancels only a downed unit's actions;
AP is the skill level; `attack_range > 1` makes a unit ranged; tells show each enemy's first action; the loadout is
5 class + 3 extra families with every basic free; passives are always on; the Dojo has no resolver, the Combat Arena
has; a new data field joins the parity strips (`AFTER_GODOT`).

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
| `no_stances` | [kit hook] | option | stances, switch cost, matchup factor, `disrupt_break`, `ends_in_stance` inert; units stay Neutral | today's stances |
| `deck` | [kit hook] | option | section 9: base moves, 3 actives, 2 passives, 1 keystone | 5 class + 3 extra, all basics |
| `poise` | [kit hook] | extension | section 5 (with Active Guard's halving, 6) | no poise, no stagger, no breaks |
| `recovery` | [kit hook] | extension | `recovery_blocks` booked after the impact (4.2) | field ignored |
| `chains` | [kit hook] | extension | follow-ups start in the lead's recovery (4.3) | recovery always paid |
| `ap_cost` | [kit hook] | extension | the skill field replaces level as the AP base (4.4) | AP = level rule |
| `pierce` | [kit hook] | extension | `armor_piercing` skips the armour term (7) | field ignored |
| `reach` | [kit hook] | extension | family rule `reach`: melee band 1-2 (8) | reach 1 |
| `plan_badges` | [kit hook] | extension | impact badges and the reveal flag (10) | today's tells only |

### 3.3 Classic stays byte-identical

- Every new branch reads the flag first; classic builds no poise, draws no extra random number, writes no new trace,
  log, metrics or report key, and adds no plan entry.
- New data fields default to today's behaviour (`ap_cost` -1, `recovery_blocks` 0, the rest 0/false/empty).
- Catalog rows are only appended (skills are addressed by index: `skill.rs:9`); no Ashlands row changes.
- Bug fixes go to the core for both rulesets; one that moves a classic trace stops the task (CLAUDE.md parity rule).

## 4. Timing: windups, recovery, chains, AP `[kit hook]` (RULESET1-T)

### 4.1 Relative windups `[data]`

Speed tiers are data through the existing `blocks`: Snap 1, Standard 2, Heavy 3-4. No new field. The **windup** of
an action is every block from its start to its **impact block** (its last windup block), where it lands as today.

### 4.2 Recovery blocks

- T1. A skill's `recovery_blocks` (0-2, default 0) are booked after the impact block: the action occupies
  `start .. impact + recovery`. `ScheduledAction` gains `impact`; the resolver lands actions on `impact == block`
  (classic: `impact == end`, so the same set lands in the same order).
- T2. The whole action, recovery included, must fit the round (`impact + recovery <= total_blocks`), as `schedule`
  checks today (TQ4: no spill into the next round).
- T3. Recovery is not windup (stagger x1, 5.2). A unit in recovery takes no other action unless a chain starts (T4).
- T4. Moves never chain; a move after a lead starts after its recovery.

### 4.3 Chained commands

Under revision (user 2026-10-25): chains become same-stance follow-ups inside a round stance (CANON 2.1 "Round
stance"); the base-move slots leave the deck (section 9); recovery for player units, kiting, flanking and chain order
are being clarified. T5-T9 below are superseded once the revision lands.

- T5. A lead skill lists the skill families that may follow it: `follow_ups` (at most 4; the lead needs
  `recovery_blocks >= 1`). TQ9 covers the follower-side alternative.
- T6. When a unit's next planned step after the lead is a skill of a listed family, the follow-up starts at
  `impact + 1` and the lead's recovery is waived (its booking ends at its impact). Otherwise the next step starts after
  the recovery. Each command pays its own AP at its own start; there is no chain discount.
- T7. A follow-up may itself be a lead (chains of chains are data). A break (5.3) cancels a follow-up not yet landed.
- T8. Input (the battle screen, RULESET1-U): under a lead skill the skill list shows one **chain pick** per affordable
  follow-up in the deck ("Heavy Cleave > Shoulder Check"); one pick and one target plan both steps on that target (a
  self-targeted follow-up plans on self): two inputs instead of four. The strip draws a link between the two and no
  recovery hatching on the waived blocks.
- T9. AI: after drafting a lead, the AI takes a listed follow-up next when its draft would add another attack on the
  same target and the follow-up is in its deck and affordable; otherwise unchanged.

### 4.4 AP cost apart from level

- T10. Skill field `ap_cost` (-1 = unset, else 0-10; per rank). Unset: today's rule exactly (`skill.rs:149`). Set:
  `max(0, ap_cost + ap_cost_modifier)`, at most `BASE_AP`; Move, Rest, Target and Status keep their rules. Guideline
  (CANON): Cold Steel 2-4, Inoculated 1-2 plus Strain (Act 2). `level` keeps gating unlocks and the AI `skill_cap`.

## 5. Poise and breaks `[kit hook]` (RULESET1-P)

### 5.1 Max Poise

- P1. Max Poise = the derived row named by the ruleset's `poise_stat` (DERIVE1; TQ5). Data: `max_poise` =
  `CONS * 2 + gear_mitigation * 1.0`, where `gear_mitigation` is a new formula input (sum of the unit's worn
  `effective_mitigation`, `gear.rs:156,298`; Q-51 b: the item's mitigation x a factor, factor 1.0 a planner pick).
  Passives add poise through DERIVE1's `stat` effects on `max_poise` (Q-51: "Braced Poise +20").
- P2. Unit field `poise` (-1 default = the derived value; 0 = no poise bar, never breaks; above 0 = a fixed Max Poise,
  for bosses and the Dojo dummy).
- P3. Poise refills to Max Poise at the start of every round (`begin_round`, before planning) and when a unit enters
  the battle (waves, ENCOUNTER2 room moves keep the current value). A unit swap (phase effect) recomputes it.

### 5.2 Stagger and multipliers

- P4. An ATTACK skill's `stagger` (0-100, default 0, per rank) is dealt on impact to each unit it damages (single or
  area), after the damage and its status roll. Riposte counters, items, round-end ticks and hazards deal none in M1.
- P5. A unit is **in windup** when it has a started, not cancelled, not yet landed action of kind Attack, Area,
  Support, Focus, Item or Rest. Defend, Walk, Disengage, Target and lost-block entries are never windup.
- P6. Stagger taken = `roundi(stagger x windup x guard x repeat x passive x hook)`, floor 0, the factors multiply:
  windup = `stagger_windup_mult` (2.0) in windup; guard = `stagger_guard_mult` (0.5) while a Defend command of the
  unit runs (6); repeat = `stagger_repeat_mult` (0.5) per P10; passive = the product of the unit's `stagger_taken`
  effects that match the action's `blocks` (a Snap-proof trait: mult 0, max_blocks 1); hook = 1.0 (the Act 2
  Flesh-Tension x2 plugs in here through the meter spec; nothing in M1 sets it).

### 5.3 The break

- P7. When stagger takes poise to 0 or below, the unit **breaks** (in any state; TQ11): poise stays 0 until the next
  refill; every scheduled action of the unit that has not landed is cancelled, including one later in the same block
  (no mutual trades) and its running Defend (its `guard` drops at once); AP already paid stays spent; actions not
  started pay nothing. Recovery of a landed action ends. Nothing else (no evasion or damage rider).
- P8. A unit breaks at most once per round; further stagger that round is not recorded as a break.
- P9. The log gets a break line (`[Break] <unit>'s <skill> is broken`); metrics count breaks, windup breaks and
  chances rolled per unit (timeline-only keys).

### 5.4 Repeated breaks

- P10. A unit broken in the previous round takes stagger x `stagger_repeat_mult` this round: its second break costs
  double stagger (CANON 2.1). The same-round case cannot occur (P8).

### 5.5 Heavy enemy windups

- P11. `windup_poise` (0-100, per rank): poise added when the action starts and removed when it lands or is
  cancelled (never below 0 on removal: the unit keeps `max(0, poise - bonus)`).
- P12. `unbreakable` (family flag): while this action is in windup the unit's poise cannot drop below 1. The status
  row `steadfast` (new, a buff: "cannot be broken") does the same for every action while active.

### 5.6 Stagger intent: break chance and guaranteed break (Q-50)

A break is a tactical choice: windups double stagger, so the player reads a tell or badge and times a hit into it.

- P13. Break chance (ruleset `break_chance_scale`, 0 = off; `break_chance_cap`): a hit on a unit in windup whose
  stagger taken leaves poise above 0 rolls `chance = min(cap, floor(100 x taken / poise_before x scale))` with one
  `randi_range(1, 100) <= chance` draw on the resolver RNG, after the hit's damage, status and riposte draws; a
  success breaks the unit. No roll for unbreakable or steadfast units or when `taken` is 0.
- P14. Guaranteed break, recommended form (TQ1 a): a family flag `sure_break` on ATTACK skills: a hit that lands in a
  windup breaks the target whatever its poise, unless unbreakable or steadfast. The cost is the skill itself: its AP
  (author it 1 above a plain skill of its tier), a deck slot and its own stagger. A meter cost (Strain, Act 2) is a
  later field on the same flag (`sure_break_cost`), not built now.

| Option (TQ1) | Feel | Cost to build |
| --- | --- | --- |
| a. `sure_break` skill flag | a few signature interrupts (the prologue's Pommel Strike lesson) | one flag, data |
| b. "Commit" on any attack, +N AP | every attack can become an interrupt; more choice per round | plan field, UI toggle, AI rule |
| c. Meter cost (Strain) | risk and reward, Act 2 | waits for the meter spec (Q-63) |

### 5.7 Stagger rescale `[data]`

Lore stagger was set for bars of 30-70; at P1 the bars run about 20-60 (CONS 6-18 x 2 plus 2-22 of armour). First-sim
rule: engine stagger = lore stagger x 0.5, rounded (TQ6): Pommel Strike 25 -> 12, Jaw-Breaker 30 -> 15, wall slam 20
-> 10. Tier guideline: Snap 8-12, Standard 6-12, Heavy 15-25. In windup a Snap of 12 deals 24: it breaks light units
(about 20-30) and needs a second hit, the chance roll or `sure_break` on heavies (about 40-60). The balance lane sets
the final factor.

## 6. Active Guard `[kit hook]`, `[data]`

- G1. Active Guard is a base move (section 9), the data row `active_guard`: DEFEND, blocks 2, `ap_cost` 1, mitigation
  0.5. It runs through the existing guard term: `min(GUARD_CAP 0.9, guard + cover)` (`damage.rs:202`).
- G2. While any Defend command of the unit runs (not broken), stagger taken is x0.5 (P6). Under timeline the classic
  `guard` basic is not offered (the deck's base list replaces it).
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
  stays melee for cover (none), riposte, AI role and deployment; off-hand jabs, LINE and SWEEP stay (1, 1).
- K3. Distance is the grid's Manhattan distance as today; no line of sight (Q-54): a reach-2 strike passes over a unit
  or an obstacle between.
- K4. Movement: a dash (`dash_tiles`) stops at the nearest tile in the band, so a polearm stops 2 tiles away when it
  can; the chase (`follow_tiles`) steps only while the target is outside the band. Holding ground is free.
- K5. Riposte answers only at distance 1 (`command.rs:782`): a reach-2 strike from distance 2 is not countered.
- K6. Guard: Active Guard and cover apply as to any melee hit. Previews: the battle screen's reach highlight and the
  attack forecast use the band; the hover card names "Reach 1-2".

## 9. The deck `[kit hook]` (RULESET1-D, Q-48)

- D1. Slot kinds: base, active, passive, keystone; counts from the ruleset row (`deck_slots`: active 3, passive 2,
  keystone 1). Base moves are `deck_base` (`move`, `active_guard`): always owned, shown, never changed. `deck_system`
  (`rest`, `switch_target`, and `strike` per TQ3) stays usable and is not shown. No other basic is usable.
- D2. Active slots hold skill families the unit owns (class path or learned), as today's slots; the class and extra
  split does not apply.
- D3. Passive rows gain `slot`: `""` (source-granted, always on: today), `"passive"` or `"keystone"`. Under the
  `deck` rule a slottable passive applies only while slotted; classic ignores `slot` (applies as today).
- D4. A unit owns a slottable passive when a source grants it (world character, discipline, faction grant, named
  item) or, for a catalog unit, when its `deck` names it.
- D5. Data shape: catalog unit and party member field `deck` = `{"active": [families], "passive": [ids],
  "keystone": [ids]}`; empty or missing keys = auto (the first families the class path reaches, as today's default;
  no passives). Members save it (a new key; save migration step, companion 1.8).
- D6. UI: under a timeline world the party screen's Skills tab shows four buckets instead of 5 + 3: Base (two locked
  cards), Actives (3), Passives (2), Keystone (1), filled by click or drag like today's slots; the Dojo shows the same
  buckets for its heroes. The AI drafts from the base, system and active skills only.

## 10. Enemy plan visibility `[kit hook]` (RULESET1-V, Q-91)

- V1. Default (`plan_badges` on): today's tell (each enemy's first action, `intent.rs:136`) plus an **impact badge**
  on that enemy's strip row at the impact block of every live action other than a move, with a coarse kind (hit for
  Attack and Area, else other) and no skill, target or start (TQ7).
- V2. Full plan: when the battle's `plan_reveal` is set, every enemy is revealed as foresight reveals it today: each
  action with its windup, impact, recovery, target and moves (the strip and the board path).
- V3. `plan_reveal` is set by data: the world manifest field `plan_reveal_flag` names a game-state flag (Tess's
  reconnaissance, set by her quest in M2 data); the bridge sets `plan_reveal` when the flag is set. The Arena and
  Dojo have a toggle; the sim ignores it (the AI never reads tells).
- V4. Foresight gains an optional `range` (Q-58, the Crag Hawkeye reveal): with `range` N the holder reveals every
  enemy within N tiles at planning time; without it, today's rule.
- V5. `Intent` gains `impacts` (block, kind); the trace writes it only under timeline (the classic shape stays).

## 11. Statuses and skill conventions `[data]` (Q-57, Q-58, Q-59, Q-61, Q-72)

- C1. A new status maps onto one of the fifteen when that row carries the effect within its caps; else it is a new
  row through a logic task (the table is code, `status.rs:37`). Suggested for the data pass (Q-57 decides with the
  data): corrosion, "shreds armour" -> `expose`; disarmed, mutilated -> `weaken` (cap 25 stays; Kaelen's -50%
  rescaled); frenzy -> `empower`; Arterial Rupture -> `bleed`; Pinned -> `root`; mark, blind, Flammable: no row fits.
- C2. Cut-rider skills (Q-58: Ghost of the Undercroft, Viscous Shadow, Cinder-Child's Reflex) are rebuilt on three
  levers: `exploit` power against a status, the attack chase (`follow_tiles`), and foresight (V4).
- C3. Q-59: lore flat values become `power_mult` (and `heal_base`/`heal_mult`) at the asset pass; no skill points;
  Inoculated skills are `learned_only`; `riposte` stays a kit skill under timeline, its numbers reviewed with the data.
- C4. Q-61: Maren's Bile Phial is a trait counter, a per-unit battle meter; not built here (section 13).
- C5. Q-72: area shapes come from the engine's `Area` list (PLUS, BURST, LINE, SWEEP); a shape the data needs and
  the list lacks ("behind the target") becomes a kit task when that data is written.
- C6. No taint mechanics in Act 1: no Act 1 row may name Strain, Calcification or a founder's taint; `strain_cost`
  and the meter wait for Act 2.

## 12. What the Dojo and the Arena show (RULESET1-G, -U, -X)

- Gear Dojo: a Classic / Timeline toggle. Under timeline: no stance controls; Max Poise with its DERIVE1 breakdown;
  poise bars for hero and dummy (dummy `poise` 40); "Dummy in windup" beside "Dummy guards"; hit breakdowns add
  `stagger 12 x windup 2 x guard 0.5 = 12 -> poise 40 -> 28` and "armour pierced"; AP by the `ap_cost` rule; "Next
  round" refills poise; the deck buckets (D6).
- Combat Arena: a World / Classic / Timeline select, the extension toggles (3.2), "Reveal enemy plans", a ruleset chip.
- Battle screen: windup shading, impact markers, recovery hatching, chain links and BREAK markers on the strip; badges
  (V1) or full plans (V2); poise bars on unit and hover cards; a target forecast ("in windup: stagger x2, breaks";
  "may break: 40%"); the reach band (K6); a round summary line: breaks, windups broken of windups hit, chains, guards.
- Exit evidence: golden traces (companion 4.3), Arena recordings of `tl_duel_windup` and `tl_skirmish` copied to
  `games/emberwake/playtests/`, and the sim report (RULESET1-B).

## 13. Out of scope

- Strain, the crisis, Calcification, grafts; Act 2 hooks beyond their named slots (P6 hook, G3, P14).
- The per-unit battle meter (Q-61 Bile Phial, Strain Q-63): a separate design spec.
- Forced movement, ground hazards from skills, auras, arena hazards, line of sight (own M1 items, Q-54, Q-55).
- ENCOUNTER2 (rooms, stay-behind objectives, phase effects); its unit swaps recompute Max Poise (P3), its heroic stand
  may grant `steadfast`.
- The founders' class kits and the bestiary as data (a later M1 asset task; Q-73, Q-74).
- Act-then-move maneuvers (Q-54); a re-planning AI (Q-50 kept INTENT1); any Ashlands row, label or help text change.

## 14. Open questions for the user

Decided (user 2026-10-25): every TQ question as recommended. Chains apply to any skill type, not only heavy skills: a
chain adds a quick follow-up on the target and cuts the selecting and clicking per character; each member sets an intent
for the round and acts only within it. The intent model is being clarified with the user; section 6 (chains) will be
revised after.

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| TQ1 | The guaranteed break's cost (Q-50) | a, b, c of the 5.6 table | a for M1, c as the Act 2 field; b only if the sim shows too few interrupt choices |
| TQ2 | Break chance in M1 | (a) on: scale 0.5, cap 50, toggle in the Arena; (b) off by default, sim only | a: the user asked for some randomness; the sim reports both |
| TQ3 | Basic Strike under timeline | (a) a system move outside the deck, with Rest and Switch Target; (b) gone: attacks come from the 3 actives | a: every unit keeps an attack at level 1 (Q-41) and the AI keeps its cheapest strike |
| TQ4 | Recovery at the round's end | (a) the action and its recovery must fit the round; (b) recovery spills into the next round's first blocks | a: simple and makes a late Heavy impossible, a planning commitment |
| TQ5 | Where Max Poise lives | (a) DERIVE1 row `max_poise` with a `gear_mitigation` input; (b) numbers in the ruleset row, formula in code | a: one formula system; passives add poise as data |
| TQ6 | First-sim stagger factor | (a) lore x 0.5; (b) lore x 0.65 (bar 32 / 50) | a: a break should need a windup plus a second hit or a tool |
| TQ7 | Badges without reconnaissance | (a) every non-move impact with hit / other; (b) damaging impacts only, no kind | a: reads danger and timing without the plan |
| TQ8 | Reach-2 families | (a) `polearm` and `scourge` families with the `reach` rule (Q-71 b); (b) a per-item reach field | a: one rule per family, as Q-70 named polearms, billhooks and scourges |
| TQ9 | How chains are authored | (a) the lead lists `follow_ups`; (b) followers list `chains_from` | a: the lead owns the recovery it waives |
| TQ10 | Max Poise armour factor (planner pick) | (a) 1.0; (b) 0.5 | a: armour poise 2-22 matches CONS x 2 in weight |
| TQ11 | Breaks outside a windup | (a) at 0 poise in any state (CANON "break at 0"); (b) only in windup | a, per CANON; b if guarding units break too often in the sim |
