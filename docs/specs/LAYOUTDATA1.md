# LAYOUTDATA1: screen layouts as data, authored by a UI agent

Status: proposed (the user decides section 14; nothing is approved yet)
Depends on: LAYOUT720-E4 (done), LAYOUT720-M0 (built in lane 1, uncommitted), LAYOUT720-E5 (for slice D2)
Lane: design for this spec; the slices are logic and a new `ui` lane (section 8)
Touches: see the slice table in section 10

## 1. Goal and recommendation

Goal (user 2026-11-03, decision "Engine goal: agents build, humans judge feel"): UI changes happen without a logic builder
writing screen layouts. A design or UI agent builds layouts; logic builders add widgets and bindings.

Recommendation, in short:

1. A screen's layout is a small JSON tree in the asset layers, `ui/layouts/<screen id>.json`, read by `aegis_ui`.
2. The tree places things. Widgets, state and actions stay in Rust and are named by slot. Anything hard is a slot.
3. `tools/fc.sh layout` gives an agent schema, contract, layout and text checks and shots, with no rebuild. The game
   re-reads a changed file while running, so the user can judge feel on the spot.
4. Convert the main menu first: it is the one screen with a signed-off picture to compare against.
5. Then make LOOK1-M2 the first UI-agent work. Total 5 runs of new work plus E5; M2 lands about 1 to 2.5 hours of lane time
   later and costs about 0.5 run more, against about 2.25 runs to redo it later (section 7).

## 2. The authoring form

The question: what does a UI agent write?

| | A: declarative tree with slots (recommended) | B: Rust builder convention | C: small script (Rhai, Lua) | D: tree naming widget kinds, with a binding table |
| --- | --- | --- | --- | --- |
| What the agent writes | JSON: cuts, stacks, frames, tokens, named slots | Rust in agreed `layout_*.rs` files | A script computing rects | JSON including `button`, `slider` and each widget's arguments |
| Edit to picture | Saved, picture changes; hot reload for the user | Rebuild, relaunch | Saved, picture changes | Saved, picture changes |
| Can it break logic | No: no access to state | Yes: all of Rust; only review stops it | Partly: depends on what is exposed | No |
| Action types | `A` stays typed in Rust | Typed | Strings to Rust | Strings to `A`, per screen |
| New engine | About 2 runs | None | A VM dependency, bindings, determinism rules; 3 runs or more | About 3.5 runs |
| Add a button with no code | No | Yes, but it is code | Yes | Yes, for an action that exists |
| Main risk | Grows into a language (rule below) | "Without a logic builder" is true in name only | Second language, no static shape, harder web port | Every widget's arguments become a schema to maintain |

Why A: the goal needs an agent that cannot break logic and a picture that changes without a build. D adds little: a new widget
nearly always needs a new action, hence code, and it can come later as one more leaf. B costs nothing but does not meet the
goal. HISTORY has no row on any of these.

**The rule that stops it growing:** a closed list of node kinds, no arithmetic, no variables except tokens, conditions are
one flag. Anything harder is a slot. A new node kind needs its own task row and a reason.

**Asset, not content.** The file lives beside `ui/theme.json` and works like it: found over the layers, edited with the
Edit tool, validated when loaded, errors shown and the previous version kept (`theme.rs:816`, `:862`, `:1091`). The
`aegis content` writer does not apply: its rule covers `games/*/data/**` (`docs/STRUCTURE.md`), catalog rows with a store format
and a byte-for-byte resave test. A layout is one tree, outside the content fingerprint, which reads the data folder
(`emberwake_gui/src/main.rs:127`), so an edit touches no save, recording or sim. HISTORY's lesson ("hand-edited JSON and ad-hoc
string patch scripts corrupted files") is about catalog files; the guard here is strict loading plus `tools/fc.sh layout` in the gate.

## 3. Schema contract

Schema-first: yes. File `ui/layouts/<screen>.json`, first hit over the layers (game, kit, engine; `Layers::find`,
`aegis_core/src/assets.rs:70`). A file replaces a lower layer's whole file; trees do not merge. Unknown keys are errors,
as in the theme. Only screens on `Grid::Native` may use one.

### 3.1 File

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `version` | int | required | 1 |
| `screen` | string | required | Equals the file's stem |
| `design` | `[w, h]` | `[1280, 720]` | Both above 0 |
| `tokens` | object, name to number | `{}` | `snake_case`, 0 or more; a name in `Metrics` is an error |
| `corner_size` | number | none | Passed to `Ui::corner_size` before drawing (one corner size per screen) |
| `root` | node | required | |

### 3.2 Values

| Value | Written | Meaning |
| --- | --- | --- |
| Design number | `470` | `canvas * 470 / design`, on that axis, rounded down; exactly 470 on the design canvas. Relative, and reads like the mockup |
| Token | `"$row"` | Units, fixed. A screen token or a `Metrics` field: `margin`, `gutter`, `padding`, `grid`, `frame_title_h`, `tab_h`, `row_h`, `focus_glow`, `hair`, `body`, `small`, `title`, `head`, `line`. A fractional value used as a length rounds up |
| Ratio | `"1/3"` | Of the room the parent gives. Both whole, at most 65,535 |
| Fill | `"fill"`, `"fill:2"` | What is left, by weight |
| Content | `"content"` | Measured (rule R5) |
| Floor or text growth | `{"is": 320, "min": "content", "with_text": true}` | `is`: any of the above but fill. `min`: number, token or `"content"`. `with_text`: `is` times the Text size level, snapped up to `Metrics.grid` |

Sizes (`size`, `w`, `h`) take all of these. `margin` takes a number, a token or `{"l", "t", "r", "b"}` of those. `gap`, `pad`
and slot `params` take tokens only: spacing never scales (decision "Engine layout is relative").

### 3.3 Nodes

Common keys on every node:

| Key | Type | Default | Meaning |
| --- | --- | --- | --- |
| `id` | string | none | Unique in the file. `frame` and `image` push a widget under it (`ui_tree`) |
| `about` | string | none | A comment; ignored |
| `when` | string | always | `flag`, `!flag`, `art:<path>`, `!art:<path>`. Nothing else |
| `margin` | see 3.2 | 0 | Taken off the given rect first |
| `place` | anchor | none | One of `top_left`, `top`, `top_right`, `left`, `center`, `right`, `bottom_left`, `bottom`, `bottom_right`. Needs `w` and `h` |
| `w`, `h` | size | fill | Used with `place` |
| `side` | `top`, `bottom`, `left`, `right`, `rest` | required in `cuts` | The edge this kid cuts from; `rest` takes what is left and ends the list |
| `size` | size | required in `cuts` except `rest`; `fill` in `split`, `stack` | Along the cut or the axis |
| `frame` | frame role | none | Draws through `frame_body`; the node's kind fills the content rect. May join one kind key |
| `pad` | token | 0 | Inside the frame or the rect |

Exactly one kind key below; none makes a spacer.

| Kind | Keys | Meaning |
| --- | --- | --- |
| `layers` | list of nodes | Every kid gets the same rect |
| `cuts` | list of nodes | Each kid cuts a band; the rect keeps the rest (`cut_*`) |
| `split` | list; `axis` `x` or `y` (required); `gap` | Shared edges (`split_x`, `split_y`) |
| `stack` | list; `axis` (required); `gap` | One after another (`Stack`). Overflow is reported |
| `grid` | slot name; `cols` (required); `rows` (default from the count); `gap` | That slot once per cell, up to its `count` binding (`Rect::grid`) |
| `slot` | name; `params` (name to token); `mock` | Code draws here. `mock` is what a preview draws (section 6.3); it is ignored once code fills the slot |
| `image` | path or list (first found wins); `fit` `contain` (default), `cover`, `stretch`; `anchor` (default `center`); `veil` 0-255; `vignette` 0-1; `optional` | A picture |
| `text` | literal or `@name`; `style` `body` (default), `small`, `title`, `head` or a token; `color` theme role, default `text`; `face` `body`, `display`, `mono`; `anchor` (default `top_left`); `shadow` token; `shadow_color` role | One line; cut to its rect with the full text on a tip. Wrapped text is a slot |

Reserved, refused in version 1: `pick`, `overflow`, `max`, `give_way`, `repeat`. They name the later work of section 9.

## 4. Rules

| Id | Rule | Acceptance check |
| --- | --- | --- |
| R1 | Tree order is draw order and focus order (widgets push as drawn) | Reordering two slots in a fixture reorders `ui_tree` widgets and draws the later on top |
| R2 | The walk is sequential in `layers` and `cuts`: `when` and any measure are read as the kid's turn comes, after earlier kids drew. `split`, `stack`, `grid` read theirs on entry, as sizes are worked out together. So a flag set by a slot's draw (a hover) works for a later `cuts` or `layers` kid in the same frame | A slot sets a flag in draw; a later `cuts` kid shows that frame |
| R3 | A node that is off takes no room, no gap and no measure | `when` false in each container kind: siblings lay out as if it were absent |
| R4 | Sizes go through `aegis_layout::resolve` and `cut_*`: shared edges, whole units, never negative, nothing panics. A size that cannot fit becomes 0 and the audit reports it | Every canvas of `the_menu_layout_follows_the_canvas` and the E5 profiles, on a fixture of every kind: no panic, no negative size |
| R5 | `content` measures bottom up: `text` by its line and width, `slot` through `measure`, a token or number as itself, `fill` and ratios as 0; `stack` and `cuts` sum along the axis with gaps and take the largest across; `frame` adds its insets (`Ui::frame_outer`) and `pad`. Measured sizes snap up to `Metrics.grid` (LOOK1 6) | A frame with two text lines and a `$face` row: height equals insets plus lines plus face, snapped, at each text size |
| R6 | `place` clamps to the parent, as `Rect::place` does. A floor larger than the parent is clamped and reported as `overflow` | Fixture: floor past the parent gives a clamped rect and one issue |
| R7 | Design numbers resolve against the canvas, never the parent | The same node nested at two depths has the same size |
| R8 | A bad file never crashes and never strands the player (section 5.3) | Each error row in 5.3 has a test; with a broken file, `click` on the required Quit slot's widget still works |
| R9 | Nothing is kept across frames; the walk runs each frame from a parsed file | A resize and a reload re-lay with no screen code |

Crate placement: the pure part (`Node`, `walk`, `measure`, a `Visitor` trait) goes in `aegis_layout::tree`, keeping `no_std` and
no dependencies. JSON reading, the store and drawing go in `aegis_ui::layout_doc` (`serde_json` is already there). So the
walk is tested with no `Ui`, and a preview is only another visitor.

## 5. Bindings

### 5.1 What code provides

```rust
pub trait LayoutHost<A> {
    fn contract(&self) -> &'static [Binding];     // every name below, with `required` and a one-line `about`
    fn art(&mut self) -> &mut Art;
    fn flag(&mut self, name: &str) -> Option<bool> { None }
    fn text(&mut self, name: &str) -> Option<String> { None }
    fn count(&mut self, name: &str) -> Option<usize> { None }
    fn measure(&mut self, ui: &mut Ui<A>, s: &Slot) -> Option<(i32, i32)> { None }    // units
    fn draw(&mut self, ui: &mut Ui<A>, s: &Slot) -> bool;                             // false: no such slot
}
pub struct Slot<'a> { pub name: &'a str, pub index: usize, pub r: Rect, pub host: Rect, /* params */ }
```

`Slot.host` is the nearest enclosing frame's outer rect, or `r`. A trait over one per-frame struct (borrowing the screen and
`cx`) avoids the double borrows a closure per slot would hit. A screen's draw becomes: `Ui::new`, build the host,
`layouts::of(self.id()).draw(&mut ui, &mut host)`, keep `ui.widgets`. Actions stay `A`; `act`, `key`, `tick`, `back`,
`auto_focus` and `state` do not change.

### 5.2 Who changes what

| Change | UI agent alone | Logic builder |
| --- | --- | --- |
| Move, resize, reorder; change a gap, margin or token | Yes | |
| Hide an optional slot; add a frame, picture or fixed text | Yes | |
| Change text size or colour role on a `text` node | Yes | |
| A new action, widget, state or slot; a new flag or `@text` | | Yes: one contract line and one match arm |
| A new kit widget or node kind | | Yes, with a task row |

Left in code, as slots or whole screens: the battle board, region map and all drawn on it, timeline, Stage, Dojo, drag and
drop, portrait effects; the themed ask (`ask_layout` measures its own text) and the vignette player. A custom board can later
be one slot in a data-laid screen, with its own cell math inside.

### 5.3 Errors

| Problem | Running game | `tools/fc.sh layout` and the gate |
| --- | --- | --- |
| File missing, unreadable; bad key, type, token or node | `show_errors`; the previous file stays; on first load, the bare layout | Fail |
| Slot, flag or `@text` not in the contract | One notice; it draws nothing | Fail (`unknown_slot`, `unbound`) |
| A required slot not placed | One notice; the whole screen uses the bare layout | Fail (`missing_slot`) |
| `image` finds no file and is not `optional` | Draws nothing | Fail (`missing_art`) |

The bare layout: every contract slot in contract order, in a `Stack` of `row_h` rows inside `margin`. Ugly, but every required
action can be reached.

## 6. Checks: what makes a layout trustworthy without eyes

### 6.1 `tools/fc.sh layout [screen|all] [shots=<dir>]`

Runs `emberwake_gui`'s test table (`src/layouts.rs`: per screen, named fixture states, such as `main_menu`: `idle`,
`no_save`, `over_continue`) through the headless shell. It reads files from disk, so an edit needs no rebuild once the
worktree is built. Each state runs at the four text sizes, at 1280 x 720 and 1920 x 1080. Output under 40 lines, details in a
file. The gate runs it with no shots, so every shipped layout stays valid.

| Check | Comes from | Fails on |
| --- | --- | --- |
| Schema, tokens, `when` shape | D1 | Any error |
| Contract | D2 | `unknown_slot`, `unbound`, `missing_slot`, `missing_art` |
| `layout_audit` | E5, D2 adds kinds | `overflow`, `off_canvas`, `zero_size`, `corner_shrunk`, `hover_only`; new: `stretched_art` (a `stretch` image more than 2 % off its aspect), `widget_overlap` (two enabled widgets overlap and neither holds the other) |
| `text_audit` | Exists (`shell.rs:855`, `issues_on` in lane 1) | `cut`, `edge`, `overlap`, `spill` |
| Screen tests | Exist, in code | Ids and actions unchanged; they do not depend on the file's numbers |
| `shot_diff` | Exists (`shot_diff.rs`) | Only where a task says the picture must not move (the proving case) |

`ui_tree` gains a `layout` key: the file, its layer, each node's id or slot with its rect, and optional slots not placed. An
agent reads relations from it, and reads the shots to make a board. E5's other profiles report only (LAYOUT720 section 10).

### 6.2 Hot reload for the user

The shell stats the top screen's layout file about twice a second and re-reads it on change, as `TuningFile` does for the
tuning file (`tuning.rs:29`), in every build, since `run.bat` builds release. A bad file gives a notice and the old layout stays. Reload
data (`runner.rs:298-310`) re-reads all layouts beside the theme. Nothing else is needed: screens keep no rects.

### 6.3 Gallery preview and mocks (takes in LAYOUT1)

A new Gallery entry, "Layouts" (`gallery.rs:38` `ENTRIES`; not a tab on `Specimens`, which would change pinned hashes). It
lists the layout files over the layers, and shows each as:

- **Wireframe:** node rects and slot names, for structure.
- **Mock:** images real, `text` real, each slot drawn from its `mock` key: `button:<label>`, `tab:<label>`, `row:<label>`,
  `slot_card`, `portrait`, `text:<line>`, or a plain named box. Inert, through the real kit widgets.
- **Live:** when a screen is registered for that file, the real screen, through `preview::view_in`.

This is LAYOUT1's "mockups built in the engine, filled with the game's assets", and decision "No upkeep on unused debug tools"
keeps tools for human mockups. A `MockHost` lets `tools/fc.sh layout <file>` audit a file no code backs yet. **The mock is the
layout**: the file approved on a board is the file that goes live, so nothing is rebuilt from a picture. Drag handles
(LAYOUT1's "resizable elements") are not built; see question 5.

## 7. The first screens

### 7.1 The main menu is the proving case (recommended)

It is the only finalized screen and M0 left before and after shots of it (`C:/dev/localgen/review/layout720/m0/`: 720p and
1080p, Normal and Extra large, the typeset path, the peek). So the proof is objective. Its 7 rects hold the hard parts: a contained
picture, a conditional panel measured from content, a column that widens for its labels. Cost 0.75 run.

**Proof, slice G2:** with no Rust edit after G1, the UI agent's file gives `shot_diff` clean against M0's `after_*` shots,
`ui_tree` ids unchanged, `tools/fc.sh layout main_menu` clean.

**Stop rule:** if parity needs a new node kind or any arithmetic, or the file passes about 60 lines, stop and report. M2
then goes to code on the relative tools (route A2), with E1 and E2 kept, as they stand alone. What is lost is at most D1, D2
and G1, about 2.5 runs.

### 7.2 LOOK1-M2: the real decision

Nothing depends on LOOK1-M2 in `docs/TASKS.md`, and decision "Priorities after the visual pass" puts the menu rebuild last, so
M1 and the M2 content chain do not wait in any route. A run is one builder plus review, about 17 to 47 minutes (decision
"Efficiency pass"). The M2 sizes are my estimates.

| | A: M2 in code now, convert later | A2: M2 in code now, never convert until redesigned | B: form first, M2 by the UI agent (recommended) |
| --- | --- | --- | --- |
| M2 itself | About 2.5 runs | About 2.5 | About 3: bindings 1.25, files 1, code-only part 0.75 |
| Form, whenever built | 4.75 | 4.75 | 4.75 |
| Rework of M2's screens | About 2.25 (split into slots, write files) | 0 now; each redesign pays | 0 |
| Total | About 9.5 | About 7.25 | About 7.75 |
| M2 on screen | Soonest | Soonest | 3.25 serial runs later, with E5 beside them: about 1 to 2.5 hours, plus 0.5 run |
| Who can change M2's screens after | UI agent, after the rework | A logic builder | UI agent |
| Risk | Same screens rebuilt twice, what LAYOUT720 section 3 argued against | Six more screens outside the goal | The form is unproven: the stop rule bounds it |

Recommendation: **B**. It costs M2 a short delay and 0.5 run, on screens nothing is waiting for, and tests the form on six real
screens. B also moves E5 up: that decision has "E5 and G0 later", and `layout_audit` is what the agent is checked by. G0 stays later.

### 7.3 M2 under route B

The row splits. Bindings carry no layout, so they need not wait for the user's board, and decision "Efficiency pass" (no
builder on a visual change before a mock) still holds: the builder no longer changes the picture.

| Row | Who | What | Size |
| --- | --- | --- | --- |
| LOOK1-M2-L | `ui` | Files for the screens below, mock first, a board for the user | 1 |
| LOOK1-M2-B1 | logic | Hosts for New game, Slots, Settings (`menus.rs`) | 0.75 |
| LOOK1-M2-B2 | logic | Hosts for Game menu, Run over, Campaign notice (`hub.rs`, `runover.rs`) | 0.5 |
| LOOK1-M2-C | logic | Stays code, on the relative tools: `Cx::ask` through the themed ask, story and vignette cards with the banner and ornate rules | 0.75, needs only E1 and E2 |

M2-C can start once E1 and E2 land. L and B run in either order against an agreed contract. Numbers for the files come from LOOK1
section 6, doubled. Where it gives none, the UI agent chooses and the board shows it. Spacing comes from `Metrics`.

| Screen | Layout | Slots (draft; the code's contract rules) |
| --- | --- | --- |
| New game | World cards over a backdrop | `grid` `world_card` (count: worlds), `party_line`, `start`, `back` |
| Load, save | Six slot cards, two columns, 580 x 168 at Normal, `with_text`; thumbnail 224 x 126 inside the card | `grid` `slot_card` (count 6), `back`, `@title` |
| Settings | Tab rail left, 240 wide; the form right; popup 1200 x 600 at Normal, `with_text`, centred | `tabs`, flags `tab_<name>`, one slot per control, `close` |
| Game menu | Over the dimmed map, 50 % veil: left rail, party box | `rail`, `party_box` |
| Run over, Campaign notice | `ornate` frame | Text slots, answers |

After M2, each later UI task (UIR-4 first) splits the same way. No sweep: the rollout rule of decision "Engine layout is
relative" stands, and a dense screen may still move in code.

## 8. The UI agent's lane

New lane value `ui` in `docs/TASKS.md`, with a role note `.claude/agents/ui-author.md`. Draft of what it says:

| Topic | Rule |
| --- | --- |
| Writes | `games/*/assets/ui/layouts/*.json`; review boards in `C:/dev/localgen/review/<task>/` |
| Writes only if the row's Touches names it | The `metrics` and `game_colors` keys of `ui/theme.json` |
| Never | `*.rs`, `data/**`, `tuning.json`, tests, goldens, kit or engine layers; never commits |
| Reads | The row's brief, `engine/docs/authoring/layout_files.md`, `preferences/ui.md`, the contract (`ui_tree` and the bridge's `layout_contract`) |
| Done means | `tools/fc.sh layout <screen>` clean, a board at 720p and 1080p at Normal and Extra large, `shot_diff` if the task says the picture holds |
| Gaps | A missing slot, flag or node kind: stop and report it as a binding gap with the exact need, as the asset-builder does with a schema gap. Two failed tries at one problem: stop |

Landing: one file per screen, so layout tasks can run in parallel across screens. A file-only change passes `tools/fc.sh
layout all` and `tools/fc.sh docs`, and gets the lighter review of decision "Efficiency pass" (4). `staging` to `develop` still
takes the full gate. The wider split of roles is AGENTROLES1, which this feeds.

## 9. Later work of LAYOUT720 section 11.1

| Later work | Data form needs | Now? |
| --- | --- | --- |
| Image fit rule | `fit`, `anchor` (gap 7); `stretched_art` | **Yes**, E2 and D2 |
| Minimum caps on parts | `min` (gap 2) | **Yes**, `Part::min`, on shares, ratios and tokens only, E1 |
| Minimums on text and targets (`min_text_dp`, `small_text`, `small_target`) | Nothing yet | No: this game has one fixed canvas |
| Give-way | Reserved key | No: it needs a canvas that can be too small. A floor that does not fit is reported |
| `pick` and size classes in a file | Reserved key | No: same reason. A second canvas or a compact arrangement brings it |
| Scroll in a `stack` | Reserved `overflow` | No: LOOK1 6's screens fit. Journal or Results may need it, as a slot until then |

## 10. Slices

One run is one builder plus review. Sizes are estimates.

| Id | What | Layer | Lane | Depends on | Touches | Runs |
| --- | --- | --- | --- | --- | --- | --- |
| LAYOUTDATA1-E1 | `ratio(n, d)`, `Part::min`, `Stack` on an axis. Stands alone | engine | logic | - | `aegis_layout/src/lib.rs`, `tests/layout.rs`, `reactive_layout.md` | 0.5 |
| LAYOUTDATA1-E2 | `panes::menu_column_in`, `panes::tab_rail_in`, `Ui::picture` with `Fit`, `Ui::text_in`, `Ui::frame_outer`. Stands alone | engine | logic | LAYOUT720-M0 (same `ui.rs`), E1 | `aegis_ui/src/{panes,ui}.rs`, `tests/panes_native.rs` | 0.5 |
| LAYOUTDATA1-D1 | The tree: schema, walk, measure, reader, store over the layers, `layouts::reload` | engine | logic | E1 | new `aegis_layout/src/tree.rs`, new `aegis_ui/src/layout_doc.rs`, `aegis_ui/src/lib.rs`, new tests, new `engine/docs/authoring/layout_files.md` | 1 |
| LAYOUTDATA1-D2 | Drawing, `LayoutHost`, contract, bare layout, hot reload, `ui_tree` `layout`, bridge `layout_contract`, audit kinds | engine | logic | D1, E2, LAYOUT720-E5 | `aegis_ui/src/{layout_doc,shell,layout_audit,screen}.rs`, new tests | 1 |
| LAYOUTDATA1-M | Gallery "Layouts": wireframe, mock, live; `MockHost`. Closes LAYOUT1's mockups | engine + game | logic | D2 | new `aegis_ui/src/layout_preview.rs`, `emberwake_gui/src/gallery.rs` (one entry) | 0.75 |
| LAYOUTDATA1-G1 | `MainMenu` on a host and contract; the test table; `tools/fc.sh layout`; gap 8. Lands together with G2 | game | logic | D2 | `emberwake_gui/src/menus.rs` (`MainMenu` and its tests), new `emberwake_gui/src/layouts.rs`, `tools/fc.sh` | 0.5 |
| LAYOUTDATA1-G2 | `main_menu.json` by the UI agent: the proof of section 7.1 | game | ui | G1, R | `games/emberwake/assets/ui/layouts/main_menu.json` | 0.25 |
| LAYOUTDATA1-R | Role note, lane value, decision rows | process | planner | D1 | `.claude/agents/ui-author.md`, `docs/WORKFLOW.md`, `docs/STRUCTURE.md`, decisions | 0.25 |

**Total 4.75 runs**, plus LAYOUT720-E5 (1, already approved), plus M2's 3 under route B. Longest chain from now: E1, D1, D2, G1,
G2 = 3.25 runs. Parallel: E1 and E5 and M2-C's start; E2 after M0 lands; M alongside G1. D2 and E5 share `shell.rs`, so not
together. M0 lands first; it needs no change for this. G1's rewrite of M0's pinned tests goes in its commit, as relations.

No golden and no pinned hash is re-recorded: `menu_column` and `tab_rail` keep their forms for old callers
(`specimens.rs:1241`, `tests/panes.rs:205`), and the T1 hashes draw only those.

### 10.1 The pilot's gaps

| # | Gap (evidence in lane 1) | Fix | Stands alone? | Slice |
| --- | --- | --- | --- | --- |
| 1 | Per mille too coarse; the logo needs "a share of the right half" (`menus.rs:58-64`, `:243`) | `ratio(n, d)` in `Part`; in a file, a design number | Yes | E1 |
| 2 | Share with a content floor by hand (`menus.rs:311`) | `Part::min(n)`; in a file, `min: "content"` | Yes | E1, D1 |
| 3 | Opaque 954, 417, 969 (`menus.rs:61-75`) | In a file: `cuts` and design-number margins ("120 from the right, 300 down"). In code: `ratio` | Partly | E1, D1 |
| 4 | `menu_column` takes x, y, w, h, step (`panes.rs:451`, `:458`) | `menu_column_in(ui, id, r, row_h, gap, items, style)`. Step 22 old is row 36, gap 8 | Yes | E2 |
| 5 | `Stack` is vertical only (`aegis_layout/src/lib.rs:286`) | `Stack::row`, or an axis argument. Wrapping flow is not needed by any listed screen | Yes | E1 |
| 6 | Row 18, step 22, gap 12, text 28 have no token (`menus.rs:70-84`) | A file's `tokens`. Code keeps named consts and `m.u(n)` | No, form only | D1 |
| 7 | Physical blit for the title; `image_in` has no aspect rule (`menus.rs:189-201`, `ui.rs:256`) | `Ui::picture(art, path, room, Fit, Anchor) -> Option<Rect>`: moves `logo_px` and the blit into the engine, over `Art::image_fit` (`art.rs:141`). `Fit` is `Stretch`, `Contain`, `Cover` | Yes | E2 |
| 8 | Crest centre snapped to `m.u(1)` (`menus.rs:171`) | Not an engine fix. Centre on the title with `Anchor::Center`; T4's 1 unit covers it. Only the typeset path draws it, which the shipped game does not reach while `ui/logo_title.png` exists | Yes | G1 |
| 9 | Content panels by hand; no text by anchor (`menus.rs:216-221`, `:322-324`) | `Ui::frame_outer(role, w, content_h)` over lane 1's `frame_content`; `Ui::text_in(r, text, style, Anchor)`; in a file, `content` | Yes | E2, D1 |

Also found: `tab_rail` sets its own width (`panes.rs:495`), hence `tab_rail_in`; LOOK1 6's popups grow with text, hence
`with_text`; the peek reads a hover from the column drawn before it (`menus.rs:313-317`), hence rule R2.

## 11. Assets: the main menu file

G2 writes this, or better. Numbers derive from M0's test (`menus.rs:1236-1245`): column at (840, 300), 320 wide; room's right edge
816; logo (780, 26, 470, 124); typeset point (1240, 80); peek at least 320 x 128.

```json
{
  "version": 1,
  "screen": "main_menu",
  "tokens": { "row": 36, "row_gap": 8, "peek_gap": 24, "face": 40, "face_gap": 4, "peek_pad": 6 },
  "root": { "layers": [
    { "id": "menu.backdrop", "fit": "cover", "vignette": 0.4,
      "image": ["backgrounds/menu/title.png", "backgrounds/menu/main_menu.png"] },
    { "id": "menu.logo", "when": "art:ui/logo_title.png", "image": "ui/logo_title.png", "fit": "contain", "anchor": "top_right",
      "place": "top_right", "margin": { "t": 26, "r": 30 }, "w": 470, "h": 124 },
    { "when": "!art:ui/logo_title.png", "slot": "logo_typeset", "margin": { "t": 80, "r": 40 } },
    { "cuts": [
      { "side": "top", "size": 300 },
      { "side": "right", "size": 120 },
      { "side": "right", "size": { "is": 320, "min": "content" }, "slot": "menu",
        "params": { "row": "$row", "gap": "$row_gap" } },
      { "side": "right", "size": "$peek_gap" },
      { "side": "rest", "when": "peek", "frame": "ornate", "pad": "$peek_pad",
        "place": "top_right", "w": { "is": 320, "with_text": true }, "h": { "is": 128, "min": "content" },
        "cuts": [
          { "side": "top", "size": "content", "slot": "peek_lines" },
          { "side": "rest", "slot": "peek_faces", "place": "bottom", "w": "fill", "h": "$face" }
        ] }
    ] },
    { "text": "Version dev · Aegis engine", "style": "small", "color": "dim",
      "anchor": "bottom_right", "margin": "$margin" }
  ] }
}
```

Contract: slots `menu` (required), `peek_lines`, `peek_faces`, `logo_typeset`; flag `peek`. `face` is `m.u(panes::PORTRAIT)`
and `face_gap` `m.u(PEEK_FACE_GAP)` (`panes.rs:37`, `menus.rs:85`). For R5 to measure `peek_lines`,
the host's `measure` returns two body lines plus the face gap. `peek_lines` comes first because it pushes `menu.peek` over
`Slot.host`, with its tip when the place name is cut, and the portraits' widgets must follow it, as in the pilot. Under `min: "content"`, the column measures through `measure` as the widest label plus two paddings.

Known difference: with no `ornate` frame the pilot drops the peek's padding (`menus.rs:212`) and the file does not; the shots use the
shipped theme, so parity holds. The typeset logo stays a slot; dropping it, now that `ui/logo_title.png` ships, is a separate cleanup.

## 12. Verified and assumed

Verified at `C:/dev/tactical-rpg-lane1` (M0, uncommitted, on `71e4c8b3`): every `file:line` above. `layout_audit.rs` exists in
neither lane 1 nor `C:/dev/tactical-rpg-stage`, so E5 is not built. `Part`'s fields are private (`aegis_layout/src/lib.rs:203`), and
`resolve` is one function (`:244`). Tests drive the shell with no window (`s.bridge`, `shell.rs:1744`). No row in `docs/TASKS.md`
lists LOOK1-M2 as a dependency.

Reasoned, not run: nothing was compiled. The main menu file is by hand against M0's numbers; G2 tests it.

Assumed: all run counts, including M2 at 2.5 in code (LAYOUT720 gave only "20-30 %" and no base); focus order follows widget push
order; the walk's cost per frame is negligible (a small tree of integer math; not measured); the timing of rule R2 suits every M2
screen.

## 13. Out of scope

Touch, portrait, the web port, `Fit` and `Dp` for this game; a UI scale setting; moving any screen not named here; a sweep; drag
handles; conditions beyond one flag; expressions; wrapped text in a `text` node; a data form for battle, region map or Dojo; merging
trees across layers; removing `menu_column`, `tab_rail` or the old mapping.

## 14. Open questions (recommendation first)

1. **Which form?** Recommend A, the tree with slots (section 2); a HISTORY row each for the builder convention and the script.
2. **M2 route?** Recommend B (section 7.2): M2 about 1 to 2.5 hours of lane time later and 0.5 run bigger, no rework later.
   Alternative A2 if you want M2 soonest and accept a logic builder for changes on those six screens.
3. **Convert the main menu as the proof?** Recommend yes, 0.75 run, with the stop rule of 7.1. You should see no change.
4. **Pull E5 forward?** Recommend yes (only E5, not G0). It changes the order in decision "Priorities after the visual pass".
5. **LAYOUT1's "resizable elements in the game".** Recommend not now: hot reload plus describing a change to an agent covers it.
   LAYOUT1 stays an idea for that one part only. Say if you want drag handles in the Gallery's Mock view; about 1 run.
6. **A new agent role, `ui-author`, or a mode of `asset-builder`?** Recommend a new role note: it reads pictures and audits, not
   rows. Editing agents and `CLAUDE.md` needs your approval either way.
