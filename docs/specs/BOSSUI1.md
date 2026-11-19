# BOSSUI1: the boss bar, phases, passives and boss poise

Status: draft, Design 2026-11-19 `[PROVISIONAL]`, from the user's review of battle take 24 (2026-11-19 18:09).
Art/UI mocks it as take 26 (`AI-35`); the Build row (`PR7-54`, section 3) is promoted after the user signs it off.
Builds on `SCREEN_CONTRACT_BATTLE.md` section 9 (boss fight variant) and the encounter `phases` field (BOSS1).
Layer: kit (phase spans, passive tags and `poise_pips` read kit fields, never a boss id), drawn by the game's battle
screen.

## 1. User's points

- "one health bar which gets basically refilled every phase with a phase marker or like phase count, so that we can
  have different phase name for the bosses, like ... states or forms of the boss".
- "there are some boss phases where we fight one enemy in phase one, but we fight a different enemy in phase two".
- "whatever boss passives are there ... as a tag ... hover over it, it shows you the description".
- "The attack impact row looks good."
- "a higher poise break bar ... not have boss break easily ... increase the requirement for one break".
- Statuses: "the no number look is probably better in the UI. For the expanded view, we can always have the round
  count".

## 2. Rules for the bar

- BU1. Phase HP. The bar shows the current phase's HP span as a full bar: for a phase that ends on `hp_below` N %
  of a unit, the span runs from where the phase began to N %; it refills when the next phase starts. The last phase
  runs to 0. A phase that ends on `round` only keeps the plain HP bar for its span. Display only: no rule changes.
- BU2. Phase count. A row of phase markers beside the name (one per phase of the encounter, at most 4; done ones
  filled, the current one lit).
- BU3. Name per phase. The bar's title is the phase's `name` when it has one (a form or state, for example "the
  Foundry Master, Unbound"), else the boss unit's name and title. When a phase brings a different boss unit (a wave
  effect whose unit is the encounter's boss for that phase), the bar follows that unit: its name, HP and poise.
- BU4. Passive tags. Under the title, one tag per boss passive (the unit's `passives`, by display name); hover or
  focus shows the passive's description. No phase change or enrage tags (user 2026-11-19 18:12: the header shows the
  phase; enrage is the baseline rule of `ENRAGE2.md` for every enemy). No other text on the bar.
- BU5. The attack impact row stays as mocked in take 24.
- BU6. Status tabs on the bar and on plates are icon only, no pips or numbers; the expanded list shows the rounds
  left as a number (supersedes the pip line of `SCREEN_CONTRACT_BATTLE` section 6 once signed off).

## 3. Boss poise

- BU7. A unit may carry its own `poise_pips` (kit field on `content.units`, default the ruleset's), so a boss shows
  more pips (recommended 10) on its wider bar. Each pip stays the size the ruleset gives (`T = max_poise /
  poise_pips`), so a boss needs about twice the stagger of a regular unit to break.
- BU8. The field and its display land with no data change (no golden moves). Setting the bosses' `poise` and
  `poise_pips` moves the timeline and sim goldens of any boss fight they hold, so that data step waits for the user's
  approval of the re-record (`NEEDS_YOUR_EYES`).

## 4. Open for the mock

- Where the phase markers sit at 1:1 (left of the name or under the bar).
- How the bar changes between phases (refill sweep, name crossfade) within the existing phase transition.
