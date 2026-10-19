# SKILLUI1: Skills tab drag and drop and rank choice

Status: proposed (spec 2026-10-20; the user answers section 9 before the build)
Depends on: UIKIT2 (engine drag and drop) for SKILLUI1-U; SKILLUI1-L has no dependency
Lane: logic (two parts: SKILLUI1-L rules, state, save and battle; SKILLUI1-U the Skills tab). No asset part.
Touches: SKILLUI1-L `games/emberwake/crates/emberwake_rules/src/unit.rs`, `emberwake_data/src/lib.rs` (one catalog
helper), `emberwake_world/src/{state,save,progression}.rs`, new `emberwake_world/src/skill_text.rs`,
`emberwake_battle/src/plan.rs`, tests; SKILLUI1-U `games/emberwake/crates/emberwake_gui/src/party.rs` (Skills tab only)

Sources: decision "Skill rank choice (user playtest 2026-10-20)" (`games/emberwake/docs/decisions/skills.md`),
preference "Character popup" (`games/emberwake/docs/preferences/ui.md`), Godot reference
`legacy/scripts/ui/menus/{loadout_model,loadout_view}.gd` (no rank choice there).

## Goal

The Skills tab shows the loadout as fixed slots (5 class, 3 extra) beside the owned skill list. A skill is dragged from
the list onto a slot, between slots or back to the list (clicks keep working). Each filled slot has < > arrows over the
ranks the hero has unlocked; a newly slotted skill starts at its highest unlocked rank. The chosen rank is saved per
member and is the rank the hero uses in battle.

## 1. What the battle does today (why section 9 Q1 matters)

Today a party hero's battle menu lists every unlocked rank of every slotted family as separate entries (`Book::available`
filters by `Unit::can_use`; "Mend", "Mend II", "Mend III" all appear), and the hero AI picks among them (the AI tries
the cheapest heal first). ROADMAP "Open engine items" still lists "Skill-tier picker is not rendered in the planner UI".
The spec below is written for the recommended answer to Q1 (option A: the chosen rank is the only rank of that family a
hero can use in battle); options B and C change only rule R3.

## 2. Schema contract (SKILLUI1-L)

Schema-first: yes (no catalog or `schema.json` change; member state and save only).

| Where | Field | Type | Default | Meaning |
| --- | --- | --- | --- | --- |
| `UnitState` (`state.rs`) | `loadout_ranks` | `IndexMap<String, i64>` | empty | family id -> chosen rank; a family without an entry uses its highest unlocked rank |
| save, member dict | `loadout.ranks` | object `{family: int}` | absent | written only when non-empty (a member without pins saves exactly as today) |
| `Unit` (`emberwake_rules`) | `rank_choice` | `Option<IndexMap<String, i32>>` | `None` | `None` = rule off (catalog units, enemies, guests, sims); `Some` = a hero built from member state |
| `Catalog` (`emberwake_data`) | `fn family_rows(&self, family) -> Vec<&SkillDef>` | helper | - | the family's rows sorted by `tier` |

Read: `from_dict` reads `loadout.ranks` with `json::ints`; missing or not an object = empty. Save version: `VERSION`
10 -> 11 with a no-op step `10 => {}` (nothing to add: absent means "top rank"); the bump makes a pre-SKILLUI1 build
refuse a save that may carry pins ("newer version") instead of dropping them silently.

Validation (`UnitState::loadout_errors`, same message style as today), per entry:

- V1 the family is owned and not basic: `"{at}: rank set for family '{f}' that is not owned."` /
  `"... '{f}' is a basic family (basics have one rank)."`
- V2 the family has a row with `tier == rank`: `"{at}: family '{f}' has no rank {r}."`

Not errors (resolved at build, rule R2): an entry for a family that is not in the active loadout (ignored), a rank above
the highest unlocked rank (clamped down).

## 3. Rules (SKILLUI1-L)

- R1 Selectable ranks of a family for a unit: `Unit::selectable_ranks(cat, family) -> Vec<i32>`, ascending, the tiers
  of `family_rows` the unit owns (`owns_skill`) and has unlocked (`tier_unlocked`). Class-derived heroes own whole
  ladders, so this is tiers 1..top; an explicitly learned row (`learned_skills`) gives only that row's tier.
- R2 Effective rank: `Unit::effective_rank(cat, family) -> Option<i32>`: no entry -> the highest selectable rank; entry
  `c` -> the highest selectable rank <= `c`, else the lowest selectable rank; `None` when nothing is selectable (the slot
  shows "locked" and the family is unusable, as today).
- R3 Battle (Q1 option A): `Unit::can_use` gains `rank_allowed`: true when `rank_choice` is `None`, for basics, system
  and always-active skills, and for families outside the active loadout (they fail `knows_skill` anyway); otherwise true
  only for `skill.tier == effective_rank(family)`. `Plan::apply` refuses another rank with
  `"{hero} uses {family name} at Rank {roman} (set on the Skills tab)."` (after the existing unlock check). The host's
  `skills` command, the battle menu and the hero AI all go through `can_use`, so each slotted family appears once.
- R4 Build: `progression::build_with` sets `rank_choice = Some(state.loadout_ranks)` after `set_loadout` (gear and
  passives are applied first, so unlocks are final). Ranks are fixed for the battle like the loadout (SK2-L "frozen while
  the loadout is locked"). A family learned mid-battle (`learn_skill`) has no entry and uses its top rank (R2).
- R5 Following the top: an unpinned slot (no entry) moves up by itself when a new rank unlocks (Q2). A pinned slot stays
  at its rank. The Skills tab never stores the top rank: stepping up to the highest selectable rank removes the entry.
- R6 Unslotting a family removes its entry; slotting a family that was not slotted adds none (decision: "a newly slotted
  skill starts at its highest unlocked rank"). Moving a family between slots keeps its entry.
- R7 Scope: the rule is on only for units built from member state (`progression::build_with`). Catalog units, world
  characters, enemies, guests and balance sims (`emberwake_battle::sim`) keep today's behaviour, so the BALANCE_REPORT
  numbers stand and enemy AI still picks any unlocked tier.
- R8 Rank text: `skill_text::rank_lines(cat, unit, family) -> Vec<RankLine>` with `{tier, unlocked, unlock_level, ap,
  text}`: `ap` from `emberwake_battle::skill::ap_cost(row, Some(unit))` (the battle's own number); `text` lists AP,
  blocks and only the fields that differ between the family's rows, using the table below; a locked rank's text is
  `"unlocks at Lv {n}"` (`skill_unlock_level`).

| Field(s) | Text |
| --- | --- |
| `power_mult` | `power x{:.2}` |
| `heal_base`, `heal_mult`, `heal_stat` | `heal {base} + {mult:.2} x {stat}` |
| `status_effect`, `status_potency`, `status_magnitude`, `status_rounds` | `{status} {potency} ({magnitude}) for {rounds} rounds` |
| `mitigation` | `blocks {pct}% damage` |
| `boost`, `boost_rounds` | `attack x{:.2} for {n} rounds` |
| `grant_status`, `extra_grants` | `grants {status}` plus `+{n} more effects` |
| `cleanse_statuses` | `cleanses {a, b}` |
| `reach_min`, `reach_max`, `area` | `reach {a}-{b}`, `area {area}` |
| `dash_tiles`, `jump_tiles`, `recover_ap` | `dash {n}`, `jump {n}`, `+{n} AP back` |

## 4. Skills tab (SKILLUI1-U)

Canvas 640x360; frame and tabs as today; content band y 102-266; detail box y 275; buttons y 312.

| Part | Rect / place | Content |
| --- | --- | --- |
| Class header | x 64, y 104 | `Class slots {n} / 5` (INFO); right of it `Default loadout` or `Custom loadout` (DIM) |
| Class slots | x 64, y 116 + i x 17, 236 x 15, i 0-4 | filled: name (fit 112 px), `<` 13 px at +120, rank numeral +134 (22 px), `>` 13 px at +157, `{ap} AP` at +174, `x` 14 px at +219; empty: dim border, `Empty` |
| Extra header | x 64, y 205 | `Extra slots {n} / 3` (INFO) |
| Extra slots | x 64, y 216 + i x 17, i 0-2 | as class slots |
| Owned list | x 310-604, y 104 header `Owned skills (drag onto a slot or click)`, rows from y 118, step 22, 6 rows, wheel scrolls | `Slot` / `Slotted` / `Full` button (50 px) and `Name [Grade] Rank {top}` with the top rank's description (SMALL) under it, as today |
| Buttons | y 312 | `Reset`, `Use default`, `Confirm` (as today) |

Slots are compact: filled slots first in loadout order, then empty ones; a drop on an empty slot appends.

Widgets, drag payloads and drop keys:

| Widget id | Kind | Drag payload | Drop key | Click |
| --- | --- | --- | --- | --- |
| `skills.family.<f>` (list row) | row | `skill.<f>` | - | slot `f` (as `skills.slot.<f>`) |
| `skills.slot.<f>` | button | - | - | slot `f` into the first free slot of its group (today's `PAct::Slot`) |
| `skills.list` (whole list column, recorded under the rows) | area | - | `list` | - |
| `skills.<g>.<i>` (`g` = class or extra) | row | `slot.<g>.<i>` when filled | `slot.<g>.<i>` | none (hover fills the detail pane) |
| `skills.rank.<g>.<i>.down` / `.up` | button | - | - | step to the previous / next selectable rank |
| `skills.rank.<g>.<i>` (numeral) | row | - | - | none (hover card) |
| `skills.unslot.<g>.<i>` | button `x` | - | - | unslot (today's `PAct::Unslot`) |

`can_drop(from, to)` and `accept_drop(from, to)`:

| From | To | Allowed when | Result |
| --- | --- | --- | --- |
| `skill.<f>` | `slot.class.<i>` | `f` is on the class path (Q3 option A) | empty: `f` appended; filled: `f` replaces it, the old family goes back to the list (its rank entry dropped, R6); if `f` was in another slot it leaves that slot (it keeps its entry) |
| `skill.<f>` | `slot.extra.<i>` | `f` owned, not basic | same as above |
| `slot.<g>.<i>` | `slot.<h>.<j>` | not the same slot; each family fits its new group | filled: the two swap; empty: move (append to `h`) |
| `slot.<g>.<i>` | `list` | always | unslot (entry dropped) |
| anything else | - | never | - |

Click path without drag: clicking a list row or its `Slot` button puts the family into the first free slot of its group
(class path family -> class slots, else extra; Q3 option A also lets a class family use a free extra slot when the class
slots are full); a full group shows the notice `"{Group} slots are full: drag onto a slot to replace one, or take one
out with x."`. `x` unslots. Every slot change sets `using_default = false` (as today); a rank change does not (ranks
are kept with the default loadout too, R2 ignores entries for inactive families). `Use default` drops entries of
families outside the default set. `Reset` reloads slots and ranks from the member; `Confirm` writes `loadout_class`,
`loadout_extra` and `loadout_ranks` after `loadout_errors` passes. `has_draft` also compares the ranks.

Rank arrows: `<` and `>` step over `selectable_ranks` (R1) of the member built with confirmed gear; disabled (DIM) at the
ends, no wrap; both disabled with one selectable rank. The numeral is TEXT when the slot follows the top (no entry) and
GOLD when pinned below the top. Stepping up onto the top rank removes the entry (R5).

Hover texts (tip title in GOLD, then lines; the detail pane shows the second line, PARTY2-B style):

| Widget | Title | Lines |
| --- | --- | --- |
| `<` / `>` | `Rank {roman}` | the target rank's `rank_lines` text; `Cheaper: {ap} AP (now {ap})` or `Stronger: ...` |
| numeral | `{name}: Rank {roman} of {top}` | one line per rank of the family (locked ranks `unlocks at Lv {n}`), the chosen one marked `(chosen)`; then `Follows the top rank: moves up when a new rank unlocks.` or `Kept at Rank {roman} when new ranks unlock.` |
| slot (filled) | `{name}` | the effective rank's description; `Drag to another slot or back to the list.` |
| slot (empty) | `Empty {group} slot` | `Drag a skill here, or click a skill in the list.` |

While dragging (`ui.dragging()` is `Some`), the detail pane reads `Drop on a class slot (class path skills) or an extra
slot; drop on the list to take it out.`; over a filled valid target it reads `Replaces {name}.` (or `Swaps with {name}.`).

`Screen::state` adds `{"class": [...], "extra": [...], "ranks": {f: r}, "using_default": bool}` (the draft) so the
agent link can assert the tab without reading pixels.

## 5. Acceptance checks (rule-derived test names)

SKILLUI1-L (`emberwake_rules`, `emberwake_world`, `emberwake_battle` tests):

- `catalog_unit_keeps_every_unlocked_tier` (R7): `rank_choice` None, a family with 3 unlocked tiers lists 3 skills.
- `member_without_ranks_uses_the_top_rank` (R2, R4): one entry per slotted family, at `max(selectable_ranks)`.
- `chosen_rank_is_the_only_usable_tier` (R3): pin 1 of 3; `can_use` true only for tier 1; `skills` lists tier 1 only.
- `planner_refuses_another_rank` (R3): planning tier 2 with tier 1 pinned returns the R3 message.
- `pin_above_unlocked_falls_to_highest_below` (R2): pin 3, level allows 2 -> effective 2.
- `learned_row_selects_only_its_tier` (R1).
- `basics_ignore_the_rank_rule` (R3).
- `family_learned_mid_battle_uses_its_top_rank` (R4).
- `unpinned_slot_moves_up_on_level_up` and `pinned_slot_stays_on_level_up` (R5).
- `rank_entry_for_unowned_family_is_an_error`, `rank_without_a_tier_row_is_an_error` (V1, V2).
- `ranks_round_trip_through_a_save` and `member_without_ranks_saves_no_ranks_key` (section 2).
- `version_10_save_loads_unchanged` (no-op step; Godot fixtures load as before).
- `hero_ai_uses_only_chosen_ranks` (R3, a member-built priest with Mend pinned at 2 never plans Mend I or III).
- `rank_lines_list_only_changing_fields` (R8, power_mult differs, reach equal -> no reach text).

SKILLUI1-U (agent link, by widget id; `state` after each step):

- drag `skills.family.<f>` to `skills.extra.0` -> `extra` holds `f`, no rank entry; Confirm -> member saved.
- click `skills.slot.<f>` with class slots full -> notice shown, loadout unchanged.
- drag a list family onto filled `skills.class.1` -> replaced, the old family's entry gone.
- drag `skills.class.0` to `skills.class.2` -> swapped, entries kept; drag `skills.class.0` to `skills.list` -> unslotted.
- drag a non-path family to `skills.class.0` -> `can_drop` false, nothing changes.
- `skills.rank.class.0.down` -> `ranks` holds `{f: top-1}`; `.up` back to the top -> entry removed.
- a press on `skills.family.<f>` without a move slots it (click path); Esc during a drag changes nothing.

## 6. Parity note

Godot had no rank choice (`loadout_view.gd` shows only the active rank): this is an intended design change. Expected
parity impact is none: the rule is off for every unit the parity suites build (battle traces and world traces use
catalog units and world characters, not member state, R7), and Godot save fixtures carry no ranks (they load through the
no-op step and write no `ranks` key). The builder runs the whole gate; if a parity check still fails because of the
rank rule, it is retired by name in the commit with a HISTORY row (CLAUDE.md "Godot parity"), never re-recorded.

## 7. Out of scope

- A rank picker inside the battle (option A makes it unnecessary for heroes); enemy rank choice.
- Loadout presets, keyboard rank shortcuts, wheel-on-slot rank change.
- The Gear tab (GEARUI1), the detail pane on other tabs (PARTY2-B), the dojo (PORT-DOJO).
- Catalog unit `loadout` ranks in data, `schema.json`, family rename carrying member ranks (members are save data).
- Balance changes; no new sim run (R7: sims are unaffected).

## 8. Task split

| Id | Layer | Lane | Depends on | Notes |
| --- | --- | --- | --- | --- |
| SKILLUI1-L | game | logic | user answers (section 9) | sections 2, 3, 5 (L tests); no GUI |
| SKILLUI1-U | game | logic (UI/UX) | SKILLUI1-L, UIKIT2, PARTY2-B (same file) | sections 4, 5 (agent-link checks); serialize with GEARUI1 (same file) |

## 9. Questions for the user

1. In battle, what does the chosen rank do? A (recommended): it is the only rank of that skill the hero can use (one
   menu entry per slot; the choice is made before the fight like the loadout; "Skill model v2" planning-time tiers then
   apply only to enemies). B: it is a ceiling (lower ranks stay usable in battle). C: it is only the default shown first
   (every unlocked rank stays in the battle menu, as today).
2. When a new rank unlocks, does a slot left at the top move up by itself? Recommended: yes; a slot set lower stays put.
3. Which skills fit which slots? A (recommended): class slots take class path skills only, extra slots take any owned
   skill (a sixth class skill can use an extra slot). B: class slots path only, extra slots non-path only. C: any skill
   in any slot.
4. Click path when a group is full: recommended a notice (drag replaces, `x` frees a slot); alternative: click a slot to
   select it, then click a skill to put it there.

### Answers (user 2026-10-20; these override the sections above where they differ)

1. Q1: A. The chosen rank is the only rank of that skill the hero can use (R3 as written).
2. Q2: never. A slot never moves up by itself. R5 and R6 change: slotting a family stores its highest unlocked rank at
   that moment as its entry, stepping to the top rank keeps the entry (it is never removed), and Confirm writes an entry
   for every slotted family. A family without an entry (a save from before SKILLUI1, or a class default loadout the
   player never confirmed) uses its highest unlocked rank (R2) until the tab is confirmed. The numeral is GOLD when the
   entry is below the highest unlocked rank. Tests: `unpinned_slot_moves_up_on_level_up` becomes
   `slot_keeps_its_rank_on_level_up`; `slotting_stores_the_highest_unlocked_rank` is added.
3. Q3: A. Class slots take class path skills only; extra slots take any owned skill.
4. Q4: a notice when the group is full, plus: right click on a filled slot takes the skill out (in place of the `x`
   button; the slot's hover card says so), and a `Clear all` button empties every class and extra slot in the draft
   (Reset and Confirm as for any draft). The notice reads `"{Group} slots are full: drag onto a slot to replace one,
   or right click a slot to take it out."`. The screen's `back` (right click) handler unslots the slot under the mouse
   and only closes the popup when the mouse is over no slot.
