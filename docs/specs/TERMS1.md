# TERMS1: neutral term ids, the game's terms table, the lore-word layer check and the asset layering

Status: proposed (spec 2026-11-01, from decision "INPUT1 answers, part 3" (names and assets layering) and part 2 Q13
in `games/emberwake/docs/decisions/combat.md`, and the TASKS row TERMS1)
Depends on: CAMPAIGN1's purse block (built: `rpgkit_data::Purse`, `campaign::purse::purse`, `hud::purse_word_in`);
CAMPAIGN1-U for the Results and HUD screens it shares; INPUT1-U for the battle screen's intent labels
Lane: logic (kit, then game screens, then the tools check); no asset task now (section 5.3)
Touches: by slice, section 11
Paths: `rpgkit_*` crates are under `kits/rpg/crates/`, `emberwake_*` under `games/emberwake/crates/`.

History: `docs/HISTORY.md` holds no row on term names or lore words in code. CAMPAIGN1 P1 already names the currency
and the upkeep per world (the manifest's `purse` block) and HOOKS1 CN1 the classes per world (`classes`); this spec
builds on both and adds no second place for them.

## 1. Goal

Kit system code names a game-system word only by a neutral term id (`currency`, `upkeep`, the intents) with a plain
default, read through one resolver. The game maps term ids to its lore names in a `terms` table in its data, written
through `aegis content`. Results and the journal read the resolver, so a world whose purse is Scrap says "Scrap +12".
The layer check also fails on the game's lore words in kit code. The Ashlands texts and every classic text stay as
they are.

## 2. Survey (develop at 955e0d1; string literals outside tests and comments)

| Where | What is spelled out | Count |
| --- | --- | --- |
| `rpgkit_data/src/lib.rs` | `CURRENCY_NAME` "Gold", `UPKEEP_NAME` "Supplies", icons and tips (the purse defaults) | the defaults |
| `rpgkit_world/src/shop.rs` | "Not enough gold." (x3), "Gold is capped at ..." (x2), "You can carry at most N supplies." (x2) | 7 |
| `rpgkit_world/src/quest.rs`, `dialogue.rs` | "Not enough gold." (x2), "Gold is capped at ..." | 3 |
| `rpgkit_world/src/explore.rs`, `progression.rs` | "Supplies are capped at ...", `REFUSE_SUPPLIES` "Making camp needs 3 supplies." | 2 |
| `rpgkit_battle/src/round_intent.rs` | `RoundIntent::name()`: Assault, Guard, Support, Maneuver, Channel (log.rs and metrics.rs read them) | 5 |
| `emberwake_gui/src/hub.rs` | Results "Gold +N", "Objective bonus: Gold +N"; journal rewards "N gold" | 3 |
| `emberwake_gui/src/shop.rs`, `party.rs`, `region.rs` | "Supplies" labels, "N gold", "Total: N gold", "Gold: a -> b", "Not enough gold: ...", "Level ... Gold N" | 12 |
| `emberwake_gui/src/hud.rs`, `worldmap.rs`, `region.rs`, `runover.rs` | already through `purse_word` / `purse_word_in` (CAMPAIGN1 P1) | 14 calls |
| `emberwake_gui/src/battle.rs`, `intent_text.rs` | intent names on the battle screen | 4 |
| Kit `src/` lore words | one doc comment, `rpgkit_battle/src/log.rs:341` ("Nemeth and Guard fall.") | 1 |
| Kit `tests/` lore words | fixtures named Scrap, Warded Tallow, The Red Tithing, Kaelen, The Crucible | about 20 |

Not terms (unchanged): data keys and dev-facing errors that name a field (`party.gold`, "Rewards gold must be a whole
number", "gold must be 0..9999" in save validation, content check texts), ids in traces, sims, saves and metrics.

## 3. The term register (kit, `rpgkit_data::terms`)

- TM1. One register in the kit: `TermDef { id, name, short, word, tip, scope }` per term; `scope` is `world` (named per
  world by an existing world block) or `game` (named by the game's `terms` table). The register is append-only: a
  term joins it with the task that first needs it.
- TM2. The defaults are today's texts, so with no data every text is unchanged.

| Id | `name` (title) | `short` | `word` (in a sentence) | Scope | Named by |
| --- | --- | --- | --- | --- | --- |
| `currency` | Gold | Gold | gold | world | the manifest's `purse.currency` (CAMPAIGN1 P1) |
| `upkeep` | Supplies | Supplies | supplies | world | the manifest's `purse.upkeep` |
| `intent_assault` | Assault | Assault | assault | game | `terms` |
| `intent_guard` | Guard | Guard | guard | game | `terms` |
| `intent_support` | Support | Support | support | game | `terms` |
| `intent_maneuver` | Maneuver | Maneuver | maneuver | game | `terms` |
| `intent_channel` | Channel | Channel | channel | game | `terms` |

`currency` and `upkeep` keep `CURRENCY_NAME`, `CURRENCY_TIP`, `UPKEEP_NAME` and `UPKEEP_TIP` as their defaults (the
register points at them, no copy); `RoundIntent::name()` returns the register's default name.

## 4. One resolver

- TM3. `Term { name, short, word, tip }`. Kit code with a world calls `rpgkit_world::terms::term(d, world, id)`
  (helpers `name`, `short`, `word`); kit code inside a battle calls `Catalog::term(id)` on the battle's catalog. Both
  read the same register; nothing else spells a term.
- TM4. Order for a `world` term: the world's own block through the lineage (the purse: `name`, `short`, `tip`), else
  the default. For a `game` term: the game's `terms` row, else the default.
- TM5. `short` empty = `name`. `word`: the row's `word` when set; else the default word when `name` equals the default
  name; else `short` as written. This is `hud::purse_word_in`'s rule (CAMPAIGN1 P1: "gold" stays lower case, "Scrap"
  stays "Scrap"), moved into the kit; `purse_word_in` and `purse_word` become thin callers of it (no second rule).
- TM6. Forms by use: HUD, labels and Results lines use `short` ("Gold +12", "Scrap +12"); sentences use `word` ("Not
  enough gold.", "12 Tallow"); a sentence that starts with the term uses `short` ("Gold is capped at 9999"). The one
  sentence whose verb agrees with the term ("Supplies are capped at N (M left behind).", `explore.rs`) keeps its text
  while the upkeep has its default name, and otherwise reads "`{short}`: capped at N (M left behind).".
- TM7. An unknown id is a programming error: `debug_assert!` and the id itself as the text in release.

## 5. The game's `terms` table

### 5.1 Schema contract (`content.terms`, catalog kind, file `data/catalog/terms.json`; schema-first: yes)

| Key | Type | Default | Validation (error text) |
| --- | --- | --- | --- |
| `id` | id | - | a `game`-scope register id ("Unknown term '`{id}`'. Terms: intent_assault, ..."); a `world` term: "'`{id}`' is named per world: set the world's purse block." |
| `name` | string | "" | 1-24 characters ("A term needs a name.") |
| `short` | string | "" | 0-16 characters; "" = `name` |
| `word` | string | "" | 0-24 characters; "" = TM5 |
| `tip` | text | "" | 0-160 characters; "" = the default tip |

Plus the fields every catalog kind carries (`origin`, `tags`), as `hazards`. A missing file is an empty table. The
Content Forge shows the kind from the schema (the `id` help lists the register). Write path: `tools/fc.sh content
upsert terms @file.json`, then `tools/fc.sh content check`.

### 5.2 World-level overrides or one game-level table (open question Q-T1)

| Option | Effect | Cost | Risk |
| --- | --- | --- | --- |
| (a) Game table, and the existing world blocks for `world` terms | Currency and upkeep per world (the Ashlands Gold, a campaign world's Scrap), every other term once per game | This spec as written | A term that later needs a per-world name moves to `world` scope with a world block or (b) |
| (b) Game table plus a world kind `terms` (lineage, by id) | Any term per world | A second kind, the purse names migrate or are read in two places | Two places for the currency name, which the decision rules out |
| (c) World kind only | Every term per world | Each world repeats every row | Classic texts change when a world forgets a row |

Recommend (a): the per-world names that exist (purse, classes, NPC service lines) stay where CAMPAIGN1 and HOOKS1 put
them, and the table refuses their ids, so each term has one place.

### 5.3 Data now

No rows: Emberwake has picked no lore name for an intent, and the purse names belong to the worlds' purse blocks
(CAMPAIGN1's data tasks). Rows come when the user names a term (Q-T2); no asset task until then.

## 6. Where the terms are read (rules with checks)

- TU1. Kit (TERMS1-L): every site of section 2's kit rows reads TM3; the battle log and the round-intent texts read
  `Catalog::term` for intent names.
- TU2. Results (`hub.rs`): "`{short(currency)}` +N" and "Objective bonus: `{short(currency)}` +N" in the battle's
  world (decision "INPUT1 answers, part 1": Results in campaign worlds use the world's purse name through one helper).
- TU3. Journal (`hub.rs` reward lines): "N `{word(currency)}`" in the game's world.
- TU4. Shop, party sheet, region notes: the game GUI rows of section 2 read TM3 in the world being played; a catalog
  screen (the Arena, the Dojo) reads the defaults.
- TU5. Battle screen and intent texts (`battle.rs`, `intent_text.rs`): intent names through `Catalog::term` (after
  INPUT1-U, which reworks those labels).
- As built (TERMS1-L, 2026-11-01): the register is `rpgkit_data::terms` (`REGISTER`, `TermDef`, `TermScope`,
  `Term::resolve`, `TermRow`, `TERMS_KIND`) with `Catalog.terms` and `Catalog::term`; the resolver with a world is
  `rpgkit_world::terms` (`term`, `name`, `short`, `word`) and the kit sentences are built there
  (`not_enough_currency`, `currency_capped`, `needs_currency`, `upkeep_carry_max`, `upkeep_capped`). Shop, quest,
  dialogue (also the gold lock "Needs N gold", which section 2 did not count) and explore read them;
  `progression::REFUSE_SUPPLIES` stays a const, because only a world without a campaign reaches it, and there the upkeep
  is always its default (a test ties the text to the register). `RoundIntent::name()` is the register default;
  `RoundIntent::term_id()` and `label(cat)` read `Catalog::term`. The intent texts in `round_intent.rs`,
  `round_intent_plan.rs` and `plan.rs` still call `name()` (the same text while the table is empty); they move to
  `label` with TERMS1-U, after INPUT1-U. No intent name is spelled in `log.rs` or `metrics.rs`. `hud::purse_word_in` now
  calls `terms::word`. The `terms` kind carries `origin` only (as `hazards`; no catalog kind has `tags`).
- As built (TERMS1-U, 2026-11-01): the screens read the resolver through `hud::{play_world, term_short, term_word}`
  (the world being played; "" without a game, so the catalog screens read the defaults); the Results take the
  currency's short form in the game's world (`reward_text(.., currency, ..)`). Beyond section 2's rows the shop's trade
  log ("bought N items for M gold"), its supply detail ("Travel supplies: ...") and the region chest line ("N gold",
  "N supplies") read the purse too. The intent names read `RoundIntent::label(cat)` on the battle screen, in
  `intent_text::round_head`, `timeline_view` (not in the touches list; same labels), the deck screen and the kit texts
  of `round_intent.rs`, `round_intent_plan.rs` (its refusal label now a `String`) and `plan.rs`. The intents are
  `game` terms, so the deck screens read them too: `DeckTerms.intent_names` (`party::intent_names(cat)`) names them in
  the deck refusals, and the Dojo's order headings read `label` on its catalog.

## 7. The layer check gains lore words (G3; `tools/fc.sh layers`, the same step in `check_project.ps1`)

- LW1. Sources, per game folder: (1) every world id under `games/<id>/data/worlds/` (whole word, case-insensitive,
  `_` as a separator, as G1's game ids); (2) `games/<id>/lore_words.txt`: one name or phrase per line, `#` comments,
  matched as a fixed string, whole word, case-sensitive.
- LW2. Scope (git pathspecs, which must match the whole path): `'kits/*/crates/*/src/*'`, `'kits/*/schema.json'`,
  `'kits/*/kit.json'`. Kit `tests/` and READMEs are not scanned (fixtures borrow names; Q-T4). A hit prints `[layers] file:line names lore word "X" of game <id>: ...` and
  fails the gate like G1.
- LW3. False hits: the list holds proper nouns and coined phrases only; a plain English word is listed only inside its
  lore phrase ("Warded Tallow", not "Tallow"; "Inquisition Alert", not "Alert"); case-sensitive matching keeps
  lower-case code words apart from capitalised names. There is no allow marker: a hit is fixed in the code.
- LW4. Seed list (TERMS1-G; the planner adds a name when a spec introduces one): Kaelen, Vesper, Maren, Hollis,
  Dunstan, Elyse, Nemeth, Quell, Vane, Grendel, Grakka, Mara Voss, Sunk Ward, Ashlands, Iron Reaches, Tidewrack, The
  Crucible, Castellan, Inquisition, Warded Tallow, Red Tithing.
- LW5. Today's one hit (`log.rs:341`) is rewritten as "A and B fall." in the same task, so the check lands green.
- As built (TERMS1-G, 2026-11-01): `layers_check` in `tools/fc.sh` and step [3/3] of `check_project.ps1` run one
  `git grep --untracked -E` per game folder: a world id becomes a case-folded class (`[Aa][Ss]..`) with G1's
  boundaries, a lore line an escaped phrase with `_` as a word character, so both lists share one case-sensitive call.
  A line is reported once, with the first lore word found on it. `games/emberwake/lore_words.txt` holds the LW4 list.

## 8. Asset layering (the decision's rule, and what moves now)

| Layer | Holds | Where |
| --- | --- | --- |
| Engine | Asset behaviour and configuration: formats, rigs, animation, effects, options; no art with lore or genre | `engine/crates/aegis_*` |
| Kit | The generic library on the kit's formats: tiles, effects, icons, rig bodies, common portraits, props, statuses, HUD icons of the default terms | `kits/rpg/assets/` |
| Game | Lore-bound art on the kit's formats: named characters, faction portraits, story scenes, backdrops of its places | `games/<id>/assets/` |

Survey: `kits/rpg/assets/` holds audio, item icons by type, map markers, common portraits by class (with the sample
world's goblin set), common props, skill kinds, statuses, the UI sets (badges, chips, HUD icons, intents, markers,
pins, services) and rig bodies by class; nothing named by a story. Recommend no moves now: new work goes to its layer
(CAST1_ASSETS, ART3), and `ui/hud/gold.png` and `supplies.png` keep their names (they are asset keys of the default
terms; a rename touches every reference for no player-visible gain; Q-T5).

## 9. Tests (rule-derived; scratch data)

| Test | Checks |
| --- | --- |
| `rpgkit_world/tests/terms.rs` | TM2: with no data every term is its default; TM4, TM5: a purse world gives `short` "Scrap", `word` "Scrap", the Ashlands "Gold" / "gold"; a `terms` row renames an intent; TM6's forms |
| same | TU1: "Not enough Scrap." in a Scrap world, "Not enough gold." with the defaults (shop, quest, dialogue) |
| `rpgkit_content/tests/terms_checks.rs` | Each error text of 5.1, one per mistake; a `currency` row refused |
| `emberwake_gui` (hub.rs, runover.rs tests) | TU2, TU3: the existing "Gold +12" test unchanged; a purse world reads "Scrap +12" and "12 Scrap"; `purse_word_in`'s tests unchanged |
| layer check (by hand in TERMS1-G) | A temporary line with a listed word fails `tools/fc.sh layers` and `check_project.ps1`; removed, both pass |

## 10. Parity

Classic texts are unchanged: the Ashlands have no purse block, the game table cannot name `currency` or `upkeep`,
intents never show in classic, and every default is today's text; the classic and timeline goldens, the recorded
traces and every existing text test run unchanged. The loader parity (`emberwake_world/tests/data_source.rs`) adds the
kind `terms` to KINDS_AFTER_GODOT; no recorded expectation changes.

## 11. Slices and task rows (the planner writes `docs/TASKS.md`)

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| TERMS1-G | Layer check G3: world ids and `lore_words.txt`; the `log.rs` comment | kit (tools) | logic | - | `tools/fc.sh`, `check_project.ps1`, `games/emberwake/lore_words.txt` (new), `rpgkit_battle/src/log.rs` (comment) |
| TERMS1-L | Term register, `content.terms`, the resolver, kit texts | kit | logic | TERMS1-G (log.rs) | `rpgkit_data/src/{lib,terms (new)}.rs`; `kits/rpg/schema.json`; `rpgkit_content/src/{source.rs,check/}`; `rpgkit_world/src/{terms (new),shop,quest,dialogue,explore,progression,lib}.rs`; `rpgkit_battle/src/{round_intent,log}.rs`; `data_source.rs`; tests |
| TERMS1-U | Screens read the terms: Results, journal, shop, party, region, HUD helper; intent labels | game | logic | TERMS1-L, CAMPAIGN1-U, INPUT1-U | `emberwake_gui/src/{hud,hub,party,shop,region,worldmap,runover,battle,intent_text}.rs` |

TERMS1-U's battle part may land as a second slice after INPUT1-U if the rest is ready first.

## 12. Docs delta, ROADMAP and HISTORY rows (for the planner)

- STRUCTURE: `rpgkit_data/src/terms.rs`, `rpgkit_world/src/terms.rs`, `data/catalog/terms.json`, `games/<id>/lore_words.txt`.
- GAME_BRIEF: one line (terms through `rpgkit_world::terms`, the table, the purse names stay per world).
- `engine/docs/authoring/layer_rule.md`: kit code names term ids, not words; the lore-word check; the asset layers table.
- Kit README: the register and how a term joins it.
- ROADMAP: a TERMS1 row, `proposed` (approved by the decision), M2.
- HISTORY: "Godot parity over the catalog kinds" narrowed: `terms` joins KINDS_AFTER_GODOT.

## 13. Out of scope

Plural forms and grammatical gender; translation; terms beyond the register's seven (`camp`, `poise`, `morale`, `ap`,
stat names come with the task that needs them, Q-T6); renaming asset files; scanning kit tests and READMEs; moving
assets; changing class names (HOOKS1 CN1) or NPC service lines (`service_lines`).

## 14. Open questions (recommended option first)

- Q-T1. World overrides or one game table (5.2): (a) game table plus the existing world blocks; (b) a world `terms`
  kind as well; (c) world only. Recommend (a).
- Q-T2. Lore names now: (a) none (the table ships empty; the user names a term later); (b) name the intents now.
  Recommend (a).
- Q-T3. Where the lore-word list lives: (a) `games/<id>/lore_words.txt` plus the world ids read from the folders;
  (b) a `lore_words` field in `game.json`; (c) derived from the data's names (false hits on "Guard", "Alert").
  Recommend (a).
- Q-T4. Scan kit tests too: (a) not now; a CLEANUP item renames the fixtures to neutral names, then the scan widens;
  (b) now, with the fixtures renamed in TERMS1-G. Recommend (a).
- Q-T5. Rename `ui/hud/gold.png` and `supplies.png` to the term ids: (a) no; (b) yes. Recommend (a).
- Q-T6. More terms now (`camp`, `poise`, `morale`, `ap`): (a) no, each with the task that needs it; (b) now.
  Recommend (a).
