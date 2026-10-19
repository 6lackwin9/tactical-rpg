# S11 baseline rules: NPCs, dialogue, shops and faction reputation

Status update (2026-10-20): answered (decision "S9-S11 baseline answers (user 2026-10-17)") and built as DLG1 and REP1. The status line below is the original.

Status: proposed (baseline rules, design-brainstorm 2026-10-17; nothing approved; the open questions in section 3 block the DLG1 and REP1 specs)
Depends on: M4-L merged (flags, `QuestRunner`, resolved giver / turn_in, variants, `RewardGrant`, `loot_claimed`); EXP1-L for NPC markers on maps and `BattleBridge` (the `fight` effect)
Lane: design (this file); later logic + asset (section 4)
Touches (later build, not this file): new `scripts/dialogue/*`, `data/dialogue/` (base, lore-free service templates), `scripts/world/world_schema.gd` + `world_catalog.gd` (kinds `dialogues`, `shops`; character fields), `scripts/progression/party_state.gd` (gold), `scripts/game_state/*` (save bump), `scripts/quests/*` (condition ops, `rewards.gold`; REP1), new `dialogue_test_suite.gd`

Source inputs: ROADMAP "Inputs for placeholder systems" (S11 bullet: flat flag registry, static NPC prompts with branching choices, quest flags unlocking exits or story events; later reputation meters, barter shops, timed events), DECISIONS world-campaign "M4 quest answers" (no branching inside a quest; all linkage through flags; metadata altered by flags) and "M4 variant answers" (giver/turn-in are metadata only until S11; no character-level "gone" field yet), archived brainstorm P-S11 (recommended: trees as data with flag conditions and effects, shops as a dialogue effect; reputation after flags exist).

## 1. Scope

S11 is talking to world characters: a dialogue tree per NPC with flag-gated choices and whitelisted effects, the quest services (offer, turn in) that make M4's `giver` / `turn_in` binding, an NPC presence rule (gone characters), shops with a currency, and (second slice, REP1) faction reputation as gating and price input.

S11 is NOT:

- a quest-branching system: dialogue may set flags and start or turn in quests; quest consequences stay M4 variants driven by flags (DECISIONS "M4 quest answers");
- a passive or skill source: dialogue effects give only items and gold; passives and skills stay granted by quests, loot entries, factions, disciplines and named items (GAME_BRIEF "Passives", HISTORY "MasterySystem");
- party recruitment, romance, companions' banter, voice or portraits (later);
- random skill checks (Q6 proposes deterministic checks only);
- timed world events (later; they would read the S10 clock);
- lore in the base layer: `data/dialogue/` holds only lore-free service lines with tokens; every authored line is world data.

## 2. Baseline rules

### 2.1 Data model sketch

World kind `dialogues` (`data/worlds/<id>/dialogues.json`; inherited and overridable like every kind):

| Field | Type | Default | Rule (sketch) |
| --- | --- | --- | --- |
| id | id | required | unique in the world |
| speaker | ref characters, optional | "" | default speaker of every node; "" = the character the player talks to |
| start | string | required | a node id |
| nodes | json | required | Array (1..64) of nodes, below |
| services | bool | true | append the generated service choices (2.3) to the start node |
| tags | tags | [] | |

Node: `{"id", "speaker" (optional), "text", "choices": [...], "effects": [...], "next": node id, "end": bool}`; exactly one of non-empty `choices`, `next`, or `end: true`. `effects` run when the node is shown. Choice (1..6 per node): `{"text", "to" (node id; "" = end), "requires" (conditions), "unless" (conditions), "effects" (0..8)}`; a choice whose conditions fail is hidden (or shown disabled with `show_locked: true`).

Conditions (`DialogueRules.CONDITION_KINDS`, one grammar shared with M4 and S9/S10 markers):

| Kind | Keys | Holds when |
| --- | --- | --- |
| flag (default) | `key`, `value`, `op` (REP1 adds ops; DLG1 = equality only) | `QuestRules.condition_met` |
| quest | `quest` ref, `status` (one of the M4 statuses) | `QuestRunner.status` equals it |
| items | `item` base id, `count` (1..99) | the stash holds at least `count` |
| gold | `min` int | `PartyState.gold >= min` |
| stat (Q6) | `stat` (STR..KNOW), `min` 1..30 | the best active member's stat >= `min`; the choice text shows `[KNOW 14]` |

Effects (`DialogueRules.EFFECTS`, applied in list order, all-or-nothing per choice: validated first against the current state, any failure = nothing changes and the choice is refused):

| Effect | Keys | Behaviour |
| --- | --- | --- |
| set_flag | `key`, `value` | `QuestRunner.set_flag` (so quest `refresh` runs) |
| add_flag (REP1) | `key`, `delta`, optional `min` / `max` | int flag += delta, clamped; missing flag reads 0 |
| start_quest | `quest` | `QuestRunner.start`; must be `available` |
| turn_in | `quest` | opens the turn-in step (recipient + loot picks) then `QuestRunner.complete`; must be `ready` |
| give | `items` {base id: count}, `gold` | stash / gold via `RewardGrant` (gold key added in DLG1) |
| take | `items`, `gold` | must be held (auto-precondition) |
| open_shop | `shop` ref | ends the node, opens the shop; returns to this node afterwards |
| fight | `encounter` ref | must be the last effect; ends the dialogue and starts `BattleBridge` (S9 2.4) |

Character fields (additive on `characters`):

| Field | Type | Default | Rule (sketch) |
| --- | --- | --- | --- |
| dialogue | ref dialogues, optional | "" | "" = the base service template only (2.3) |
| gone_flags | json | [] | M4 conditions; ANY holding = the character is gone (Q3) |

World kind `shops` (`data/worlds/<id>/shops.json`):

| Field | Type | Default | Rule (sketch) |
| --- | --- | --- | --- |
| id | id | required | unique |
| name | string | "New Shop" | display |
| keeper | ref characters | required | a gone keeper closes the shop |
| stock | json | [] | `[{"item": base id, "count": -1 or 1..99, "price": -1 or 1..99999, "requires_flags": []}]`; -1 count = unlimited, -1 price = derived |
| loot | json | [] | `[{"loot": world loot ref, "price": 1..99999}]`; item loot entries only; claim-once through `loot_claimed` (a sold unique leaves every quest pool, same rule as M4) |
| buy_mult | float | 1.0 | 0.5..3.0, player buy price factor |
| sell_mult | float | 0.5 | 0.0..1.0, player sell price factor; 0 = the shop buys nothing |
| buys | json | all but `QUEST` | item types the shop buys |
| requires_flags | json | [] | failing = closed |

Base layer `data/dialogue/services.json`: lore-free service lines with tokens (`{npc}`, `{quest}`, `{shop}`, `{gold}`), for example "Do you have work for us?", "About {quest}...", "We finished: {quest}.", "Show me your wares.", "Farewell." A world may override a line by id (world kind override), never the rules.

### 2.2 Prices and gold (DLG1)

- `PartyState.gold` int >= 0 (default 0; `NewGame.create` sets a start amount from data).
- Buy price = `price` if set, else `ceil(item.value x buy_mult x rep_mult)`; sell price = `floor(item.value x sell_mult x sell_rep_mult)`; `rep_mult` = 1.0 until REP1. Items come from and go to `PartyState.stash` only (equipped gear must be unequipped first; QUEST items are never bought back).
- Limited stock is saved: new `GameState.shops` `{shop qid: {item id: sold count}}` (only for `count > 0` entries); no restock in the baseline (a later rule may read the S10 clock).
- Gold sources (Q4): quest `rewards.gold` (new M4 template key, soft cap like xp), chest `gold` (S9 marker key), selling, and kill gold if Q4 = A. Every price and amount is tuned after the balance pass (archived balance map: "S11 shops: freeze rules only, tune prices later").

### 2.3 Quest services and enforcement (makes M4 giver / turn_in binding)

When the player talks to character C (present, 2.4), the start node gets generated choices appended after the authored ones (unless `services: false`), in catalog order:

1. "About {quest}" for every quest whose RESOLVED `giver` is C and whose status is `available` -> a service node with the quest's resolved description and choices Accept (`start_quest`) / Not now.
2. "We finished: {quest}" for every quest whose resolved `turn_in` (empty = giver) is C and whose status is `ready` -> `turn_in`.
3. "Show me your wares" for every open shop whose keeper is C.
4. "Farewell" (end).

Enforcement rule (Q2): a quest with a non-empty resolved giver starts ONLY through C (a service choice or an authored `start_quest` effect in a dialogue spoken by C); with an empty giver it starts by `auto_start`, a marker or a dialogue effect. A quest completes ONLY through its resolved turn-in character; when both giver and turn_in are empty it completes from the journal. `QuestRunner.start/complete` stay unrestricted APIs for tests and tools; the rule lives in `DialogueRunner` and the UI. Variants that change `giver` / `turn_in` (M4 V1) re-route the services automatically because services read the resolved quest.

Authoring lint (`DialogueRules.quest_warnings(world_id)`, warnings only): for each quest whose giver or turn-in character has `gone_flags`, warn unless some variant whose `when_flags` contain that gone condition replaces that field or sets `available: false`. This is the check that keeps the M4 "NPC dies -> quest changes" promise from becoming a stuck quest.

### 2.4 NPC presence

A character is gone when ANY of its `gone_flags` holds (re-read on every check, no saved state, same as M4 variants). A gone character: no dialogue, no services, its S9 `npc` markers are absent, its shops are closed; quests it gives or takes are handled by variants (2.3 lint). Death or departure is therefore always a flag set by some effect (a quest's `complete_flags`, a dialogue choice, a won fight's marker flag), which M4 already reads in `when_flags`.

### 2.5 Runtime and save

`DialogueRunner` (pure logic): `open(gs, character_ref) -> Dictionary` (the start node with visible choices), `choose(gs, index) -> Dictionary` (validates and applies effects, returns the next node or end plus events). The conversation itself is not saved; saving is refused while a dialogue or shop is open. Save bump (one migration step): `PartyState.gold` (0) and `GameState.shops` ({}). Repeat-once choices are authored with a flag (`unless` the flag, `set_flag` it), no hidden "seen" state.

### 2.6 Reputation (REP1, second slice)

- Storage (Q5): int flags `rep.<faction qid dotted>` (-100..100, default 0) in the flat registry. Changed by `add_flag` effects (clamped by the `rep.` prefix rule) and by a new world quest field `reputation` `{faction ref: delta -50..50}` applied on `complete`.
- Conditions gain `op` (`==` default, `!=`, `>=`, `<=`, `>`, `<`; ordering ops need int values on both sides). This also extends quest `requires_flags` and variant `when_flags` for free ("honoured with the guard" gates a quest).
- Bands (`RepRules.BANDS`, display + price): hostile <= -50, unfriendly -49..-10, neutral -9..9, friendly 10..49, honoured >= 50. Shop `rep_mult` by the keeper's faction: hostile = closed, unfriendly 1.15, neutral 1.0, friendly 0.95, honoured 0.9 (sell factor mirrored: 0.85 / 1.0 / 1.05 / 1.1). Starting values; tuned after balance.
- Ripple through faction `relations`: Q7.
- Faction membership granting `grants` passives to party members stays out of scope (a later source rule).

### 2.7 Acceptance-check themes (the DLG1 / REP1 specs turn these into rule-derived checks)

Tree validation (missing start, dangling `to`, node with both `choices` and `end`, 65 nodes, 7 choices, unknown effect, `fight` not last, unresolved refs); hidden vs locked choices; all-or-nothing effects (a `take` that fails changes nothing); service generation (giver variant re-routes the offer; ready quest offered only at the resolved turn-in; empty giver + turn_in completes from the journal); gone flags hide the NPC, close its shop, and the lint warns for an un-rerouted quest; prices (derived, override, ceil/floor), limited stock saved and refused at 0, loot sale claims once and leaves quest pools; REP1: clamping, ops on non-int flags refused, band boundaries, price multipliers, quest `reputation` applied once.

## 3. Open design questions (Q1-Q6 block DLG1; Q5, Q7 block REP1)

Q1. Where do service lines and dialogue live?

- A. World kind `dialogues` for authored trees, plus base lore-free service templates and generated service choices (sketch). Every quest giver works with zero authored text; base stays lore-free.
- B. Everything authored per NPC in the world. Full control; every giver needs a tree before its quest is playable.
- C. Services only (no trees) in DLG1, trees later. Smallest; no story prompts, which the ROADMAP input asks for.
- Recommendation: A.

Q2. How strict is the giver / turn-in binding?

- A. Strict through the resolved character, with the variant lint warning for gone NPCs (sketch). Honours M4's "metadata altered by flags" design; holes are caught at authoring time.
- B. Strict, but a quest whose resolved turn-in is gone may be completed from the journal. No stuck quests ever; dulls the consequence of an NPC's death.
- C. Soft: start and turn in from the journal; NPCs are flavour. Simplest; makes giver/turn_in meaningless.
- Recommendation: A.

Q3. How is an NPC removed (death, departure)?

- A. Character field `gone_flags` (ANY holds = gone), re-read on every check (sketch). One rule for dialogue, markers, shops and the quest lint; no saved state. Adds the field M4 deferred (V5), now that S11 needs it.
- B. Per-marker and per-shop conditions only. No character field; the same condition copied to every place the NPC appears.
- C. A saved "dead" list in `GameState`. Explicit; a second progress store beside the flag registry (against "all world progress through flags").
- Recommendation: A.

Q4. Currency and gold sources?

- A. Gold in `PartyState`; sources: quest `rewards.gold`, chests, selling, and kill gold `round((2 + enemy_level) x tier_mult)` per enemy via `LevelUp.award_encounter`. Steady income; one more balance lever (buying gear).
- B. Gold without kill gold (quests, chests, selling only). Economy fully authored and boundable; mob fights feel unrewarding.
- C. Barter only (trade by item `value`, no currency). No new state; awkward for buying single items and for loot sales.
- Recommendation: B for DLG1 (authored income keeps the reward budget the only lever until the balance pass); add kill gold later if pacing needs it.

Q5. Reputation storage (REP1)?

- A. Int flags `rep.<faction>` plus comparison ops in the shared condition grammar (sketch). One registry (DECISIONS "M4 quest answers"); quests and markers gain rep gating for free.
- B. Separate `GameState.reputation` block with its own condition kind. Clear typing; a second progress store and a second condition path.
- Recommendation: A.

Q6. Stat checks in dialogue?

- A. Deterministic threshold on the best active member's stat (`stat` condition, shown in the choice text). Rewards builds without RNG; testable. (The archived P-S11 deferred stat checks because they "add randomness"; this version has none, so that reason does not hold.)
- B. None in the baseline. Smallest; the README "skill checks" item stays open.
- C. Rolled checks (stat vs d20-like). Drama; randomness the archived brainstorm wanted to avoid.
- Recommendation: A, in DLG1 (one condition kind).

Q7. Does reputation ripple through faction `relations` (REP1)?

- A. No ripple: only explicit deltas (sketch). Predictable; authors write each change.
- B. Ripple: +X to a faction gives +X/2 to its allies and -X/2 to its enemies. Living politics; side effects hard to author around and to test.
- Recommendation: A; B can be a later rule on top of A.

## 4. Milestone shape (coarse)

| Milestone | Lane | Goal | Depends on |
| --- | --- | --- | --- |
| DLG1-L | logic, one branch; commit 1 = schema (kinds `dialogues`, `shops`; character `dialogue`, `gone_flags`; base `data/dialogue/services.json` envelope; condition and effect validation; quest lint), commit 2 = runtime (DialogueRunner, services, presence, shops and prices, gold, `rewards.gold`, save bump, suite), commit 3 = minimal dialogue and shop panels (render only) | engine + tests + debug panels | M4-L merged, EXP1-L merged (markers, BattleBridge); answers Q1-Q4, Q6 |
| DLG1-A | asset, one branch (seed) | base service lines; trees for the M4 sample givers (Ashlands), `gone_flags` matching the M4 NPC-death variants (`mara_fallen`, `tev_fallen`), one shop with one loot sale | DLG1-L commit 1, M4-A merged |
| REP1-L | logic, one branch (schema first) | condition ops, `add_flag`, `rep.` clamp, quest `reputation` field, bands, shop multipliers | DLG1-L; answers Q5, Q7 |
| REP1-A | asset | reputation deltas on the sample quests, one rep-gated choice and one rep-gated quest | REP1-L commit 1 |

Order per ROADMAP "Suggested build order": DLG1 in step 5 (after EXP1, before TRAV1), REP1 in step 6. Hub files shared with EXP1 and TRAV1 (`world_schema.gd`, `world_catalog.gd`, `game_state.gd`, save files) force sequential runs. REP1-L also edits `scripts/quests/quest_rules.gd` (condition ops), so it must not run beside an open M4 follow-up.

## 5. HISTORY check

- "MasterySystem" (deprecated: traits come from sources, not the class) and GAME_BRIEF "Passives: granted only by sources": dialogue effects grant no passives or skills; faction membership passives stay a later source rule.
- Row 1 (10-block combat layer, deprecated): the S11 inputs come from the same early notes; only the non-combat parts are kept.
- The archived brainstorm (not HISTORY) deferred stat checks for randomness; Q6 proposes a deterministic version and says why the reason no longer applies.
- Nothing in HISTORY deprecates dialogue trees, shops, currency or reputation.
