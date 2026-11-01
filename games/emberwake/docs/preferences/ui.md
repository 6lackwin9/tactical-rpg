# UI preferences

Owner: UI/UX thread. Firm rules: `docs/decisions/`. UI picks Tushar approves from options are preferences unless the choice came from him (2026-10-18 08:37). Format: preference (date, source).

## Navigation and popups

- Right-click goes back or closes on most settings and UI popups; an X button for back (2026-10-18 playtest).
- One-time screens use explicit buttons (2026-10-18).
- Popups over full screens where possible. The level-up popup shows current stats, with Auto and Revert next to the allocation and Confirm kept separate (2026-10-18).
- Dialogue sits in a bottom dock (2026-10-18). Refined: a 400 px box centred at the bottom, portrait and name on the left, the NPC's line above the
  replies (user playtest 2026-10-22, PLAYUI1-D).

## Maps

- The map legend can be collapsed (2026-10-18 playtest).
- Click-to-move on region maps; WASD and hold-to-walk (2026-10-18 playtest).
- Legend rows and edge arrows pan the camera; a second row click walks; Q or a click on the leader switches the exploring leader (user 2026-11-02: "enable the option to switch the active unit for exploration"; MAPNAV1-U).

## Combat (COMBAT-UI3)

- Layout A with continuous, colour-coded timeline blocks (2026-10-18).
- A confirm-round warning that a setting can turn off (2026-10-18).
- The combat log expands on click (2026-10-18).
- Compact party cards plus a selected-hero panel (2026-10-18).
- Hostile cards show HP beside the name (2026-10-18, his own choice).
- Strip block labels in a smaller font, the move symbol when a name does not fit, details on hover (2026-10-18, his own choice).
- Timeline battles: one input flow, target then skill, the intent read from the skill, the planner's ghost fill with an
  "Auto-fill the rest" setting, keys 1-9 as skill hotkeys; no intent buttons, Manual or Fill (decision "Combat input
  feedback", 2026-10-31; built INPUT1-U slice 2, 2026-11-01). Selection: a right click on any unit inspects it (the
  threat intel panel pinned, only what the party knows), a click on the selected hero or Esc selects nothing, with no
  hero selected a click on an enemy inspects it (same decision; built INPUT1-U slice 3, 2026-11-01).

## Test tools

- Dev-only UI appears only in test scenes (Forge, scenario scenes), never over the real game; the Scene Runner pill follows this (2026-10-18).
- The UI Gallery is where every UI element is judged on its own, screen by screen and panel by panel, then shared panels; the aim is screens composed from those fixed elements, laid out in the engine with resizable elements and described to the AI in words (2026-10-20, LAYOUT1).

## Character popup (user playtest 2026-10-20)

- Skills tab: drag a skill from the list onto a loadout slot (clicking still works); each filled slot has < > arrows over the unlocked ranks (decision "Skill rank choice").
- Skills tab as built (SKILLUI1-U, 2026-10-21): five class and three extra fixed slots beside the owned list; drag list to slot, slot to slot (swap) and slot to list; right click on a filled slot takes it out; Clear all empties the draft; a full group's click shows how to make room; the rank numeral is gold when kept below the highest unlocked rank.
- Gear tab: three columns like the Godot gear dojo: character info and stats | equipped slots (click to unequip, with the stat change) | party stash; the battle pouch under the slots; drag and drop both ways (stash to a gear or pouch slot equips, an equipped or pouch item back to the stash unequips; user 2026-10-20), clicks still work; the detail pane explains the hovered row.
- Every row (stats, skills, gear) is hoverable and fills the detail pane; "Move to a row" must never stay on screen while the mouse is over a row.
- Drag and drop wherever moving a thing between two places makes sense, on every screen, once the UI kit has it (UIKIT1, then DRAG1); clicking keeps working everywhere (user 2026-10-20).

## Text size (user 2026-10-20)

- Text size works the Godot way: four levels (Small 0.85, Normal 1.0, Large 1.15, Extra large 1.3), every font scales, and layouts grow with the text; screens that still overflow are fixed one by one (TEXTFIT1), with positions and widths computed from text widths rather than fixed. Checked with the agent link's `text_audit` at each size (TEXTFIT1, 2026-10-21); a list row's one-line description and a timeline span may shorten (the full text is on hover or click).

## Battle overlays and the clarity pass (user 2026-10-30)

- While the fight logic is built, every battle overlay, badge and readout stays (tells, intents, impact badges, reach band,
  facing chevrons, revealed walks, status looks): they are good to have for debugging. Once the fight logic is built, a
  visual clarity pass decides what the actual game view keeps; the rest stays reachable for debugging (VISCLAR1).
- Slice A's outcome (VISCLAR1-A, 2026-11-01; decisions "Combat input feedback" and "INPUT1 answers", parts 1 and 4):
  every overlay group stays on by default but two new ones, off: `first_move` (the enemy's first-move kind chip on its
  token, a debug view) and `all_bars` (every unit's token bars; by default only the selected hero's, its target's, the
  hovered unit's and the inspected unit's; Alt shows all). One quiet dashed target arrow per unit (alpha 90) replaces
  the tell arrows, planned-move lines and revealed walks, the selected and inspected unit's by default, all with Alt,
  only while planning; `moves` and `walks` now switch its move leg. Debug builds add "Party foresight off" at the foot
  of the Battle tab. The intent glyphs, the destination plates (planning), the facing chevrons and the arena telegraph
  stay.
- Playback aids, built to be trimmed after playing (INPUT1-PB, 2026-11-01; decision "INPUT1 answers, part 2" Q8): a
  short tracer from each actor to what its action hit (about 0.4 s, dashed for a miss, none for self; overlay group
  "Impact tracers"), a gold frame on the strip's resolving block, playback speed 1x / 0.5x / block by block (chips
  during playback, P, N, remembered), the combat log grouped by round and block, and the full timeline view (T)
  pausing the playback at the block's end. Nothing else draws on the board during playback.
