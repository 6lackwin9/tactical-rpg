# S12S-L: status expansion (slow, expose, fortify, regen, poison, burn), registry magnitude ranges, round-end order, sim channels, AI use

Status: spec (design-brainstorm 2026-10-17; direction in DECISIONS combat "S12 baseline answers" and docs/specs/S12_RULES.md 2.0-2.2; awaiting approval)
Depends on: ITEM1-L and CLASS1-L merged (shared hubs: `combatant.gd`, `content_schema.gd`, validators); first of the S12 slices (S12S, then S12T, then S12C)
Lane: logic (schema first; commit 1 releases S12S-A)
Touches: `scripts/combat/status/{status_registry,status_effects,support_rules}.gd`, `scripts/combat/actions/defend_command.gd`, `scripts/core/combat/combatant.gd` (`move_per_block`, status source dicts), `scripts/core/combat/damage_calculator.gd` (`armor_rating` only), `scripts/combat/arena/combat_resolver.gd` (`apply_round_end_effects`), `scripts/combat/session/combat_session.gd` (snapshot fields), `scripts/content/skill_fields.gd`, `scripts/content/skill_kinds.gd` (sections), `scripts/content/validators/skill_validator.gd`, `scripts/content/skill_text.gd` (token), `scripts/combat/ai/tactical_ai.gd` (AI1 section), `tools/sim/sim_metrics.gd`, new `scripts/tests/status_expansion_test_suite.gd` + one line in `scripts/tests/suite_registry.gd`; `scripts/art/status_art.gd` glyph rows if S14-L1 has merged

## Goal

Six registry statuses that act only through existing inputs (tiles per move block, armor rating, the round-end tick, the heal path), a per-status magnitude range replacing the weaken special case, magnitudes on granted buffs, a fixed round-end order, sim metric channels so conservation checks keep holding, and AI use. `DamageCalculator.compute` is unchanged; S1 keeps `balance-ready`.

## Schema contract (logic lane, schema first: yes)

### Registry row fields (added to every `StatusRegistry.ROWS` row)

| Field | Type | Default | Validation (`row_errors`) |
| --- | --- | --- | --- |
| `magnitude_min` | int | 0 | 0..100 |
| `magnitude_max` | int | 0 | `>= magnitude_min`; 0 with min 0 = no magnitude |
| `magnitude_unit` | enum `pct`, `flat`, `pct_max_hp` | `pct` | one of the three |

`StatusEffects.has_magnitude(status)` becomes `StatusRegistry.row(status).magnitude_max > 0` (was: `status == WEAKEN`).

### Rows (new rows; numbers are PLACEHOLDERS for the balance pass)

| id | kind | resistable | default_rounds | stacking | cap | magnitude min..max (unit) | display | grant |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `weaken` (existing, gains range) | debuff | true | 2 | refresh_max_duration | 25 | 1..25 (pct) | Weakened | "" |
| `slow` | debuff | true | 1 | refresh_max_duration | 0 | 0..0 | Slowed | "" |
| `expose` | debuff | true | 2 | refresh_max_duration | 20 | 1..20 (flat) | Exposed | "" |
| `fortify` | buff | false | 2 | refresh_max_duration | 20 | 1..20 (flat) | Fortified | `support` |
| `regen` | buff | false | 2 | refresh_max_duration | 10 | 1..10 (pct_max_hp) | Regenerating | `support` |
| `poison` | debuff | true | 3 | stack_potency | 30 | 1..15 (flat) | Poisoned | "" |
| `burn` | debuff | true | 2 | refresh_max_duration | 10 | 1..10 (pct_max_hp) | Burning | "" |

The existing seven rows get `magnitude_min/max` 0 and keep all other values. New `StatusEffects` constants `SLOW`, `EXPOSE`, `FORTIFY`, `REGEN`, `POISON`, `BURN` and `SLOW_TILES: int = 1` (structural, mirror of `HASTE_TILES`). Fortify may also be granted by DEFEND skills: the `grant` value stays `support` (queued next round); DEFEND grants of support buffs already queue (M2C2B-L X2).

### Skill fields

| Field | Change |
| --- | --- |
| `status_magnitude` (`skill_fields.gd`) | label "Status magnitude"; drop the `when` on `status_effect in [weaken]`; help "Strength of the status (unit and range from the status registry: weaken %, expose/fortify armor points, regen/burn % max HP, poison HP per round)."; min 0, max 100 |
| SkillKinds sections | `status_magnitude` belongs to ATTACK (with `status_effect`), SUPPORT and DEFEND (with `grant_status`) |
| SkillText token | `{magnitude}`: the row's magnitude with its unit (`15%`, `8 armor`, `5% max HP`, `4 HP`) |

Validation (`SkillValidator`, per tier; replaces the weaken branch). The "magnitude status" of a row is `status_effect` if set, else `grant_status`.

| Case | Message (exact) |
| --- | --- |
| magnitude status has a range and magnitude outside it | `"%s needs a magnitude %d-%d" % [status, min, max]` |
| magnitude status has no range (or none set) and magnitude != 0 | `"magnitude only applies to %s" % ", ".join(statuses with a range)` |

`grant_status` options: `fortify` and `regen` join `StatusEffects.support_buffs()` automatically (registry `grant`). Passive `resist` keys, `cleanse_statuses` options and the Forge pick the new debuffs up automatically (registry checklist).

### Combatant fields

| Field | Type | Meaning |
| --- | --- | --- |
| `status_sources` | Dictionary status -> String | unit name of the last unit that applied the ACTIVE instance |
| `pending_status_sources` | Dictionary status -> String | same for the queued instance |

`StatusEffects.queue`, `try_inflict` and `grant_now` gain a last param `source: String = ""`; `roll_over` moves pending sources to active (the newer source wins). Snapshots (`CombatSession` snapshot at lines ~596 and ~682) store and restore both dicts and `pending_status_magnitudes`, so undo rewinds them.

## Rules (logic lane)

1. Slow: `move_per_block()` = `clampi(base, 1, 4) + HASTE_TILES x hasted - SLOW_TILES x slowed`, floor 1. Haste and slow on one unit cancel.
2. Expose and fortify: `DamageCalculator.armor_rating(target)` = `maxi(0, base + fortify_magnitude - expose_magnitude)`, where `base` is today's value (gear + STR / 4, or `armor_override` when set). Input only; every caller (compute, planner estimate, forecast, sim stats) sees the same rating.
3. Round end (`CombatResolver.apply_round_end_effects`), fixed order per unit in `registered_combatants` order, each step skipped for a unit that is no longer active:
   1. bleed (unchanged: 6% max HP, min 1);
   2. poison: `take_damage(magnitude, true)` (total stacked magnitude, ignores guard and armor);
   3. burn: `take_damage(maxi(1, roundi(max_hp x magnitude / 100.0)), true)`;
   4. regen: heal `maxi(1, roundi(max_hp x magnitude / 100.0))`, capped at missing HP, through the same HP-gain call `SupportRules` uses.
   The return value stays `Combatant -> damage dealt` (bleed + poison + burn); new `last_round_end_heals: Dictionary` (Combatant -> healed) is filled each call. Log lines (exact): `"[Poison] %s takes %d poison damage (HP %d/%d)%s"`, `"[Burn] %s burns for %d damage (HP %d/%d)%s"`, `"[Regen] %s regenerates %d HP (HP %d/%d)"`; the `%s` suffix is `" -> %s DEFEATED!"` as for bleed. A unit defeated in step 1-3 does not regenerate.
4. Poison stacking: `stack_potency` sums magnitudes (pending with pending in `queue`, pending with active in `roll_over`), capped at 30; rounds take the longer. Already generic in `StatusRegistry.stack`; this is the first content use.
5. Granted magnitudes: `SupportRules.apply` and `DefendCommand` pass `skill.status_magnitude` and the caster's name to `queue` / `grant_now`.
6. Resist and cleanse: the new debuffs roll resistance like every attack status; `cleanse_statuses` `all` removes them (active and pending, existing rule).
7. Planner and forecast: the expected hit uses the exposed or fortified rating (rule 2); the plan's move preview uses the slowed tiles per block; status chips list the new statuses with `{magnitude}`.
8. AI (smart path only, `tactical_ai.gd` AI1 section):
   - debuff options: a foe that already carries a `stack_potency` status is still a valid target while `active + pending magnitude < cap`; for other statuses the current "skip if has_any" rule stays;
   - expose: among valid foes, prefer the actor's ranked first target, then a foe adjacent to an ally;
   - slow: prefer the nearest foe that can reach a fragile ally (lowest max HP) this round;
   - fortify (ally buff): target the ally or self with the most adjacent foes (ties: taunting unit, then lowest armor rating), skip a unit already fortified;
   - regen (ally buff): target the most wounded ally below the profile's heal threshold (`_needs_heal`) without regen; a unit that can heal this round heals first.
9. Sim metrics (`tools/sim/sim_metrics.gd`): new per-unit fields `regen_received`, `dot_dealt`. At round end a gain on a unit with active `regen` counts as `heal_received` and `regen_received` of that unit and as `heal_done` of `status_sources["regen"]` (not `other_gain`); a round-end loss stays `dot_taken`, and `dot_dealt` is credited to the source of the status that dealt it (bleed, poison, burn; split per step using the resolver's per-step amounts, exposed as `last_round_end_steps: Array[Dictionary]` `{unit, status, amount, source}`). `buff_rounds` / `debuff_rounds` count the new statuses (status diff, existing logic). Conservation (`hp_start - dmg_taken - dot_taken + heal_received == hp_end`) must hold with no `METRIC FAIL`.

### Commits

1. Schema (releases S12S-A): registry fields and rows, `has_magnitude` from the registry, skill field and section changes, validator messages, `{magnitude}` token, Combatant source dicts and snapshot fields; behaviour unchanged except validation (weaken tests unchanged).
2. Effects: rules 1-7 (slow, armor inputs, round-end order and logs, grants with magnitude, previews).
3. AI and sim: rules 8-9, `StatusArt` glyph rows (if S14-L1 merged).

## Acceptance checks (rule-derived; `status_expansion_test_suite.gd`)

1. `StatusRegistry.row_errors()` is empty; every `StatusEffects` id constant has a row (existing registry test) including the six new ones.
2. A skill row with `status_effect: weaken` and magnitude 0 or 26 gives "weaken needs a magnitude 1-25"; `expose` 21 gives "expose needs a magnitude 1-20"; `stagger` with magnitude 5 gives a message starting "magnitude only applies to"; a SUPPORT row granting `regen` with magnitude 4 is valid; all shipped skills still validate (`validate_all` `{}`).
3. Slow: a unit whose base tiles per block is 1 still moves 1 when slowed; base 3 slowed gives 2; hasted and slowed gives base.
4. Expose with magnitude above the base rating gives rating 0; fortify 8 on rating 10 gives 18; `compute` and the planner estimate use the same rating (equal expected hit before crit).
5. Round end with bleed, poison 6, burn 5 and regen 5 on one 200 max HP unit at 150 HP: HP after = 150 - 12 - 6 - 10 + 10 = 132; the log lines appear in that order; a unit reduced to 0 by poison does not regenerate.
6. Two poison applications of 10 and 25 queued in one round become one active instance of 30 (cap); a third of 10 next round keeps 30.
7. A regen grant from a SUPPORT skill with magnitude 5 starts next round (pending first), lasts its `status_rounds`, and records the caster in `status_sources`.
8. Undo (`CombatSession.reset_round`) restores statuses, magnitudes and sources exactly (snapshot equality).
9. `cleanse_statuses: ["all"]` removes active and pending slow, expose, poison and burn, never fortify or regen.
10. AI: with a stacking-capable poison skill and a foe at poison 10 (cap 30), the debuff option is offered; at 30 it is not. A wounded ally below the heal threshold receives regen when no heal is affordable.
11. Sim: `tools/fc.sh sim party 5` and `role 5 quick` with a fixture loadout that casts regen and poison print no `METRIC FAIL`; `heal_done` of the regen caster equals the sum of its targets' `regen_received`.
12. Existing suites pass unchanged (weaken, D1, riposte, combat extensions, AI skills, BAL2 sim).

## Out of scope

Blind and silence (S12_RULES Q-S1 option C), terrain hazards (S12T-L applies these statuses from tiles), skill content (S12S-A), exploit payoffs (S12C-L), balance numbers beyond the marked placeholders, Forge UI changes beyond automatic option lists.

## Open questions

None (Q-S1 answered: option B).
