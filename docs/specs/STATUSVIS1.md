# STATUSVIS1: show a status where it acts

Status: accepted (user 2026-11-19 19:39 on take 33, take 34 as amended; build row `PR7-57`); drafted by Design 2026-11-19, from the user's principle (2026-11-19 18:05). Art/UI mocks it (`AI-34`); Build
rows follow the user's sign-off of the mock. Layer: kit (the mapping reads each status's `primitives`, `rating`,
`targeting` and `fx` in `kits/rpg/crates/rpgkit_rules/src/statuses.json`, never a status id), drawn by the battle
screen (game).

## 1. Principle

User: "have the visual impact of the status effect clearly visible wherever it impacts it ... we can still keep the
sidebar for all the statuses". So:

- SV1. Each status shows first where it changes play: the timeline, the HP bar, the numbers of a move, the movement
  range, the targeting, the sprite (its `fx`, already built).
- SV2. The plate's status column stays, but holds the statuses that have no such place first; a status already shown
  in place yields its plate slot when the column is full.
- SV3. The full list (hover "+N" or press the plate, `SCREEN_CONTRACT_BATTLE` section 6) always lists every status.
- SV4. The rule is by primitive, so a new status with a known primitive needs no new UI.

## 2. Where each primitive shows

| Primitive (statuses today) | In place | Plate priority |
| --- | --- | --- |
| `block_lock` (stagger, stun, root) | Timeline: the locked blocks on the unit's lane are drawn locked (dark hatch, the status icon on the first locked block, no plan allowed there). Root also shows on the grid: no move range, an iron ring at the feet (`fx`). | 3 (low): in place only when the column is full |
| `tick` (bleed, poison, burn, regen) | HP bar: the coming tick as a pending slice at the round's end (loss oxblood, regen moss), stacked into the plan preview; a small tick mark at the round's end on the unit's timeline lane. Sprite `fx` as now. | 2 (mid): the amount and stacks matter |
| `modifier` `attack_power` (weaken, empower) | The numbers: skill menu, strip block card and hit preview show the changed damage or heal with a small down or up arrow (oxblood or gold) and the status icon on hover. | 3 |
| `modifier` `move_rate` (haste, slow) | The grid: the move range drawn at its changed size, lost tiles as a dim ring (slow) or gained tiles gold-edged (haste); the move block's length on the strip. | 3 |
| `modifier` `armor`, `guard` (expose, fortify, guard) | The target's hit preview: incoming damage already includes it, with a small shield glyph and sign beside the number. | 3 |
| `targeting` (taunt, stealth) | Targeting: taunt pulls enemy intent lines to the taunter (crimson hoop `fx`); stealth greys the unit as a target while choosing (fade `fx`). | 3 |
| `reactive` (riposte) | The hit preview against that unit shows a counter chip ("Riposte: N back"). | 1 (high): it punishes a choice |
| `rule_flag` (steadfast) | None in place yet. | 1 (high): the plate is its only home |

## 3. Plate order

Inside the 4 slots: priority 1, then 2, then 3; inside a priority the soonest to run out first, ties to the one
applied first (as now); statuses a planned move adds come last in their priority. Statuses that do not fit go to "+N".

## 4. Open for the mock (answered by Art/UI, take 25, awaiting sign-off)

- Whether priority 3 statuses still show a slim tab when there is room (recommended: yes, as now).
- How the locked timeline block reads at 1:1 next to the planned-move colours (`SCREEN_CONTRACT_BATTLE` section 1).

Answers (take 25, `/mnt/project-files/art-ui/battle_ux14/battle_take25_status_where_2x.png`): priority 3 statuses keep a
tab when there is room; a locked block is near-black with a 45-degree hatch 5 px apart in the status's rim colour and
the icon framed on the first locked block, the only patterned fill on the strip. The riposte chip reads "Riposte -9".

## 5. Take 25 review (user 2026-11-19 19:26)

- SV5. Locks. Hatched blocks or grey blocks with the status icon both work (keep the hatch of take 25). The rules
  match the user's reading: stagger locks blocks 1 and 4 (`lock_blocks` [1, 4]), so no 3-block move fits and 2-block
  moves must be paced; stun locks the first 2 (`lock_blocks` [1, 2]), delaying the opening. Stun stays at 2 blocks now
  (user 19:28). Criterion for the later balance call: if stun is meant to be much rarer than stagger (which follows
  every poise break), it becomes a strong CC locking blocks 1-3, half a round; the balance lane counts stun versus
  stagger landings per fight and proposes it when stuns are rare.
- SV6. One HP loss. The HP bar shows one planned-loss slice for hits and ticks together (no second negative mark for
  poison or bleed); hover breaks it down (hits, each tick).
- SV7. Numbers after modifiers. Every deterministic number (damage, heal) is shown after all modifiers: the
  attacker's (weaken, empower) and the target's (armour, guard, fortify, expose). The cue is only its colour and a
  small up or down arrow when it differs from the unmodified value; no original number, no shield glyph. Hover shows
  the unmodified value and each modifier with its percent. Crits keep their own display. Replaces the `attack_power`
  and `armor` / `guard` rows of section 2.
- SV8. Taunt on the unit. A taunted unit shows a small icon (or loop) over its sprite, not a ring at the feet, which is
  often hidden. Stealth keeps its fade.
- SV9. Riposte. The chip reads "Riposte" with no number; hover shows the likely return damage on that hero (it depends
  on the hero's armour).
- SV10. Loadout passives apart. The 2 passive slots of a loadout (always on) show as two small icons under the
  portrait on the unit's plate or profile, apart from the buff and debuff column; steadfast as a status stays
  priority 1 in the column.
- SV11. AP (Design 2026-11-19, Art/UI question on take 33): the display follows the rules and changes none; a lock
  takes the blocks away, and the AP row shows the AP the rules leave that round (Build reads it from the battle rules
  when building, `rpgkit_battle` poise and status code), so a lock that also costs AP shows on the row, one that does
  not leaves it.
- SV12. Passive icons: the loadout passives need their own icon set (kit, generic, `kits/rpg/assets/passives/`), asked
  of Art/UI (`AI-39`); placeholder skill icons until then.
- SV13. Take 34 (user 19:39): hovered numbers have no frame, only the breakdown; the 2 loadout passives sit in two
  dedicated sockets centred under the portrait (an empty one dim), each with its own hover (name and description);
  passives never enter the status column (it is for timed statuses). The full timeline view (T) is calmer (kind
  tints, 24 px icons, red only for blow ticks); kept for players and the log as is.
