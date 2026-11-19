# UISPLIT1: screen behaviour apart from layout and look

Status: active (user 2026-11-18: "is it possible to have the behavior of the UI separated from the actual like the
layout and the design of the UI, so that we can like have build work on the feature side, independent of the mock work
and the UI implementation that we do?"; then "Yeah, go ahead with the split."). Choices marked US are `[PROVISIONAL]`
(Design 2026-11-18, `P-ARCH`, lowest undo cost). Layers: engine (`aegis_ui`: the layout format, components, loader,
check, preview), game (`emberwake_gui`: per-screen view-models; `games/emberwake/assets/ui/layouts/`: layout files).
Builds on the theme (`aegis_ui::theme`, LOOK1-E1), the panes (`aegis_ui::panes`), the grid (`LAYOUT720`) and the
style rows `PR7-25` and `PR7-36`. A first step toward `ENGINE-8` (`KIT-UI1`).

## 1. Three parts per screen

- US1. **View-model (behaviour, Build)**: each screen has a plain `view(&state) -> <Screen>View` that lists what it
  shows (named fields: texts, numbers, lists of rows, pictures by asset path, enabled flags) and the actions it offers
  (named ids with arguments), and an `act(action) -> Nav` that runs them. No drawing, no rects, no colours. Testable
  headless; the agent link reads the same view.
- US2. **Layout file (layout, Art/UI)**: `ui/layouts/<screen>.json` over the asset layers (engine, kit, game, as the
  theme). It names regions on the screen grid (anchor, size in grid tokens, optional breakpoints by text size), and for
  each region one component from a fixed set with its bindings to view-model fields and actions. Look (colours, fonts,
  frames) stays in `theme.json`; a layout names theme roles only, never raw colours.
- US3. **Components (engine)**: the fixed set a layout can use, game-agnostic: panel shell (main ornate or popup double
  line, `PR7-36` item 2), list row (`PR7-36` item 9), tab rail, slot card, menu column, portrait (F2 frame, `PR7-25`),
  icon grid, stat block, text block, button bar, bar meter, tooltip, and a `custom` slot that hands its rect to screen
  code (the battle board, the world map, any drawn stage).

## 2. Defaults and fallback

- US4. A screen with no layout file draws a plain default: its view-model fields top to bottom in a panel shell, actions
  in a footer button bar. Build ships a feature on the default; Art/UI's layout replaces it without code changes.
- US5. A layout that names a missing field or action, or an unknown component, fails the gate check; at run time the
  screen falls back to the default and the shell shows the problem (as the theme does).

## 3. Authoring and preview (Art/UI)

- US6. Each view-model ships a fixture (`<screen>.view.json`, sample values) so a layout can be previewed with no
  game running: a UI Gallery page draws any layout against its fixture; in a native debug build, layout files reload on
  save. `ui_layout shot <game dir> <screen>` writes the PNG for review beside the HTML mock. Fixtures live in `ui/views/<screen>.view.json` over the asset layers (Build); layouts in `ui/layouts/` (Art/UI) (`FB-319`).
- US7. HTML mocks in `tools/mock_ui/` stay the place to explore; a signed-off mock is turned into a layout file by
  Art/UI. Build does not port mocks once a screen is split.

## 4. Migration

- US8. New screens are born split. Existing screens move one at a time, in the order their new mocks are signed off
  (`AI-21`: battle, story, character, shop and board, save and load, then the rest). The battle board, region and world
  maps keep their stage code inside a `custom` region.
- US9. Rows already queued (`PR7-27`..`PR7-33`) are built as written; a screen they touch is migrated when its mock is
  signed off. No golden moves; screen baselines (`PR7-4` part 2) wait as already ruled.

## 5. Ownership and lanes

- Engine framework (US2, US3 loader, US4, US5, US6): tools lane, row `PR7-38`.
- View-models and fixtures: main Build, per screen, starting with the pilot row `PR7-39` (shop and battle report).
- Layout files `games/emberwake/assets/ui/layouts/*.json`: Art/UI (edited by hand like `theme.json`, checked by the gate).

## 6. Tests

Engine: layout parse and check fixtures (missing field, unknown component, bad anchor), default layout from a fixture,
one layout drawn against a fixture to a shot. Game: each split screen's view-model on a recorded save matches the
fields its old drawing used; GUI audit of the split screen shows no change before a new layout lands.
