# INPUT2: battle playtest feedback of 2026-11-02 (as built)

Task row INPUT2 (`docs/TASKS.md`), decision "Playtest 2026-11-02: AP gauge, party hotkeys, ghost fill"
(`games/emberwake/docs/decisions/combat.md`). It changes INPUT1 (`docs/specs/INPUT1.md`): AP6, AP7 and AP10's look,
HK1-HK4's keys and SL6 while a ghost shows. Layers: engine (the gauge widget, the Shift digit key), kit (start AP,
accepting the ghost), game (the battle screen, Help, data seed).

## 1. AP gauge, style D (engine `aegis_ui::pips`, game `battle.rs`)

The user picked style D of the prototype board (`C:/dev/localgen/review/input2/pip_styles_fixed.png`).

- Fixed slots: a slot per AP up to the unit's max (at most 12; above it a slot is `ceil(max / 12)` AP as before), 5 x 8
  px, 1 px gap (12 slots: 71 px). The row never moves or resizes with the plan; the strip keeps its room as before.
- `PipGauge::planned(now, blocks, max, cap)`: each `PipBlock {ap, colour, ghost}` is a capsule over its slots, laid from
  the running total (rounding never adds slots), filled with its colour (a ghost block at `GHOST_ALPHA`), with its own
  rim (its colour 55% toward white, corners left out) so neighbours of one colour read as two, and ticks (45% toward
  black) between its slots. Free AP (held, not planned) are dark sockets with a light outline; slots above the AP held
  are dim crossed slots. A debt crosses every slot and keeps "-n" before them; `.short(n)` turns the outlines red with
  "-n" after.
- `PipGauge::live(start, have, paid, max, cap)`: what was paid this round is one block in `paid` at the left, the AP
  held now are sockets, crossed above the round's start. Levels `spent` and `edge` are fractional while animating (AP10:
  150 ms).
- Reading (the widget text and the agent link's `ap_gauge.pips`): "B1 B1 B1 B2 B2 O O X X": the n-th block's slots,
  free sockets, crossed slots.
- The game passes one block per plan entry that costs AP, coloured by the skill's kind with the strip's
  `kind_color` (ATTACK, MOVE, DEFEND, SUPPORT, FOCUS, DISENGAGE, ...); ghost entries as ghost blocks; playback's paid
  block is grey (`AP_PAID`). Next round's gain is no longer drawn (it stays in the tip, AP9).

## 2. AP numbers: max 12, start 10 (kit, data)

- The AP cap is the encounter's `unit_ap` (or a unit's own `ap`); a battle used to start every unit full. New encounter
  field `start_ap` (kit `schema.json`, both encounter tables; default 0 = full): above 0 every unit starts round 1 with
  at most that much AP, its max kept, so later rounds' `ap_regen` banks up to the max (`EncounterDef.start_ap`,
  `Session::apply_start_ap` after round 1's start, whose refill would top it up; the start snapshot shows it). The
  sims copy the key.
- Seed `games/emberwake/data/seeds/input2.json`: every encounter (catalog and worlds) at `unit_ap` 12, `start_ap` 10,
  applied (user OK 2026-11-02). The PHASEFOLD golden (`emberwake_world/tests/phasefold.rs`) was re-recorded as an
  intended change (`docs/HISTORY.md`); no other golden moved.

## 3. Keys and the ghost (game, kit, host)

- Keys 1-4 select the party's 1st to 4th member (`World.allies`, the party cards' order; any battle; an open menu
  closes; a member off the field says so). The skill hotkeys move to Shift+1-9 (engine `Key::ShiftDigit`, read by the
  digit key's place whatever the layout types; agent link name "shift+1"); menu rows keep their digit label and their
  tip says "Key: Shift+n"; plain 5-9 do nothing.
- While the selected hero's round is its ghost (no pick holds a target), a click on another ally selects that ally (it
  used to become the hero's round target, SL6); Ctrl+click on the ally keeps SL6 (the hero's support menu on it,
  "Select `<ally>`" last; user 2026-11-02); with a target-holding pick SL6 stands for every click.
- Key F accepts the selected hero's whole ghost as picks; a click on a ghost block of the strip accepts the ghost up to
  that entry (the command becomes the latest pick). Kit `round_intent_plan::ghost_picks` / `accept_ghost` turn each
  ghost step into the pick that plans it (a skill on a unit, a tile or self, an item, Rest, a step after; a move before
  a skill is that skill's implied move; a lone first move is Move here) and `pick` them atomically; host request
  `accept_ghost {unit, upto?}` (recorded; reply as `pick` plus `added`). A ghost waiting for Confirm (IN6) sends its
  change first.
- Help (Settings, Controls, Battle): "Select party member 1-4: 1-4", "Skills of the selected hero (timeline):
  Shift+1-9", "Accept the selected hero's auto-fill (timeline): F", "Support an ally while only the auto-fill shows
  (timeline): Ctrl+click".
