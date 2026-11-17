# BUNKS1: the hub bunks and bunk talks

Status: active (user 2026-11-17: "all recs bunks", "all recs bunks 2"; lead swap BK9-BK11 `[PROVISIONAL]`, proposed
the same day). Layers: kit (talk conditions, pair counter, world kind `talks`), game UI (`emberwake_gui` hub) and game
data (`sunk_ward` talks). Art states are request `AI-17`. Builds on PK3 roster housing (`bunk`, `crypt`, `crypt_only`)
and the region leader (`GameState::explore_leader`, MAPNAV1-U).

## 1. Bunk states (I1)

- BK1. Each hub bunk is bound to a member by `housing: "bunk"` in roster order; there are as many bunk props as the
  bunk cap, and unused ones are empty.
- BK2. States, drawn on the bunk prop: empty (bare, rolled mattress); benched member (made bed, the member resting
  there, keepsake on the post, name plate); active member (made bed, kit packed at the foot, name plate); Stage IV in
  the crypts (made bed with a cloth draped over it, name plate). `crypt_only` Martyrs have no bunk.
- BK3. Hover shows the name and one condition word: resting, wounded, high Strain or ready.
- BK4. Clicking an occupied bunk opens a small card: portrait, name, condition, a waiting talk if any, and "Take into
  the squad" or "Stand down" (`roster::toggle_active`, with `party_min_refusal` and the Martyr `fuse_refusal` prompt
  as today). The party screen keeps working as before.

## 2. Bunk talks (S1)

- BK5. New world kind `talks`: `{id, pair: [a, b] | template: true, requires_flags?, min_together?, temperament?,
  origin?, dialogue, reward?}`. `dialogue` names an existing `dialogues.json` entry; `reward` is an item id (often a
  keepsake). No battle stats.
- BK6. A pair counter `party.together` (`"a|b"` sorted, skip-if-empty) adds `1` for each pair fielded together in a won
  or withdrawn battle. `min_together` reads it.
- BK7. Founder pairs and named recruits (Talia's runaway, named rescues) get written `pair` talks. Generated recruits
  get `template` talks picked by their `temperament` or `origin` and paired with any founder, with `{a}` and `{b}`
  filled from the names; each template plays at most once per recruit.
- BK8. A talk is waiting when its conditions hold and its seen flag (`talk.<id>`) is unset. At most one waiting talk
  per member is shown, the oldest first. A waiting talk lights a candle on both members' bunks and a speech mark over
  each of them where they idle. Interacting with either member plays it through `DialogueRunner`; it sets the seen
  flag and gives the reward.

## 3. Idle spots and passing the lead `[PROVISIONAL]`

- BK9. Every member in the hub who is not the lead idles: active members at the social tables, benched members at
  their bunks. Spots come from the hub map's spot list in a fixed order; Stage IV members in the crypts and Martyrs are
  not shown.
- BK10. Passing the lead (Q as in the regions, or "Take the lead" on a member's click card): the two trade places. The
  new lead stands up from their spot and is controlled from there; the old lead walks to the freed spot and takes its
  idle pose. No other member moves, and the camera follows the new lead.
- BK11. Talks are unaffected by the lead: markers stay on their members, and a talk involving the lead plays when the
  lead interacts with the other member.

## 4. Tests

Kit: the pair counter, talk conditions, template pick and the seen flag; content check of `talks` rows. GUI audit:
the four bunk states, the click card refusals, and the lead swap (positions after Q). `0` golden diffs.
