# DRAG1: drag and drop candidates

Status: candidate list (screen pass 2026-10-21); the user picks, then each pick becomes a build row
Depends on: UIKIT2 (the kit's drag and drop, done)
Lane: design (this list); the picked builds are logic rows (game layer, `games/emberwake/crates/emberwake_gui/src/*`)

Sources: TASKS row DRAG1, preference "Character popup" (`games/emberwake/docs/preferences/ui.md`), the screens as they
are on 2026-10-21 (`emberwake_gui/src/*.rs`).

## Already done

- Character popup, Gear tab (GEARUI1): stash to slot or pouch, slot or pouch back to the stash.
- Character popup, Skills tab (SKILLUI1-U): list to slot, slot to slot, slot back to the list.
- Gear and Skill Dojo: the same gear drops as the Gear tab.

## Candidates

Every candidate keeps its clicks; a drag is a second way to do the same thing.

| # | Screen | Drag | Does today (click) | New UI needed | Size |
| --- | --- | --- | --- | --- | --- |
| C1 | Battle planning | A skill from the hero's action list onto the plan strip: the step goes in at that point (before the step on that block), later steps move back | Pick a skill, then a target; the step goes after the last one | none | medium |
| C2 | Battle planning | A planned step along the strip to reorder it; off the strip to remove it | Undo and Clear only (the last step, or all) | none | medium |
| C3 | Battle planning | A planned action's target arrow onto another foe or tile (retarget) | Undo, then plan it again | none | medium |
| C4 | Shop | An offer onto the party's bag (buy one); a bag item onto the counter (sell one) | Click an offer (buy) or a bag row (sell) | a bag pane beside the counter | small |
| C5 | Character popup (or a Party pane) | A member between Active and Reserve, and up or down for the order (who deploys first) | "In party" checkbox on the Stats tab; no order | a roster strip (Active / Reserve) | medium |
| C6 | Save slots | A save onto another slot (copy) | none (save over a slot only) | none | small |

Not candidates (looked at, no second place to move a thing to): main menu, settings, new game, journal, world map
(travel is a click on a node), region map (walk or click a marker), results, the Dojo's skill (one picker, no
loadout). "Loadout presets" in the row has no feature
behind it yet (a preset store would come first); "the battle pouch in camp" is the Gear tab's pouch, done in GEARUI1.

## Recommendation

C1 and C2 first: the plan strip is where a player orders the round (a plan is a sequence of steps whose blocks follow
from their AP), and today a planned step can only be undone from the end. C4 next (small, and the shop is the second
most used screen). C5 once there are more members than the four active places (the roster then matters). C3 and C6 are
low value for their cost.

## Question for the user

Which candidates to build (any of C1 to C6), and for C5 whether the roster lives on the Character popup or in its own
Party pane on the game menu.
