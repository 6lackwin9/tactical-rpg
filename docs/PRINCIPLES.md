# Guiding Principles & Autonomous Decision Framework

Authoritative cross-cutting heuristics distilled from settled decisions (`docs/DECISIONS.md`, `docs/decisions/workflow.md`, `games/emberwake/docs/decisions/*.md`), user taste (`games/emberwake/docs/preferences/*.md`), architecture contracts (`engine/docs/authoring/layer_rule.md`), and canon (`games/emberwake/docs/lore/CANON.md`).

- **`docs/DECISIONS.md`** records *specific historical rulings* per system.
- **`games/emberwake/docs/lore/CANON.md`** tracks *campaign & worldbuilding maturity* (`[LOCKED]`, `[PROVISIONAL]`, `[PLACEHOLDER]`, `[MISSING]`).
- **`docs/PRINCIPLES.md` (this file)** records *reusable decision rules* so any session can test options against project principles and keep moving when the user is away.

## 0. Autonomous Decision Protocol (When the User Is Away)

Whenever an open choice arises in `.1` spec authoring, `FB-nn` / `UN-nn` Build feedback triage, `CL-nn` lore/data gaps, or audit follow-ups, run this 5-step gate without stalling Build:

1. **Check `[LOCKED]` & Settled Decisions First**: Verify `CANON.md` (`[LOCKED]`) and `docs/DECISIONS.md`. Never silently override an explicit user ruling; if a new finding suggests changing one, keep the current rule running and log a review proposal.
2. **Score Candidate Options Against `P-ARCH` .. `P-UI` Below**: Eliminate any option that violates a principle (e.g., game-layer rule hacks, silent golden drift, player-UI text bloat, dead loot, or hardcoded `[PLACEHOLDER]` lore).
3. **Apply Deterministic Tie-Breakers**: When two options both satisfy the principles, pick the option with:
   - **(a) Lowest undo cost**: data/JSON or asset swap `<` schema/ruleset config `<` isolated kit helper `<` cross-crate state change.
   - **(b) Cleanest reusable kit generality**: parameterised `kits/rpg/` behaviour over one-off `emberwake_*` special cases.
   - **(c) Zero unintended golden movement**: strict no-op defaults unless the slice explicitly intends a named golden change.
4. **Apply Immediately as `[PROVISIONAL]` & Log in `docs/DESIGN_REVIEW.md`**:
   - Put the chosen option straight into `docs/READY.md` (Section 1 or 4), the `.1` / feature spec, or `CANON.md` Section 5.1 tagged `[PROVISIONAL]` so Build stays unblocked (`workflow.md` `"Build feedback on the recommended option"`).
   - Append a row to `docs/DESIGN_REVIEW.md` (`Date`, `Item`, `What was asked`, `Applied` citing principle IDs like `P-ARCH-01, P-COMBAT-02`, `Where`, `Undo cost`) so the user can approve or flip it asynchronously at low cost.
5. **Hard-Stop Exceptions (Never Auto-Decide)**: Always pause and ask the user first before:
   - Irreversible git/release actions (`push` or PR merge to `main`, `itch.io` deploy, deleting user branches or files).
   - Paid external API spend (e.g. Gemini image API batches without prior count/model approval; local ComfyUI generation is free to run).
   - Breaking shipped release saves without a migration path, or resolving a direct contradiction between two `[LOCKED]` user rulings.

## 1. Architecture, Layering & Data Hygiene (`P-ARCH`)

- **`P-ARCH-01` — Three-Layer Question Test & Kit-First Rule Library** (`layer_rule.md`, `SPLIT1`, `"Engine scope: a rule library"`):
  - *Rule*: Generic tech any genre needs belongs in `engine/` (`aegis_*`); reusable RPG mechanics, schemas, validation, and AI belong in `kits/rpg/` (`rpgkit_*` + `schema.json`) with zero game/lore nouns; `games/<id>/` holds thin shell/UI crates plus data and assets.
  - *Option test*: **Prefer** building a clean, parameterised `kits/rpg/` rule capability **over** coding game-unique rule logic, deck/squad validators, hardcoded sim tables, or English combat-log string scraping in `emberwake_*`.
- **`P-ARCH-02` — Additive Engine Options & Ruleset Isolation** (`"Engine options are additive"`, `"Ruleset placement"`):
  - *Rule*: The engine and kit accumulate reusable rule options (`classic` vs. `timeline`, `strain_enabled`, `melee_diagonal`); each world opts into a coherent set, and dropped game experiments stay available in the kit if cleanly isolated.
  - *Option test*: **Prefer** gating new or incompatible behaviour behind explicit ruleset flags/numbers that default to a strict no-op (`0` unintended golden diffs) **over** mutating shared baseline formulas.
- **`P-ARCH-03` — Fixed Behaviours in Code; Numbers & System Terms in Data** (`"Behaviours fixed, stats as data"`, `TERMS1`, `"Tier numbers"`):
  - *Rule*: Code defines *how* a behaviour works; data defines *names, scaling inputs, and numbers*. System labels (`currency`, `upkeep`, intent names, `"Cold Steel"`) resolve through one `terms` table, and skill rank rows store explicit numbers rather than hidden engine formulas.
  - *Option test*: **Prefer** `schema.json` fields, `tuning.json`, and `terms.json` **over** magic constants in Rust or hardcoded lore labels.
- **`P-ARCH-04` — Schema, Consumer & Content-Writer Unity** (`Builder Authority Matrix`, `"Content edits"`):
  - *Rule*: Every new data field lands in `source.rs`/`store.rs` *and* `kits/rpg/schema.json` together, is wired across every consumer (battle resolver, planner, AI, Dojo, Party UI, Content Forge, CLI/sim, save/load, content checks), and ships data exclusively via `aegis content`.
  - *Option test*: **Reject** raw-JSON row keys that bypass `schema.json`, partial "done with caveats" wiring, shadow legacy fields, or hand-edited `games/*/data/*.json`.
- **`P-ARCH-05` — Bottom-Up Silo Limit-Testing Before Integration** (`"Silos: data to tweak"`, `"A silo's goal is a limit test"`, `"Master plan first"`):
  - *Rule*: Build and stress-test each `MASTERPLAN1` block in isolation on direct/placeholder values to find where the mechanic breaks before coupling cross-system pipelines. Once a silo locks, allow only bug fixes and data tuning.
  - *Option test*: **Prefer** testing a system's bounds inside its own silo seam **over** entangling unfinished systems or growing the master task list sideways.
- **`P-ARCH-06` — Additive State & Disposable Dev Saves** (`"Saves and data edits"`, `"Save upgrades at game version updates"`):
  - *Rule*: Keep a single clean save version during pre-release development (while preserving the migration hook for post-release upgrades), and gate mid-campaign mechanics by act (`durability.from_act`, `tracks`, `body`).
  - *Option test*: **Prefer** additive optional save/data keys with clear load notes **over** maintaining v1..vN development save migrations.
- **`P-ARCH-07` — Single-Writer Sessions & Small Feel Tuners** (`"Two-session split"`, `"Engine goal: agents build, humans judge feel"`, `"No upkeep on unused debug tools"`):
  - *Rule*: Respect single-writer boundaries (`READY.md` = Design, `FEEDBACK.md` = Build, `ART_UI_REQUESTS.md` + `assets/**/README.md` = Art/UI handshake). Build small, game-agnostic tuners/arenas for subjective feel checks, and prune debug scaffolding nobody uses.
  - *Option test*: **Prefer** a tiny reusable Dojo/Gallery/HTML tuner for human feel sign-off **over** guessing subjective feel in full-game loops or maintaining dead debug modes.

## 2. Combat, Pacing & AI Symmetry (`P-COMBAT`)

- **`P-COMBAT-01` — Deterministic Telegraphing Over Hit/Miss RNG** (`CANON` 2.1, `DR1`, `FX1-L2`, `ACTS23` Q-A4):
  - *Rule*: Combat uses deterministic hits—no accuracy/evasion rolls, stealth/facing cones, or d100 tables. Enemy first-block intents and impact timings are telegraphed (`DR1`), and rare crisis randomness (`roll_table`) uses small, seeded, bounded pools.
  - *Option test*: **Prefer** readable spatial/timeline counterplay (wind-ups, poise breaks, flanks, area telegraphs) **over** output RNG that invalidates a clean plan.
- **`P-COMBAT-02` — One-Click Intent Commitment & Derived Modes Over Duplicate Catalogs** (`INPUT1`, `ACTS23` 5.1, `PK1`):
  - *Rule*: Clicking target -> skill commits the round's single intent and target while ghost-filling follow-ups; the Channel opener is a non-locking mode toggle that swaps planned skills in place. Mode shifts (`Field` bonuses, `Channel` `1.25x` power + pierce) derive from base skills with optional per-skill overrides, and no skill appears in more than two menu rows.
  - *Option test*: **Prefer** derived mode rules + compact per-skill override blocks **over** authoring parallel duplicate skill families or adding extra per-block clicks.
- **`P-COMBAT-03` — Brisk Encounter Pacing & Two-Act Bosses** (`PACE1`, `MORALE1`, `preferences/design.md`):
  - *Rule*: Fights stay sharp: mobs `~3` rounds, elites `~5`, bosses `~8` split into Phase 1 (`~5` rounds mechanic check) and Phase 2 (`~3` rounds lethal enrage race at `50%` HP or round `6`, revealing full boss plans), while group morale routs clean up stragglers.
  - *Option test*: **Prefer** decisive phase enrages, morale breaks, and light-touch data tuning (`DR1/DR2`) **over** high-HP attrition, boss self-heal stalls, or free mid-boss party buffs.
- **`P-COMBAT-04` — Fun From Level 1 & Active Offence on Every Role** (`CANON` Q-41, `"Every role attacks"`):
  - *Rule*: Onboarding starts at level 1 with real tactical choices immediately, and all six classes (including Priest/support) attack regularly in normal rounds so attack-driven economies (like weapon durability) engage the whole party.
  - *Option test*: **Prefer** hybrid support/control kits that weave attacks between heals/guards **over** passive back-row heal-bots or late-blooming empty level-1 kits.
- **`P-COMBAT-05` — Symmetric Resolver Rules & Non-Cheating AI** (`RULESET1_BALANCE1`, `AI1`, `PK1`):
  - *Rule*: Allies and enemies obey the same positioning, area friendly-fire, status registry, and trigger rules unless an explicit asymmetric pillar governs them (player `0` recovery + AP vs. enemy recovery + `0` AP), and AI never reads hidden player tells.
  - *Option test*: **Prefer** shared kit AI heuristics (`smart_skills`, `channel_ai_edge`, role tables) **over** bespoke per-encounter AI scripts or hidden enemy rule exemptions.
- **`P-COMBAT-06` — Self-Paid Risk Costs Never Self-KO** (`PK1` recoil & `Vent` rules):
  - *Rule*: Voluntary or meter-recoil HP costs (Strain recoil, `Vent` `-15%` max HP) either clamp at `1 HP` or grey out when HP is insufficient; an enemy or hazard must still finish the unit.
  - *Option test*: **Prefer** leaving reckless casters exposed at `1 HP` (or blocking unpayable casts with a clear hover reason) **over** self-inflicted knockouts during planning/resolution.

## 3. Class Identity, Progression & Economy (`P-PROG`)

- **`P-PROG-01` — Equal Stat Budgets, Distinct Shapes & Unique Main Weapons** (`"Class balance"`, `"Class weapon lineup"`):
  - *Rule*: Every playable class shares the exact same level-1 stat sum (`62`) and per-level growth (`5`), plus `3` masterable weapon families (`3 / 2 / 2`) where each class's `main` (`proficiency 3`) weapon family is unique across the six classes.
  - *Option test*: **Prefer** expressing class fantasy through stat distribution, class deck, and unique main-weapon techniques **over** raw stat budget inflation or overlapping signature weapons.
- **`P-PROG-02` — Parallel Asymmetric Branches With Separate Substrates** (`PK2_GRAFTS`, `"Cold Steel weapon techniques"`):
  - *Rule*: From Act 2, Cold Steel and Inoculation mirror each other in power budget and progression depth (`3-4` heavy slots + `1` shared keystone slot of either theme) while taxing separate substrates: Cold Steel taxes **weapons, mastery & Scrap/salvage** (Hearth, durability), while Inoculation taxes **the body & Strain/Calcification** (Slab, monster grafts, `5/15/30/50` Strain floor).
  - *Option test*: **Prefer** structural parity with contrasting costs (`marrow` never spends durability; Cold Steel never spends Strain) **over** letting one doctrine dominate or merging their currencies.
- **`P-PROG-03` — Blunt Secondary Thresholds Over Micro-Bookkeeping** (`ACTS23` 5.2, `PK2`, `ITEMUSE1`, `FORGE1`):
  - *Rule*: Reserve multi-band tension for the headline combat meter (Strain); keep secondary trackers simple and binary—weapon durability penalizes *only* at `0` (`-50%` dulled, zero intermediate brackets), battle pouches hold only player-packed items (`0` free class potions), and gear budgets warn softly before hard gates.
  - *Option test*: **Prefer** a single unmistakable threshold (`0` durability, `50` Strain gate for `Vent`) **over** fiddly `75%/50%/25%` wear brackets on secondary items.
- **`P-PROG-04` — Zero Dead Loot & Anti-Grind Authored Economy** (`PK2` salvage/organs, `PURSE1`, `REWARD1`):
  - *Rule*: Every drop has a clear sink—items break down into tier/faction salvage + Scrap for Hearth repairs/reforges, and aberration cores + slot materials fuel Slab grafts. Story encounters pay authored purses once, and partial fights pay per defeated enemy weight so road grinding barely covers rest upkeep.
  - *Option test*: **Prefer** bounded, authored encounter rewards and universal salvage sinks **over** vendor-trash clutter or repeatable gold/XP farm loops.
- **`P-PROG-05` — Expressive Camp Preparation, Fast In-Battle Execution** (`SKILLUI1`, `PK2`):
  - *Rule*: Loadouts (`5` class + `3` extra skills), pinned skill ranks (`< >`), follow-up priorities, techniques, grafts, and `3`-slot pouches are configured *outside* combat and locked once battle starts.
  - *Option test*: **Prefer** rich between-battle buildcraft in camp/hub popups **over** mid-battle rank pickers or sprawling combat submenus.

## 4. Campaign Agency, Stakes & Lore Maturity (`P-CAMP`)

- **`P-CAMP-01` — Protected Act 1 Onboarding; Choice-Then-Fate Fail-Forward in Acts 2–3** (`CANON` 1–2, `"Losses: choice, then fate"`, `DR1`):
  - *Rule*: Act 1 protects onboarding (non-lethal `wounded` until rest, zero Strain/Calcification/durability). The first permanent loss (`M5` Crucible) is a deliberate player choice; Act 2–3 losses emerge from systemic risk (bleed-out, capture, Stage V, sieges), while standard fight losses fail forward via clock advance, hub fallback, or fallen-unit keepsakes/organs rather than dead-end Game Overs.
  - *Option test*: **Prefer** lasting narrative/roster consequences that keep the campaign moving **over** reload-forcing Game Over screens or unwarned Act 1 punishment.
- **`P-CAMP-02` — Flat Flag Reactivity & Shared World Loot Pools** (`M4 quest/variant answers`, `DLG1`):
  - *Rule*: Quests stay internally linear while world reactivity (`gone_flags` when NPCs die/leave, variant givers/objectives, shop return guards) runs through one flat flag registry. Unique quest rewards draw from a shared world pool (`loot_picks`) while generic rewards always pay out.
  - *Option test*: **Prefer** flat, inspectable `quest.<id>.*` / world flags and shared choice pools **over** hidden per-quest state machines or missable one-quest-only build-defining loot.
- **`P-CAMP-03` — Lock Mechanics Early (`[LOCKED]`); Keep Flavour Replaceable (`[PLACEHOLDER]`)** (`CANON` Sections 3–5, `Q-90`):
  - *Rule*: Lock the campaign spine and gameplay-linked roles/hooks; treat unfinalized character names, ages, bios, dialogue voices, portraits, and palettes as `[PLACEHOLDER]` or `[PROVISIONAL]` data behind stable, never-renamed catalog IDs.
  - *Option test*: **Prefer** shipping `[PROVISIONAL]` data-driven placeholders in `games/<id>/data/**` + `terms.json` **over** blocking Build on unfinished writing or hardcoding lore strings in Rust.
- **`P-CAMP-04` — Earned Story Rewards, Qualitative States & Grounded Tone** (`PACE1`, `MORALE1`, `CANON` 2.1):
  - *Rule*: Narrative buffs (`Empowered`) are earned only through optional story rescues/choices; morale and doctrine show qualitative states (`Steady..Broken`) instead of raw engine integers; and under-18 characters remain strictly non-combat narrative stakes.
  - *Option test*: **Prefer** diegetic state labels and earned world consequences **over** exposed engine math in story UI or shock-value combat tropes.

## 5. UI, Visual Clarity & Asset Pipeline (`P-UI`)

- **`P-UI-01` — Subtractive Player UI vs. Telemetry-Rich Debug Surfaces** (`preferences/ui.md`, `preferences/design.md`):
  - *Rule*: Debug tools (Dojo, Forge, `T` inspector) show full math and telemetry; player screens ask *"what can be removed while keeping the signal?"* Use visual pips for small caps (`<= 5`), compact `xN` numerals above `5`, never repeat costs already shown on the timeline strip, and format hover cards as `compact tag row + 1 effect line + 1 highlighted intent delta`.
  - *Option test*: **Prefer** icons, pips, and scannable tags **over** prose paragraphs, exposed multipliers/coordinates, or duplicate `3/5` text counters.
- **`P-UI-02` — Calm Default Board & Progressive Inspection** (`REGIONGROUND1`, `VISCLAR1`, `INPUT1`):
  - *Rule*: Ground textures stay muted and low-contrast beneath crisp `48 px` units/props. By default, show bars and one quiet dashed target arrow only for the selected, targeted, hovered, or right-click-inspected unit (`Alt` shows all), and hide planning clutter during playback.
  - *Option test*: **Prefer** hover/`Alt`/right-click progressive disclosure **over** always-on board spaghetti, rainbow terrain mixes, or playback arrow clutter.
- **`P-UI-03` — Consistent Ergonomics, Scalable Text & Bounded Data Layouts** (`TEXTFIT1`, `DRAG1`, `LAYOUTDATA1`, `"Compact frame"`):
  - *Rule*: Prefer popups over full screens (`Right-click` / `X` closes); support drag-and-drop alongside click workflows everywhere; scale all text across 4 sizes (`0.85x..1.3x`, `dim` `#9ca3af` contrast) verified by `text_audit`; keep one frame corner size per screen; honour `reduce_motion`; make every floating panel unlockable-to-drag and minimizable toward the corner holding its minimize button so a named button (e.g. `"Legend"`, never `"Show HUD"`) toggles in place and drags with press-and-drag while unlocked (starting open, centered, and unlocked when every map corner holds a node or feature); and send `ui/layouts/<screen>.json` back to Rust if it needs new node kinds, arithmetic, or `> 60` lines.
  - *Option test*: **Prefer** computed text-fit layouts and shared widget conventions **over** fixed pixel widths, drag-only inputs, or complex logic inside JSON layouts.
- **`P-UI-04` — Placeholder-First Assets & Layered Fallbacks** (`"Placeholder-first assets"`, `"Asset layers and reuse groups"`):
  - *Rule*: Missing art or audio never blocks code/data slices: Build wires the final path, copies the nearest valid asset as a placeholder, and logs it in `FEEDBACK.md` so Design queues an `ART_UI_REQUESTS.md` Inbox item for in-place replacement; missing assets fall back `game -> kit -> engine`.
  - *Option test*: **Prefer** landing green features with final-path placeholders **over** parking gameplay tasks on art or inventing temporary code fallbacks.
- **`P-UI-05` — Lightweight Preview Labs First in `/art-ui-session`** (`ART_UI_STUDIO.md`, `"Efficiency pass"`):
  - *Rule*: Never iterate UI layouts, VN choreography, or animations directly in Rust or inside the full `emberwake_screen_flow.html` bundle; prove the look first in a tiny standalone HTML/GIF lab (`tools.mock_ui.anim_lab`) inside `/art-ui-session` and build small tuners for user-tuned visuals.
  - *Option test*: **Prefer** a focused 1-screen mock/GIF sign-off **over** multi-round Rust rebuilds on unapproved visuals.
- **`P-UI-06` — Local-First Generation, Faction Readability & Mandatory Self-Review** (`preferences/assets-world.md`, `"Faction lighting"`):
  - *Rule*: Draft and limit-test locally first (`FLUX.2 Klein 4B` / `ERNIE-Image Turbo`, alpha `"good enough"` bar) before proposing paid image-API runs; apply faction silhouettes and rim lighting; obey VN 4-pose rules (no leg crossover, flat-planted feet, threat via weapon motion); and **visually inspect every generated/edited image against prompt + reference** before accepting.
  - *Option test*: **Prefer** code recolours/fixes and local ComfyUI takes verified by immediate visual inspection **over** unreviewed generations or defaulting to external API calls.

## 6. Quick Scorecard for Option Selection & Audits

When choosing between `Option A` and `Option B` (or auditing a landed slice), verify:

1. **Layer & Kit Purity (`P-ARCH-01..04`)**: Does it live in the lowest valid layer, keep `schema.json` + all consumers unified, and leave `classic`/untouched goldens at `0` unintended diffs?
2. **Tactical Clarity (`P-COMBAT-01..06`)**: Is it deterministic, derived rather than duplicated, brisk (`3/5/8` rounds), active for all 6 classes, AI-symmetric, and non-self-KO?
3. **Asymmetric Economy (`P-PROG-01..05`)**: Does it preserve the `62 / +5` class budget, unique main weapons, separate Cold Steel vs. Inoculation currencies, blunt secondary thresholds, and zero dead loot?
4. **Malleable Lore & Fail-Forward Stakes (`P-CAMP-01..04`)**: Does it protect Act 1, fail forward in Acts 2–3, use flat flags, and keep `[PLACEHOLDER]` flavour 100% data-driven?
5. **Subtractive UI & Placeholder Flow (`P-UI-01..06`)**: Does it remove redundant player-UI text, keep the board calm, ship placeholders rather than blocking on art, and log any autonomous `[PROVISIONAL]` call in `docs/DESIGN_REVIEW.md`?
