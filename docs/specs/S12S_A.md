# S12S-A: first skills using the new statuses (five new families on class paths)

Status: spec (design-brainstorm 2026-10-17; DECISIONS combat "S12 baseline answers"; awaiting approval)
Depends on: S12S-L commit 1 (registry rows, magnitude validation, `{magnitude}` token)
Lane: asset
Touches: `data/catalog/skills.json` (5 new families, ranked storage via `ContentCatalog.upsert_skill`), `data/catalog/classes.json` (5 path steps via `upsert`), seed `data/seeds/s12s.gd`, `data/seeds/README.md`

## Goal

Give each new status of the first slice except burn one shipped carrier, so the role limit tests can opt in, without changing any default loadout or any shipped skill. Burn ships with the terrain hazards (S12T-A). Every number below is a PLACEHOLDER starting value for the balance pass, built from the tier defaults (T2 +1 AP, x1.25; T3 +2 AP, x1.5, DECISIONS skills "M2c-2b design answers") and the nearest existing family.

## Authoring rules used

- New families sit on the class path at level 6, after every level-5 family, so the default loadout rule (first five path families) and every explicit unit loadout stay as they are; nothing is slotted. Unlock: tier 1 at 6, tier 2 at 8, tier 3 at 10 (SK2 offsets 0/2/4).
- Grade `novice`, no `requires`, not `learned_only`, origin `user`.
- Existing families are NOT edited (BAL2 just ran; swaps such as sunder to expose, firebomb plus burn or renewal haste to regen are balance-pass candidates, listed in out of scope).

## Assets

### Families (base row = tier 1; ranks list tier 2 and tier 3 overrides)

| Family id | Name | Kind | Class | AP (T1/T2/T3) | Blocks | Weapon gate | power_mult (T1/T2/T3) | Status | Potency (T1/T2/T3) | Magnitude (T1/T2/T3) | Rounds (T1/T2/T3) | Other |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `crippling_shot` | Crippling Shot | ATTACK | ranger | 2/3/4 | 2 | bow, crossbow; min proficiency 1 | 0.7/0.9/1.05 | slow | 75/80/85 | 0 | 2/2/3 | - |
| `rend_armor` | Rend Armor | ATTACK | warrior | 2/3/4 | 2 | sword, axe, mace, hammer; min proficiency 1 | 0.8/1.0/1.2 | expose | 70/75/80 | 6/8/10 | 2/2/3 | - |
| `envenom` | Envenom | ATTACK | rogue | 2/3/4 | 1 | dagger; min proficiency 1 | 0.6/0.75/0.9 | poison | 80/85/90 | 4/5/6 | 3/3/3 | - |
| `aegis` | Aegis | SUPPORT | priest | 2/3/4 | 1 | none | - | grant fortify | - | 6/8/10 | 2/2/3 | `target_mode` ALLY_OR_SELF, `heal_base` 0, `heal_mult` 0.0 |
| `rejuvenate` | Rejuvenate | SUPPORT | priest | 2/3/4 | 2 | none | - | grant regen | - | 4/5/6 | 2/3/3 | `target_mode` ALLY, `heal_base` 0, `heal_mult` 0.0, `heal_stat` KNOW |

AP is the skill `level` field (base row `level` 2; ranks `level` 3 and 4). Blocks are the same on every tier.

### Descriptions (SkillText tokens; literal fallback is not needed)

| Family | `description` |
| --- | --- |
| `crippling_shot` | `{name}: {power} power, {potency} to Slow for {rounds}. Needs {weapons} training {proficiency}.` |
| `rend_armor` | `{name}: {power} power, {potency} to Expose (armor -{magnitude}) for {rounds}. Needs {weapons} training {proficiency}.` |
| `envenom` | `{name}: {power} power, {potency} to Poison ({magnitude} per round, stacks) for {rounds}. Needs {weapons} training {proficiency}.` |
| `aegis` | `{name}: grants Fortify (armor +{magnitude}) for {rounds} starting next round.` |
| `rejuvenate` | `{name}: grants Regen ({magnitude} per round) for {rounds} starting next round.` |

### Class path steps (`skill_path` append, through `ContentCatalog.upsert("classes", ...)`)

| Class | Step |
| --- | --- |
| ranger | `{level: 6, skill: crippling_shot}` |
| warrior | `{level: 6, skill: rend_armor}` |
| rogue | `{level: 6, skill: envenom}` |
| priest | `{level: 6, skill: aegis}`, `{level: 6, skill: rejuvenate}` |

## Acceptance (rule-derived)

1. `ContentCatalog.validate_all()` is `{}`; `warnings_all()` has no new entry for these families.
2. Every class's default active loadout (`Combatant.active_class_families` of a class-derived unit at levels 1, 5, 7 and 10) equals the list before the seed; every shipped unit's explicit loadout is unchanged.
3. A level 6 ranger owns `crippling_shot` and can cast tier 1 only; at level 10 tier 3 (SK2 rule).
4. Each family's rendered description shows the tier's magnitude with its unit (S12S-L token).
5. `tools/fc.sh sim duel 5` and `role 5 quick` results equal the pre-seed run (no default kit changed).
6. Seed listed in `data/seeds/README.md`.

## Out of scope

Edits to shipped families (sunder, firebomb, renewal, quick_stab and others) and role loadout changes in `tools/sim/sim_roles.gd`: both are balance-pass choices once the new statuses are measured; burn carriers (S12T-A terrain); enemy units; world characters; passives that resist the new statuses.

## Open questions

None.
