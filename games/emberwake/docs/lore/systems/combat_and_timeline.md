# Combat and the timeline

Status: canon for the Act 1 rules (ruleset option "timeline", the Cold Steel branch); proposal for Strain, the
Aberration Crisis and the Inoculated details, which switch on in Act 2.

The numbers follow [CANON](../CANON.md) sections 2.1-2.2; where this file and CANON differ, CANON wins. Related:
[rules engine architecture](rules_engine_architecture.md) (what to build where), [magic and
corruption](magic_and_corruption.md) (Strain and Calcification), [classes and skill trees](classes_and_skill_trees.md),
the game's [GAME_BRIEF](../../GAME_BRIEF.md) (today's combat rules).

Layer tags (the layer rule of [CANON](../CANON.md)): `[engine]` technology any game needs (here only the timeline
strip widget); `[kit hook]` a ruleset option or a generic shape any story could fill, "(exists)" where today's
battle and rules crates already carry it (the code lands in the `emberwake_*` crates until a second game shares rules, see
[STRUCTURE](../../../../../docs/STRUCTURE.md)); `[data]` content filled through `aegis content`; `[game code]` a
mechanic only this story uses.

## 1. Two rulesets on one resolver

The Low Sleep worlds (first `sunk_ward`) play the ruleset option **timeline**; the Ashlands (the kit's sample and
parity world) keeps **classic** `[kit hook]`. The ruleset is a world field with a Dojo toggle (user 2026-10-25, Q-52).
Both run on the same block resolver; timeline adds poise, recovery blocks, chained follow-ups and (from Act 2) Strain,
and switches stances off. Classic is what the Godot parity suites replay, so the timeline option must leave classic
results unchanged: a rule incompatible with classic is built as an option, a compatible one as an extension that can be
disabled, and bug fixes go to the core (they may need an Ashlands or kit revisit). Every engine and kit piece the
vertical slice needs is built before it (Q-55, Q-56).

| Rule | Classic (today, the Ashlands) | Timeline (the Low Sleep) |
| --- | --- | --- |
| Round | 6 blocks, all sides resolved simultaneously `[kit hook]` (exists) | same |
| AP | 10 max, +4 per round; an action pays AP when it starts and lands on its last block; AP debt allowed `[kit hook]` (exists) | same |
| Stances | 4 stances, matchup x1.25 / x0.75, +1 AP per switch, stance breaks | off (DEC-LORE-11) `[kit hook]` |
| Interrupts | none (the `stagger` and `stun` statuses come from skills and AP debt) | poise bar; a break cancels the action `[kit hook]` |
| Recovery | none | per-skill recovery blocks `[kit hook]` |
| Guard | basic `guard` skill, mitigation per rank | Active Guard base move (section 5.4) `[kit hook]` |
| Armour | hyperbolic, rating / (rating + 50) `[kit hook]` (exists) | same, plus binary pierce for Inoculated attacks `[data]` |
| Strain | none | per-unit meter, Act 2 on (section 7) `[kit hook]` |

## 2. The round as the engine runs it today `[kit hook]` (exists)

1. Planning: the enemy plans are drafted first and shown as intent tells (INTENT1, `emberwake_battle::intent`); the
   player then plans each operative's blocks on the timeline strip and confirms.
2. Resolution, block by block (`Resolver::prepare_block`): actions that start in this block pay their AP and run their
   start effects (a dash, a chase, a guard grant); actions whose last block is this block land, highest initiative
   first. Initiative = command priority x 1000 + the unit's speed (speed = max(1, 8 + DEX - armour burden)).
3. A unit downed earlier in the block loses its own landing and everything it still had scheduled
   (`Resolver::execute_single` cancels it).
4. Round end (`Resolver::apply_round_end_effects`, `finish_round`, the session's status roll-over): statuses tick and
   AP refills by +4 (plus any `ap_regen` bonus) up to the maximum.

The real names are `Resolver::schedule`, `prepare_block`, `execute_single`, `apply_round_end_effects`,
`finish_round`, and `ScheduledAction { unit, command, start, end, cancelled }`. There is no `step_tick`, no per-tick
loop and no `SpeedTier` type; the lore's "ticks" are the engine's blocks.

## 3. Maneuvers: windup, impact, recovery `[kit hook]`

A maneuver occupies its own blocks on the strip:

- **Windup**: the action's blocks (the skill's `blocks`). The impact lands at the end of the last windup block, as
  every action lands today. A unit in windup scales incoming stagger by `stagger_windup_mult` (`2.0` default; `1.0` in
  current `timeline` ruleset tuning) and can be broken (section 5).
- **Impact**: damage, stagger, statuses and forced movement resolve, in initiative order within the block.
- **Recovery** (optional, per skill; 0 by default): blocks after the impact in which the unit is committed. Recovery
  is the price of heavy maneuvers; most skills have none.
- **Chains** `[kit hook]` (user 2026-10-25, Q-54): commands that work together chain. A follow-up skill can start in
  the blocks a previous action leaves; otherwise the unit spends them in recovery. This makes a heavy skill a
  commitment: plan its follow-up or pay the recovery. Which skills chain into which is the spec's call.

### 3.1 Speed tiers are relative windups

| Tier | Windup | Use |
| --- | --- | --- |
| Snap | 1 block | interrupts, parries, smoke, throws, daggers |
| Standard | 2 blocks | sword swings, thrusts, aimed bolts, field sutures |
| Heavy | 3-4 blocks | two-handed cleaves, hammers, channelled blights; often 1-2 recovery blocks |

The impact lands at the end of the action's own blocks wherever it starts: a Snap started in block 3 lands at the end
of block 3. There are no fixed tick anchors (the old "Snap lands T2, Standard T4, Heavy T5-6" text is retired).

### 3.2 AP cost is separate from the tier `[data]`

A skill's AP is its `ap_cost` field (default: the skill level, which is today's rule). Guideline: Cold Steel 2-4 AP,
Inoculated 1-2 AP plus Strain. With 10 AP and +4 per round, sustained play is one to two Cold Steel actions per round,
with bursts above that from the banked pool.

### 3.3 Worked example (the prologue's lesson)

Kaelen plans Pommel Strike (Snap) in block 2. An Inquisitor's tell shows a Heavy Cleave (4 blocks) from block 1 to
block 4. Pommel Strike lands at the end of block 2 while the Cleave is in windup: its stagger is doubled, the
Inquisitor's poise breaks, the Cleave is cancelled with the rest of his blocks this round, and the AP he paid at the
start stays spent.

## 4. Same-block impacts `[kit hook]` (exists)

- Impacts in one block resolve one after another in the engine's initiative order. Each sees the state the previous
  one left (no "defensive snapshot" at the start of the block).
- No mutual trades: a unit downed or broken by an earlier impact in the block does not land its own.
- Exact ties fall to the engine's deterministic sort; the lore's DEX x 2 + PERC + tier-weight formula is dropped.

## 5. Poise, stagger and interrupts `[kit hook]`

### 5.1 The poise bar

- `max_poise = 2 * CONS + armour_poise` from gear, displayed and resolved as **5 discrete Poise pips** with threshold
  `T = max(1, roundi(max_poise / 5))` (`PK1` / `S2_POISE.md`). CONS runs 6-18 in the stat budget (`12-36` before armour).
- An impact with `eff_stagger >= 1` chips `clamp(floor(eff_stagger / T), 1, 5)` pips; a single hit can strip up to all
  5 pips, and Poise refills to 5 pips at round end (or immediately after a break).
- Skills deal stagger (poise damage) on impact, separately from HP damage.

### 5.2 Stagger multipliers (they multiply)

| Condition on the target | Stagger taken |
| --- | --- |
| In windup when struck | `stagger_windup_mult` (`x2.0` default; `x1.0` in current `timeline` tuning) |
| In Flesh-Tension (Strain 50-74, Act 2 on) | x2 |
| Holding Active Guard | x0.5 |
| Broken on the immediately preceding round (`stagger_repeat_mult`) | x0.5 |

Skill texts state only their base stagger; windup, Flesh-Tension, Active Guard, and repeat-break multipliers are
general ruleset terms, never skill riders.

### 5.3 A break and repeated breaks

- **Break**: at 0 pips, the action in windup is cancelled together with the unit's remaining blocks that round
  (recovery included), AP already paid stays spent, Poise resets to 5 pips, and `stagger` (`[1, 4]`, `-4 AP` next
  round) is queued (the old "-50% evasion, +25% damage taken" rider is dropped).
- **Repeated breaks**: breaking a unit on consecutive rounds halves incoming `eff_stagger` (`stagger_repeat_mult = 0.5`,
  effectively requiring double stagger to break again back-to-back) so fast stun-locks cannot chain indefinitely.
- **Stagger intent** (user 2026-10-25, Q-50 a): a break is a tactical choice to disrupt an enemy action, not an
  automatic lock; heavy bosses carry high `max_poise` or `unbreakable` windups.

### 5.4 Active Guard (base move)

Halves incoming stagger; grants **35% damage mitigation** (`0.35`, or **20%** on the Snap `Brace` variant) through the
kit's guard term, capped by `GUARD_MITIGATION_CAP = 0.90` with cover; from Act 2 it vents 10 extra Strain at round end.
It replaces the classic `guard` skill in the timeline deck.

## 6. Movement and forced movement

- **Move** `[kit hook]` (exists): the basic move, up to 2 blocks, tiles per block from speed, 1 AP per tile, once per round.
  Acute Resonance adds +1 tile per move block.
- **Advance** (move, then act) `[kit hook]` (exists): the move blocks come first in the unit's sequence, then the windup, as the
  engine's `dash_tiles` gap-close already does.
- **Disengage** `[kit hook]` (exists): the engine's `DISENGAGE` kind jumps first in the round (`backflip`). An act-then-move
  maneuver (strike, then retreat) is not in Act 1: only the first-in-round jump (Q-54).
- **Chase when the target moves during a windup**:

| Reach | Rule | Engine |
| --- | --- | --- |
| Reach 1 (daggers, knuckles, beasts) | steps after the target to keep contact; fails at walls | attack `follow_tiles` chase `[kit hook]` (exists) |
| Reach 2 (spears, billhooks) | holds ground; lands if the target is still in reach | reach band `[kit hook]` (exists) |
| Ranged (arbalests, thrown flasks) | lands if the target is still in range; blocking line of sight needs a check the engine lacks | `[kit hook]` |

- **Knockback, pull and wall slam** `[kit hook]` (forced movement is new): a unit pushed into a wall, obstacle or
  unit stops there; each unspent tile deals lore +15 blunt damage and +20 stagger (lore values; rescale with the poise
  formula). A unit knocked into another unit hurts both.
- **Teleport** (Void-Flicker, Act 2) `[kit hook]`: new.
- **Ground zones** (fire, smoke, tar) `[kit hook]` (exists) terrain hazards with round-end effects, filled as `[data]`; smoke
  gives cover guard in Act 1; smoke that blocks line of sight waits for the Act 2 line-of-sight check (Q-54).

## 7. Strain in combat (Act 2 on; proposal) `[kit hook]`

Act 1 has no Strain on anyone. From Act 2 each unit carries a Strain meter (0-100) with the brackets from CANON:

| Strain | Bracket `[data]` | Effect (engine scale) |
| --- | --- | --- |
| 0-24 | Dormant | none |
| 25-49 | Acute Resonance | +15% crit chance, +1 tile per move block, -25% healing received |
| 50-74 | Flesh-Tension | +25% damage, +4 armour rating, stagger taken x2 |
| 75-99 | Aberrant Edge | +50% damage, recoil 10% of damage dealt |
| 100 | Rupture | crisis roll at round end |

- **Gain**: an Inoculated skill adds its `strain_cost` (guideline 15-35) when the action starts, so the bracket can
  change before its own impact lands (a skill started at Strain 40 with cost 15 lands in Flesh-Tension). Hazards and
  eldritch attacks add more.
- **Decay**: at round end -10 if the unit used no Inoculated action that round, a further -10 if it held Active
  Guard; never below the unit's dormant floor (grafts raise it, 0-50). After battle Strain resets to the floor.
- **Aberration Crisis** `[game code]` unless the kit meter's triggers can express it: at round end, a unit at 100
  rolls 1d3 (equal odds, seeded): 1 Unchecked Spasm (a full-power strike on the nearest unit, ally or foe, then the
  engine's `stun` next round); 2 Biological Collapse (0 AP next round, Bleed 3 rounds, armour 0 for the rest of the
  battle); 3 Flesh Breach (+1 Calcification stage). Strain after the crisis is open (CANON section 6).

Sustained use: decay is 10-20 per round against 15-35 per Inoculated action, so an operative can hold about one
Inoculated action per round; more is a burst that climbs toward Rupture.

## 8. The timeline strip `[engine]` `aegis_ui::timeline`

- At the TOP of the battle screen (COMBAT-UI3 layout A): one row per unit, continuous blocks colour-coded by kind.
- Timeline additions `[kit hook]`: windup shading, recovery blocks hatched, an impact badge on each action's landing
  block, a BREAK marker on cancelled blocks; Strain bracket and poise on the unit cards.
- Enemy windups are read through the existing intent tells; the multi-track "ghost scrubber" is cut (start with impact
  badges on the existing strip).

## 9. Cut / later

| Idea | Status |
| --- | --- |
| Evasion and accuracy rolls (incl. "+15% evasion", "-15% hit chance", Fear as -accuracy) | cut |
| Facing, backstab bonuses, rear arcs, stealth and detection cones | cut |
| Mutual trades (both same-block strikes land) and the defensive snapshot | cut |
| The d100 crisis (Flesh Mastery, Marrow Rupture, Predatory Frenzy, Spontaneous Calcification, Metamorphosis) and "Composure" in its formula | cut; replaced by the 1d3 table (Composure itself returns as a derived stat, the corruption resist, CANON 2.1) |
| Fixed tick anchors (T2, T4, T5-6) and tier-wide recovery windows | cut; relative windups |
| Stances and stance switching in the Low Sleep | cut (DEC-LORE-11); classic keeps them |
| Multi-track ghost scrubber | later; start with impact badges |
| A per-tick resolver (`step_tick`) and per-tick hazard damage | cut; the engine resolves by block |

## 10. Gaps

| Gap | Options | Recommendation |
| --- | --- | --- |
| Armour poise per gear piece (the CANON formula's second term) | A: a flat poise per armour weight class; B: poise = the item's mitigation x a factor | B: one number per item, no new field to author |
| Lore stagger values (Pommel 25, Jaw-Breaker 30, wall slam 20) were set for flat bars of 30-70 | A: keep them and tune poise; B: rescale to a fraction of a typical Max Poise | Decided (2026-10-25, user, Q-50): B, with the stagger intent of 5.3 (break chance, guaranteed break at a cost); then a balance-lane sim: at x2 windup a 25-stagger Snap breaks nearly any 12-36 bar (the interrupt-lock risk in [risk and reward](risk_reward_and_corruption_balance.md)) |
| Act-then-move maneuvers (Disengage strike-and-retreat) | A: allow a move step after an action in the plan; B: only the engine's first-in-round jump | Decided (2026-10-25, user, Q-54): B in Act 1 |
| Line of sight (smoke zones, ranged chase, full cover) | A: a kit line-of-sight check on the grid; B: smoke gives cover guard instead | Decided (2026-10-25, user, Q-54): B in Act 1, a kit line-of-sight check in Act 2; cover is only the existing cover guard (the brigandine's half-cover trait and "full cover" pillars are dropped); cover and line of sight become engine or kit systems after testing |
| Strain above 100 within a round, and Strain after a crisis | A: cap at 100, set to 75 after the crisis; B: no cap, reset to the floor | open in CANON (Q-62); A keeps the crisis a one-off per round |
| Recovery default per tier | A: 0 everywhere, set per skill; B: Heavy gets 1 by default | Decided (2026-10-25, user, Q-54): A, plus chains (section 3) |
| Maneuver hooks the engine lacks (forced movement, ground hazards a skill leaves, auras, poise-break triggers, scheduled arena hazards) | A: the first playable uses existing statuses and phases; B: build them all first | Decided (2026-10-25, user, Q-55): A in spirit, but every hook the slice uses is built as a reusable kit or engine system before the slice, starting with forced movement and ground hazards |
