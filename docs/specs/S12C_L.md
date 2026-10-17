# S12C-L: combos (flow moves `ends_in_stance`, stance-break `disrupt`, status payoff `exploit`)

Status: approved (user 2026-10-17) (design-brainstorm 2026-10-17; DECISIONS combat "S12 baseline answers" and docs/specs/S12_RULES.md 2.4)
Depends on: S12S-L merged (exploit reads the new debuffs); S12T-L merged (both edit `combat_plan.gd`; run in sequence)
Lane: logic (schema first; commit 1 releases S12C-A)
Touches: `scripts/content/skill_fields.gd`, `scripts/content/skill_kinds.gd` (sections), `scripts/content/skill_ranks.gd` (family-level keys), `scripts/content/validators/skill_validator.gd`, `scripts/content/skill_text.gd` (tokens), `scripts/content/content_factory.gd` (SkillDef build), `scripts/combat/planning/{skill_def,combat_plan,plan_entry}.gd`, `scripts/combat/actions/{battle_command,attack_command,area_attack_command}.gd`, `scripts/combat/arena/combat_resolver.gd`, `scripts/combat/status/riposte_rules.gd` (order only), `scripts/combat/ai/tactical_ai.gd`, new `scripts/tests/combo_rules_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`

## Goal

Three skill fields that create combos inside the existing stance and status systems: a flow skill ends in a chosen stance so the next action skips the switch cost (the per-skill override B1 left open), a stance-break hit knocks a guarding or attacking target into Tactical, and an exploit skill hits harder against a target carrying a named debuff. Each changes only inputs (the stance, `skill_mult`); `DamageCalculator.compute` is unchanged.

## Schema contract (logic lane, schema first: yes)

### Skill fields (family level: one value for all ranks; ranks may not override them)

| Field | Type | Default | Kinds (section) | Validation (exact message) |
| --- | --- | --- | --- | --- |
| `ends_in_stance` | enum `""`, `OFFENSIVE`, `TACTICAL`, `DEFENSIVE` | `""` | ATTACK, DEFEND, SUPPORT, FOCUS | equal to the skill's own `stance()`: `"Ends in stance equals the skill's stance; leave it empty."`; on a DEFEND that grants riposte: `"A riposte skill cannot flow: a stance change ends riposte."` |
| `disrupt` | enum `NONE`, `STANCE_BREAK` | `NONE` | ATTACK | on a zero-power ATTACK: `"A zero-power attack cannot break a stance."` |
| `exploit` | Array of `{status: String, power_mult: float}` | `[]` | ATTACK | at most 2 entries: `"Exploit takes at most 2 statuses."`; status a registry debuff: `"Exploit status '%s' is not a debuff."`; duplicates: `"Exploit lists '%s' twice."`; `power_mult` 1.05..2.0: `"Exploit power must be 1.05-2.0."`; on a zero-power ATTACK: `"A zero-power attack cannot exploit."` |
| `exploit_consume` | bool | false | ATTACK | true with an empty `exploit`: `"Consume needs an exploit status."` |

A rank modifier that sets any of the four keys: `"%s is set on the family, not per rank."` (SkillRanks key list).

SkillText tokens: `{flow}` -> `"ends in Defensive"` (stance display name), "" when empty; `{exploit}` -> `"x1.3 vs Exposed, x1.2 vs Slowed"` (display names), "" when empty; `{disrupt}` -> `"breaks stance"` or "".

`SkillDef` gains `ends_in_stance: int` (-1 = none), `disrupt: int`, `exploit: Array[Dictionary]`, `exploit_consume: bool`, built by `ContentFactory`.

## Rules (logic lane)

1. Flow, planning: `SkillDef.stance_after(current)` returns `ends_in_stance` when set (before the keep rules); `PlanEntry.stance_after` therefore carries it and the next entry's `switch_cost` is computed against it (`STANCE_SWITCH_COST`, B1). The planner shows `"flows into <stance>"` on the entry.
2. Flow, resolution: `BattleCommand` gains `stance_on_end: int = -1`, set from the skill. The unit fights the command's blocks in the stance it entered at start (`stance_on_start`, unchanged); right after `execute()` on the resolving block, if the unit is still active and `stance_on_end >= 0`, `set_stance(stance_on_end)` with no AP cost and log `"[Flow] %s flows into %s stance"`. `set_stance` side effects stay (a change out of Defensive ends riposte). A cancelled or failed command does not flow.
3. Stance break: in `AttackCommand` and `AreaAttackCommand`, for each target that took damage > 0 from this skill and is still active, if `disrupt == STANCE_BREAK` and the target's stance is OFFENSIVE or DEFENSIVE: after the hit's reactions (a riposte counter triggered by this hit resolves first, it reacted to the hit), `target.set_stance(TACTICAL)`, `target.defense_mitigation_ratio = 0.0`, and every running `DefendCommand` action of the target in the resolver's schedule is marked `guard_broken = true` so `prepare_block` stops re-applying its mitigation for its remaining blocks. Log `"[Break] %s breaks %s's stance (now Tactical)"`. Not a status: no resist roll, no registry row. AP already paid and the target's later actions are not recomputed (they enter their planned stance on start, paying what was planned).
4. Exploit: per target hit, `mult = max(power_mult of entries whose status is ACTIVE on the target)` (pending statuses do not count), default 1.0; the command passes `skill_mult x mult` to `compute` (input). With `exploit_consume`, after the hit the matched status with the highest multiplier is removed (`StatusEffects.remove`, active instance only). Log suffix `" (exploits %s)"`. The riposte counter (`RiposteRules`) never applies exploit. The planner estimate applies exploit for statuses active at planning time (they stay active for the whole round), so the preview is exact.
5. Order inside one hit: compute (with exploit) -> damage -> status roll of the skill -> riposte reaction -> stance break -> consume.
6. AI (smart path): (a) flow: no special rule, the cheaper chain wins through the existing AP estimate; (b) a STANCE_BREAK skill is preferred over a plain strike against a foe in DEFENSIVE stance or carrying riposte (planned or active) when it is affordable and in reach; (c) target ranking uses the estimate, so exploit-able foes rank higher; an exploit skill is only chosen over the plain strike when its multiplier applies to the chosen target.

### Commits

1. Schema (releases S12C-A): four fields, sections, rank key rule, validation, tokens, SkillDef build; no behaviour change.
2. Rules 1-5 with planner previews and logs; suite.
3. AI rule 6; suite part.

## Acceptance checks (rule-derived; `combo_rules_test_suite.gd`)

1. Every validator message fires for one bad row; a rank that sets `exploit` is refused; all shipped skills validate unchanged.
2. Flow: an ATTACK with `ends_in_stance: DEFENSIVE` followed by Guard plans the Guard with switch cost 0 (without the field: 1); after resolution the unit is Defensive and the log has the flow line.
3. A DEFEND riposte row with `ends_in_stance` fails validation; a non-riposte DEFEND flowing to OFFENSIVE drops its own guard only after its last block.
4. Break: a target guarding over blocks 2-4 hit on block 2 by a STANCE_BREAK attack takes the guarded hit on block 2 (break comes after the hit), then unguarded hits on blocks 3-4 (equal to `compute` with guard 0); its riposte counters the breaking hit once, then is gone.
5. Break on a Tactical or Neutral target does nothing (no log line); a killed target is not broken.
6. Exploit: against a target with active `expose`, damage equals `compute(..., skill_mult x 1.3)` for `{expose, 1.3}`; with only pending expose it equals the plain hit; with two matching statuses the larger multiplier is used; with `exploit_consume` the active status is gone after the hit and the second hit of the same round is plain.
7. Planner estimates equal resolved damage (no crit) for exploit and for the post-break hits.
8. Undo restores stances, `guard_broken` flags and consumed statuses (snapshot equality).
9. AI: given a foe in DEFENSIVE stance and both a strike and an affordable STANCE_BREAK skill, the AI plans the break skill.
10. Existing suites pass unchanged (B1 stance tests, riposte, combat extensions, AI skills).

## Out of scope

Interrupt of multi-block actions and momentum (S12_RULES Q-C1 option C), forced movement, combo chains across units beyond exploit, skill content (S12C-A), new statuses.

## Open questions

None (Q-C1 answered: option B).
