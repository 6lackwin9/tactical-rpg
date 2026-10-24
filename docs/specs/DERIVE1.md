# DERIVE1: data-defined derived stats

Status: proposed (spec 2026-10-25, from the user decision Q-47a: Composure returns as a derived stat, the corruption
resist, and the engine and kit gain data-defined derived stats so any game can add them as data)
Depends on: none (DERIVE1-E); the kit parts follow KIT1's crate paths if KIT1 lands first (section 8)
Lane: logic (E, S, R, U), then asset (D)
Touches: `engine/crates/aegis_core/src/{formula.rs,lib.rs}` (new module); `games/emberwake/schema.json`
(`content.derived_stats`); under `games/emberwake/crates/`: `emberwake_data/src/lib.rs`,
`emberwake_content/src/{source.rs,store.rs,check/*}`, `emberwake_rules/src/{derived.rs,unit.rs}`,
`emberwake_world/tests/data_source.rs`, `emberwake_gui/src/{party,dojo,battle}.rs`;
`games/emberwake/data/catalog/derived_stats.json` (new, asset lane)

History: `docs/HISTORY.md` (2026-10-25) lists "Composure as a unit stat: not adopted" because Composure was never one
of the six stats. That reason still holds and this spec respects it: Composure is not a seventh base stat but a value
computed from the six (Q-47a, `games/emberwake/docs/decisions/lore.md`). The gap 5 recommendation "drop it" in
`games/emberwake/docs/lore/specs/rpg_kit_and_system_architecture.md` is superseded by the same decision.

## 1. Goal

A game declares derived stats as catalog rows: an id, a name, a formula over the base stats and other derived stats,
caps, and where it shows. Gear, passives (which also carry traits, recruit quirks and Calcification stage effects) and
active statuses add flat and percent modifiers through the grammar they already use. Rules read a derived stat by id
and can run a seeded resist check against it. The engine gets only a game-agnostic formula evaluator. Emberwake's
Composure is the first row. Nothing in the Ashlands "classic" reads a derived stat, so every parity trace stays
the same.

## 2. Survey: how stats are computed today

### 2.1 Base stats and modifiers

| What | Where | Rule |
| --- | --- | --- |
| The six stats | `emberwake_data/src/lib.rs:13` `STATS` | STR DEX INT CONS PERC KNOW |
| Core values | `emberwake_rules/src/stats.rs:193` `CharacterStats::new` | class base + spend + class growth per level |
| Modifier | `stats.rs:81` `StatModifier` | `stat`, `flat` i32, `percent` f64, `source`, `context` (`weapon:<type>`), `duration`; `permanent` upper-cases `stat` (`stats.rs:94`) |
| Effective stat | `stats.rs:216` `effective` | `roundi((base + Σflat) × (1 + Σpercent))`, minimum 1, modifiers matching the weapon context |
| Gear bonuses | `unit.rs:209` `sync_from_stats_and_gear` | item `stat_bonuses` become source `gear` modifiers, then tier bonuses (`unit.rs:233`, read without tier modifiers, `gear.rs:113-175`), then gear effects |
| Passive effects | `unit.rs:279` `apply_effects` | `{"type": "stat", "stat", "value", "context"}` adds a flat modifier; `resist` adds per-status resist (`unit.rs:295`, read by `resist_for`, `unit.rs:753`) |
| Statuses | `status.rs:37` `ROWS` (Rust table) | change damage, armour and actions; none adds a stat modifier |
| Poise | none | does not exist yet (the timeline ruleset task adds it) |

### 2.2 Hard-coded derived numbers

| Number | Formula today | Code | Shown |
| --- | --- | --- | --- |
| Max HP | `60 + CONS × 8 + STR × 2`, × `hp_scale` | `stats.rs:236`, `unit.rs:226` | party, Dojo |
| Speed | `max(1, 8 + DEX − burden) − carry penalty` | `stats.rs:240`, `unit.rs:228` | party, Dojo, battle inspect |
| Armour rating | gear mitigation `+ roundi(STR × 0.25)`, `+ fortify − expose` | `stats.rs:244`, `damage.rs:122-134` | party, Dojo, Forge |
| Range bonus | `+1` if PERC ≥ 18 and base range > 1 | `stats.rs:248` | battle inspect (Reach) |
| Crit % | `5 + max(0, DEX − 10) × 1.0` + proficiency (f64) | `stats.rs:253`, `damage.rs:140` | party, Dojo, Forge |
| Debuff resist % | `INT × 2.5` (f64); status chance `clamp(potency − resist − per-status, 0, 100)` | `stats.rs:257`, `status.rs:193-213` | party, Dojo |

The party screen lists them in `emberwake_gui/src/party.rs:87` (`derived_of`) with hand-written tooltips in
`party.rs:113` (`stat_desc`) and `party.rs:128` (`derived_desc`); the Dojo in `dojo.rs:307-314`; battle inspect in
`battle.rs:1515`; the Forge in `forge.rs:154`.

### 2.3 Evaluators, constants, loading

- No arithmetic expression evaluator exists in `aegis_core` (modules: `lib.rs`), `aegis_content` or the game crates. The
  nearest things are data conditions: `quest.rs:17` `condition_met` (flag comparisons) and gear tier `when`
  (`gear.rs:113`). `aegis_core::json::roundi` (`json.rs:31`) is Godot's round half away from zero.
- Rule constants are Rust `const`s (`stats.rs:6-19`, `damage.rs:10-19`). `tuning.json` holds only battle feel values,
  read live by `aegis_ui::tuning::TuningFile` (`aegis_ui/src/tuning.rs:14`, `emberwake_gui/src/tuning.rs:1`).
- Catalog kinds load in `emberwake_content/src/source.rs:94` (`CATALOG_KINDS`, `source.rs:27`); a missing file is no
  error (`aegis_content/src/file.rs:20`). The writer maps kinds to files in `store.rs:146-165` (`BASE_KINDS`).
- Content checks reject unknown stat names in item modifiers (`check/item.rs:478`), tier `when` (`item.rs:560`), tier
  bonuses (`item.rs:634`), passive effects (`check/simple.rs:73,108`) and dialogue stat conditions (`dialogue.rs:258`).
- The loader's parity test compares Godot's schema whole and catalogs per `CATALOG_KINDS`; fields added after the
  freeze are stripped by `AFTER_GODOT` (`emberwake_world/tests/data_source.rs:20-34`, `87-96`).
- Out-of-battle rolls are per-save streams (`emberwake_world/src/rolls.rs:15-30`); battle rolls use the session's
  `GodotRng` (`status.rs:199`).

## 3. Engine: the formula evaluator `[engine]` (DERIVE1-E)

New module `engine/crates/aegis_core/src/formula.rs`. It knows names and numbers, never a stat or a game.

Grammar (whitespace ignored):

| Rule | Form |
| --- | --- |
| expr | term (`+` or `-` term)* |
| term | unary (`*` or `/` unary)* |
| unary | `-` unary, or primary |
| primary | number, name, call, or `(` expr `)` |
| number | digits with an optional `.` and 1-4 decimals; no exponent; at most 1000000 |
| name | `[A-Za-z_][A-Za-z0-9_]*`, not a function name |
| call | `min(a, b, ...)` and `max(a, b, ...)` (2-8 arguments), `clamp(x, lo, hi)`, `floor(x)`, `ceil(x)`, `round(x)` |

Rules:

1. Parse once: `Formula::parse(src) -> Result<Formula, FormulaError>`; the formula is immutable and `Clone`.
2. `Formula::names() -> &[String]`: every name used, first appearance order, no repeats (the caller validates them).
3. `Formula::eval(&self, input: &mut dyn FnMut(&str) -> f64) -> f64`, left to right, operands in source order.
4. Numbers are f64 inside (Q2); `round` is `roundi` (half away from zero); `x / 0` is 0; a result that is not finite is
   0. No NaN or infinity leaves `eval`. Callers turn the result into an integer with their own rounding.
5. Limits: source at most 256 characters, nesting at most 16, at most 64 tokens.
6. `FormulaError { column, message }`, displayed as `column 9: expected a number, a name or '('`. Errors: empty
   formula, unknown character, number format or size, unbalanced parenthesis, wrong argument count, unknown function
   (`pow(...)`), a name used as a call, `/ 0` with a literal zero, limits exceeded, trailing tokens.
7. `Formula::source() -> &str` returns the text as written (the UI shows it).

Tests (unit tests beside the code, as the rest of `aegis_core`): precedence and associativity against the same sum
written in Rust over random inputs (`a - b - c`, `a + b * c`, `a / b * c`); unary minus; each function's identity
(`min(x, y) <= x`, `clamp(x, lo, hi)` within bounds, `round` equals `json::roundi`); `x / 0 == 0`; every error kind
above with its column; `names()` order and uniqueness; parse twice gives the same result for the same inputs.

## 4. Kit hook: derived stats as data `[kit hook]` (DERIVE1-S, DERIVE1-R)

Written story-free in today's genre crates (`emberwake_data`, `emberwake_content`, `emberwake_rules`) so KIT1 moves it
unchanged. No game name, no Composure in code.

### 4.1 Schema contract (`content.derived_stats`, file `data/catalog/derived_stats.json`, schema-first: yes)

| Key | Type | Default | Validation |
| --- | --- | --- | --- |
| `id` | id (key field) | "" | `[a-z][a-z0-9_]{1,23}`; unique; not a base stat (any case), not `level`, not a function name |
| `name` | string | "New Stat" | 1-24 characters |
| `label` | string | "" | at most 16 characters, unique among rows; empty = `name` (table header) |
| `description` | text | "" | the tooltip's first line |
| `formula` | string | "0" | parses (section 3); every name is a base stat, `level`, another row's id or a key of `constants` |
| `constants` | json | {} | object of name to number; names follow the name rule and collide with no base stat, row id or `level`; numbers finite, at most 1000000 in size |
| `min` | int | 0 | `min <= max` |
| `max` | int | 100 | as above |
| `status_modifiers` | json | [] | list of `{status, flat, percent}`; `status` in the status table (`status.rs:37`); `flat` int −100..100, `percent` float −1.0..1.0, not both 0; one entry per status |
| `rulesets` | strings | [] | empty = every ruleset; values from the ruleset option (`classic`, `timeline`) (Q3) |
| `show_on` | strings | ["party", "dojo"] | subset of `party`, `dojo`, `battle` |
| `order` | int | 0 | display order among derived rows; ties keep file order |
| `suffix` | enum | "" | `""` or `"%"` (display only) |
| `origin` | enum | "system" | as every catalog kind (system, user) |

Cross-row validation (DERIVE1-S, `emberwake_content/src/check/derived.rs`):

- Cycles: the rows form a graph by the derived ids their formulas name; a cycle is an error on each row in it, naming
  the path (`grit -> composure -> grit`). The loader orders rows topologically (ties in file order).
- Stat fields accept a derived id once a row defines it: item `stat_bonuses` and modifier `stat_bonus`
  (`check/item.rs:478`), tier bonuses (`item.rs:634`), passive `stat` effects (`check/simple.rs:108`). Tier `when`,
  weapon scaling and dialogue conditions keep the six base stats (out of scope, section 9).
- Passive `stat` effects gain an optional `percent` (float −1.0..1.0), accepted only when `stat` is a derived id; base
  stats keep flat values only (the classic grammar is unchanged).
- Loading: a row that fails is skipped with an error in `Source.errors` (the loader's rule); the typed catalog holds
  only valid rows with their parsed `Formula`. The writer takes the kind (`BASE_KINDS`); `aegis content check` reports
  the same errors.

### 4.2 Value rule (DERIVE1-R, `emberwake_rules/src/derived.rs`)

Inputs of a unit's formula: the six base stats as `CharacterStats::stat` gives them (effective, in the unit's weapon
context), `level`, the final value of every derived row it names, and the row's `constants`.

```text
raw   = formula(inputs)                                   (f64, section 3)
flat  = Σ flat of the unit's modifiers whose stat is the id (case-insensitive; gear, tier, passive sources)
      + Σ flat of status_modifiers whose status is active on the unit
pct   = Σ percent of the same modifiers and active status_modifiers
value = clamp(roundi((raw + flat) × (1 + pct)), min, max)
```

The order follows `CharacterStats::effective` (`stats.rs:216`) with the row's caps in place of the minimum 1. Values
are computed on read and never stored on `Unit`, `UnitState` or a save. A row whose `rulesets` excludes the current
ruleset reads as absent (`None`), so no rule and no screen sees it.

API: `derived::value(cat, unit, id) -> Option<i32>`; `derived::breakdown(cat, unit, id) -> Option<Breakdown>` (raw,
each input with its value, each flat and percent part with its source name, the cap hit if any, the value), for the
tooltips and the Dojo.

### 4.3 Resist checks (DERIVE1-R)

A derived row with a `min` of 0 and a `max` of 100 can serve as a resist. Recommended rule (Q1, option a), the same
shape as the debuff roll (`status.rs:193-213`):

- `derived::resist_chance(cat, unit, id, potency) -> i32 = clamp(potency − value, 0, 100)`: the chance the effect
  lands.
- In battle the caller draws `randi_range(1, 100) <= chance` from the session's `GodotRng` when the effect resolves:
  one draw per check of a living target, whatever the values. Out of battle the caller uses `rolls::chance(gs,
  "resist", chance)` on a new stream `resist` (no existing stream's counter moves).
- An absent row (ruleset off) means the effect lands with its `potency` as the chance, so a consumer works in both rule
  sets.
- Consumer shape (reserved for the hazard, nightmare and quirk tasks; validated by `derived::check_errors`):
  `{"resist": "<derived id>", "potency": 1-200}`. DERIVE1 ships the helper, no consumer.

### 4.4 Display (DERIVE1-U, `[game code]` on the screens)

- Party screen: after the built-in rows of `derived_of` (`party.rs:87`), one row per derived row with `party` in
  `show_on`, sorted by `order`, header `label` or `name`, value with `suffix`. The detail pane (`derived_desc`,
  `party.rs:128`) shows `description`, the formula text, then the breakdown (`KNOW 14 × 2 + CONS 12 = 40; Salt charm
  +3; Vein-Blackening −5; = 38 (0-100)`).
- Dojo (`dojo.rs:307`) for `dojo`, battle inspect (`battle.rs:1515`) for `battle`, the same text in their tooltips.
- No layout change when no row is shown (the classic screens look as today).

## 5. Data: Composure `[data]` (DERIVE1-D)

| Key | Value |
| --- | --- |
| `id` | `composure` |
| `name` / `label` | Composure / Composure |
| `description` | "Resists corruption: the Strain of the deep, the bells, the titan's tremors and the nightmares." |
| `formula` | `base + KNOW * per_know + CONS * per_cons` (placeholder shape) |
| `constants` | `{"base": B, "per_know": K, "per_cons": C}`: the balance lane sets B, K, C; first sim guess B 0, K 2, C 1 |
| `min` / `max` | 0 / 100 (a d100 resist, section 4.3) |
| `status_modifiers` | [] (none named by CANON yet) |
| `rulesets` | per Q3 (recommended `["timeline"]`) |
| `show_on` | `["party", "dojo", "battle"]` |
| `order` / `suffix` | 0 / "" |

Why KNOW and CONS, KNOW weighted: KNOW today only shortens skill unlocks (`stats.rs:46`), so Composure gives it a
combat role; CONS is the body holding against calcification, but CONS already drives HP and the planned Poise, so a
lower weight keeps it from dominating; INT is left out because it already drives debuff resistance (`stats.rs:257`),
which would let one stat answer both resists. With stats 6-18 the first guess spans about 18-54 before gear, so
Calcification I's −5 (CANON 2.3) is a felt but not crippling loss.

Modifier sources, all through existing grammar once DERIVE1-S lands (each its own later data task): Calcification I
as a stage passive `{"type": "stat", "stat": "composure", "value": -5}`; Preservationist gear and salt charms as item
`stat_bonuses`; recruit quirks and the Martyr mercy reward as passives. Asset path:
`tools/fc.sh content upsert derived_stats @<file>` with the row above, then `tools/fc.sh content check`.

## 6. The classic stays byte-identical; later migrations

- No classic rule reads a derived value; nothing is stored or saved; no RNG draw happens unless a consumer checks.
- `legacy/data` has no `derived_stats.json`, so the loader parity test sees no new error and no new catalog kind
  (`CATALOG_KINDS` does not gain it; the rows ride beside it like `item_families`).
- `data_source.rs` compares Godot's schema whole: DERIVE1-S strips the new kind `content.derived_stats` from the
  engine's schema before comparing (a kind-level twin of `AFTER_GODOT`), named in its commit and a HISTORY row. Every
  key compared today is still compared.
- `checks_parity`: Godot's data names no derived id, so its reports are unchanged.
- Trace, world, journey, save and sim parity run untouched; DERIVE1 edits none of their expectations.
- Switch-off (the user rule for compatible additions): `rulesets` without `classic`, or no row, and no screen or rule
  sees the stat.

Hard-coded numbers that could later move onto the system (not in DERIVE1; one task each, the parity suites as proof):

| Number | Needs first |
| --- | --- |
| Max HP | non-stat inputs (`hp_scale`) from the kit |
| Speed, tiles per move | non-stat inputs (burden, carry penalty, haste and slow) |
| Armour rating | gear mitigation as an input; fortify and expose already fit `status_modifiers` |
| Crit %, debuff resist % | fractional values (they are f64 today): a `decimals` field |
| Range bonus | a comparison function (`step(x, edge)`) |

Recommendation: keep them in code until a second game or a ruleset needs to tune them.

## 7. Tests (rule-derived)

| Task | Test | Expectation from |
| --- | --- | --- |
| E | section 3 list | the grammar and rules 1-7 |
| S | a scratch data folder with two rows naming each other | one cycle error per row, naming the path |
| S | rows with each section 4.1 violation | one error each, the field named |
| S | a passive `stat` effect naming a derived id; a `percent` on a base stat | first accepted, second rejected |
| S | `legacy/data` load and `data_source` | no errors; parity unchanged |
| R | a built unit with no modifiers | `value` equals the formula evaluated over `stats.stat(...)`, clamped |
| R | add a passive `stat` +k on the id | value rises by k unless a cap is hit |
| R | a `status_modifiers` entry | applies only while the status is active |
| R | caps | `min <= value <= max` over random stats 1-40 |
| R | `resist_chance` | equals `clamp(potency − value, 0, 100)`; absent row gives `potency` clamped |
| R | a seeded battle with a check vs the same without | one extra draw per check; no check, identical draws |
| U | a test row in a scratch catalog | the party table gains the row; with none, the same rows as today |
| D | `tools/fc.sh content check` | clean |

## 8. Tasks

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| DERIVE1-E | Formula evaluator (section 3) | engine | logic | - | `aegis_core/src/{formula,lib}.rs`; README delta |
| DERIVE1-S | Derived-stat rows: schema, typed rows, loader, checks, writer, parity strip (4.1, 6) | kit | logic | DERIVE1-E | `schema.json`, `emberwake_data`, `emberwake_content/src/{source,store,check/*}`, `data_source.rs` |
| DERIVE1-R | Derived-stat rules: value, breakdown, resist chance (4.2, 4.3) | kit | logic | DERIVE1-S | `emberwake_rules/src/{derived,unit,lib}.rs`, `emberwake_rules/tests/derived.rs` |
| DERIVE1-U | Derived stats on the party screen, Dojo and battle inspect (4.4) | game | logic | DERIVE1-R | `emberwake_gui/src/{party,dojo,battle}.rs` |
| DERIVE1-D | The Composure row (section 5) | game | asset | DERIVE1-S | `data/catalog/derived_stats.json` |

If KIT1 lands before S or R, their files go to the kit crates KIT1 names (same modules). Docs deltas: STRUCTURE rows
for `formula.rs` and `derived.rs`, `GAME_BRIEF.md` stats section, and an authoring note "derived stats as data" in
`engine/docs/authoring/` (the generic method, per the promote-generic-methods practice). The balance lane sets
Composure's constants when the first corruption consumer exists (no sim can measure it before).

## 9. Out of scope

- Every consumer: the chimes, the shockwave, nightmares, the Martyr reward, recruit quirks, Calcification stages.
- Poise and the ruleset option themselves; a per-unit stage track as a formula input (stage effects arrive as passives).
- Derived ids in tier `when` conditions, weapon scaling, dialogue stat conditions and the stat budget.
- Migrating any hard-coded number (section 6); the Content Forge's editor for the new kind beyond what the schema gives.

## 10. Open questions (recommended option first)

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| DQ1 | What "resist" means mechanically | (a) a seeded d100: chance = potency − value, like debuffs; (b) a threshold, no roll: the effect lands if potency > value; (c) a scale: the effect is cut by value % | (a): one rule players already learn from debuffs; (c) suits poise losses if randomness feels bad in playtest |
| DQ2 | Number type inside formulas | (a) f64 inside, integer result by `roundi` (as `stats.rs:216` does); (b) fixed point (thousandths in i64) | (a): matches every derived number today, so later migrations stay byte-identical; IEEE + - × ÷ are deterministic in Rust |
| DQ3 | Is Composure on in the Ashlands | (a) `rulesets: ["timeline"]`, off in the classic and on with the Dojo's ruleset toggle; (b) on everywhere, display only in the Ashlands | (a): the classic screens stay as they are and nothing there uses it |
| DQ4 | Composure's starting shape | (a) KNOW-weighted KNOW + CONS; (b) KNOW only; (c) KNOW + INT | (a), for the reasons in section 5; the balance lane sets the numbers |
| DQ5 | Percent modifiers from passives | (a) an optional `percent` on passive `stat` effects, derived ids only; (b) flat only, percent only from statuses | (a): CANON's uses are flat today, but quirks may want "−10% Composure" |
