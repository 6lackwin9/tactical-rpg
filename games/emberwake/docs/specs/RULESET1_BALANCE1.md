# RULESET1_BALANCE1: what the first timeline feel run asks for

Status: proposed (design 2026-10-30, from the RULESET1-B report in `games/emberwake/docs/BALANCE_REPORT.md`, section
"RULESET1-B 2026-10-30")
Depends on: RULESET1-B (done as a report); the user's answers to section 7
Lane: logic (RULESET1-BX, RULESET1-G2, RULESET1-AI2), asset (RULESET1-A2), balance (RULESET1-B2)
Touches: per row in section 6

## 1. Goal

The first feel run (500 fights, five `tl_` encounters) says the timeline rules work but the test fights cannot show
them: the AI never maneuvers, nobody times a hit into a windup, two duels have no winning line, enemy decks are thin
and the sim and the engine's own driver disagree. This spec sorts the ten issues into bugs, data fixes and rule
questions, fixes the tool first, and then reruns. The `tl_` units are test data: CAST1's bestiary replaces them, so
the data fixes are the fewest that let each encounter show what it judges (RULESET1_TASKS 2.7 "Judges"), not a
difficulty pass (preference "Light-touch battle tuning"). Every change is timeline-only; classic stays byte-identical
(RULESET1 3.3, the inert check of RULESET1-X).

## 2. Triage

Kind: BUG (investigate in code), TOOL (sim gap), DATA (tl_ rows), RULE (a user question, section 7).

| # | Issue (report number) | Kind | Recommendation | Row |
| --- | --- | --- | --- | --- |
| 1 | Maneuver unreachable: AI row 4 needs `Role::Skirmisher`, no `tl_` unit is one; carried moves 0 of 4392 | DATA (+ RULE Q2) | `tl_spiker` `ai_role` SKIRMISHER | A2 |
| 2 | tl_duel_windup 8%: 0 poise breaks on the Enforcer; both sides Guard at 40% HP round after round (party Guard 52.8%, enemy 45.1%) | RULE (Q3, Q4) + DATA on trigger | a "timed" party driver in the sim (Q3); no second Guard round without a hit (Q4); levers L1-L2 only if still under band | G2, AI2, B2 |
| 3 | tl_duel_armour 2%: Crushing Judgment 110 a round kills the 210 HP Spiker in round 2; Marrow Spike 15 vs the Captain, 20 vs the Footman | BUG + DATA (Q6) | find the 0.75 term first; then Crushing Judgment `power_mult` 2.2 to 1.5 (or 1.3) | BX, A2 |
| 4 | Standards weak in tl_flank (82% sim, 40 of 40 driver) | DATA, deferred | no change now: tier multipliers are global (classic moves); judged by the band (Q1) after the rerun | B2 |
| 5 | Thin enemy decks: assaults only; 29.8% enemy rounds with no active of their intent; 1676 enemy Rest blocks | DATA (Q5) | a Guard active in every `tl_` enemy deck; close OI1 with 3/2/1 kept | A2 |
| 6 | Break chance is 37% of breaks (366 of 936, 22.8% a roll) | none now | keep `break_chance_scale` 0.5 (TQ2); with timed hits poise breaks rise and the share falls; re-read in B2 | B2 |
| 7 | Flank and reach move no win rate (0-5 points) | DATA (diagnostic) | keep `flank_power_mult` 1.0 (TQ12 a); a `tl_flank_elite` fight; re-read once Maneuver fires | A2, B2 |
| 8 | Sim and driver disagree (windup 8-12% vs 0/40, flank 82% vs 40/40, skirmish 67% vs 13/40); `preset=` inert | BUG | one auto driver for both; the battle screen's autoplay is the reference | G2 |
| 9 | RULESET1 A3 says pierce "gains 33-100%" | DOC | planner edit, text in 5.5 | planner |
| 10 | `sim feel` gaps: no capped count, no armour or tuning key, `preset=` inert | TOOL | outcome columns, `armor=`, `tune=`, `tactic=`, `party=` | G2 |

Order: BX and G2 first (both change what the numbers mean), AI2 and A2 next, then B2. Nothing else in the report
needs a change: Rest gives 0.40 AP a block (TQ13 b, the no-AP fallback, is not triggered), lock rate 1.7% is a
player-skill readout, 0 PLAN FAIL.

## 3. Schema contract (logic lane)

Schema-first: no. No content field is added; the sim gains keys (G2) and the AI one rule (AI2, if Q4 = b).

### 3.1 `sim feel` keys (RULESET1-G2)

| Key | Values | Default | Validation |
| --- | --- | --- | --- |
| `tactic=` | `balanced`, `all_out`, `focus_weakest`, `guard_counter`, `kite`, `economy` (the AUTO1 tactics, mapped by RULESET1_INTENT AI5) | `balanced` | unknown name: error listing the six |
| `preset=` | classic preset names | `balanced` | under timeline any value but `balanced` is an error: "preset is classic-only; use tactic=" (today it is silently ignored) |
| `party=` | `auto` (the battle screen's autoplay), `timed` (5.3) | `auto` | `both` runs each and prints two blocks |
| `armor=` | `<unit_id>:<0-200>`, repeatable, comma list | none | the unit must be in a fought roster; sets the unit's `armor` for this run only |
| `tune=` | `<ruleset number>:<value>`, repeatable | none | keys and ranges of RULESET1_TASKS 1.2 (`stagger_*_mult`, `flank_*_mult`, `break_chance_*`, `rest_*`); out of range: error |

Output per encounter gains: `won`, `lost`, `capped` (reached `max_rounds` with both sides standing), `fled` counts that
sum to the fights; `median rounds (won)` beside today's median (PACE1 reads won-fight medians); the driver name. The
header prints every non-default key.

## 4. Rules (logic lane)

### 4.1 RULESET1-BX: the Marrow Spike numbers (bug probe)

- BX1. Marrow Spike (`power_mult` 0.9, `armor_piercing`) deals 15 to `tl_captain` (armour 50) and 20 to `tl_footman`
  (armour 25) from the same `tl_spiker` (report, seed 11). With the armour term at 1.0 (`damage.rs` `compute`:
  `armor_ratio` 0 when `ignore_armor`) the only terms left are stance matchup, guard plus cover, crit and the attack
  boost; 15 / 20 = 0.75 is exactly a lost stance matchup (`Stance::matchup`, 0.75) or a 0.25 guard or cover term.
- BX2. Read the hit breakdown of both hits (trace `Hit` fields: `armor_pierced`, guard ratio, stance factor) and name
  the term. Expected under timeline: stance matchup 1.0 for every hit (`round_intent` makes classic matchup inert,
  RULESET1 3.2); guard only while a Defend command of the target runs; cover per S12T.
- BX3. If the term is unintended (most likely the stance matchup reaching a timeline hit), fix it in the core for the
  timeline path only; the classic traces do not move (the inert check passes untouched).
- BX4. If the term is intended (for example the Captain guards), name the cause in the builder's report (B2 quotes it)
  and change no code.
- Acceptance: a test that a pierced hit from one attacker deals the same damage to two targets that differ only in
  armour (25 and 50) under timeline; the classic inert check unchanged. A traced change re-records
  `timeline_golden.json` with the cause in the commit and a `docs/HISTORY.md` row (RULESET1_TASKS 4.2).

### 4.2 RULESET1-G2: one driver, the tool gaps

- G2-1. Find why `sim feel` and `emberwake_host battle <enc> seed=N` disagree: run both on one encounter and seed with
  `jitter=off` and diff the traces to the first different step. Leads, in order: the party side is planned by
  different code (the round intent AI table or AUTO1 Balanced in one, classic `PlanPresets` or another profile in the
  other); the party's AI profile is read from the class in one and the unit in the other; capped fights are counted
  differently (the driver prints "fled" at round 8 though every `tl_` unit has `flee_ratio` 0).
- G2-2. Reference driver: what the battle screen's autoplay runs for a hero on Balanced (AUTO1). `sim feel`, the host
  `battle` command and the `timeline_golden` fights all call it (the BREPORT1 rule: one source for the report and the
  sims). If `sim::fight` is that driver, only the host command changes; otherwise re-record the golden as BX says.
- G2-3. Under timeline `tactic=` picks the party's AUTO1 tactic per AI5; `preset=` follows 3.1.
- G2-4. The keys and columns of 3.1. `armor=` and `tune=` change the run's in-memory copy only, never data.
- Acceptance: for each `tl_` encounter and seeds 11, 23, 35 the host command and `sim feel` (jitter off) give the
  same outcome and rounds; `tactic=all_out` and `tactic=guard_counter` give different intent shares in tl_skirmish;
  `won + lost + capped + fled` equals the fights; `armor=tl_footman:50` changes a plain hit on the Footman by the
  curve `r / (r + 50)` and leaves a pierced hit unchanged.

### 4.3 The `timed` party driver (RULESET1-G2, if Q3 = b)

The intended play of tl_duel_windup and tl_duel_reach (RULESET1_TASKS 2.7: "a Snap timed into a Heavy (manual)",
"the sure break") is manual by design (RULESET1_INTENT MM4: a delayed opener is what only manual can do). The sim
plays it with a second party driver that stands for a player who reads the tells; the in-game planner stays as is.

- TD1. It sees only what the player sees: the tell's first action of each enemy (DR1: skill, so its windup blocks)
  and the impact badges (V1: the impact block only). Visible windup blocks: every windup block of the tell action;
  the impact block of every other hit badge. It never reads a hidden plan, AI state or the RNG.
- TD2. After the AUTO1 Balanced pick (intent and target), if the target has a visible windup block, it builds the
  round by manual steps (MM2): Rest blocks, then the Assault skill that lands in the earliest visible windup block
  of the target, preferring in order a `sure_break` skill, then the most stagger per AP; then "Fill the rest" (MM3).
- TD3. No visible windup block, a refused step, or an intent other than Assault: the Balanced round unchanged.
- TD4. Deterministic; no random draw (PL rule). Enemies keep their AI (they never read the party's plan).
- Acceptance: on a scripted tl_duel_windup round where the Enforcer's tell is Overhead Smash at blocks 1-3, the
  timed plan lands a Vanguard hit in block 1, 2 or 3; with no tell the plan equals Balanced's.

### 4.4 RULESET1-AI2: the low-HP Guard stalemate (if Q4 = b)

- AI2-1. Row 3 (HP share at most `defend_below`, Guard) and the HOOKS1 `brace_after_hit` row are skipped for a unit
  that was in the Guard intent last round and took no attack hit during it; it goes on down the table (Engage,
  Assault). The next round row 3 may fire again. Timeline AI table only; classic `TacticalAI` untouched.
- AI2-2. A round in Guard that took a hit keeps today's rule (the guard bought something).
- Acceptance: two units at 30% HP facing each other, both Guarding in round 1 with no hit: neither picks Guard in
  round 2; a unit hit while guarding in round 1 may Guard again; tl_duel_windup capped fights fall (B2 reads it).

## 5. Assets (asset lane, RULESET1-A2)

Through `tools/fc.sh content upsert`, then `content check`; only `tl_` rows. The golden's frozen kit copy
(`timeline_kit.json`) is not touched, so these rows move no golden.

### 5.1 Units

| Unit | Field | Now | New | Why |
| --- | --- | --- | --- | --- |
| `tl_spiker` | `ai_role` | "" (AGGRESSOR from `tl_drill`) | `SKIRMISHER` | issue 1 (Q2 a): AI row 4 Engage and the low-HP Disengage (`ai.rs` row 4 and its Disengage gate) |
| `tl_footman`, `tl_footman_b` | `deck.active` | snap_jab, thrust | snap_jab, thrust, `tl_brace_counter` | issue 5 (Q5) |
| same | `deck.order.guard` | none | [`tl_brace_counter`, `active_guard`] | issue 5 |
| same | `extra_skills` | [`tl_snap_jab`] | [`tl_snap_jab`, `tl_brace_counter`] | the deck's actives not on the path |
| `tl_enforcer` | `deck.active`, `deck.order.guard`, `extra_skills` | overhead_smash, thrust; none; [`tl_overhead_smash`] | add `tl_brace_counter` to actives and extras; guard [`tl_brace_counter`, `active_guard`] | issue 5 |
| `tl_captain` | same | crushing_judgment, thrust; none; [`tl_crushing_judgment`] | add `tl_brace_counter` likewise | issue 5 |
| `tl_crossbowman` | same | aimed_shot, snap_shot; none; [both] | add `tl_fade`; guard [`tl_fade`, `active_guard`] | issue 5 (a ranged unit: no riposte) |

Keep `defend_below` 0.4 and `flee_ratio` 0 on every `tl_` unit.

### 5.2 Skills

| Skill | Field | Now | New | Rule |
| --- | --- | --- | --- | --- |
| `tl_crushing_judgment` | `power_mult` | 2.2 | 1.5 if BX finds and fixes an unintended term (Marrow Spike then 20 against the Captain); 1.3 if BX finds 15 correct | issue 3 (Q6 a). 110 a hit at 2.2 becomes about 75 (Spiker dies to the 3rd hit, round 3) or 65 (4th hit, round 4); the Spiker deals 80-133 a round against 315 HP, so the fight turns on the Spiker's damage: boss favoured, winnable |

Trigger-only levers (applied in a later A2 pass only if B2 shows tl_duel_windup under its band with `party=timed`):
L1 `tl_overhead_smash` `windup_poise` 20 to 10; L2 `tl_shoulder_check` `stagger` 10 to 12. Estimate from RULESET1_TASKS
2.6 (Max Poise about CONS x 2 + plate): the Enforcer sits near 34, 54 during the Smash; three timed Shoulder Checks
(3 x 20 = 60) break it today, two (2 x 24 = 48 against 44) with L1 and L2.

### 5.3 Encounter

| Id | Party | Enemies | Settings | Judges |
| --- | --- | --- | --- | --- |
| `tl_flank_elite` | `tl_vanguard`, `tl_spiker` | `tl_enforcer` | as RULESET1_TASKS 2.7 shared (zone `stone_arena`, max_rounds 8, total_blocks 6, ap_regen 4, unit_ap 10, rng_seed 1337, rout, standard deployment, morale false) | the flank bonus on a heavy target: `rules=-flank` against on (issue 7) |

Add it to the game's feel list (`SimGame.feel_encounters`) only if the planner wants it in the default run; else B2
names it with `enc=`.

### 5.4 RULESET1-B2 (balance lane): the rerun

On the merged result of BX, G2, AI2 and A2: `sim feel seeds=100 party=both`; `seeds=40` with `rules=-flank` (all six
fights), `rules=-pierce`, `rules=-poise`; pierce at 25/35/50: `enc=tl_flank seeds=40 armor=tl_footman:25`, `:35`,
`:50`, each with `rules=-pierce`; `tune=break_chance_scale:0.25` and `:0` on tl_duel_windup and tl_skirmish. Report
against the bands of Q1 and the mechanic floors below, with the old baseline beside each number.

Mechanic floors (each encounter shows what it judges): carried moves above 0 in tl_flank and tl_skirmish; poise breaks
above 0 in tl_duel_windup (`party=timed`); sure breaks above 0 in tl_duel_reach (`party=timed`); capped fights at most
10% in every encounter; enemy rounds with no active of their intent under 15%.

### 5.5 Doc fix (planner, issue 9)

RULESET1 A3 now reads "(cut 33%, 41%, 50%), so pierce gains 33-100% over a plain hit". Replace with "(cut 33%, 41%,
50%), so a pierced hit deals +50%, +70%, +100% over a plain hit". The report's "+69%" rounds the kept share (59%);
exact is 85 / 50 = 1.7.

## 6. Task rows wanted

| Id | Title | Layer | Lane | Depends on | Touches |
| --- | --- | --- | --- | --- | --- |
| RULESET1-BX | Pierced hit damage differs by target (Marrow Spike 15 vs 20): find and fix the term (4.1) | kit | logic | RULESET1-B | `kits/rpg/crates/rpgkit_rules/src/damage.rs`, `kits/rpg/crates/rpgkit_battle/src/command.rs` (read first), a kit test; maybe `games/emberwake/crates/emberwake_battle/tests/fixtures/timeline_golden.json` |
| RULESET1-G2 | One auto driver for `sim feel` and `battle`; feel keys `tactic=`, `party=`, `armor=`, `tune=`, outcome columns; `preset=` error; the timed driver (3.1, 4.2, 4.3) | kit + game | logic | RULESET1-B; Q3 for 4.3 | `kits/rpg/crates/rpgkit_sim/src/{args,modes,feel,runner}.rs`, `games/emberwake/crates/emberwake_host/src/cli.rs`, `kits/rpg/crates/rpgkit_host/src/lib.rs`; maybe the golden fixture |
| RULESET1-AI2 | No second low-HP Guard round without a hit (4.4) | kit | logic | Q4 = b | `kits/rpg/crates/rpgkit_battle/src/ai.rs`, `kits/rpg/crates/rpgkit_battle/tests/` (AI tests); maybe the golden fixture |
| RULESET1-A2 | `tl_` data: Skirmisher Spiker, enemy Guard actives, Crushing Judgment power, `tl_flank_elite` (5.1-5.3) | game | asset | RULESET1-BX (the power pick); Q2, Q5, Q6 | `games/emberwake/data/catalog/{units,skills,encounters}.json` |
| RULESET1-B2 | Feel rerun against the bands and floors (5.4) | game | balance | RULESET1-BX, RULESET1-G2, RULESET1-AI2, RULESET1-A2 | `games/emberwake/docs/BALANCE_REPORT.md` |

BX, G2 and AI2 may each re-record the golden: run them one after another (shared fixture), each naming its cause.

## 7. Open questions for the user

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| Q1 | Targets for the `tl_` fights (sim, after the fixes) | (a) sanity bands: tl_duel_windup 30-70 (timed driver), tl_duel_reach 30-70, tl_duel_armour 15-45, tl_flank 50-85, tl_skirmish 40-70, capped at most 10%, plus the floors of 5.4; real bands come with CAST1-B; (b) the balance agent's tier line (elite about 50%, standard beats a hero) as a gate; (c) no bands, floors only | a: duels are a diagnostic with a 30-70 band (decision "BAL2 role answers" (3)) and the `tl_` units are test data; tl_flank's 82% sits inside 50-85, so no standard is retuned (issue 4) |
| Q2 | How does the AI decide to Maneuver? | (a) by role: `tl_spiker` becomes SKIRMISHER (data only); (b) by deck: any unit with a Maneuver active Engages when a flank tile is in range (a kit AI rule for every timeline unit); (c) by temperament, when ROSTER1-T lands | a now: one data field makes carried moves, Hamstring Cut and Disengage measurable; b or c is worth deciding with CAST1's bestiary |
| Q3 | Who times a hit into a windup in the sim? | (a) the in-game auto planner does it for everyone (changes MM4: a delayed opener would no longer be manual-only); (b) a `timed` sim driver only (4.3), reading only the tells and badges a player sees; (c) nobody: data-only fixes | b: the sim measures both the autoplay floor and a tell-reading player, and manual keeps its edge. Note RULESET1 V3 says the AI never reads tells: still true for every in-game AI |
| Q4 | The low-HP Guard stalemate (both sides Guard round after round to the cap) | (a) keep row 3; (b) no second Guard round without a hit taken in the first (4.4); (c) data: `defend_below` 0 on elites and bosses (they never turtle, as they ignore morale) | b: symmetric, uses only the past round, and keeps a guard that is working |
| Q5 | Thin enemy turns and OI1 (3 actives for four intents) | (a) every `tl_` enemy deck gets a Guard active (5.1) and OI1 closes with 3/2/1 kept: the party's gaps are authored, and CAST1's bestiary rule becomes "an enemy deck holds an active for each intent its AI can pick"; (b) a 4th active slot for enemies; (c) leave as is | a: the run shows enemies, not heroes, are thin (enemy 29.8% vs party 6.2% empty rounds) |
| Q6 | tl_duel_armour (unwinnable: lost in round 2) | (a) Crushing Judgment `power_mult` 2.2 to 1.5 (1.3 if BX finds 15 correct); (b) keep it and judge the duel by pierce share and the unbreakable windup only; (c) more Spiker HP (moves tl_flank and tl_skirmish too) | a: one skill row the Captain alone uses; the boss stays favoured |

Not asked (kept as decided, re-read in B2): break chance 0.5 (TQ2, issue 6); flank power 1.0 (TQ12 a, issue 7); Rest
0.5 AP a block, cap 2 (TQ13 a).

## 8. Out of scope

- Global tier multipliers, any Ashlands or classic row or code path, the CAST1 bestiary and its bands (CAST1-B).
- A tell-reading or re-planning AI in the game (Q-50 kept INTENT1; V3); the in-game auto planner's opener timing.
- The UI (RULESET1-U, VISCLAR1); HOOKS1 rows; ENCOUNTER2.
- Re-freezing the golden's kit copy to the new `tl_` data (a separate choice after B2).

HISTORY check: nothing here re-proposes a dropped idea. The cut list of 2026-10-25 (no rolls, row 75) keeps the break
chance as the one roll and this spec leaves it; Rest's +2 AP a block (row 81) is not revisited.
