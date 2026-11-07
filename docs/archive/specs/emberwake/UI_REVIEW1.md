# UI-REVIEW1: UI/UX review pass over menus, combat HUD, dialogue and shop

Status: approved (user 2026-10-17 23:37, decision card "Approve all": every fix in sections 1-2 and the recommended option for Q1-Q6; the two small open questions stay open)
Depends on: SETTINGS-POPUP (in progress; settings layout is out of scope here)
Lane: design (this file); fix batches in section 4 are lane logic
Touches (when built): see section 4 per batch

## Goal

List the top issues per screen with a concrete fix, raise the bigger design choices the user may not have considered,
and split the work into small logic batches. Sources: renders in `/mnt/project-files/` (ui1, combat-layout, playable,
dlg1, menu-polish, load-game) checked against `scripts/ui/menus/*`, `scripts/ui/overworld/*`, `scripts/ui/combat/*`.
"(render old)" marks a finding whose render predates MenuStyle; the code check is noted where it changes the finding.

## 1. Cross-screen findings (apply to every menu screen)

| # | Issue | Fix |
| --- | --- | --- |
| X1 | Only the main menu is centred; every other screen is left-aligned with a big empty right half | Shared screen frame: title centred, content in a centred column (max width 1000 px), action bar at the bottom |
| X2 | `MainMenuView.button` sets left-aligned text and 260 px; row buttons (`small`) fit their text, so widths vary per row | Centre text by default; fixed width classes: wide 260, row 140, step 48; never fit-to-text in a row of peers |
| X3 | Back sits in a different place on each screen (under the cards, bottom left, inside the Confirm row) | Back is always the first button of the bottom action bar; primary action (Confirm, Start, Continue) is last and right |
| X4 | Draft screens show Confirm, Cancel and Back; Cancel and Back look the same to a player | Drop Cancel; rename it Reset (clears the draft, stays); Back with a draft asks "Discard changes?" (router rule 4 already asks) |
| X5 | Selection marker "▶ " is prepended to the label (party, journal), so the text shifts on select | Selected state via a style (MenuStyle `selected` box: gold edge, filled), label text never changes |
| X6 | Destructive questions focus Yes (overwrite, delete); `_open_overlay` defaults to index 0 | Focus No for overwrite, delete and quit-without-saving; Yes stays first for harmless questions |
| X7 | Nothing scrolls; party detail already reaches y=715 at Normal text, so its Back is pushed off-screen (11_party) | Content area in a ScrollContainer; action bar outside it, pinned; acceptance: every screen at 1280x720 at Extra large shows its action bar |
| X8 | Text size setting reaches only the menu host (`apply_text_scale`); combat HUD, dialogue and shop ignore it | One `UiTheme.text_scale()` read used by the combat HUD, dialogue and shop panels too |

## 2. Per-screen findings

Main menu (menu-polish/after): good. Add a small version and build label bottom right (muted), useful for playtest
reports. Continue appears only with a save (by design).

New game (02, render old): two full-width cards each with its own Start button. Fix: cards are selectable (click or
arrows), one Start button in the action bar; show the party as four small portraits or class icons instead of a text
list; when worlds share a party (today both do) the line can stay but should not repeat Lv on every member.

Hub (03b, 10; render old): party card spans 1184 px for four short lines; "Party •" dot is cryptic; Journal disabled
without a reason. Fixes: party as four member tiles (name, class icon, level, HP) in one row; badge text "Party (9
points to spend)"; a disabled entry shows why on focus ("No quests yet"); rename Explore to "Return to map".
See Q1 for hub as a popup.

Save / load (load-game/after_load, after_save, 05): empty slots show a disabled Load button; empty and filled rows
differ in height so the list jumps after a save; overwrite question gives no detail. Fixes: fixed row height (two
lines always: header line plus "Empty" or location and party); row actions in fixed slots at the row's right
(Load or Save here, Export, Import, Delete; absent actions leave the slot empty, no disabled buttons); the overwrite
question names what is lost ("Overwrite slot 2: The Ashlands, Warden Keep, 14:21?") and focuses No (X6).

Party (11, render old): Back off-screen (X7); stat table and derived list in one tall column; Active/Reserve button
shows a state, not an action. Fixes: two columns for detail (stat table left, derived values right) so it ends near
y=500; toggle becomes a CheckBox "In party (4/4)" disabled with a reason when full; unspent badge as "+9" pill in a
fixed-width column; Level up / Skills / Gear move into the action bar (or tabs, Q2).

Level-up (12): "STR 0 +1" reads as two numbers. Fix: columns Stat, Now, Added, New (23, +1, 24) with -/+ at fixed
positions; the permanence note stays; Auto, Reset, Back, Confirm in the bar (X4). Show what a stat drives on focus
(Q4).

Skills / loadout (13): the "Slot" column is greyed for slotted skills with no state text; no AP cost or description;
"T1, T2 (Lv 7), T3 (Lv 9)" is dense; empty Extra slots give no hint. Fixes: each row shows name, grade, AP cost and
the next locked tier ("T3 at Lv 9"); the slot button reads Slotted / Add / Full; empty extra section reads "No extra
skills learned yet"; chips row keeps a fixed chip width so removing one does not reflow the rest.

Gear (14, render old; Unequip exists in code now): "(default)" on every line is noise; "Head: -" reads like a
glitch; no stat preview; two paths (list and Gear board) for one job. Fixes: "(default)" only when it differs from
the class default, empty slots read "empty"; selecting a stash item shows the derived-stat delta like level-up (green
or red); stash rows use fixed-width target buttons. Board vs list is part of Q2.

Battle results (15, render old): Level up buttons follow text of varying width, so they zig-zag; Warrior has no
arrow while others do; no gold or loot line. Fixes: grid columns (name, XP, level from -> to, button) with the
button column fixed; show loot and gold rows (M4 RewardGrant) or "No loot"; Continue in the action bar, primary.

Journal (18, 19): good detail pane. Fixes: X5 marker; Back moves into the bar next to the list; empty states per
group ("No completed quests"); a quest list with more than ~10 entries needs the scroll from X7.

Combat HUD (combat-layout/after_hero, after_enemy, playable/*): most text is 10-11 px and ignores text size (X8);
truncations ("next block 1 • ...", "SIMULTANEOUS PLANNIN..."); the Victory banner overlaps the timeline and the
enemy roster with no dim; the planning hint appears twice (bottom-left box and log placeholder); "[+ Hero Slot 3]"
looks clickable; "12x9 COMBAT GRID" is a dev label; the settings cog (top right) is 12 px. Fixes: minimum 14 px at
Normal (scaled by X8); the hostile-timeline notice becomes a one-line lock badge in the timeline bar, full text on
focus; the end banner is a centred popup over a dim of the whole arena with MenuStyle buttons (Review replay,
Continue); one hint location (bottom-left box), the log shows "No actions yet"; empty hero slots render as muted
outlines without "+"; replace the grid caption with "Round 1 - Planning"; cog becomes a 32 px bordered button.
Inspector and forecast panels leave ~250 px empty at the bottom: see Q3.

Dialogue and shop (dlg1, debug region map): 13 px text, no MenuStyle, choices centred; shop shows Buy and Sell as
labels over one list with no item detail; panels sit beside the map with no dim. Fixes: both become MenuStyle popups
over the map (same layer as the settings popup), choices left-aligned and numbered with 1-9 hotkeys, locked choices
show their reason; shop as Buy / Sell tabs with an item detail line (stats, owned count, who can use it) and
"Nothing to sell" when empty; Esc and right click close (dialogue: same as Farewell when allowed).

## 3. Option questions

Q1 Hub: (A) full screen as now; (B) popup over the paused map (Esc toggles; map visible behind the dim);
(C) side drawer on the map. A: no work, map hidden on every visit. B: matches "popups for secondary screens", reuses
the settings popup layer, one less screen swap. C: map stays usable but needs a new layout per text size.
Recommend B.

Q2 Character screens: (A) separate screens as now (party -> level-up / skills / gear); (B) one Character popup with
tabs Stats, Level up, Skills, Gear and a member strip on top (Q/E or shoulder buttons switch member); (C) B plus the
Gear board replacing the gear list. A: no work, four hops per member. B: fewer screens, the member stays in view,
drafts per tab need one "Discard changes?" rule on tab switch. C: one gear path but the board is a mouse-first
modal. Recommend B; keep the gear list in the tab and the board as an optional view.

Q3 Combat HUD density: (A) keep five fixed panels, only fix fonts and labels; (B) merge arena rules into a one-line
strip and give the freed space to a bigger inspector (threat intel, statuses); (C) collapsible side panels (grid
grows when collapsed). A: smallest change, keeps empty space. B: same frame, more useful content, fits 1280x720.
C: grid size changes shift everything, against "nothing should shift". Recommend B.

Q4 Explanations: (A) hover tooltips; (B) a fixed detail line or pane that shows the focused item; (C) both.
A: mouse only, hides at larger text. B: works with keyboard and controller, fixed space so nothing shifts.
Recommend B, tooltips only for icon-only buttons.

Q5 Shared frame: (A) per-screen layout as now; (B) one ScreenFrame helper (title, centred content, scroll,
pinned action bar with Back first and primary last). A: drift continues. B: one place for X1-X7 and for future
screens. Recommend B (first batch).

Q6 Controller: (A) none (UI_MENUS_RULES 2.1 rule 5 baseline; no HISTORY row rejects it); (B) menus, dialogue and
shop only (joypad events on `ui_*` actions, shoulder buttons for tabs, B = back); (C) full, including a combat grid
cursor. A: no work. B: cheap since focus navigation exists. C: grid cursor and timeline editing are a real task.
Recommend B now, C after G2. The baseline's reason was scope, so revisiting it is allowed.

## 4. Proposed task split (lane logic; each ends with a 1280x720 check at all four text sizes)

| Id | Title | Depends on | Touches |
| --- | --- | --- | --- |
| UIR-1 | ScreenFrame, button defaults (X1-X7), selected style | SETTINGS-POPUP merged; Q5 | `main_menu_view.gd`, `menu_style.gd`, new `scripts/ui/menus/screen_frame.gd`, `menus_test_suite` |
| UIR-2 | Main menu label, new game, hub, save / load fixes | UIR-1; Q1 | `main_screen_view.gd`, `new_game_view.gd`, `hub_view.gd`, `hub_model.gd`, `save_slots_view.gd`, `menu_router.gd` (if Q1 B) |
| UIR-3 | Party, level-up, skills, gear fixes (or Character popup) | UIR-1; Q2, Q4 | `party_view.gd`, `level_up_view.gd`, `loadout_view.gd`, `gear_view.gd`, their models, `menu_router.gd` (if Q2 B) |
| UIR-4 | Battle results and journal fixes | UIR-1 | `battle_results_view.gd`, `battle_results_model.gd`, `journal_view.gd` |
| UIR-5 | Combat HUD readability, end banner, text scale (X8) | Q3 | `scripts/ui/combat/*`, `scripts/ui/overworld/battle_screen.gd`, `ui_theme` |
| UIR-6 | Dialogue and shop as MenuStyle popups | UIR-1 | `dialogue_panel.gd`, `shop_panel.gd`, `region_map_view.gd` |
| UIR-7 | Controller for menus, dialogue, shop | UIR-1, UIR-6; Q6 B | `project.godot` input map, `main_menu_view.gd` |

UIR-2, UIR-4 and UIR-6 can run in parallel after UIR-1; UIR-5 is independent of UIR-1.

## Out of scope

Settings layout (SETTINGS-POPUP), art and theme pass (S14 and later), Forge, Dojo and Scene Runner tools, the debug
region map's own layout, any rule or data change.

## Open questions

Q1-Q6 above. Also: should the main menu show a version label (main menu finding)? Is the "(default)" gear marker
needed at all?
