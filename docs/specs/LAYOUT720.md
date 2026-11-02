# LAYOUT720: reactive, relative layout as an engine feature, and the 1280 x 720 grid

Status: approved (user 2026-11-02: option B, tools first; only the main menu is finalized; wrapper for what the user plays or judges by eye, namely Battle, the Region map and all drawn on it, Character, Gallery, Frame tuner and Dojo, quadrant for the other unmoved screens; M0 kept; the touch rule per function; text, spacing, thin lines and frame parts stay fixed tokens from the layout).
The direction here is the planner's relay of the user's messages of 2026-11-02.
Depends on: nothing to start; section 8 gives the order against other work
Lane: logic
Touches: see the slice table in section 7

**The model (user 2026-11-02, relayed by the planner):** once a layout is selected, the position and size of each element are
calculated from it. Moved screens have no fixed positions, only relative positions and sizes; the actual size at render
time comes from the layout. Image rules, minimum size caps and space priority are later work (section 11.1).

This is not a one-shot migration. Phase 1 is the foundation and the layout tools. After that a screen moves only when it
is being built or redesigned anyway. There is no sweep and no end date for the old mapping.

## 1. Every impacted screen

### 1.1 Classes

| Class | Meaning | Phase 1 | Later |
| --- | --- | --- | --- |
| Finalized | Real screens the user has signed off | Exactly today's picture, no work; moved with a check only if it needs it | Moves with a check when a task changes it |
| Placeholder | Real screens likely to be redone | Old numbers in the top-left quadrant. It only has to draw and click: no shots, no parity, no fixes | Converted by the task that rebuilds it |
| Debug or tools | Screens the player never sees | Same quadrant, same bar | Never, unless a task wants it |

"Quadrant at 720p" below is my reading of each screen's density, for the choice in section 4. "Wrapper" means a candidate
for keeping today's full-window picture instead.

| Screen | File | Class | Evidence | `Rect::new` | Phase 1 change | Moves when | Quadrant at 720p |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Main menu | `menus.rs` `MainMenu` | Finalized | LOOK1-M1 done; `preferences/ui.md` main menu, user 2026-11-02 | 7 | Pilot move, slice M0 | M0 | Not applicable |
| New game | `menus.rs` `NewGameScreen` | Placeholder | LOOK1-M2 rebuilds it (LOOK1 6) | 3 | None | LOOK1-M2 | Stays as today until M2 |
| Load and save | `menus.rs` `Slots` | Placeholder | LOOK1-M2 rebuilds it | 4 | None | LOOK1-M2 | Stays as today until M2 |
| Settings | `menus.rs` `SettingsScreen` | Placeholder | LOOK1-M2 rebuilds it | 16 | None | LOOK1-M2 | Stays as today until M2 |
| Game menu | `hub.rs` `Hub` | Placeholder | LOOK1-M2 rebuilds it | 7 | None | LOOK1-M2 | Stays as today until M2 |
| Run over | `runover.rs` `RunOverScreen` | Placeholder | LOOK1-M2: `ornate` frame | 3 | None | LOOK1-M2 | Stays as today until M2 |
| Campaign notice | `runover.rs` `CampaignNotice` | Placeholder | M2 "ornate one-time cards" | 3 | None | LOOK1-M2 | Stays as today until M2 |
| Story stage, vignette cards | `story.rs`, `vignette.rs` | Placeholder | LOOK1 6: `ornate` frame in M2 | 3 + 11 | None | LOOK1-M2 | Stays as today until M2 |
| Ask dialog, Notice | `screen.rs` | Placeholder | M2: `Cx::ask` through the themed ask | 6 | None | LOOK1-M2 | Wrapper: pops over any screen |
| Journal | `hub.rs` `Journal` | Placeholder | UIR-4 approved, not built | 4 | `grid()` line | UIR-4 | Wrapper: overlay, text-heavy |
| Battle results | `hub.rs` `Results` | Placeholder | UIR-4 approved, not built | 5 | `grid()` line | UIR-4 | Wrapper: overlay, text-heavy |
| Region map, HUD and world | `region.rs`, `mapview.rs`, `props.rs`, `ground.rs`, `hud.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | MAPNAV1-U: "UI preference, not final: user UI review later"; REGIONVIEW1, REGIONGROUND1, LOOK1-L2, WALK2 still change it | 57 + 19 + 11 + 1 + 7 | `grid()` line, or none if finalized | Its UI review | Wrapper: main screen, game menu sits over it |
| Dialogue (inside region map and battle) | `talk.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | `preferences/ui.md`: refined in user playtest 2026-10-22 | 5 | Follows its host | With its host | Wrapper: reading is the point |
| Shop (inside region map) | `shop.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | SCREENS: PLAYUI1-P; DRAG1, UIR-7 open; no sign-off found | 16 | Follows its host | With its host or DRAG1 | Wrapper: three dense columns |
| Mission board (inside region map) | `board.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | STORYUI1-U2 "kit screens later, KIT2-UI" | 8 | Follows its host | KIT2-UI | Wrapper with its host |
| World map | `worldmap.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | SCREENS "done"; LOOK1 6: no layout change; no sign-off found | 14 | `grid()` line, or none if finalized | Its UI review | Usable; small pins and labels |
| Character (4 tabs) | `party.rs`, `portraits.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | `preferences/ui.md` Character popup: user playtest 2026-10-20, SKILLUI1-U; DRAG1 open | 55 + 12 | None if finalized, else `grid()` line | DRAG1 or its next redesign | Wrapper: dense rows, drag and drop |
| Battle, HUD and board | `battle/{draw,board,marks,tips,input}.rs`, `stage.rs` | Placeholder (user 2026-11-02: only the main menu is finalized) | `preferences/ui.md` COMBAT-UI3; VISCLAR1 "decide with the user" open; LOOK1 6: theme only | 65 + 23 + 11 + 3 + 2 + 45 | `grid()` line, or none if finalized | VISCLAR1 or its review | Wrapper: small chips and strip labels |
| Scene Runner | `runner.rs` `SceneRunner` | Debug or tools | SCREENS "Dev tools" | 18 | `grid()` line | Never | Usable |
| Debug Fight | `runner.rs` `DebugFight` | Debug or tools | A debug screen by name | 4 | `grid()` line | Never | Usable |
| Content and World Forge | `forge.rs` | Debug or tools | SCREENS "Dev tools" | 6 | `grid()` line | Never | Hard: long fields in small text |
| Gear and Skill Dojo | `dojo.rs` | Debug or tools; **flag** | SCREENS "Dev tools"; dense windows, 11-unit buttons | 63 | `grid()` line | Never | Hard: wrapper candidate |
| UI Gallery, previews | `gallery.rs`, `specimens.rs` | Debug or tools; **flag** | Named debug, but `preferences/ui.md`: "where every UI element is judged" | 31 + 77 | `grid()` line | Never | Misleading: judged at half size |
| Frame tuner | `frame_tuner.rs` | Debug or tools; **flag** | Decision "Efficiency pass": the user tunes frames with it | 13 | With the Gallery | Never | Misleading: tuned at half size |
| Recordings, note view | `recordings.rs` | Debug or tools; **flag** | Dev-only, but the user reads playtest notes here | 20 | `grid()` line | Never | Usable at 1080p and up |
| Playtest note box | `notes.rs` | Debug or tools; **flag** | F8 over the real game while the user types | 4 | `grid()` line | Never | Small field |

Counts by grep: game 539 non-test occurrences (a few in test modules: `menus.rs` 7, `hub.rs` 1); engine screens 131 of the
278 in `aegis_ui`. The other 147 there are shared widgets, panes and shell code (section 6.4). `kits/rpg` draws nothing.

### 1.2 What the user confirms

1. The "User to confirm" and "flag" rows. The class decides only one thing now: a finalized screen must not go to the
   quadrant. Either answer costs the same.
2. Per screen, quadrant or wrapper (section 4).
3. Dialogue, Shop and Mission board are drawn inside Region map, so they follow it.

### 1.3 Cost of phase 1

One run is one builder plus review (13-30 and 4-17 minutes, decision "Efficiency pass"). Rough.

| Part | Runs |
| --- | --- |
| Engine core and layout tools: E1-E5 | 5 |
| Quadrant lines plus a smoke test: Q | 0.25 |
| The game declares its canvas and window list: G0 | 0.25 |
| Main menu pilot with a check: M0 (optional, section 7) | 0.5 |
| **Phase 1 total** | **6** |

Not in the total: each later move, part of that screen's own task. LOOK1-M2 is the first; expect about 20-30 % on top of
its size for the first use of the tools (estimate).

## 2. Goal

Screens and engine pieces declare structure: position and size relative to a parent or the screen. Once a layout is
selected, the engine calculates every element from it for the canvas it has, so a future layout change needs few or no
edits. This is a game-agnostic `aegis_ui` capability
(decision "Port goal and the end of Godot"): engine widgets and panes adopt it first and the game gets them for free.
`Rect` and every existing call stay valid for screens not moved. The unit of new screens is 1 / 1280 of the fixed canvas,
so decision "Layout grid moves to 1280 x 720" still happens, without a flag day.

The long-term aim is a web build on any screen with suitable controls. This spec does not build touch, portrait layouts
or the web port. It names what must not be blocked (section 11).

## 3. The choice for the user

| | A1: one sweep, double every number | A2: double, screen by screen | B: layout tools, then moves inside each screen's own task (recommended) | C: B, then convert every screen now |
| --- | --- | --- | --- | --- |
| Size (runs, rough) | 10-12 on one branch | 14 | 6 for phase 1; later moves in their tasks | 5 + 20-25 |
| Build and look safe at each slice | No: nothing builds until all of it lands | Yes | Yes | Yes |
| Other UI work during it | Frozen | Blocked per file | Free, except files in the current slice | Blocked per file for a long time |
| Main risk | One huge diff; `/ 2` and `+ 1` need judgement, so regex is not enough | Parity work on screens to be redone; numbers still fixed | Mixed screens for a long time; widgets carry a unit factor | Regressions in the largest files (`region.rs` 4,423 lines, `party.rs` 3,027) for no visible gain |
| Finer tuning | All, at the end | Moved screens | Moved screens, from M2 | All, at the end |
| Art lines up with layout | All, at the end | Moved screens | Moved screens | All, at the end |
| 16:10, ultrawide without bars | No | No | Moved screens only | Yes, at the end |
| UI scale | No | No | Mechanism; useful once a game's real screens have moved | Yes, at the end |
| UI keeps its size at higher resolutions | No | No | Mechanism; it contradicts "sharper rather than roomier", so it needs its own decision | Same |
| Small screens, any browser window | No | No | Mechanism and audits; each screen supplies its compact arrangement | Same |

Why B: most current screens are not final, so parity work on them is wasted. And the numbers are not mostly positions: in
`emberwake_gui` about 120 of 539 `Rect::new` have literal x and y, and the rest compute from variables and consts. A sweep
would edit all of it and leave every screen as rigid as now.

What B does not give: board and map geometry stay cell math; an unmoved screen gains nothing; a compact arrangement is
design work per screen (the battle timeline on a phone is a design problem, not a layout problem).

### What happens to the 1280 x 720 decision

It stands, with its cost moved. The grid stays as the unit everything snaps to: a screen states fractions and anchors, the
engine resolves them to whole grid units for the current viewport, then scales to pixels (section 6.3). Its size is 1280 x
720, set in one place. It arrives in E1 for new screens only. Some screens never move, so the old mapping stays for good:
about 30 lines (`View.g`, `Grid`, the old consts), with no removal task. The fixed canvas stays for this game. 720p stays
the lowest size this game offers (section 10).

## 4. Unmoved screens: quadrant or wrapper

Both come from one field in `View`, and a screen picks with one line. Neither edits a screen's numbers.

| | Quadrant (`Grid::Quadrant`) | Wrapper (`Grid::Old`) |
| --- | --- | --- |
| Mapping | `s` halved, `g = 1`: old numbers on the 1280 canvas | `s` as today, `g = 1`: exactly today's `View` |
| Picture | Everything at half size, top left; the other three quarters stay `theme.bg` | Exactly today, filling the window |
| Screen cost | 1 line | 0 lines; it is the default |
| Hit tests | Correct with no work: `to_logical` uses the screen's view, rects stay old numbers, a click outside hits nothing | Unchanged |
| Agent link | Ids unchanged; `ui_tree` reports `canvas: [640, 360]` and the grid | Same |
| Blank area | Free: the shell clears the canvas each frame and nothing draws there | None |

So the quadrant's real cost is close to zero, and the wrapper's is lower. The default in code must be the wrapper, or the
first slice would shrink every screen, finalized ones included. Slice Q then sets the quadrant on approved screens. It is
reversible per screen by deleting the line.

**Shared widgets.** They read the unit from the view (`m.u(n)` = `n * g`), and both modes have `g = 1`. A button's rect,
text, padding and frame all come out in old numbers and map together: uniformly half size, nothing mismatched. Physical
floors do not halve (`panel`'s 1 px border, the 6 px text floor at `painter.rs:478`), so lines look relatively heavier.

**Text in physical px** (`size * text scale * s`, rounded, floor 6). Readability is reasoned, not measured.

| Window | Wrapper: body, small, head | Quadrant: body, small, head | Quadrant readable? |
| --- | --- | --- | --- |
| 1280 x 720 | 14, 12, 24 | 7, 6, 12 | Poor: a glance, not reading. Extra large text gives 9, 8 |
| 1920 x 1080 | 21, 18, 36 | 11, 9, 18 | Fine for tools; tight for play |
| 2560 x 1440 | 28, 24, 48 | 14, 12, 24 | As today's 720p: the quarter is pixel-identical to today's 1280 x 720 picture |

Text also fits a little worse, because `px` rounds: body at 1080p is 11 for a true 10.5, and Small text at 720p hits the
floor. Expect more `...` on quadrant screens. Under the "draw and click" bar that is accepted.

**Frames.** Sidecars hold art px and `scale`, so `k = s / scale` halves too. No sidecar changes.

| Frame | Today at 720p | Quadrant at 720p |
| --- | --- | --- |
| `button`, `card`, `inset` and others (slice 4, `scale` 2) | 4 px corner | 2 px corner, every other art px sampled, so a 1-px detail can vanish |
| `panel` (slice 8, `scale` 2) | 8 px corner | 4 px corner, sampled |
| `ornate` (slice 200, `scale` 8, averaged) | 50 px corner | 25 px corner, averaged, softer |

**Targets.** A 16-unit button is 16 px tall at 720p; window buttons 11 px, grips 8, scroll bar 3. Workable with a mouse,
fiddly in the Dojo.

**Overlays.** A quadrant popup over a full-window screen dims only its quarter. It works, since only the top screen takes
clicks, but the rest stays bright. The game menu over a quadrant map looks the same way in reverse.

**Before the user picks:** a mock board of 4-5 screens at half size at 720p and 1080p, from the shots in
`games/emberwake/docs/screens/engine/`, so the choice rests on real pixels (decision "Efficiency pass").

## 5. Tests and proof, phase 1

| Check | What it proves | Where |
| --- | --- | --- |
| T1, unmoved path is bit-identical | `default_hashes.json` and `pane_hashes.json` pass with no edit (`aegis_ui/tests/theme.rs:99`, `theme_panes.rs:263`) | Every slice |
| T2, the mapping | `native.rect(2r) == old.rect(r)` exactly at all `WINDOW_SIZES`. Halving `s` and doubling `x` in binary floats give the same product; the test pins it | E1 |
| T3, widget equivalence | A table of widget and pane calls drawn native with doubled rects against old, at 1280 x 720 and 1920 x 1080, through `shot_diff` | E3, E4 |
| T4, careful move | Before and after shots at the same two sizes through `shot_diff`; `ui_tree` rects: after = 2 x before, each edge within 1 unit | Finalized screens only (M0) |
| T5, smoke | Each quadrant screen opens through the shell, a click by physical position reaches the right widget | Q |

`shot_diff` (new, pure, in `aegis_ui`): a pixel matches if some pixel within radius `ceil(s)` in the other image is within 3
per channel. Any unmatched pixel fails, and the report gives the bounding box. Before shots go to
`C:/dev/localgen/review/layout720/<screen>/`, not into the repo. For animated areas the builder freezes the clock or masks
them; I did not verify a frozen clock on every screen.

Non-finalized screens get no shots, no parity check and no layout fixes. A rebuilt screen goes through the mock rule.

**No golden changes and none is retired.** The logic goldens (`timeline_golden`, `rules_golden`, `sim_golden`,
`checks_golden`, `journey`) name no `Rect`, logical unit or `ui_tree`, by grep. The tracked `parity/` folder has no files in
this checkout. Ground fixture PNGs pin art px. The two hash files draw Gallery pages, which stay old, so they stay valid.
Never re-record them; a slice that needs them changed reports a finding to the planner.

**Tests on screens left behind keep passing in old numbers.** `tests/{ground,fx,fx_loops,clips,autotile,pips,portrait_fx,
kit_widgets,panes,frame_tuner,theme_panes}.rs`, `battle/tests/*`, `menus.rs` tests and `emberwake_gui/tests/theme.rs` use
`View::fit` or set `g.view` directly, so neither `grid()` nor Q touches them. They change only when their own screen moves,
in that screen's task, position pins becoming relations and named in its commit. `text_audit` gets the top screen's
canvas instead of `W`, `H` (`shell.rs:821`), which is the same value for old screens.

## 6. The engine design

Immediate mode allows all of this with no retained widget tree. Each frame a screen cuts rects top-down from the canvas,
then draws. A parent sized by its content measures first, as `panes::ask_layout` already does (`panes.rs:651`).

### 6.1 New crate `engine/crates/aegis_layout`

`#![no_std]` with `alloc`, no dependencies, whole numbers only. `no_std` proves at compile time that it uses no file,
thread or socket, and needs no wasm toolchain in the gate. `Rect` moves here with its methods; `aegis_ui::canvas`
re-exports it, so `Rect::new`, `inset`, `contains` and all callers stay as they are.

```rust
pub struct Insets { pub l: i32, pub t: i32, pub r: i32, pub b: i32 }
pub enum Anchor { TopLeft, Top, TopRight, Left, Center, Right, BottomLeft, Bottom, BottomRight }
pub struct Part { /* private: a share, a fill weight or a token's units; section 11.1 attaches here */ }
pub fn frac(per_mille: u16) -> Part;
pub fn fill(weight: u16) -> Part;
impl From<i32> for Part { /* a token's value, in units */ }
pub enum SizeClass { Compact, Medium, Expanded }
pub struct Need { pub min_w: i32, pub min_h: i32 }

impl Rect {
    pub fn pad(self, by: Insets) -> Rect;
    pub fn place(self, at: Anchor, w: impl Into<Part>, h: impl Into<Part>) -> Rect;
    pub fn at(self, fx: u16, fy: u16, at: Anchor, w: impl Into<Part>, h: impl Into<Part>) -> Rect;    // per mille of the parent
    pub fn offset(self, dx: i32, dy: i32) -> Rect;
    pub fn cut_top(&mut self, h: impl Into<Part>) -> Rect;    // also cut_bottom, cut_left, cut_right
    pub fn split_x(self, parts: &[Part], gap: i32) -> Vec<Rect>;    // also split_y
    pub fn grid(self, cols: i32, rows: i32, gap: i32) -> Vec<Rect>;
}
pub struct Stack { /* area, gap, used */ }
impl Stack {
    pub fn new(area: Rect, gap: i32) -> Self;
    pub fn next(&mut self, h: impl Into<Part>) -> Rect;
    pub fn overflow(&self) -> i32;    // units past the area, 0 when it fits
}
pub fn pick<T: Copy>(area: Rect, choices: &[(Need, T)], last: T) -> T;
```

Rules, each with a test in the crate:

- A size that cannot fit becomes 0. Nothing panics and nothing goes negative.
- Token sizes are taken first, then shares of what is left after gaps, then fills by weight, all in one function,
  `resolve(parts, room)`, so later steps have one place to go.
- `split`, `grid` and `Stack` resolve shared edges, not sizes (section 6.3), so the parts and gaps always add up to the
  area and neighbours never gain a gap. A test runs every part count from 1 to 12 over every width from 1 to 2,000.
- `place` and `at` clamp to the parent, and a centred odd remainder rounds toward the top left.
- `pick` returns the first choice whose `Need` fits `area`, else `last`. It works on any rect, so a pane can rearrange
  inside itself.

### 6.2 What is calculated, and what the layout supplies

On a moved screen every position and every panel, pane or element size is calculated from the selected layout. Selecting
means `pick` or a size class, or the only layout a screen has. Review checks moved code for `Rect::new` with numbers.
Text, spacing and thin lines are not stretched by a fraction: the layout supplies them as tokens (question 6 in section 18).

| Kind of value | How it is written | Examples |
| --- | --- | --- |
| Position | Anchor, cut, split, stack; `at` for a free point. Never an absolute canvas number | A dialog, a logo, a panel in its screen; a close button in its frame |
| Panel, pane and element size | `frac` or `fill`, or a token for a control's height | Dialog width, side rail, columns, list height |
| Inner layout of a widget | Relative to its own rect | Icon centred in a button, slider knob by value |
| Spacing | Tokens from `Metrics`, not fractions | Margin, gutter, padding; bands of a frame |
| Thin things | Tokens; `Metrics.hair` at small scales | Lines, borders, focus ring, scroll bar, grips |
| Text | Token times the Text size setting; boxes grow from measured text | Body, head, line step |
| Small icons and controls | Tokens | Glyph icons, checkbox, stepper, row height |
| Frame parts | Sidecar corner and band sizes, not fractions; stretch or tile only along the edge | `corner_scale`, `straddle` (section 9) |
| Images | Drawn in the rect the layout gives, through `Ui::image_in`; the per-image rule is later (section 11.1) | Portraits, thumbnails, title art |
| Board and map | Cell math, its own world scale, not layout | Cells, sprites on cells, fx |

A per mille is 1.28 units across the 1280 canvas, so a share lands within 1 unit of any target, inside T4's tolerance.
Exact alignment to art uses an anchor plus a token. Under `Fixed` nothing here moves, so this game loses nothing by having
no caps yet: on another canvas a share grows and shrinks freely until the caps of section 11.1 land.

Rollout:

| When | What switches to relative |
| --- | --- |
| Phase 1, E3 | `ui.rs`, `focus.rs`: inner layout of every widget; literals become anchors plus `m.u(n)` or tokens |
| Phase 1, E4 | `panes.rs`: `ask_layout`, `tab_rail`, `slot_card`, `menu_column`, `backdrop_pane` |
| Phase 1, E4, units only | `pips.rs`, `timeline.rs`, `window.rs`: `m.u(n)` so they draw right on all grids; positions stay until next touched |
| Phase 1, new code | Layout specimen and all of `aegis_layout` |
| Stays old for now | `specimens.rs`, `frame_tuner.rs`, `vignette.rs`, `recordings.rs`, `notes.rs`, `screen.rs` (M2 takes Ask) |
| Rule from now on | Any engine layout code a task touches moves to relative in that task. My reading: at function level, not the whole file, and a change that touches no layout moves nothing. New engine code is relative from its first line |

The old path must stay bit-identical, so T1 limits E3 and E4: a pane's existing inner positions become anchors and tokens
that resolve to the same numbers at `g = 1`. Where a share cannot reproduce today's value exactly on both grids, the piece
keeps `m.u(n)` and a share arrives with its first native caller. Existing minimums inside panes (`ASK_MIN_W`) stay as they are. Callers keep passing a `Rect`: `tab_rail(ui, id, r, ...)`
is unchanged, and on a moved screen `r` itself comes from a share.

### 6.3 From fractions to grid to pixels, and rounding

Relative positions still lie on the grid. Each frame runs three steps, and a changed viewport re-runs them with no screen
edit:

| Step | Input | Output | Rule |
| --- | --- | --- | --- |
| 1. Viewport | Window px, `CanvasPolicy` | Canvas in whole grid units, `s` | Section 6.4. Under `Fixed` the canvas is always 1280 x 720; only `s` changes |
| 2. Resolve | Anchors, shares, tokens, the canvas | Rect edges in whole grid units | Round down, edges not sizes (below) |
| 3. Scale | Grid edges, `s` | Physical px | `View::rect` rounds each edge on its own (`painter.rs:35-39`), so one grid edge is one pixel edge |

**The rounding rule: resolve shared edges, not sizes.** A split works out each boundary from its running total, rounded down
once, and each cell's width is the difference between its two boundaries. For `Fill` parts over `room`, boundary `i` is
`room * sum(weights up to i) / sum(all weights)`, rounded down; the last is exactly `room`. Fixed gaps are added after. Then:

- Two neighbours read the same number for their shared edge, so there is no gap or overlap in units.
- Step 3 maps one number to one pixel, so there is none in pixels either, at any scale.
- The total always equals the room, so nothing is lost at the far end.
- Cells can differ by 1 unit. That is the whole cost, and it never accumulates.

`cut_*` share their edge by construction. `Stack` and `grid` use running totals the same way. A share rounds down. Parts with
no fill whose shares total 1,000 end exactly on the area's edge. `place` and `at` round the start down
and add the size, so a lone element has one rounding only. Never add a rounded size to a rounded position across
neighbours: a screen that needs a shared edge cuts or splits.

**Worked example.** A screen: margin 16, a rail of `frac(186)`, gap 12, then a list and a detail pane split `fill(2)`,
`fill(1)` with gap 12.

| | Canvas 1280 x 720 (`Fixed`; every 16:9 window) | Canvas 1720 x 720 (`Fit`; a 3440 x 1440 window) |
| --- | --- | --- |
| Area after margin | x 16 to 1264, 1248 wide | x 16 to 1704, 1688 wide |
| Rail | 186 per mille of 1248 is 232.1, so 232; x 16 to 248 | 313.97, so 313; x 16 to 329 |
| Room for the panes | 260 to 1264, less the gap: 992 | 341 to 1704, less the gap: 1351 |
| Boundary at 2 of 3 | 661.3, so 661 | 900.7, so 900 |
| List | x 260 to 921 (661 wide) | x 341 to 1241 (900 wide) |
| Detail | x 933 to 1264 (331 wide) | x 1253 to 1704 (451 wide) |
| Ends on the area's edge | Yes | Yes |

Rounding sizes instead would give 661 + 330 and 900 + 450, ending at 1263 and 1703: the off-by-one gap this rule prevents.
The rail grows with the canvas; a cap on it is later work. Step 3 at 1920 x 1080 (`s` 1.5): the rail's right edge 248 is 372
px, the list's edges 260 and 921 are 390 and 1382 px, and every window of the same canvas gives the same units.

**Grid size, in one place.** E1 adds `GRID = (1280, 720)` beside `LW`, `LH` in `painter.rs`, and `View.g` becomes
`GRID.0 / LW`. E5 lets a game's `canvas` key set it. Changing it re-resolves anchors and shares with no screen edit. Theme
`metrics` are in units, so they change with it, in one file. Numbers written as `Units` in code do not change, which is why
those go through tokens or `m.u(n)`. The width must be a whole multiple of 640 for the old mapping to stay whole.

| Question | Answer |
| --- | --- |
| Does the grid's size still matter? | Less. `Rect` is whole units, so the unit is the rounding step: on 640 a split can be up to 4 px off at 1440p, on 1280 up to 2. Fixed tokens also stay in units, and art lines up only at 1 unit = 1 px at 720p. It matters enough to change, not enough to sweep |
| Does the unit become a float? | No. `Rect` stays i32 (`canvas.rs`, 830 call sites). Shares are per mille in whole numbers, so layout is deterministic and portable. The painter is already f32 where it needs it |
| Hit tests? | Unchanged: `Widget.rect` records the resolved rect, `ui::hit` reads it, `to_logical` floors. The agent link works by widget id |
| New tests? | Pin relations, not numbers: "Yes is left of No, inside the frame, one gutter apart", over several canvases |

Scale per offered size on the new grid: 1280 x 720 is 1.0; 1366 x 768 about 1.067; 1600 x 900 is 1.25; 1920 x 1080 is 1.5;
2560 x 1440 is 2; 3840 x 2160 is 3.

- **Even values** land exactly at 1080p. **Odd values** are allowed: at 1.5 they fall on a half pixel and round up, so a
  3-unit bar is 4 or 5 px. Repeated thin elements (pips, rulers) use even steps, or uneven gaps show at 1080p.
- **A division after doubling** can differ by 1 unit, closer to true centre: old `(640 - 101) / 2` is 269 (538 new); new
  `(1280 - 202) / 2` is 539. Accept it; T4's tolerance covers it.
- **Hit edges**: `to_logical` floors while drawing rounds, so an edge can differ by under 1 px at a non-whole scale, as
  it does today at 1366 x 768.
- **Below scale 1** (`Fit` in a small window) a 1-unit fill can round to 0 px: use `Metrics.hair` = `max(1, ceil(1 / s))`.
  The vector `panel` border is already one physical pixel (`painter.rs:545`).

Fine positions pay in: text centred in buttons and rows, odd-sized icon art, frame corners and banners against a panel,
portrait marks, and small art at 1 unit = 1 art px (the 48 px sprite frame is 24 old units, `mapview.rs:15`). They do not
pay in cell math or text wrapping.

### 6.4 `aegis_ui` pieces, and what stays in the game

| Piece | Contract |
| --- | --- |
| `View` | Gains `g: i32`, units per old unit: 1 unmoved, 2 native. `View::fit` stays, so its 40-plus callers do not change. `View::for_grid(self, grid)` gives a screen's view |
| `Screen::grid()` | New trait method: `Grid::Old` (default), `Grid::Quadrant`, `Grid::Native` |
| `Ui::screen()` | The canvas as a rect: `(0, 0, 640, 360)` for `Old` and `Quadrant`; `(0, 0, 1280, 720)` for `Native` under the fixed canvas |
| `Ui::m()` | `Metrics`: text sizes, gaps, `u(n)` = `n * g`, from the theme. The one read path for text and control sizes |
| `Ui::image_in(img, r)` | Moved screens draw images through this. It draws as `Painter::image` does today |
| `Ui::env()` | `Env { canvas, dpr, ui_scale, pointer, width_class, height_class }`. `pointer` is `Mouse` or `Touch`; no shell produces `Touch` yet |
| `Ui::frame_body(role, r)` | Draws the frame, returns the content rect (section 9) |
| `Ui::corner_size(f32)` | One corner size for this frame of drawing (section 9) |
| Layout specimen | New engine module, one new Gallery entry, not a tab on `Specimens` (that would change pinned hashes). Shows anchors, shares, fills and `pick`; the audit's subject |

`CanvasPolicy` is how a game relates canvas to window (E5; this game uses only `Fixed`):

| Policy | px per unit | Canvas in units | Example |
| --- | --- | --- | --- |
| `Fixed { w, h }` | `min(pw / w, ph / h)` | always `w` x `h`, with bars | Today's rule, for 1280 x 720 |
| `Fit { w, h }` | `min(pw / w, ph / h) * ui_scale` | `pw / s` x `ph / s` | 1280 x 800 gives 1280 x 800; 3440 x 1440 gives 1720 x 720 |
| `Dp` | `dpr * ui_scale` | `pw / s` x `ph / s` | A browser or phone: 1 unit = 1 CSS px, any aspect |

Size classes are decided on the canvas in units, so a larger UI scale moves a screen to a smaller class by itself.
Defaults (theme `metrics.breakpoints`; assumed, tune with the first real target): width Compact under 800, Medium under
1200, else Expanded; height Compact under 480, Medium under 700, else Expanded. A narrow tall canvas is width Compact, and
its `pick` chooses a stack, so portrait needs no special case. The engine provides the mechanism; the screen supplies the
choices.

What stays in `emberwake_gui`: screen structure, the choices at each size class, and cell math. No generic widget code.

Example, the settings popup. Today (`menus.rs:794`): `let r = Rect::new(95, 25, 450, 310);`. On the new grid:

```rust
let (m, area) = (ui.m(), ui.screen());
let r = area.place(Anchor::Center, frac(703), frac(861));
let mut body = ui.frame_body("panel", r);
let tabs_row = body.cut_top(m.tab_h);
let page = body.pad(Insets::all(m.padding));
```

## 7. Slices, phase 1

| Id | What | Layer | Lane | Depends on | Touches | Size |
| --- | --- | --- | --- | --- | --- | --- |
| LAYOUT720-E1 | `View.g`, `Grid`, shell per-screen view and mouse, painter fixes, nine scale, previews, `ui_tree` grid | engine | logic | - | `aegis_ui/src/{painter,shell,screen,preview}.rs`, one new test file | 4 files, 1 run |
| LAYOUT720-E2 | `aegis_layout` crate, `Rect` moved, `shot_diff`, the authoring note | engine | logic | - | new crate, root `Cargo.toml`, `aegis_ui/src/canvas.rs`, new `shot_diff.rs`, `engine/docs/authoring/` | 1 crate, 1 run |
| LAYOUT720-E3 | `Metrics`, `Env`, theme `metrics`, widgets relative and correct on all grids | engine | logic | E1, E2 | `aegis_ui/src/{ui,focus,theme}.rs` | 3 files, about 90 numbers, 1 run |
| LAYOUT720-E4 | Panes relative; pips, timeline, desk in units | engine | logic | E3 | `aegis_ui/src/{panes,pips,timeline,window}.rs` | 4 files, 1 run |
| LAYOUT720-E5 | `CanvasPolicy`, `layout_audit`, size profiles, layout specimen, game-set window sizes | engine | logic | E3 | `aegis_ui/src/{painter,shell,screen}.rs`, new `layout_audit.rs`, new specimen module | 3 files + 2 new, 1 run |
| LAYOUT720-Q | One `grid()` line per approved screen, T5 | game + engine | logic | E1, table approved | those screens' files, one line each | 0.25 run |
| LAYOUT720-G0 | The game declares `Fixed` and window sizes from 1280 x 720 up | game | logic | E5 | `game.json`, `emberwake_gui/src/{main,screens}.rs` | 0.25 run |
| LAYOUT720-M0 | Main menu pilot with T4: the tools' first real user | game | logic | E4 | `emberwake_gui/src/menus.rs` (`MainMenu` only) | 0.5 run |

Parallel: E1 and E2 together (no shared files). After E3, E4 and E5 together (no shared files). Q right after E1 in any lane
free of the listed files. E1 and E5 touch `shell.rs` and `painter.rs`, so no other task on those runs with them. M0 and M2
share `menus.rs`, so M0 goes first.

M0 is optional. The main menu looks right on the wrapper, so its picture does not need it. It is here as a low-risk pilot
before M2 (7 rects). Dropping it saves 0.5 run and makes M2 the first user.

Later, inside other tasks and not costed here: `Ui::old`, a sub-scope for an unmoved world layer inside a new screen (rects
doubled on `push`, mouse halved inside), built by the first task that moves a HUD and keeps its board old (about 0.5 run).

### 7.1 E1 detail

Unmoved screens under `Grid::Old` get exactly today's `View`, so they are identical by construction.

- **Shell** (`shell.rs:322`): before each `draw`, sets `gfx.view` from `screen.grid()`. After the stack it leaves the
  top screen's view in place for the focus ring and open lists. It keeps the mouse in physical px and derives
  `self.mouse` for the top screen (`shell.rs:1261`). Screens under an overlay get `(-1, -1)` already, so only the top
  needs it. It recalculates when the stack changes.
- **Also in units of the top screen**: `pointer_req` (`shell.rs:1111`), `SCROLL_STEP` (`ui.rs:109`, `shell.rs:420`), the
  focus ring's literals, ghost and option lists.
- **Agent link**: `ui_tree` reports `canvas` and `grid` for the top screen. Rects and `drag` `[x, y]` are in its units.
- **Painter primitives that treat `s` as a thickness or a step** use `s * g`: `line` (`painter.rs:837`), `ellipse_ring`
  (`:866`), `circle` (`:886`), `stage.rs:849-850`. Without this 1080p lines thin from 2 px to 1 on new screens.
- **Nine** (`painter.rs:699`): `k = v.s * g / scale`. `pad`, the `skin` slice (`:781`) and `nine_content_rect` results
  are multiplied by `g`. Sidecars never change: `scale: 2` keeps meaning 2 art px per old unit, 1 per new unit.
- **Previews** (`preview.rs:25,44`): `screen_in` takes the shown screen's grid.
- **Desk**: `desks.<id>` in settings (`shell.rs:783`) holds old units. Desk screens are tools and stay old.

Right on every grid already: `icon_px` and callers of the form `size * v.s` (`ui.rs:597`, `region.rs:1617`). Data in old
units (`tuning.json:6,8,18,31-33`, sidecars, theme `layout`, `shake_px`, `lean_px`, `bob_px`, `shiver_px` at
`rpgkit_data/src/lib.rs:1457-1476`) keeps its meaning for good and is read through `u`. No content edit.

## 8. Order against other work

| Task | Order | Why |
| --- | --- | --- |
| REGIONVIEW1, LOOK1-G2, SPRITEFIT1 | Land first, as planned | No conflict with E2; E1 follows them to avoid a rebase on `painter.rs` and `art.rs` callers |
| ARTSCALE1 | Any time after SPRITEFIT1 | Its fixes are on physical px, grid-neutral |
| LOOK1-M2 | After E4 and M0, native and relative from the start | Otherwise the same files get rebuilt twice. E1-E4 delay it by up to 4 runs. Its brief (`LOOK1_TASKS.md`) stays the contract; layouts go through the mock rule |
| REGIONGROUND1-W, REGIONGROUND1-S, LOOK1-L2 | No wait; author in old units | Region map stays old. "24 logical" keeps its meaning (48 new units) |
| Other queued `-U` rows | Old units unless the task rebuilds its screen | The screen's `grid()` says which |

## 9. Frames, banners and one corner size

Today's nine-slice code already follows a resized panel. Corners keep one size on any panel (`corner_scale`,
`painter.rs:368`), and edges stretch or tile. So a reactive panel needs only:

- **Body**: `frame_body` returns `nine_content_rect` (`painter.rs:376`) for the current view, then the role's padding.
  Content never reads a slice number.
- **One corner size per screen** (decision "Compact frame for side-by-side panes": set by the smallest pane): cut every
  pane first, call `Ui::corner_size` with the size for the smallest, then draw. The value joins the `reduced_nine` cache
  key through the cloned `Nine` (`painter.rs:186`). A test: no pane's `corner_scale` falls below the chosen size.
- **Too small**: `corner_scale` shrinks a corner so it never overlaps, which breaks the one-size rule. `layout_audit`
  reports it (`corner_shrunk`), and the fix is the screen's compact choice.
- **Banners**: `r.place(Anchor::Top, w, h).offset(0, -h / 2)`, where `w` is the smaller of the art's width at the frame's
  `k` and a share of `r.w`. Ends stay at art size; the middle stretches or tiles as the sidecar says.
- **Backdrops**: `cover_crop` (`panes.rs:78`) already handles any aspect and stays as it is.

## 10. Small screens and any window

Size alone is not enough. A 7-inch handheld at 1280 x 800 has the pixels but not the physical size, so the same layout
gives unreadable text and tiny targets. Phase 1 reports this; the minimums that fix it are later work (section 11.1).

**The engine supports any canvas; a game decides what it offers.** Emberwake declares `Fixed` at 1280 x 720 and a window
list starting there, so its new screens always see Expanded, Expanded and need no compact code. Today's `WINDOW_SIZES`
(`screen.rs:110`) still lists 960 x 540, 1024 x 576 and 1152 x 648, against the decision. G0 fixes this; a saved smaller size
opens at the nearest listed one. LOOK1-M2's settings rail reads the list.

| Concern | Rule |
| --- | --- |
| `dpr` | Window px per CSS px, from winit `scale_factor` or the browser's `devicePixelRatio`. Not a true measurement: a handheld can report 1, so a game targeting one also sets a default `ui_scale` (about 1.5, assumed) |
| Rearranging | `pick` or the classes choose side-by-side, tabs or a stack. The screen writes each arrangement; the engine never rearranges by itself |

Overflow: every stack or list declares one; the default is `Report`.

| Policy | Mechanism | Use for |
| --- | --- | --- |
| Scroll | `scroll_begin` (`ui.rs:302`), content height from `Stack` | Lists, long text |
| Collapse | `section` (`ui.rs:630`), or tabs through `pick` | Secondary panels |
| Truncate | `text_fit` and `mark_cut`, full text on a tip and on focus | Names, one-line labels |
| Report | `Stack::overflow()` goes into the audit; drawing is clipped to the area | Anything not designed to overflow |

`layout_audit::issues` sits beside `text_audit`, run by the same shell command on the same frame. Its results go in a
separate `layout` key, so the exact `counts` checks of the text audit stay valid (`shell.rs:1968`, `text_audit.rs:85`).
Kinds: `overflow`, `off_canvas`, `zero_size`, `corner_shrunk`, `hover_only`. `zero_size` and `overflow` show where caps and
priority will be needed.

| Profile | Window px | dpr | ui_scale | Pointer | Policy |
| --- | --- | --- | --- | --- | --- |
| desktop | 1280 x 720 | 1 | 1 | Mouse | Fixed |
| wide | 1920 x 1200 | 1 | 1 | Mouse | Fit |
| ultrawide | 3440 x 1440 | 1 | 1 | Mouse | Fit |
| handheld | 1280 x 800 | 1 | 1.5 | Touch | Fit |
| browser small | 800 x 600 | 1 | 1 | Mouse | Dp |
| phone landscape | 2532 x 1170 | 3 | 1 | Touch | Dp |
| phone portrait | 1170 x 2532 | 3 | 1 | Touch | Dp |

Each runs at the four text sizes. The layout specimen, relative panes and `Native` game screens must pass `desktop`. The
other profiles report only, because nothing keeps a minimum yet; a report does not fail the gate. They must resolve without a
panic. `Old` and `Quadrant` screens are not audited for layout.

## 11. What this must not block, and what waits

Not built here: touch input, portrait layouts, the web port, a UI scale setting, rearrangements for any game screen.

| Later task | What this design keeps open | Rule for new code now |
| --- | --- | --- |
| Web port | `aegis_layout` compiles anywhere; `aegis_ui` as a whole does not today (`bridge.rs:7,22` socket and threads, `shell.rs:1182` files; winit, pixels, rodio, arboard) and needs its own task | New layout and metric code uses no file, thread, socket or clock. Canvas comes in as a value |
| Any browser window | A resize already replaces canvas and view (`shell.rs:1230`) | Read `ui.screen()` each frame; assume no aspect; keep no `Rect` across frames on `Native` screens |
| Portrait | Classes treat it as width Compact | No `Native` screen assumes wide; one without a compact choice reports overflow |
| Touch | `Env.pointer`, `(-1, -1)` as "no pointer", the target minimums of 11.1 | No information needed to play lives only in a hover card. A tip is also reachable by focus or press: `draw_tip` (`ui.rs:747`) is keyed on the mouse today, and E3 makes its target "the hovered or focused widget". `hover_only` reports a tip on a widget that can take neither. Dragging keeps a non-drag route; right click keeps a visible button |

Added size for small screens: E5 (1 run) plus about a third of E2 and E3, already in the 5. Can wait until a game targets a
small screen: `Fit` and `Dp` in the shell, the UI scale setting, touch input and enlarged hit boxes, the `draw_tip` press path,
compact arrangements. If E5 waits too, 1 run moves out of phase 1 and nothing checks small sizes. A real web target also means
moving every screen it offers, which is not costed here.

### 11.1 Later work, and where it attaches

Not in phase 1. Each is its own later task. Phase 1 only keeps one place for it, so no caller changes when it lands.

| Later work | Where it attaches | What phase 1 does | What it adds |
| --- | --- | --- | --- |
| Images: a simple scale or fill rule, chosen per image | `Ui::image_in(img, r)` | Draws in the given rect, as today. `cover_crop` stays for backdrops | A rule argument with a default; aspect-kept drawing, so a varying canvas cannot stretch art; a `stretched_art` audit kind |
| Minimum size caps: some elements keep a minimum to stay legible | `Part` (private fields, so a `.min(n)` adds without breaking callers); `Ui::m()` for text and control sizes; `Env.dpr` | Parts have no caps; text keeps only the 6 px floor of `Painter::px` | Caps on parts; `min_text_dp` and `min_target_dp` theme keys, after the Text size setting and UI scale; `small_text` and `small_target` audit kinds; values to tune on a device (12 dp text and 44 dp targets are common guidance, assumed) |
| Space priority: what space may be lost when the minimums do not fit, such as which pane gives way first | `Part` (a `.give_way(rank)`), applied inside `resolve(parts, room)` | Too little room gives 0 size, and the audit reports it | The give-way step before boundaries are worked out; shared-edge rounding is unchanged. If nothing can give way, the screen's next `pick` choice is the fallback |

Until these land, moved screens are checked on this game's fixed canvas only, where a layout has the room it was written for.
Art on a moved screen is authored in a rect of its own proportions.

## 12. Risks

| Risk | Handling |
| --- | --- |
| Mixed screens for a long time | Accepted by design. `Screen::grid()` is one line to find; nothing forces a move |
| Quadrant makes a real screen hard to use while the game is still being tested | Delete the line: it is the wrapper |
| A native screen gets an old-unit number by habit (`BODY`, `W`, a copied literal) | T3; review checks moved files for `ui::{W, H, BODY, SMALL}` |
| E3 or E4 changes old-path pixels while going relative | T1 pins them at `g = 1` |
| Without caps, a moved screen distorts or overflows on another canvas, and art can stretch | Accepted for phase 1: this game has one canvas. The audit reports it; section 11.1 fixes it before a game offers another |
| Phase 1 blocks the later work | `Part`, `resolve`, `Ui::image_in` and `Ui::m()` are the single entry points; a crate test covers `resolve` so the later steps can rely on it |
| "Any piece touched moves to relative" grows small tasks | The function-level reading in 6.2; the review checks scope |
| Reduced-frame caches recompose on every step of a window drag | `REDUCED_MAX` bounds memory, not time. Fine on desktop; a web port measures it |

## 13. Authoring note (E2 writes it)

`engine/docs/authoring/reactive_layout.md`, naming no game, linked from `engine/docs/authoring/README.md` (decision
"Promote generic methods"). `art_bible.md` gets a pointer for canvas and unit. Contents: window, policy, canvas and unit;
writing a screen (cut from `ui.screen()`, tokens from `ui.m()`, frames through `frame_body`, panes before drawing, no rect
kept across frames); the table of 6.2; fractions to grid to pixels and resolving edges, not sizes, with the worked example of
6.3; **the rule that any engine layout code touched moves to relative**; size classes and
`pick`; overflow per container; `dpr`, Text size and UI scale; art and units (odd and even, thin lines); no fixed positions on
a moved screen; what is later and where it attaches (11.1); no hover-only information; what layout code may not use; testing by relations, profiles and `shot_diff`; sorting
existing screens into the three classes, quadrant and wrapper, moving a screen only inside a task that changes it.

## 14. Schema contract

Schema-first: yes for E3 and E5. Theme key `metrics` in `ui/theme.json`, optional; absent means these defaults, which equal
today's values doubled (`ui.rs:15-20`, theme `layout`). The `layout` block is unchanged. Unknown keys are errors, as
elsewhere in the theme. Numbers must be above 0 unless noted. On `Old` and `Quadrant`, `Metrics` returns today's constants
and `layout`, so `metrics` changes nothing there.

| Key | Type | Default | Note |
| --- | --- | --- | --- |
| `body`, `small`, `title`, `head`, `line` | f32 | 14, 12, 15.2, 24, 18 | Text sizes and line step |
| `radius` | f32, 0 or more | 5 | |
| `margin`, `gutter`, `padding`, `grid` | i32 | 16, 12, 12, 8 | `grid` can later go to 4 for finer snaps; `panes::snap` reads it (`panes.rs:34-48`) |
| `frame_title_h`, `tab_h`, `row_h` | i32 | 52, 36, 28 | `row_h` is assumed from `shop.rs:30`; a screen may keep its own |
| `focus_glow` | i32, 0 or more | 2 | |
| `breakpoints` | object: `width: [i32, i32]`, `height: [i32, i32]`, ascending | `[800, 1200]`, `[480, 700]` | Compact and Medium upper limits |

Game manifest keys (E5, read through the shell's options; the game sets them in G0): `canvas`, an object with `policy`
(`fixed`, `fit` or `dp`; default `fixed`) and `w`, `h` (i32, default 1280, 720; `w` a whole multiple of 640), and
`window_sizes`, a list of `WxH` strings (default today's list). Bad values are a notice and the default, as with `icon`.

Acceptance, from the rules above: T1-T5; crate rules in 6.1; no slice edits a sidecar, `tuning.json` or data; `aegis_layout`
builds as `no_std`; `layout_audit` has a test per kind; each profile resolves the layout specimen without a panic.

## 15. Assets

None.

## 16. Out of scope

Touch, portrait arrangements, the web port and any wasm toolchain in the gate; a UI scale setting; `Fit` or `Dp` for this
game; image rules, minimum size caps and space priority (section 11.1); changing the fixed canvas; any visual redesign; parity work on non-finalized screens; a sweep over screens; removing
the old mapping; kit crates, which draw nothing (`kits/rpg` mentions `aegis_ui` only in comments in
`rpgkit_data/src/lib.rs`, and holds only the `_px` fields above).

## 17. Verified and assumed

Verified in code at `C:/dev/tactical-rpg-stage` (`d49ea32a`): every file:line above; rect counts per file and, by line
ranges, per screen; 197 numeric consts by grep (about 174 given, depends on the pattern); about 430 text calls by my pattern
(about 550 given); 26 `Screen` impls (20 game, 6 engine); frame sidecars are `scale` 2 or 8 only.

Reasoned, not run: T2's float exactness; that `Grid::Old` is identical; the text and frame sizes in section 4 (computed from
`Painter::px` and `Painter::nine`); all readability calls; that panes can go relative while staying bit-identical at
`g = 1` (T1 will show it, and 6.2 gives the fallback). No code was compiled for this spec.

Assumed: all run counts; the M2 share; size-class thresholds; `dpr` values; `row_h`; that playtest trail `pos`
(`shell.rs:566`) is informational (E1 checks it); frozen clocks for shots. Classes marked confirm or flag are my reading of
the docs, not the user's word.

## 18. Open questions

1. **Approve the screen table** (section 1.2), the unsure rows first. Recommendation: as guessed in each row.
2. **Which option?** Recommend B. It keeps "Layout grid moves to 1280 x 720", arriving in E1 for new screens. The decision's
   "Scale: about 830 `Rect::new`" stops being the cost.
3. **Quadrant or wrapper, per screen?** Recommend wrapper for the rows marked Wrapper, Hard or Misleading, and quadrant for
   the rest, after the mock board. Both are about free, so the quadrant saves no cost; what it gives is the blank area the
   user described. The user may prefer wrapper throughout, which would also drop slice Q.
4. **Keep M0 as a pilot, or make M2 the first user?** Recommend keeping it: 0.5 run, and a problem in the tools shows on 7
   rects, not on M2's 40-odd.
5. **"Any engine piece touched moves to relative": function level or whole file?** Recommend function level (6.2).
6. **Do text, spacing, thin lines, frame parts and small control heights stay tokens under "only relative positions and
   sizes"?** Recommend yes. The layout supplies them and they re-resolve with it, but a fraction would stretch them, as the
   user said earlier. Making them relative too would need the minimum caps in phase 1.
