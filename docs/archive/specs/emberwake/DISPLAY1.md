# DISPLAY1: display settings (window mode, resolution, frame rate)

Status: approved (user 2026-10-17 20:28, decision card "Approve all"): Q1 A all three window modes, Q2 A vsync on with no cap, Q3 A web fullscreen not remembered
Depends on: SETTINGS-UI (fixed levels and settings layout; DISPLAY1 reuses its level mechanism and layout checks)
Lane: logic (no asset part)
Touches: `scripts/game_state/settings_file.gd` (new keys), new `scripts/ui/menus/display_settings.gd`, `scripts/ui/menus/settings_model.gd`, `scripts/ui/menus/settings_view.gd`, `scripts/ui/menus/main_menu_view.gd` (apply at boot), `scripts/tests/menus_test_suite.gd` (or the SETTINGS-UI layout suite)

## Goal

User ask (2026-10-17): resolution and frame-rate options, plus windowed / fullscreen / borderless fullscreen. Desktop builds get a Display section on the Settings screen: window mode, window size (windowed only), vsync and a frame-rate cap. The browser build gets one Fullscreen toggle, because the browser owns the canvas size, the frame rate and the fullscreen permission. Settings stay in `user://settings.cfg` and are never part of a save (UI_MENUS_RULES).

## Schema contract (logic lane)

Schema-first: yes (the keys land before the view). New `SettingsFile.KEYS` rows. Each is a fixed list of allowed values, using the same level mechanism SETTINGS-UI adds for animation speed and text size. A value that is not in the list reads as the default, with the existing wrong-type warning.

| Key | Type | Allowed values | Default | Platform |
| --- | --- | --- | --- | --- |
| `display/window_mode` | String | `windowed`, `borderless`, `fullscreen` | `windowed` | desktop |
| `display/resolution` | String | `960x540`, `1024x576`, `1152x648`, `1280x720`, `1366x768`, `1600x900`, `1920x1080`, `2560x1440` (logical sizes: on a high-DPI screen the window is size x `screen_get_scale()` physical pixels, and the list filter compares that) | `1280x720` | desktop, windowed only |
| `display/vsync` | bool | `true`, `false` | `true` | desktop |
| `display/max_fps` | int | `30`, `60`, `120`, `144`, `0` (0 = unlimited) | `0` | desktop |

Labels: "Window mode" (Windowed / Borderless fullscreen / Fullscreen), "Window size", "Vertical sync", "Frame rate cap" (30 / 60 / 120 / 144 / Unlimited). The web build writes no display keys.

## Rules (logic lane)

1. Mapping in `DisplaySettings` (a static helper over a small backend interface, so headless tests can use a fake backend). `windowed` maps to `DisplayServer.WINDOW_MODE_WINDOWED`. `borderless` maps to `WINDOW_MODE_FULLSCREEN`, which in Godot 4 is a borderless window that covers the screen. `fullscreen` maps to `WINDOW_MODE_EXCLUSIVE_FULLSCREEN`. Vsync maps to `window_set_vsync_mode(VSYNC_ENABLED or VSYNC_DISABLED)`. The cap maps to `Engine.max_fps`.
2. Resolution is the window size in windowed mode only. Godot 4 does not change the monitor's video mode, so both fullscreen modes always run at the screen's native size. While a fullscreen mode is selected, the Window size row is shown disabled with the hint "Used in windowed mode". Its saved value is kept.
3. The window size list only offers sizes that fit `DisplayServer.screen_get_usable_rect()` of the current screen. `1280x720` is always offered. If the saved size does not fit (a smaller monitor), the game uses the largest size that fits and does not rewrite the file. After a resize, the window is centred on its current screen.
4. Stretch: `project.godot` keeps `canvas_items` / `expand` at a 1280x720 base. Every 16:9 size therefore shows the same 1280x720 logical layout, sharper as the size grows. No layout depends on the chosen size. Other aspect ratios only happen in fullscreen on non-16:9 screens, or when the user drags the window (`resizable=true` stays). With `expand`, those give a wider or taller logical canvas (16:10: 1280x800; 21:9: about 1680x720). Dragging the window never writes the file.
5. Vsync and the cap: with vsync on, the cap still applies when it is below the refresh rate (for example 30 on a 60 Hz screen). Unlimited with vsync on means "follow the refresh rate". Both apply at once on Apply, with no countdown.
6. Apply and Revert for window mode and size. On Apply, the non-display keys are written as today. If `display/window_mode` or `display/resolution` changed, the new values are applied live and a modal opens: "Keep these display settings? Reverting in 10 s" with Keep and Revert buttons. The countdown is visible and Revert has focus. Keep writes the two keys. Revert, Esc, or the timeout restores the previous mode and size, live and in the draft, and writes nothing. The two keys are written only after Keep, so a crash or a black screen during the countdown leaves the old values for the next launch.
7. Boot: `DisplaySettings.apply_saved()` runs once before the main menu is shown, next to the existing `ui/text_scale` read in `main_menu_view.gd`. It applies the mode, the size (rule 3), vsync and the cap. The scene runner and test entries skip it.
8. Web (`OS.has_feature("web")`): the four desktop rows are hidden. A single "Fullscreen" checkbox calls `window_set_mode` from inside its `pressed` handler, because browsers allow fullscreen only during a user gesture. The checkbox shows the real state when the screen opens and follows `get_tree().root.size_changed`, because the browser's Esc leaves fullscreen without telling the game. The browser always syncs the frame rate to the display, so no cap or vsync row is shown. The itch.io embed's own Fullscreen button (docs/workflow/web.md) stays.
9. Headless (`DisplayServer.get_name() == "headless"`): `DisplaySettings` uses the fake backend and never calls window functions, so the gate and the sims are unaffected.

Acceptance checks (headless model tests, fake backend):

- Each key round-trips through `settings.cfg`. A value outside the list reads as the default. The four keys are untouched by save, load and delete of any slot.
- Rule 1: each `window_mode` value produces the matching backend call. Vsync and each cap value produce the matching calls, and no countdown is started.
- Rule 3: with a fake 1600x900 usable screen, the list is every listed size up to `1600x900` (`960x540` ... `1600x900`). A saved `2560x1440` applies `1600x900` and leaves the file at `2560x1440`.
- Rule 6: changing the mode starts a 10 s countdown. Revert, Esc and the timeout (simulated time) each restore the old mode and leave the file unchanged. Keep writes both keys. Changing only the cap starts no countdown.
- Rule 8: with the web flag forced on, the four desktop rows are absent, the Fullscreen row is present, and toggling it writes no display key.
- Layout (extends the SETTINGS-UI checks): at each text-size level, check the Settings screen, the countdown modal and the main menu at logical sizes 1280x720, 1280x800 and 1680x720. Pass: no control clipped or outside the viewport, and the labels and controls line up. Testing each 16:9 size separately would add nothing, because rule 4 makes them identical.

## Assets (asset lane)

None.

## Out of scope

Monitor selection on multi-monitor setups (the game uses the current screen). Render scale, MSAA, HDR, UI scale beyond text size. Changing the stretch mode or the 1280x720 base. Any save-format change (`SaveGame.VERSION` unchanged). Remembering a dragged window size.

## Open questions

1. Window modes: which ones to offer?
   - A (rec): all three as asked (Windowed / Borderless fullscreen / Fullscreen). Matches the ask. Exclusive fullscreen differs only on Windows (slightly lower input latency, slower alt-tab); on Linux and macOS it looks like borderless.
   - B: two (Windowed / Fullscreen, where Fullscreen is borderless). Simpler and the same on every OS, but drops the exclusive option Windows players may expect.
2. Frame-rate defaults: vsync on with an Unlimited cap, or a fixed default cap?
   - A (rec): vsync on, cap Unlimited. No tearing, follows the monitor, and nothing to tune for most players.
   - B: vsync on, cap 60. Lower power and fan noise on 120+ Hz screens; animations look less smooth there.
   - C: vsync off, cap 60. Lowest input latency per frame, but tearing is possible.
3. Web fullscreen: remember it between visits?
   - A (rec): no, session only. Browsers refuse fullscreen without a click, so a saved "on" could not be restored at load anyway.
   - B: save it and re-enter fullscreen on the first click in the game. Restores the preference, but the first click unexpectedly changes the window.
