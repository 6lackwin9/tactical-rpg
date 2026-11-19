# STATUSVIS1: show a status where it acts

Status: draft, Design 2026-11-19, from the user's principle (2026-11-19 18:05). Art/UI mocks it (`AI-34`); Build
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
