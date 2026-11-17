# Full Project Review — 2026-11-17 (`@11d320a`)

- **Date**: `2026-11-17`
- **Branch / Commit**: `origin/develop` `@11d320a`
- **Scope**: the three consistency-audit pillars plus workflow and planning, art and UI, and the engine (six read-only passes)
- **Source**: Tushar's Claude Doc "Full project review, 2026-11-17" (archived verbatim below, split in two files for the 25 KB limit; part 2: [Art, UI, engine and next steps](2026-11-17_11d320a_consistency_audit_part2.md))
- **User answer**: "all recs A-E, queue F and G afterwards" (2026-11-17)

## Summary

The code and data are healthy; the instructions, the art direction and the planning docs are not, and those three are what stand between this project and automated development. Reviewed: `develop` at commit `11d320a`, using your consistency audit's three pillars plus workflow, art and UI, and the engine.

1. **Code is solid.** The full gate passes: clean build, 3,491 tests, 0 failures. Every finding from the 11-10 audit is fixed. Four new medium mismatches between spec and code, none severe.
2. **Shipped data is clean.** No broken references. A few placeholder strings and the retired word "gold" are visible to players; seven lore questions need your ruling.
3. **The agent instructions are the weakest part.** Skills and workflow docs contradict each other on the mode in force, who may edit what, and whether Build continues by itself. This is the main blocker for automation.
4. **Art has no locked style.** Portraits are strong but off canon; battle sprites are complete but look like a different, brighter game; icons come in three styles. Lock the style first, then batch.
5. **The engine is a strong UI and agent-tooling engine, not yet a general one.** A short engine track fixes that, and its first items are the same checks automation needs.
6. **Nothing says how far the game is from shippable.** Milestone docs disagree and the task list has been frozen since 11-07.

The central recommendation is the four-step path in the next section: clean the instructions, build machine guardrails, run a supervised week, then go to full autopilot with playtest notes as your main input.

## Path to automated development

The project is closer to automation than its interactive mode suggests. The hard parts exist: a tested gate, a single-writer hand-off between Design and Build, a rule for making reversible calls without you (PRINCIPLES section 0), an audit skill, a build lease, and an engine an agent can drive headless. What is missing is a trustworthy instruction set, machine checks in place of self-reported ones, a defined path for your playtest notes, and a decision that turns the loop on.

**The target loop:**

1. You playtest and leave notes (the F8 note box, or a message in the project chat).
2. Design turns notes, audit findings and the backlog into READY rows, making reversible calls as provisional and logging them.
3. Build takes the next row, builds it, and passes review and the gate.
4. An audit checks the landing against its READY row; scenarios, replays and balance bands run as machine checks.
5. A green batch is released, and you get one digest: what changed, what to try in the next playtest, which calls were provisional.

**What blocks it today:**

| Blocker | Why it matters unattended | Fix |
| --- | --- | --- |
| Session instructions contradict each other (see Workflow) | No human is there to correct an agent that follows the stale file | One cleanup pass over the skills, cloud.md and agent files; one line that states the mode in force |
| No hand-off trigger: Build starts only on your "Build: next row", and threads may not start each other | The loop stops after every step | A recorded "autopilot" mode and routines that chain Design, Build and audit |
| Verification is self-reported: the release check only confirms that `Review:` and `Gate:` lines are present | A wrong landing passes | Audit after every landing batch with the last audited commit recorded; CI gate on `develop` |
| No visual or feel regression check: goldens pin determinism, not looks or balance | UI and balance can drift silently | Scenario image goldens, replays in CI, balance target bands as a failing check (engine track 1, 3, 4) |
| Playtest notes have no route: they reach the repo only by a desktop copy and manual push, and nobody owns the inbox | Your one manual input gets lost | Define intake: notes go to Design, become READY rows, and the digest reports what happened to each |
| READY rows with wrong assumptions about the code (six of ten queued rows) | Build stops to ask | Design corrects the rows before the queue runs |
| No definition of "shippable" and no ordered backlog once READY empties | Automation has no target and no next thing | One milestone doc with a playable-release definition; a standing backlog order |
| The balance agent only runs on `main` | It cannot measure work on `develop` | Let it run on `develop` |

**Which approvals can become automatic:**

| Class of change | Today | Proposed |
| --- | --- | --- |
| Bug fixes, audit findings, data fixes that match locked canon, doc sync, cleanup with no behaviour change | Waits for "Build: next row" | Automatic |
| Engine track items that come with tests | Not planned | Automatic, from the ranked list |
| Reversible design calls (numbers, copy, provisional defaults) | Provisional, logged for review | Automatic and provisional, reported in the digest; you flip any by naming it |
| Balance retunes inside agreed target bands | Design proposes | Automatic inside the bands; outside them it comes to you |
| Replacing placeholder art inside the locked style | Each batch asked | Automatic after the style lock; one contact sheet per batch |
| Merging `develop` to `main` | Only on "release now" | Automatic when gate, audit and scenarios are green |

**What still comes to you:**

- New systems or features, and anything that changes the game's goal.
- Changes to locked canon, or two locked rulings in conflict.
- Anything that breaks existing saves without a migration.
- The art style lock, and paid image-generation runs (or a budget you set once).
- Public deploys (itch.io).
- Changes to agents, skills or this process itself.
- A failure that two targeted fixes did not solve: the row is parked and reported, and the loop moves to the next row.

**Guardrails:**

- A cap on slices per run and per day, so a bad night is small and usage stays inside your limits.
- A stop word ("pause") that halts all routines, and automatic pause after two failed gates in a row.
- One Build at a time through the existing lease; add the same for Design.
- Every landing carries its READY row id; the audit refuses a landing that does more than its row.
- A weekly run of this consistency audit, so drift is caught on a schedule.
- One digest per day in the project chat, not a message per step.

**Getting there in four steps:**

1. **Clean the instructions (one Design pass, docs only).** Fix the contradictions, correct the six READY rows, define the playtest intake, the shippable milestone and the backlog order, and write the autopilot mode as a decision for you to approve.
2. **Build the guardrails (four to six Build slices).** Scenario image goldens, replay and balance-band checks, CI on `develop`, automatic audit with a recorded commit. Each is small and testable.
3. **Supervised autopilot for a week.** Routines chain Design, Build and audit with a cap of about three slices a night. Releases to `main` stay on "release now". You read the digest and playtest.
4. **Full autopilot.** Releases to `main` become automatic on green. Your inputs are playtest notes and the short list above.

The game queue (Acts 2 and 3) can keep moving interactively during steps 1 and 2; nothing here blocks it.

## Code and specs

The code is in good shape. I ran the full gate on this snapshot: it builds with no warnings and all 3,491 tests pass (0 failed, 4 ignored), plus the layer check, 145 tile-tool tests and 30 planner tests, in about 9 minutes. Every finding from the 11-10 audit (B1 to B12 and the spec cleanup list) is fixed in code, and the numbers checked in the newer Corruption, Grafts and Roster code match their specs.

**New spec versus code mismatches (medium):**

| ID | Finding | Evidence | Suggested fix |
| --- | --- | --- | --- |
| N1 | Sensory Rot (Stage III, -15% on ranged attacks beyond 3 tiles) does not apply to the basic ranged attack, only to ranged skills. A test locks the gap in. | `rpgkit_rules/src/unit.rs:1610`, `rpgkit_battle/src/command.rs:1075` | Use the hit's ranged flag for the basic attack, or reword the passive |
| N2 | The Cold Steel keystone needs no masterwork weapon, and one forge rite unlocks it for every weapon family. The spec says Berek forges it into a masterwork weapon of that family. | `rpgkit_rules/src/unit.rs:1935`, `rpgkit_world/src/recipe.rs:259` | Require a masterwork main hand or record the rite per family; or amend the spec |
| N3 | Graft-era results never reach the player: weapon wear, mastery gains and remains are computed after a battle but the results screen does not show them, and the -50% dulled state is invisible in battle. The host commands do not expose them either. | `emberwake_gui/src/hub.rs:597`, `rpgkit_host/src/lib.rs` | One slice: result lines, battle tips, host keys |
| N4 | The 39 shipped doctrine recipe effects (all +1 on two axes) do nothing today and do not fit the one signed gauge the Act 2 spec expects. | `data/catalog/recipes.json`, `rpgkit_world/src/recipe.rs:213` | PR6-6 defines the mapping, PR6-12 reseeds |

Lower: three pieces of schema help are stale; the skill-rank machinery is still in the kit after ranks were retired; one PK2 spec row still says Stage III is immune to fear where the data says Weaken and Stun.

**The queued Act 2 and 3 rows assume things about the code that are not true.** None is a blocker, but Build will stop and ask on each, so they are worth fixing in READY before the queue runs unattended:

- **PR6-4:** the crypts count is a fixed number with no flag, so "0 in Act 1, 2 from Act 2" cannot be expressed yet. A recruit effect and the keepsake-by-class parser already exist.
- **PR6-6:** asks for `#[serde(default)]`, but the kit has no serde derives. The battle trigger already exists; only the recipe trigger is new.
- **PR6-7:** the named type `ArenaHazard` and a `graft_floor` function do not exist; hazards already have a rounds list that overlaps `every_rounds`; the captive snapshot is already held in the departed record; `requires_calcified` would put a game concept in the kit.
- **PR6-8:** there is no `Shop` struct; shops are raw data rows.
- **PR6-9:** "rout" is already the default objective id, and units already have Fled and Surrendered fates that the new "withdrawn" must be defined against.
- **PR6-10, PR6-11, PR6-14:** feasible as written.

**Layer split.** The rules side is clean: the seven game façade crates total 351 lines, dependencies point the right way, and the layer check passes. The remaining debt is one crate.

| Layer | Source lines |
| --- | --- |
| Engine (`aegis_*`) | 34.4k |
| Kit (`rpgkit_*`) | 80.8k |
| Game façades (7 crates) | 0.35k |
| `emberwake_gui` | 55.3k (about 24k non-test) |

- **`emberwake_gui` is a 55k-line generic RPG frontend living in the game layer.** Battle, party, shop, dojo, forge and world map screens hold almost no Emberwake nouns, and there is no kit UI crate for them. A second game would have to fork all of it. This is the largest architectural gap.
- **Rule copies in the GUI:** stat formulas restated as English (`party.rs:303-335`), the AI tactic table duplicated (`battle.rs:119-135`), walk-blocking and pathfinding in the region screen (`region.rs:480-524`), and five places that still decide things by matching English log text (`battle/draw.rs:989,1296`, `battle.rs:870`, `hub.rs:712`, `battle/tips.rs:434`).
- **The layer check has blind spots:** it skips the kit README (which names Skerrit and marrow), assets and engine docs; the lore word list lacks Strain, Graft, Vent and Tallow; nothing checks the game crates for rule logic creeping upward. Whether "graft" is a kit term or an Emberwake term has no recorded ruling, and kit code uses it about 215 times.

**Stale code docs.** `engine/README.md` credits the game crates with logic that moved to the kit and has no rows for the seven kit crates. The top README, GAME\_BRIEF and STRUCTURE each name files that moved.

## Lore and data

The shipped Act 1 data is clean: every unit, skill, item, encounter, dialogue and recipe reference resolves, with no duplicate or dangling ids, and every data bug from the 11-10 audit is fixed. What remains is text the player can see and lore docs that lag the data.

**Fix in data (small, safe to queue):**

| Finding | Where |
| --- | --- |
| Four meter warnings and all four faction descriptions start with the word "PLACEHOLDER" on screen | `worlds/sunk_ward/meters.json`, `factions.json` |
| Four quest rewards say "gold"; the currency is Scrap | `worlds/sunk_ward/quests.json` (`q_hook_*`) |
| The M5 quest is named "The Drowned Sluice"; every doc and the encounter call it The Crucible | `q_a1_m5` |
| All 46 characters have an empty faction; the six founders have empty background and personality text | `characters.json` |
| About 30 unreferenced catalog rows (10 passives, 14 classic skills, a few units, one item, one map) | `data/catalog/` |

**Lore docs that disagree with the data or each other (your call which side wins):**

1. **Relic effects.** CANON 5.1 promises signature effects (Dreamer's Vertebra +30% damage and a Strain floor, Quell's scalpel vents Strain, Grendel's hammer shockwave). The data gives plain stat bonuses.
2. **Class weapon lists.** The locked CANON 3.1 text names axe, mace and wand families that do not exist; the data lists are different for five of six classes.
3. **One hand or two.** Data and CANON 5.2 make Kaelen's billhook and Elyse's sprayer one-handed. CAST1\_GEAR, CAST1\_FOUNDERS and the Vanguard blurb in `world.json` still say two-handed.
4. **Node numbering.** The hub is node 01 in one CANON line and 00 everywhere else. Act 2 node 08 and the 2.1 / 2.1b labels differ between PK8 and CANON, which matters before PR6-12 seeds Act 2.
5. **Sabina's dose.** It gives a founder Marrow Lash in an Act 1 fight, against the "Act 1 is Cold Steel only" rule, and CAST1\_GAPS calls the dose retired while the data keeps it.
6. **Big bosses.** CAST1\_BOSSES and ACTS23 say the two-cell Amalgam shipped. READY parks that feature and the data has no footprint field.
7. **Other worlds.** `iron_reaches` and `tidewrack` are stubs with no recorded keep or retire status.

**Housekeeping in lore docs:** nine docs still carry a placeholder banner that CANON has overtaken; eight of the ten open question rows are already answered in CANON and only need marking; CAST1\_GAPS has four stale rows (one still says Orlov dies at M4); `CAST1_FOUNDERS` describes a `disciplines.json` that does not exist. Two questions are really open: Q-08 (Kaelen's involuntary graft trial) and Q-27 (which act holds Berek's quest).

## Workflow, planning and decisions docs

The hand-off between Design and Build works (0 open feedback items, 281 closed), but the instructions agents load at session start are the stalest part of the repo. An agent following them literally is told different things in different files. This is the main blocker for automation, because an unattended agent has nobody to correct it.

**Contradictions most likely to misdirect a session:**

| Issue | Evidence |
| --- | --- |
| The mode in force is stated two ways: planner-ops says "standard" (design ahead, pull the queue, parallel lanes), cloud.md and the decision say interactive. No single line says today's mode. | `.claude/skills/planner-ops/SKILL.md:14`, `docs/workflow/cloud.md:25,119` |
| The design-session skill only works on the desktop: Windows paths and Git Bash, no cloud branch. It is the Design thread's start skill. | `.claude/skills/design-session/SKILL.md:6-29` |
| build-session tells Build to pick up new slices automatically, plan three parallel lanes and cut the release PR. cloud.md says one task at a time and Merge & Release owns releases. | `build-session/SKILL.md:31-41`, `cloud.md:14,29,67` |
| The planner skill tells Build to add rows to TASKS.md, which only Design may edit. | `planner/SKILL.md:56-66` |
| Merge & Release is told to write `docs/metrics/TASK_LOG.md` and usage files. That folder was deleted. | `cloud.md:14,70,111` |
| Retired area threads and the 11:52 / 17:52 scheduled releases are still referenced in six files. | `content-edits.md`, `design-review.md`, `planner`, `planner-ops`, `milestone-status`, `patch-notes-write` |
| The balance agent requires a clean `main`, but the work it must measure is on `develop` (17 commits ahead). | `.claude/agents/balance.md:3,10` vs READY PR6-13 |
| AGENTROLES1 (approved) adds `integration` and `ui-author` agents. Neither file exists, and a READY row depends on one. | `docs/specs/AGENTROLES1.md:10,193`, `READY.md` INTEG-ASSETS1 |
| The build handover file still says "read this, then run /build-session" with state from 11-07, while cloud.md says the handover folder is unused. | `docs/handover/build.md:3`, `cloud.md:90` |

**Planning docs do not say how far the game is from shippable.** ROADMAP and MILESTONES define R1 to R4 differently, R5 to R7 are used but defined nowhere, MILESTONES still says "R1.1 polish in progress" while READY records 117 slices done through R4, and TASKS.md has been frozen since 11-07. What the docs do support: the backend silos R1 to R4 and Wave 1 are released on `main`; Wave 2 is 4 of 14 slices done; the 10 queued slices are all of the Acts 2 and 3 hooks, content and endings. PATCH\_NOTES is 643 commits behind.

**Decisions.** The index covers about a third of roughly 420 decision rows. Four workflow decisions (twice-daily releases, design review cycle, batched merges, gate once per task) are superseded only in prose. The seven B1 to B7 rulings you accepted today are still tagged provisional in READY and WAVE2\_PLAN.

**Waiting on you without a clear route.** Nine playtest-review rows in DESIGN\_REVIEW (11-11 to 11-17), 27 unticked boxes in NEEDS\_YOUR\_EYES (last touched 11-05, no owner since the split), and two playtest recordings from October still in the inbox. No doc says who clears the playtest inbox or how a playtest note becomes a READY row.

**Balance data is stale.** The current report is from 11-15 and shows boss clear at 80% against a 30 to 50% target. Since then the class pass retired 47 skill ranks and five goldens were re-recorded; the newer numbers live only in WAVE2\_PLAN and DESIGN\_REVIEW.

**Doc health.** Lint is clean on 224 files. Thirteen docs exceed the size limit (LAYOUT720 at 52 KB is the worst), and the largest files are not size-checked at all (`decisions/workflow.md` at 110 KB, PATCH\_NOTES at 129 KB). Fifteen live links are dead. The known-issues table has not been edited in 147 commits.

