# COMBAT-UI2: Combat screen layout (compact columns, hover card, tactic dropdown)

Status: approved (user 2026-10-18 00:32, decision card: option C Compact columns; with it the hover card that also opens on focus and pins on click, and one party tactic with per-hero overrides as in AUTO1). The Rules section is written for A; build it with C's layout (slim left party list, slim right forecast column, one-line expandable log ticker, biggest grid). UIR-5's remaining items fold into COMBAT-UI2-L1
Depends on: UIR-5 (same files; run it first or fold it in, see Tasks), AUTO1 logic part (tactic list and Custom state)
Lane: logic (three batches, no asset part)
Touches: `scripts/ui/*.gd` combat panels (`unit_inspector_ui.gd`, `tactical_forecast_ui.gd`, `timeline_dock_ui.gd`,
`action_log_dock_ui.gd`, `roster_panel_ui.gd`, `playback_controls_ui.gd`), `scripts/ui/combat/*`,
`scenes/combat_arena.gd`, `combat_ui_framework_test_suite`

## Goal

User request 2026-10-18: tactic dropdown in the timeline ("Custom" once the draft is edited), smaller log, inspector
and forecast, forecast next to the log, inspector replaced by a hover card, allies left and enemies right of the grid,
detailed cards for the 4 heroes, no gear editing in combat. Mockups: Design canvas "Combat Screen Layouts" (artboards A, B, C, Hover; not in the repo).

## Options (each keeps the 6-block timeline row on top and the 12x9 grid)

| Option | Layout at 1280x720 | Pros | Cons |
| --- | --- | --- | --- |
| A Flanks | Party cards (4, detailed) left 300 px, hostile cards right ~290 px, bottom strip 128 px: log 60% + forecast 40%; tile 54 px | Teams sit on the side they stand on the grid (Tactics Ogre and Triangle Strategy pair unit info with the board side); a 12x9 grid is height-bound, so the side width is free; all 4 heroes always visible; same tile size as today | Enemy cards only fit up to 5 (6+ needs a scroll or compact rows); two side columns at Extra large text get tight |
| B Bottom dock | Grid centred (tile 49), 4 party cards in a bottom row with Log / Forecast tabs, thin hostile strip right | Closest to XCOM / Into the Breach (HUD at the bottom, board on top); one place for "my squad" | Smallest grid; ~320 px of empty width beside the grid; log and forecast never visible together; enemies far from their side |
| C Compact columns | Today's frame slimmed: party list 220 px, hostiles + forecast 220 px, log as a one-line ticker that opens over the grid; tile 64 | Biggest grid and tokens; smallest change from today | Log history hidden behind a click; forecast in a narrow column wraps; cards too narrow for effects and plan together |

Recommendation: A. It is the user's idea and also the better standard here: the grid cannot grow wider without
growing taller, so the flanks use space B leaves empty, and it keeps the player's eyes on one horizontal line (my team,
board, their team). Borrow from B the "hero card = timeline selector" rule and from C the 14 px ticker style for the
newest log line when the log is collapsed during playback.

## Rules (written for option A; the chosen option C uses the same parts)

1. Timeline row (top, 72 px): line 1 `Round N · Planning`, selected hero, AP, next block, lock badge
   `Hostile plans hidden`; line 2: prev arrow, 6 blocks (clips span blocks, next block gold), next arrow,
   `Tactic` dropdown, `Autoplay` toggle, `Clear`, `Confirm Round`, 32 px settings button.
2. Tactic dropdown: the AUTO1 list (Balanced, All-out, Focus Weakest, Guard & Counter, Kite, Economy) with a one-line
   description per entry (Hover artboard). Picking one redrafts per AUTO1. Any manual edit of a drafted track shows
   `Custom (edited)`; Custom is display-only (not pickable); picking a named tactic again redrafts (confirm when it
   would discard edits, per UI-REVIEW1 X4). Names, descriptions and scope come from AUTO1; this spec owns placement only.
3. Party cards (left, 4 fixed slots): token, name, level, stance chip, HP bar + value, AP bar + value, speed and
   reach, effect chips, plan state (`Plan N AP`, `No plan` in gold, `edited`). Click selects the hero for the
   timeline (replaces the top party box). Empty slots are muted outlines. No GEAR button anywhere in battle.
4. Hostile cards (right): token, name, stance, HP bar, one behaviour line (AI profile + key number). Hover links card
   and token (gold dashed ring on both).
5. Hover card (replaces the Unit Inspector): on mouse hover of a token or card, after 150 ms; contents per Hover
   artboard (enemy: stats, behaviour lines, effects, numbers vs the selected hero; ally: stats, plan steps, effects,
   gear as read-only text). Placed beside the unit inside the grid panel, never covering the hovered tile. Pin button
   or click keeps it; Esc or a click elsewhere closes. Keyboard and controller focus show the same card (Q2).
6. Bottom strip (128 px): log (3 newest lines, playback buttons, Expand opens the existing log modal) and forecast
   (selected hero vs hovered or targeted unit: distance and reach, stance factor, est. strike and crit, one arena
   rules line). Arena rules no longer have their own panel.
7. Gear is read-only in battle: the inspector GEAR button and the equipment modal path from the arena are removed;
   gear is changed in the Character popup outside battle (UI-REVIEW1 Q2).
8. Text scale: 14 px floor at Normal (UIR-5), every panel uses `UiTheme.text_scale()`; panel heights are fixed; as
   text grows, cards drop lines in this order: speed/reach, effects, behaviour line (all stay in the hover card).
   Tile size = `floor(min(area_h / 9, area_w / 12))`, minimum 48 px; side columns keep their minimum width first.

Acceptance (rule-derived): at 1280x720, 1280x800 and 1680x720 (DISPLAY1) and all four text sizes, no control is
clipped and the grid tile is at least 48 px; 4 party cards and 5 hostile cards show without scrolling; no control in
the battle scene opens the equipment modal; the hover card never overlaps the hovered tile and stays in the viewport;
one manual timeline edit sets the dropdown to `Custom (edited)` and picking `Balanced` restores it; the log strip and
forecast are each at most 128 px tall.

## Out of scope

New tactics or AI behaviour (AUTO1), grid size or art, playback logic, the log modal content, controller grid cursor
(UI-REVIEW1 Q6 C), animations.

## Tasks (planner writes TASKS.md)

| id | title | lane | deps | Touches |
| --- | --- | --- | --- | --- |
| COMBAT-UI2-L1 | Flank layout, party and hostile cards, bottom log + forecast strip, gear button removed | logic | UIR-5 (or fold its remaining items in), Q1 | `scripts/ui/*.gd` combat panels, `scripts/ui/combat/*`, `scenes/combat_arena.gd`, `combat_ui_framework_test_suite` |
| COMBAT-UI2-L2 | Unit hover card replaces the inspector (hover, pin, focus) | logic | L1, Q2 | `unit_inspector_ui.gd` (becomes the card), `combat_grid_view.gd`, `combat_ui_framework_test_suite` |
| COMBAT-UI2-L3 | Tactic dropdown + Autoplay toggle in the timeline row, Custom state | logic | L1, AUTO1 logic, Q3 | `timeline_dock_ui.gd`, `combat_timeline_panel.gd`, `combat_ui_framework_test_suite` |

## Open questions

1. Layout: (A) flanks, (B) bottom dock, (C) compact columns. Recommend A.
2. The inspector goes, but UI-REVIEW1 Q3 B ("bigger inspector") and Q4 B (fixed detail pane, tooltips only on icon
   buttons) were approved on 2026-10-17. Replace with: (a) hover card, mouse only; (b) hover card that also opens on
   keyboard/controller focus and pins on click; (c) hover card plus a one-line fixed detail strip. Recommend (b); the
   planner then supersedes those two rows.
3. Tactic scope: (a) one party tactic in the timeline row; (b) per hero, the dropdown follows the selected hero;
   (c) party tactic with per-hero override. Recommend whatever AUTO1 settles; if open, (a), with `edited` on the card
   of each hero whose track was changed.
