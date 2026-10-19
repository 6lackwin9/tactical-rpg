# S12-RULES: advanced combat baseline (status expansion, terrain, combos, skill upgrades)

Status: proposed (design-brainstorm 2026-10-17; baseline rules so S12 can be specced and built later without design stalls; nothing approved)
Depends on: S12-A (done), SKILL1 (done), BAL2-L (done; sim metrics), ITEM1-L and CLASS1-L (hub files, see section 4)
Lane: design (this file); later logic and asset tasks listed in section 4
Touches (when built): `scripts/combat/status/*`, `scripts/core/combat/{combatant,damage_calculator}.gd` (inputs only), `scripts/combat/arena/{terrain,combat_zones}.gd`, `scripts/combat/planning/*` (path cost, stance chain), `scripts/content/{skill_fields.gd,validators/skill_validator.gd}`, `scripts/combat/ai/tactical_ai.gd`, `tools/sim/sim_metrics.gd`, `scripts/art/tile_art.gd`

## 1. Scope and what it is NOT

In scope: what is left of S12 after S12-A (M1), riposte (M2a), weaken (D1) and the M2c engine pieces:

| Part | Content |
| --- | --- |
| S12-S status expansion | new registry statuses that act through existing hooks; a general magnitude range per status |
| S12-T terrain effects | the unused `Terrain.INFO` fields `move_cost`, `cover`, `hazard` become rules; hazard tiles apply statuses |
| S12-C combos | flow moves (a skill ends in a chosen stance), stance-break hits, status payoffs ("exploit") |
| S12-U skill upgrades | closed as already covered (section 2.5); only an optional source-granted modifier is left |

NOT in scope:

- No change to the `DamageCalculator.compute` formula (DECISIONS combat "Combat extensions (S12)"); every mechanic below changes an INPUT (attack power, skill power, guard ratio, armor rating, stance, tiles per move block) or runs outside damage (heal path, round-end ticks, movement).
- No balance numbers. Every number is skill, status or terrain DATA set by the asset lane and tuned by the balance lane; this file only names the fields. Where a field needs a starting value, the later asset spec proposes it.
- No new area shapes (cone stays an idea; PLUS, BURST, LINE, SWEEP exist), no forced movement (push/pull), no dual wield or weapon-switch commands (section 5, HISTORY row 23), no miss/hit roll (hits stay deterministic except crit).
- No UI beyond planner previews and log lines; animation and icons are S14 (`games/emberwake/docs/specs/S14_RULES.md`).

## 2. Baseline rules

### 2.0 Rules every S12 mechanic follows

1. Single formula: inputs only. A mechanic may change `attack_power` / `SkillDef.power_for` (skill power), `skill_mult`, the guard ratio fed to `(1 - Guard)`, `armor_rating`, the attacker or defender stance, or `Combatant.move_per_block()`. A new multiplicative term in `compute` needs a user decision (it would cost S1 its `balance-ready`).
2. Data, not constants: numbers live in skill rows, `StatusRegistry` rows or the terrain table (HISTORY row 20: tuning must be a data change). Code constants are only structural limits (caps, floors).
3. Timing: statuses inflicted or granted this round start next round for `status_rounds` (existing rule). The only immediate effects are the existing `StatusEffects.grant_now` path (DEFEND grants) and the stance-break rule of 2.4, both resolved in the resolver's deterministic block order with the seeded RNG.
4. Visibility: every effect shows in the planner preview (expected hit, path cost, stance chain) and writes a log line in `BattleCommand` results/reactions so replays explain why a sequence changed.
5. Sim conservation: any new HP change (regen, DoT, hazard) reports to the BAL2 metrics collector through its own channel (`log_regen`, `log_dot`, `log_hazard`), so `METRIC FAIL` stays meaningful (games/emberwake/docs/specs/BAL2_SIM.md section 4). New buffs count in `buff_rounds`, new debuffs in `debuff_rounds`.
6. AI: each mechanic gets one TacticalAI rule in the smart-skills path (the only path after CLN-16); a mechanic the AI cannot use is not shipped on enemy units.
7. Role fit: each mechanic names the BAL2 role KPI it serves (games/emberwake/docs/specs/BAL2_SIM.md 6.1) so the role limit tests can measure it.

### 2.1 Status model changes (S12-S, schema first)

`StatusRegistry` stays the one code table (effects live where they act; header checklist). Additions per row:

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `magnitude_min` | int | 0 | lowest allowed `status_magnitude` on a skill that inflicts or grants it |
| `magnitude_max` | int | 0 | highest allowed; 0 and 0 = the status has no magnitude (skill must leave `status_magnitude` at 0) |
| `magnitude_unit` | enum `pct`, `flat`, `pct_max_hp` | `pct` | how the effect reads the magnitude (doc + Forge token `{magnitude}`) |

Validation: `SkillValidator` replaces the special case "weaken needs 1-25, other statuses 0" with the row range. weaken becomes `1..25 pct`, cap 25 (unchanged behaviour). Magnitudes travel in the existing potency channel (`status_magnitudes` / `pending_status_magnitudes`, max wins unless the row is `stack_potency`).

Granting rule extended: SUPPORT `grant_status` and DEFEND grants may carry `status_magnitude` too (today only debuffs from attacks do); terrain hazards carry their own magnitude (2.3).

### 2.2 New statuses (candidate set; Q-S1 picks the first slice)

| Id | Kind | Resist | Stacking | Acts in (input) | Effect rule | Role KPI |
| --- | --- | --- | --- | --- | --- | --- |
| `slow` | debuff | yes | refresh_max_duration | `move_per_block()` | minus 1 tile per move block, floor 1 (mirror of haste; haste and slow cancel) | ranged `nearest_foe_dist`, tank hold |
| `expose` | debuff | yes | refresh_max_duration, cap from row | `armor_rating(target)` | armor rating minus magnitude (flat), floor 0 | striker `priority_down_round` |
| `fortify` | buff (grant support or defend) | no | refresh_max_duration | `armor_rating(target)` | armor rating plus magnitude (flat) | tank `dmg_taken` share, survival |
| `regen` | buff (grant support) | no | refresh_max_duration | round end, heal path (`SupportRules`, never `DamageCalculator`) | heals magnitude % of max HP at round end, min 1, never above max | support effective heal, `buff_rounds` |
| `poison` | debuff | yes | stack_potency, cap from row | round end tick (like bleed) | loses magnitude HP (flat, total of stacks) at round end; ignores guard and armor; can defeat | striker, ranged sustained damage |
| `burn` | debuff | yes | refresh_max_duration | round end tick | loses magnitude % of max HP; the status hazard fire tiles apply (2.3) | AoE zone control |
| `blind` | debuff | yes | refresh_max_duration | attack reach | ranged and spell reach minus 1, floor 1 (deterministic; no miss chance) | ranged counterplay |
| `silence` | debuff | yes | refresh_max_duration | `Combatant.can_use` | only basics usable (strike, guard, move, rest, switch_target, offhand_jab) | caster counterplay |

Rules shared by all: resistable keys follow the registry (passive `resist` and Forge pick them up automatically); `cleanse_statuses` may name any new debuff; `all` covers them. Bleed stays the % max HP DoT (DECISIONS combat "Bleed"); poison is the stacking flat DoT, burn the terrain/fire DoT, so the three stay distinct for resists and cleanses. Order at round end: DoTs (bleed, poison, burn) then regen, each logged; a unit defeated by a DoT does not regen.

`sunder` ("armor-breaking chop", today only stagger) is the natural first `expose` user; that is an asset choice for the S12-S asset spec, not decided here.

### 2.3 Terrain rules (S12-T)

Terrain table: the 14 `Terrain.Type` values keep `walkable`, `cover`, `move_cost`, `hazard`; `hazard` text becomes a structured record. Where the table lives is Q-T1.

| Field | Type | Default | Meaning |
| --- | --- | --- | --- |
| `move_cost` | int 1..3 | 1 | tile budget a unit spends to ENTER the tile |
| `cover` | int 0..2 | 0 | 0 none, 1 half, 2 full; the guard ratio per level is a table value (`cover_guard`) |
| `hazard_status` | status id or "" | "" | registry status applied by the tile |
| `hazard_magnitude` | int | 0 | validated against the status's magnitude range |
| `hazard_rounds` | int 1..5 | 1 | `status_rounds` of the applied status |
| `hazard_on` | enum `enter`, `round_end`, `both` | `both` | when it applies |

Movement: a move block grants `move_per_block()` tiles of budget; the planner path search becomes cost-weighted (Dijkstra over walkable tiles; ties keep today's order rules, friend-vacates fastest-first unchanged, HISTORY row 16). Minimum progress: the first tile of a block is always enterable even if its cost exceeds the remaining budget, so mud never roots a unit. Dash (`dash_tiles`) pays costs like a move; disengage (`jump_tiles`) is a jump and ignores costs but not the landing tile's hazard.

Cover: applies to hits with `is_ranged` true (ranged weapons and spells, area hits included), never to melee. A target has the higher of (a) the cover of the tile it stands on and (b) the directional cover of Q-T2. Cover adds `cover_guard[level]` to the guard ratio input: effective guard = min(`GUARD_CAP`, guard + cover), where `GUARD_CAP` is a new structural constant (value in the S12T-L spec) so guard plus cover never nullifies a hit. The planner estimate includes cover (it is known ahead, unlike guard). Riposte, which needs melee, is unaffected.

Hazards: on `enter` the status is applied once per unit per block when its move ends on or passes through the tile; on `round_end` to every unit standing on it. Terrain statuses skip the potency roll but resistable ones still roll resistance; they follow the next-round timing rule. Lava, deep water, walls and trees stay impassable (current data). New walkable hazard types (for example burning ground with `burn`, toxic bog with `poison`, ice with `slow`) are new enum values plus a `TileArt` drawing each, so they are logic-lane work; zone maps that use them are asset work.

Skill-created terrain (a fire area left by Firebomb) is Q-T3; the baseline reserves the skill fields only if the user picks it.

### 2.4 Combos (S12-C)

Flow move: skill field `ends_in_stance` (enum `""`, `offensive`, `tactical`, `defensive`; default `""` = the stance of the skill's kind, today's behaviour). The unit fights the skill's blocks in the kind's stance; after its last block the stance becomes `ends_in_stance` for free. The planner's stance chain (switch cost `CombatTypes.STANCE_SWITCH_COST`, B1 rule) reads the end stance, so the next action of that stance pays no switch. Normal consequences apply: leaving Defensive ends riposte (`Combatant.set_stance`). This is the per-skill override B1 left open ("no per-skill override yet").

Stance break: skill field `disrupt` (enum `none`, `stance_break`; default `none`). A damaging hit that the target survives sets the target's stance to Tactical immediately (a resolver-ordered exception like riposte): guard mitigation from a running DEFEND ends for the remaining blocks, riposte ends, the target's next non-Tactical action pays the switch. Area attacks break every unit hit. Stance break is not a status: no resist roll, no registry row, `stagger` resist does not apply. Serves striker (break a tank screen) and is the counter to turtling warriors.

Exploit (status payoff): skill field `exploit` = list of `{status, power_mult}` (status: any registry debuff; power_mult 1.0..2.0, at most 2 entries); when the target has one of the statuses active at the hit, the skill's power is multiplied by the highest matching `power_mult` (input: skill power via `SkillDef.power_for`). Optional `consume: bool` (default false) removes the status after the hit. It turns statuses into party setups (warrior expose, rogue payoff) and serves the striker and party-synergy targets.

Interrupt (cancel the rest of a multi-block action) and momentum (consecutive blocks in one stance scale a skill) are deferred by Q-C1 unless the user picks them.

### 2.5 Skill upgrades (S12-U)

Already covered, so no new ladder: tiers 1-3 per family chosen at planning time (SKILL1 ranks), advanced and expert families with `requires` (grade), the `skill_potency` passive and item effect, the `unlock` passive, item `status_potency` and `spell_reach`. The ROADMAP word "skill upgrades" is satisfied by these. Not re-proposed: levels 4-5 of a skill (HISTORY row 30), basic variants (row 31), AP-tied unlock (row 28). Q-U1 asks whether a source-granted modifier is wanted.

## 3. Open design questions (only choices that block a spec)

Q-S1 First status slice.

- A: inputs-only four: slow, expose, fortify, regen. Each uses a hook that exists (move_per_block, armor_rating, SupportRules); one new round-end channel (regen). Smallest test surface; covers tank, support, striker, ranged.
- B: A plus the DoT family poison and burn. Needed if S12-T hazards ship in the same milestone; poison adds stack_potency, the first real use of that rule.
- C: A plus B plus control (blind, silence). Strong counterplay but silence touches `can_use` and the AI's whole option list; highest AI and balance risk.
- Recommendation: B (A now, poison and burn with it because terrain hazards need burn and the stack rule is already built and untested in content); C later.

Q-T1 Where the terrain table lives.

- A: code table `Terrain.INFO` as today. Zero pipeline work, but tuning cover or move cost is a code change (against HISTORY row 20's principle).
- B: catalog kind `terrain` (one row per `Terrain.Type`, numbers only; types and rule kinds stay code), edited through `ContentCatalog.upsert`, validated against the status registry. Tunable by the asset lane; one more kind on the content hub.
- C: a world-layer table so worlds retune terrain. Breaks "base asset decides gameplay" (GAME_BRIEF).
- Recommendation: B; zone maps stay in `CombatZones` until S9 decides map data (the S9-S11 design agent owns that).

Q-T2 Cover geometry.

- A: on-tile only (rubble). Trivial, but walls and trees (cover 1-2 in data) never give cover.
- B: on-tile plus directional: the neighbour of the target in the step direction toward the attacker (sign of dx, sign of dy; for a diagonal, either orthogonal neighbour counts) gives its cover if it is a blocking cover tile. Cheap, readable on a 9x9 grid, previewable.
- C: line of sight (raycast blocks attacks through walls). Changes targeting, AI and the planner reach rules; much larger.
- Recommendation: B.

Q-T3 Skill-created terrain.

- A: static zone terrain only in S12-T.
- B: skill fields `leave_terrain` (type) and `terrain_rounds` on area attacks; a per-battle overlay that expires. Strong AoE identity (mage), but a new battle-state layer that save/undo snapshots must cover.
- Recommendation: A now, B as a later slice (fields not reserved until then).

Q-C1 Combo scope of S12-C.

- A: flow moves and stance break only.
- B: A plus exploit payoffs (2.4). Party synergy for the role-based balance target at the cost of one skill field.
- C: B plus interrupt and momentum. Interrupt needs a resolver rule for half-resolved multi-block actions and replay explanations; momentum adds hidden per-unit state.
- Recommendation: B.

Q-U1 Skill upgrades beyond ranks.

- A: close as covered (2.5).
- B: add a whitelisted `skill_mod` passive and item effect (family or group; fields `status_rounds` +1, `status_potency` +N, `blocks` -1 floor 1) granted only by sources (quests, loot), so S8 rewards can upgrade a skill without new rows.
- C: player-spent skill points per family. A second progression currency next to stat spend; new UI and save fields.
- Recommendation: A now; B when M4 loot needs more reward variety.

## 4. Milestone shape (coarse; each needs user approval)

| Task | Lane | Content | Depends on |
| --- | --- | --- | --- |
| S12S-L | logic, schema first | registry magnitude range fields, Q-S1 statuses and their hooks, round-end order, grant magnitudes, sim channels, AI rules, new suite `status_expansion_test_suite.gd` | ITEM1-L and CLASS1-L merged (skill validator, combatant hub) |
| S12S-A | asset | skill rows using the new statuses (candidates: sunder expose, a priest regen tier, rogue poison); seed `data/seeds/s12s.gd` | S12S-L commit 1 |
| S12T-L | logic, schema first | terrain kind (Q-T1), cost-weighted pathing, cover input, hazards, new walkable hazard types plus `TileArt`, suite `terrain_rules_test_suite.gd` | S12S-L (burn, poison, slow) |
| S12T-A | asset | terrain numbers, zone maps with hazards and cover | S12T-L commit 1 |
| S12C-L | logic, schema first | `ends_in_stance`, `disrupt`, `exploit`, planner stance chain, AI rules, suite `combo_rules_test_suite.gd` | S12S-L; not in parallel with S12T-L (both edit `combat_plan`) |
| S12C-A | asset | skill rows with flow, break, exploit | S12C-L commit 1 |

After all three: the planner freezes S12 rules (ROADMAP build order step 7) and the balance lane runs role tests with the new channels.

Rule-derived acceptance checks to carry into the logic specs:

1. A skill row with `status_magnitude` outside the registry range fails validation; weaken behaviour and the D1 tests are unchanged.
2. slow on a unit with base 1 tile per block leaves 1; haste plus slow equals base.
3. expose with magnitude above the target's rating gives rating 0, never negative; the expected hit in the planner equals `compute` with the exposed rating.
4. A unit on a cost-2 tile path with budget 1 still advances one tile per block (minimum progress).
5. A ranged hit on a target with cover equals `compute` with guard + `cover_guard`; a melee hit on the same target ignores cover.
6. A flow skill ending in its next action's stance shows switch cost 0 in the plan; a stance-break hit removes riposte in the same block.
7. Every party/role sim fight with the new statuses passes the conservation check (no `METRIC FAIL`).

## 5. HISTORY rows that touch S12

| HISTORY row | Effect on this baseline |
| --- | --- |
| 1, hand-scripted 10-block layer deprecated | flow, hold-stance and disruptive hits are concepts from that layer (ROADMAP S12-C note); only concepts are reused, no numbers; hold-stance (momentum) is deferred |
| 20, riposte numbers as code constants deprecated | all new numbers are data (2.0 rule 2) |
| 23, off-hand shortsword/knuckles/dagger and plain dual wield deprecated | README's `DualStrikeCommand` for 1H pairs is not re-proposed (off-hand slots are support items); weapon switch has no second weapon set to switch to, so it stays out |
| 28, AP-tied unlock deprecated | no upgrade may tie unlock to AP |
| 30, advanced as levels 4-5 deprecated | skill upgrades stay ranks plus separate families |
| 31, basic variants deprecated | basics scale only via potency hooks; silence keeps basics usable |
| 16, vacated tile fastest-first | cost-weighted pathing keeps the fastest-first rule |

Archived P-S12 (docs/archive/ROADMAP_2026-10.md): "skill upgrades as per-skill levels" is superseded by ranks (HISTORY row 30); "terrain and AoE patterns as slice B, combo engine last and optional" matches this order.
