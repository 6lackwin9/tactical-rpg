# AI1: AI use of disengage, sweep, cleanse-only, buff and debuff skills

Status: proposed
Depends on: D1-L (weaken status and zero-power attacks merged). Does NOT need M2C2B content: tests use shipped skills plus in-memory skill defs. Real kits arrive with M2C2B; the sim check belongs to BAL1.
Lane: logic only (no asset task, no new catalog data)
Touches: `scripts/combat/ai/tactical_ai.gd`, `scripts/combat/ai/ai_profile.gd`, `scripts/combat/ai/plan_presets.gd` (only if a preset needs the switch), `scripts/combat/planning/skill_catalog.gd` (list helpers only), new `scripts/tests/ai_skills_test_suite.gd`; one registration line in `arena_test_suite.gd`. No hub file (`content_*`, `skill_def.gd`, `combatant.gd`) is edited; if a read-only helper on `SkillDef` is unavoidable (for example `deals_damage()`), D1-L may already provide it, otherwise add it as a pure additive function and report it.

## Goal

Make `TacticalAI` (enemies every round and the player's AI presets) use the skill kinds that are authored but inert today: DISENGAGE, area attacks (SWEEP, BURST, PLUS, LINE), cleanse-only SUPPORT, self/ally buff SUPPORT (taunt, haste), the riposte DEFEND buff and zero-power debuff attacks (intimidate). A single profile switch restores the old behaviour so balance runs can A/B the effect.

## Schema contract (logic lane)

Schema-first: no (nothing else depends on it). No catalog or content-schema change.

`AiProfile` gets two fields (and `duplicate_profile` copies them):

| Field | Type | Default | Allowed | Meaning |
| --- | --- | --- | --- | --- |
| smart_skills | bool | true | true/false | false = exactly the pre-AI1 behaviour (regression and A/B switch) |
| area_min_foes | int | 2 | 1..6 | an area attack is chosen only if it hits at least this many foes |

Validation: `area_min_foes` is clamped to 1..6 on use; no catalog schema row (profiles are code/preset objects).

## Rules (logic lane)

Existing order of the round draft is kept (withdraw, SUPPORT heal, target pick, buff, guard, kite, engage, advance, rest). New steps are inserted as below; each uses `plan.add_step(..., false)` validation and is removed again if it leaves no useful strike (same pattern as `_buff_then_engage`). All new behaviour is off when `profile.smart_skills` is false.

1. Zero-power attacks are never "the attack". `_try_skills` for ATTACK kind must exclude skills that deal no damage (power_mult 0). Without this, a high-level intimidate would be picked as the main strike (the list is sorted highest level first). Debuff-only attacks are handled by rule 7 only.
2. Disengage. For roles SNIPER and SUPPORT, when a foe is adjacent (distance 1) and the unit knows a usable DISENGAGE skill: try disengage (highest tier whose AP fits, unit not rooted) INSTEAD OF the kite move, then require a strike to fit afterwards (otherwise remove it and fall back to the existing kite). For SKIRMISHER, use it in place of the retreat move under the existing trigger (`hp_ratio <= defend_below`, foe adjacent). Reckless units never disengage. The landing tile is the planner's auto-pick.
3. Area attacks. When choosing an attack, consider area skills of the unit (`SkillCatalog` list helper, not the single-target list). For each candidate compute foes hit and allies hit from the planned (simulated) positions using the same shape code the planner and `AreaAttackCommand` use (no new geometry). Choose it over the best single-target strike when foes hit >= `area_min_foes` AND allies hit == 0; ties prefer the higher expected damage (power x foes hit). SWEEP is aimed at the diagonal neighbour tile that maximises foes hit. The unit may first move/engage as the existing rules do; if the area skill cannot be aimed this round the old single-target path runs.
4. Cleanse-only. A SUPPORT skill with a non-empty `cleanse_statuses` and no heal. Any unit that owns one tries it once per round before the heal step when an ally (or itself if the skill allows self) has `SupportRules.cleansable(...)` non-empty AND is stunned or rooted (cannot act normally); otherwise after the heal step for any cleansable debuff. Target: most cleansable debuffs, then lowest HP ratio. Tier: lowest tier whose list covers all of the target's cleansable debuffs, else the highest tier that fits. Max 1 cleanse per unit per round. Skills with heal AND cleanse are already covered by `_try_heal`; leave that path unchanged but let it also run when the patient is above `heal_below` if it has a cleansable stun or root.
5. Taunt (self SUPPORT granting `taunt`). A unit casts it once per round when it has a living ally within 3 tiles of any foe and does not already hold taunt (active or pending). Only if its strike still fits afterwards.
6. Haste and other ally buffs (SUPPORT granting a buff, no heal, not taunt). Target: the ally that is not already holding that status and whose distance to the nearest foe is greatest but greater than its attack reach + 1; one cast per round; only if the caster's own strike still fits. A buff on self is allowed when the skill target mode allows it.
7. Debuff attack (zero-power ATTACK with `status_effect`, intimidate). Once per round: target the foe with the highest effective attack power that is within the skill's reach this round and is not weakened (active or pending); cast before the main strike; keep the main strike (the unit still attacks), and drop the debuff step if the strike no longer fits.
8. Riposte (DEFEND granting `riposte`). In the existing low-HP guard branch, prefer a riposte skill over a plain guard when the nearest foe is a melee unit (attack_range 1); additionally a unit with `reckless == false`, hp ratio above `defend_below` and an adjacent melee foe may cast it first once per round if it is not already riposting. Rules about the stance break stay with `RiposteRules`.
9. Tier choice stays "highest affordable tier first" (`prefer_level` still overrides); the same ordering applies to the new skill kinds.
10. Player AI presets (`PlanPresets`) use the same code, so a preset-controlled unit now uses its disengage/cleanse/buff skills; the planner UI is not changed.

### Acceptance checks (rule-derived; build kits from existing skills such as `backflip`, `sweep_kick`, `purify`, `mend`, `rally`, `taunt_shout`, `riposte` or from in-memory defs, deriving ids and numbers from the catalog)

- Switch off equals old: for 6 seeded scenarios (one per class role), the plan with `smart_skills = false` is step-for-step identical to the plan produced by the pre-change code (store the expected step lists as derived facts: no new kind appears; compare against a unit that lacks the new skills, which must be identical under both switch values).
- Rule 1: a unit with a zero-power attack at the highest level and a normal strike picks the normal strike as its main attack (step skill id check); the unit with ONLY the zero-power attack still advances/rests as an unarmed idler does and never plans it as damage.
- Rule 2: a SNIPER adjacent to a foe with a disengage skill: first step is the DISENGAGE skill, and an attack step follows; the landing is farther from the foe than the start tile. Rooted unit: no disengage step (falls back to the old kite). A unit without the skill: plan identical to before.
- Rule 3: with two foes placed so a SWEEP/BURST hits both and no ally in the shape, the plan contains the area skill; moving an ally into the shape (or a single foe) makes the plan use the single-target strike; with `area_min_foes = 1` the single-foe case picks the area skill.
- Rule 4: ally with an active stun and a cleanse-only skill known: cleanse step before the heal step and targets that ally; ally with nothing cleansable: no cleanse step; two cleanses are never planned in one round.
- Rule 5: warrior-like unit with an ally adjacent to a foe: taunt step present once; already taunting (active or pending): absent.
- Rule 6: two allies, one hasted already: the cast targets the other, farthest one; none qualifying: no step.
- Rule 7: foes with different power: the strongest in reach is the debuff target; already weakened: skipped; the main strike still follows.
- Rule 8: low-HP melee-facing unit with riposte and guard picks riposte; with a ranged foe picks guard.
- Every produced plan passes the existing validation (no `add_step` errors are swallowed: assert the step count equals the intended count) and a 50-round seeded simulation of a 3v3 mixed kit never plans an invalid step (no exceptions, no zero-AP loops, `MAX_STEPS` respected).
- Determinism: same seed gives same plans; no use of the global rng.

## Assets

None.

## Out of scope

- New skills or numbers (M2C2B), balance tuning (BAL1), perception-reactive AI or difficulty profiles (S4-B), AI using terrain, planner UI changes, taunt steering for the player planner (stays AI-only), AI weighing individual tiers by expected damage beyond "highest first".

## Open questions

1. Should enemies use the Riposte buff at all (rule 8)? It makes warriors on the enemy side slower to kill and may need a stance rule check; default: yes, once per round only when above `defend_below`. Drop rule 8 if you prefer keeping enemy tanks plain.
2. Area attacks that would hit allies: forbidden (default). Allow when foes hit >= 3 and allies hit == 1?
