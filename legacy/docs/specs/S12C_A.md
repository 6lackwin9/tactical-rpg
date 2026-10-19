# S12C-A: first combo skills (three new families on class paths)

Status: approved (user 2026-10-17) (design-brainstorm 2026-10-17; DECISIONS combat "S12 baseline answers")
Depends on: S12C-L commit 1 (fields, validation, tokens); S12S-A merged (exploit targets its statuses; both edit `skills.json` and `classes.json`)
Lane: asset
Touches: `data/catalog/skills.json` (3 new families via `ContentCatalog.upsert_skill`), `data/catalog/classes.json` (3 path steps via `upsert`), seed `data/seeds/s12c.gd`, `data/seeds/README.md`

## Goal

One shipped carrier per combo field: a rogue flow strike, a brawler stance breaker, and a rogue payoff against the warrior's Expose (S12S-A `rend_armor`) and the ranger's Slow (`crippling_shot`), the first party-synergy pair for the role-based balance target. Same authoring rules as S12S-A: new families at path level 6 or later, nothing slotted, no shipped family edited. Every number is a PLACEHOLDER starting value for the balance pass (tier defaults T2 +1 AP x1.25, T3 +2 AP x1.5 from the nearest existing family).

## Assets

### Families (base row = tier 1; ranks list tier 2 and 3)

| Family id | Name | Kind | Class | AP (T1/T2/T3) | Blocks | Weapon gate | power_mult (T1/T2/T3) | Combo fields | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `feint` | Feint | ATTACK | rogue | 2/3/4 | 1 | dagger; min proficiency 1 | 0.7/0.9/1.05 | `ends_in_stance: DEFENSIVE` | none |
| `crushing_blow` | Crushing Blow | ATTACK | brawler | 2/3/4 | 2 | unarmed, fist; min proficiency 1 | 1.0/1.25/1.5 | `disrupt: STANCE_BREAK` | none |
| `opportunist` | Opportunist | ATTACK | rogue | 2/3/4 | 1 | dagger; min proficiency 1 | 0.8/1.0/1.2 | `exploit: [{status: expose, power_mult: 1.3}, {status: slow, power_mult: 1.2}]`, `exploit_consume: false` | none |

AP is the skill `level` field (base 2; ranks 3 and 4). Grade `novice`, no `requires`, ~~not `learned_only`~~ `learned_only: true` (as built, S12C-A review 2026-10-17: keeps hand-built units' skill lists and the sim baseline unchanged; class units still get the family from the path), origin `user`. Combo fields are family level (S12C-L), so they appear once on the family entry.

### Descriptions

| Family | `description` |
| --- | --- |
| `feint` | `{name}: {power} power, {flow}. Needs {weapons} training {proficiency}.` |
| `crushing_blow` | `{name}: {power} power, {disrupt}. Needs {weapons} training {proficiency}.` |
| `opportunist` | `{name}: {power} power, {exploit}. Needs {weapons} training {proficiency}.` |

### Class path steps

| Class | Step | Unlocks (T1/T2/T3, SK2) |
| --- | --- | --- |
| rogue | `{level: 6, skill: feint}` | 6/8/10 |
| rogue | `{level: 7, skill: opportunist}` | 7/9/10 (capped at level 10) |
| brawler | `{level: 6, skill: crushing_blow}` | 6/8/10 |

## Acceptance (rule-derived)

1. `ContentCatalog.validate_all()` is `{}`; descriptions render with the S12C-L tokens (`feint` shows "ends in Defensive").
2. Default active loadouts of every class at levels 1, 5, 7 and 10, and every explicit unit loadout, equal the pre-seed lists.
3. A level 7 rogue owns `feint` and `opportunist` (tier 1 castable); `opportunist` against a target with active expose 8 deals the S12C-L exploit damage (x1.3 skill power).
4. `tools/fc.sh sim duel 5` equals the pre-seed run.
5. Seed listed in `data/seeds/README.md`.

## Out of scope

Edits to shipped families (for example `shield_bash` or `haymaker` with `STANCE_BREAK`, `assassinate` with exploit), sim role loadout changes, enemy units: balance-pass choices once the combos are measured.

## Open questions

None.
