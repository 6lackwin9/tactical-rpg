# UI preferences

Owner: UI/UX thread. Firm rules: `docs/decisions/`. UI picks Tushar approves from options are preferences unless the choice came from him (2026-10-18 08:37). Format: preference (date, source).

## Navigation and popups

- Right-click goes back or closes on most settings and UI popups; an X button for back (2026-10-18 playtest).
- One-time screens use explicit buttons (2026-10-18).
- Popups over full screens where possible. The level-up popup shows current stats, with Auto and Revert next to the allocation and Confirm kept separate (2026-10-18).
- Dialogue sits in a bottom dock (2026-10-18).

## Maps

- The map legend can be collapsed (2026-10-18 playtest).
- Click-to-move on region maps; WASD and hold-to-walk (2026-10-18 playtest).

## Combat (COMBAT-UI3)

- Layout A with continuous, colour-coded timeline blocks (2026-10-18).
- A confirm-round warning that a setting can turn off (2026-10-18).
- The combat log expands on click (2026-10-18).
- Compact party cards plus a selected-hero panel (2026-10-18).
- Hostile cards show HP beside the name (2026-10-18, his own choice).
- Strip block labels in a smaller font, the move symbol when a name does not fit, details on hover (2026-10-18, his own choice).

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

- Text size works the Godot way: four levels (Small 0.85, Normal 1.0, Large 1.15, Extra large 1.3), every font scales, and layouts grow with the text; screens that still overflow are fixed one by one (TEXTFIT1), with positions and widths computed from text widths rather than fixed.
