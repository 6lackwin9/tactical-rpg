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
- BU2. Phase count. A row of phase markers left of the name (one per phase of the encounter, at most 4). User
  2026-11-19 18:24 (take 26): all markers start lit, the current one highlighted; a phase beaten goes dim, and the
  highlight moves on, so the fight shows from the start how many phases are left.
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

## 4. Open for the mock (answered by Art/UI, take 26, awaiting sign-off)

- Where the phase markers sit at 1:1 (left of the name or under the bar).
- How the bar changes between phases (refill sweep, name crossfade) within the existing phase transition.

Answers (take 26, PR #70, `art-ui/battle_ux14/battle_take26_*`): phase markers are diamonds left of the name (done
filled gold, current lit, coming hollow iron); the change runs inside the phase transition, about 0.9 s (the span
drains, a 0.1 s gold flash as the marker fills, title and portrait crossfade 0.3 s for a new unit, the bar refills in
0.4 s); the number beside the bar is the span left ("160/450"). Enrage (ENRAGE2 EN5): the round counter shows
"Enrage next round" in amber a round ahead, then "Foes enraged +n"; an enraged plate takes a 10 % amber tint and rim.

## 5. Take 26 review (user 2026-11-19 18:24)

- BU9. Poise target (user 2026-11-19 18:26, confirming): a high break threshold plus the larger bar, so a boss
  breaks "only once every 3 rounds or something like that when fighting against a party of the same level". Target,
  same-level party focusing the boss: the first break in round 2 or 3, then about one break every 3 rounds; never a
  break in round 1 from a full bar. Breaking stays optional (poise is an interrupt, not a required mechanic): a hero
  too weak to dent the bar is fine, and building a party to break the boss is a valid choice. Tuned later in the
  `PR7-54` data step (sim breaks per fight); the golden re-record stays the user's call.
- BU10. Poise pips read at a glance: pips in groups of 5 with a gap (10 = two groups); at most 10.
- BU11. Boss intent panel (take 28, after the user's 18:34 note that take 27 was "too busy"): in the bar's space
  instead of the standard foe card, two damage rows by block ("Block 1" to "Block 6"): the selected hero's damage per
  block (gold-ringed cell) and the party's damage per block, with BREAK and the next phase's marker in the block where
  each lands; totals at the right; the boss's own attack impact row kept. No per-member chips or "After" row. It
  stays compact and never covers a unit or the selected target.
- BU12. Any intent box or card, for bosses and mobs, sits clear of the acting unit and its target on the board.
- BU13. The round banner carries the enrage cue (ENRAGE2 EN5): "Enrage next round", then "Foes enraged +n %" with
  the step the foes gained; confirmed by the user's reading of take 26.
- BU14. ~~Overflow carries into the next phase~~ (Design `[PROVISIONAL]`, replaced the same day by BU15, user).
- BU15. Phase break ends the round (user 2026-11-19 18:38: "if you do kill a phase in a round, it should trigger the
  next phase in the next round only ... the damage also should not carry forward ... the moment the phase change
  happens ... it should end the round, so none of the other actions or AP consumption happens"). A rule (kit,
  `rpgkit_battle`): when a hit takes a boss unit to or below the threshold that ends its phase (`hp_below`), its HP
  stops at the threshold (the rest of the hit is lost), the round ends at once (no later scheduled action of either
  side resolves and none spends AP), the usual end-of-round steps run (`[PROVISIONAL]`: ticks and status countdown,
  which cannot take the boss below the threshold), and the next round starts in the new phase with its effects. The
  last phase ends with the boss's defeat as today. Phases that end on `round` are unchanged.
- BU16. Boss intent rows (user 2026-11-19 18:38 on take 28): each block cell shows the impact, as the small foe
  intent bar does: HP damage, poise damage and the status chips; no skill names or icons (the strip at the top has
  them). The party row shows the phase break in the block it lands and greys the blocks after it (they will not run).
