# ENRAGE2: baseline enrage for every enemy unit

Status: accepted (user 2026-11-19 18:12), numbers `[PROVISIONAL]` (Design 2026-11-19) until the balance run
(`PR7-55`). Layer: kit rule (`rpgkit_battle`), numbers in the ruleset (kit field, game value), so the kit names no game.

## 1. User's words

"the enrage mechanic, we can add it as a baseline. So any unit, basically, if it stays on the board for till 6th
round, then it starts ... the scaling is different naturally for bosses versus elites versus the minions, but the
enrage mechanic does kick in ... for all enemy units. So we don't have to add it as a separate tag." Aim: discourage
turtling, by the AI and by players.

## 2. Rules

- EN1. From round `from_round` (6) every enemy unit on the board is enraged at the start of the round: its attack
  power rises by its tier's step, and again by the step at the start of each later round, up to `max_stacks` steps.
- EN2. Steps by the unit's `tier` (`[PROVISIONAL]`): minion 10 %, standard 10 %, elite 15 %, boss 20 % of attack
  power per step; `max_stacks` 4 (so at most +40 % to +80 %). Heroes and allies never enrage from this rule.
- EN3. An enraged unit keeps today's enraged behaviour (intent revealed, PACE1). An encounter phase `enrage` effect
  still works and may start a boss's enrage earlier; the larger of the two powers applies, steps do not add twice.
- EN4. Ruleset fields (kit schema, `content.rulesets`): `enrage_from_round` (int, 0 = off), `enrage_step` (object by
  tier), `enrage_max_stacks` (int). The game's timeline ruleset sets them; a ruleset without them has no baseline
  enrage, so other kit users are unchanged.
- EN5. UI: no enrage tag on the boss bar or plates (user: drop the phase change and enrage tags). The round counter
  shows the enrage round coming (for example "Round 5 of 6: foes enrage next round"); an enraged unit shows the
  amber tint `fx` and its raised numbers (STATUSVIS1 `attack_power` row). The battle log names it once.

## 3. Balance (`PR7-55`)

- Before the re-record, a sim run on every shipped fight with the rule off and on (`tools/fc.sh golden-bisect` with a
  test-only switch) reports: fights that reach round 6, win rate and party HP left by tier mix, and how much shorter
  long fights get. Target: fights won in 5 rounds or fewer do not move; fights that go to 8 or more get shorter; no
  shipped fight's win rate drops more than 10 points at its intended level. Tune the steps (EN2) inside that band.
- The rule moves the timeline and sim goldens of every fight that reaches round 6. The user asked for the mechanic,
  so the re-record is an intended change: name it in the commit and a `HISTORY.md` row.
