# CAST1: the Act 1 combat cast under the timeline ruleset

Status: proposed (spec 2026-10-25, from the user decisions Q-41, Q-49, Q-53, Q-55, Q-56, Q-58, Q-61, Q-71, Q-73 (a,
planner pick), Q-74 and the CANON rows "Round intent", "Combat group (1.1)" and "Content group (1.7)" in
`games/emberwake/docs/decisions/lore.md`; rewritten by CAST1-R the same day to use the HOOKS1 hooks instead of
stand-ins; every number is a first pass for the RULESET1 feel sims)
Depends on: KIT1-N; RULESET1 (R, S, T, IN, PL, P, K, V, AI, D, A), ENCOUNTER2 (T, H, K, W), DERIVE1 (S, R, D), HOOKS1
(S, F, G, Z, E, K, M, H, U, AI, C, UI, A; Q for M2), ROSTER1-L (join level, `party_min`, capture, keepsakes),
CAMPAIGN1 (the Alert inputs that read the tags), the M0 `sunk_ward` world skeleton, and CAST1-S (section 6)
Lane: logic (CAST1-S, a small schema option task), asset (CAST1-A1 to CAST1-A7), balance (CAST1-B)
Touches: `kits/rpg/schema.json` and the gear token list (CAST1-S only); through `aegis content`:
`games/emberwake/data/catalog/{item_families,items,skills,passives,units}.json` and
`games/emberwake/data/worlds/sunk_ward/{characters,items,disciplines,loot,encounters}.json`; palette files under
`games/emberwake/assets/units/*/*/palettes/` and copied placeholder icons (CAST1-A7)

Parts (each under ~300 lines / 25 KB):

| File | Holds |
| --- | --- |
| this file | goal, scope, conventions and the number basis, out of scope, tasks, content tests, open questions |
| [CAST1_FOUNDERS.md](CAST1_FOUNDERS.md) | the six founders' kits: base class, skills, passives, keystone, trait, gear, deck and order lists |
| [CAST1_GEAR.md](CAST1_GEAR.md) | new weapon families, items, consumables, disciplines and proficiency passives, relics, keepsakes, loot rows |
| [CAST1_BESTIARY.md](CAST1_BESTIARY.md) | the Act 1 regular enemies: units, skills, traits, AI and decks |
| [CAST1_BOSSES.md](CAST1_BOSSES.md) | Vane, Quell, Grendel, Orlov, the Calcified Amalgam, Nemeth: kits and phases |
| [CAST1_ENCOUNTERS.md](CAST1_ENCOUNTERS.md) | the slice's and the rest of Act 1's encounter rows, terrain rows, objectives, rewards |
| [CAST1_ASSETS.md](CAST1_ASSETS.md) | the placeholder table (existing assets only) and the new art for the user to approve |
| [CAST1_GAPS.md](CAST1_GAPS.md) | the gaps (resolved by HOOKS1, ROSTER1, CAMPAIGN1, or still open) and the contradictions |

History check (`docs/HISTORY.md`): nothing deprecated is re-proposed. The cut rolls (evasion, accuracy, Blind as a
miss chance, slip checks, the 50% hallucination) stay cut: every lore effect that leaned on one is restated on an
existing lever (Q-56) or listed as deferred. Stance damage multipliers stay cut (intents only limit skills). Composure is
not used in Act 1 (DERIVE1: corruption effects start in Act 2). The Crossbowman's "targets one in a windup" is not
adopted (HQ2 a; HOOKS1_TASKS 7 HISTORY row).

## 1. Goal

The vertical slice and the rest of Act 1 are data only (Q-55, Q-56). CAST1 is that data for combat: the six founders'
Act 1 Cold Steel kits sorted by round intent, the Act 1 enemies with their move sets and recovery blocks, the six boss
kits with phases, the new weapon families, and the encounter rows the asset lane writes with `aegis content`. The
slice (0a, 0b, 0c, the dilemma job, M3, M5) must be fun from the first fight at level 1 (Q-41): each founder has two
usable actives at level 1, every intent has at least one option through the basics and the pouch, and the prologue
enemies are tuned so the first lesson (a Snap into a windup breaks it) works on the first try. Lore effects use the
kit hooks of HOOKS1 (forced moves, ground and arena hazards, triggers and riders, unit meters, HP costs, guard bypass,
AI tactics, capstone unlocks, class names, zones), ROSTER1 (join level, minimum party, capture, keepsakes) and
CAMPAIGN1 (mission tags). A lore part no spec covers is deferred, named in its row and listed in CAST1_GAPS; no
system is designed here.

## 2. Scope

| # | Part | Where |
| --- | --- | --- |
| 1 | Founders' kits under round intents (Kaelen, Vesper, Maren, Hollis, Dunstan, Elyse), level 1 at the prologue | founders |
| 2 | Weapon families as data: polearm (billhook), scourge, censer, bone-saw, chemical sprayer (all Act 1) | gear |
| 3 | The Act 1 bestiary: Footman, Crossbowman, Bailiff Enforcer, Flagellant, Purifier, Dredger, Render-Beast, Ash-Crawler, Calcified Husk | bestiary |
| 4 | Boss kits: Vane (M1), Quell (M2, the marrow valve), Grendel (M3), Orlov (M4, the vendetta), the Amalgam (interlude, 1-cell staging), Nemeth (M5, unkillable, hold escape) | bosses |
| 5 | Encounters: 0a-0c, the dilemma job, M3, M5 (slice); M1, M2, M4, the interlude (M2b, Q-86) | encounters |
| 6 | Names, ids and the asset placeholder table | this file 3.2, assets |
| 7 | First-pass numbers, marked FP | everywhere |
| 8 | Tasks, content tests, gaps | this file 6-7, gaps |
| 9 | Open questions CQ1 onward | this file 9 |

## 3. Conventions

### 3.1 First-pass numbers (FP)

Every number in the CAST1 tables is a first pass for the RULESET1 feel sims (`aegis sim feel`) and the balance lane's
Act 1 runs; a table whose caption says FP may be retuned by the balance lane without a design pass. The basis:

| Quantity | Rule | Source |
| --- | --- | --- |
| Damage | `power_mult` = lore flat damage / 25, rounded to 0.05 (Strike 1.0 is about a lore 25) | Q-59; RULESET1 2.4 anchors |
| Stagger | lore stagger x 0.5, rounded (Snap 4-18, Standard 4-12, Heavy 12-20) | RULESET1 5.6 |
| Heals | `heal_base` + KNOW x `heal_mult` comes to about lore heal x 0.75 at level 1 | Q-59 |
| AP | the lore `ap_cost` (Cold Steel 2-4); a `sure_break` skill costs 1 more than its plain twin | CANON 2.1; RULESET1 P14 |
| Speed | `blocks`: Snap 1, Standard 2, Heavy 3-4; `recovery_blocks` (0-2) only on enemy-used rows | RULESET1 4.1-4.2 |
| Armour | armoured enemies use the unit `armor` override at 25, 35 or 50; unarmoured ones -1 (derived) | Q-49; RULESET1 A3 |
| Poise | party and most enemies derived (`max_poise` = CONS x 2 + gear mitigation); fixed `poise` for bosses and the tutorial Footmen | RULESET1 P1-P2 |
| Statuses | potency 60-90 (resisted by INT x 2.5 and `resist`); rounds 1-3; magnitudes inside the status row's range | `status.rs` rows |
| Collisions | `collide_mult` 0.5-1.2 x the attacker's power, `collide_stagger` 6-12 (HOOKS1 FM4) | HOOKS1_TASKS 6 |
| Levels | founders level 1 at 0a; regular enemies level 1-3; bosses 3-6 (tables) | Q-41 |

Level-1 founder profiles (FP, class base stats before the 4 free points; HP = 60 + CONS x 8 + STR x 2):

| Founder | Class (display, HOOKS1 12) | HP | Armour rating | Max Poise with starting gear and passives |
| --- | --- | --- | --- | --- |
| Kaelen | `warrior` (Vanguard) | 222 | 11 | 34 + 8 + 20 (Braced Poise) = 62 |
| Vesper | `rogue` (Gutter-Stalker) | 142 | 6 | 16 + 4 = 20 |
| Maren | `priest` (Heretic Acolyte) | 168 | 3 | 24 + 2 + 8 (Anatomical Precision) = 34 |
| Hollis | `ranger` (Arbalestier) | 148 | 6 | 18 + 4 = 22 |
| Dunstan | `brawler` (Slag-Brute) | 232 (with Hypertrophy) | 9 | 36 + 6 = 42 (steadfast below 50% HP, Spite) |
| Elyse | `mage` (Ichor Alchemist) | 136 | 3 | 16 + 2 = 18 |

### 3.2 Ids and names

- Ids are snake_case and stable (Q-90); names follow the naming guide (`lore/specs/naming_and_asset_guide.md`): Cold
  Steel skills and gear in smith and soldier words, enemy types as rank or role plus function, relics as owner plus
  object. Display names stay at most 14 characters where the UI lists them; a longer lore name gets a short form in
  the tables (`name` holds the short form, `description` the full one).
- Catalog rows (skills, passives, items, families, units) are global and appended only (no Ashlands row changes,
  RULESET1 3.3). World rows live in `sunk_ward`: characters, named items, disciplines, loot, encounters.
- A world enemy character and its base unit share the id (they are different kinds). If `content check` refuses
  that, the characters take the prefix `sw_` and the encounter refs follow.
- Founders: world characters `kaelen`, `vesper`, `maren`, `hollis`, `dunstan`, `elyse` (role party) on catalog units
  `founder_kaelen` ... `founder_elyse` (side party, tier hero, level 1, `stat_mode` derived, `spend` {}).
- Encounters use the lore ids: `enc_pro_0a`, `enc_pro_0b`, `enc_pro_0c`, `enc_evt_01` ... `enc_evt_06`, and the
  dilemma job `enc_job_corvo_gate`.
- Class display names (Q-53) come from the `sunk_ward` manifest's `classes` overlay (HOOKS1 12, HOOKS1-A); the
  character `title` is "" or an epithet ("Slag-Jaw").

### 3.3 Descriptions

`description` = `"<Intent>, <speed>: <what it does>."`, naming only what the data does (a deferred lore part is not
promised). Example: "Assault, Snap: a pommel jab that staggers; timed into a windup it breaks the strike." Use
`{power}`, `{stagger}`, `{ap}`, `{potency}`, `{rounds}` tokens where the skill text supports them.

## 4. What each part needs from the other specs

| Need | Provided by | Used for |
| --- | --- | --- |
| `round_intent`, `ap_cost`, `recovery_blocks`, `stagger`, `windup_poise`, `unbreakable`, `sure_break`, `armor_piercing`, `intent_buff` | RULESET1-S | every skill row |
| `deck` (active, passive, keystone, order), unit `poise` | RULESET1-S, -D | founders, enemies, bosses |
| passive `slot`, `stagger_taken`, `foresight` `range` | RULESET1-S | passives, traits |
| `polearm`, `scourge` weapon types, the family rule `reach` | RULESET1-S, -K | Kaelen, Vane, Flagellants |
| `steadfast`, `stealth` statuses | RULESET1-S | Ash-Cloud Fade, Resilience Ward, Phalanx, Spite, Nemeth |
| the `timeline` ruleset row, `active_guard`, `flank_cut`, `max_poise` | RULESET1-A, DERIVE1-D | the basics and poise |
| `rows`, `unkillable`, `timeout_outcome`, `stay_behind`, `kill` | ENCOUNTER2-T, -H, -K, -W | arenas, M2 cull, M5 |
| skill `forced_move`, `hazard`, `triggers`, `riders`, `meters`, `hp_cost`, `ignore_guard` | HOOKS1-S (rows run by -F, -G, -E, -K, -M) | founders, bestiary, bosses |
| passive and item effects `forced_move`, `trigger`, `meter`; the `hazards` and `unit_meters` rows | HOOKS1-S, HOOKS1-A | traits, passives, the pavise |
| unit `tactics`, `target_pref` SOFTEST and TOUGHEST | HOOKS1-S, -AI | Footman, Flagellant, Bailiff, Purifier, Nemeth |
| encounter `arena_hazards`, phase effect `deck`; later `spare_once`, trigger `spared`, effect `heal` | HOOKS1-S, -H, -U; HOOKS1-Q | M3, the capstones; M2 |
| world `classes`; the `zones` rows | HOOKS1-C, -Z, -A | names, every encounter's `zone` |
| `recruit` `level`, encounter `party_min`, `then` `capture`, character `keepsake` | ROSTER1-L, ENCOUNTER2-W | Dunstan, Elyse, the Crucible, keepsakes |
| the Alert inputs reading `loud` and `quiet` | CAMPAIGN1 (M3; CAMPAIGN1_TASKS 4.2) | encounter `tags` |
| `censer`, `bone_saw`, `sprayer` weapon types | CAST1-S (this spec, section 6) | Maren, Quell, Elyse, Purifier |

## 5. Out of scope

- Act 2-3: Inoculated skills, Strain, Calcification, the Channel intent, grafts, the founders' Inoculated variants
  and signatures (Vesper's Umbral Flensing, Maren's Black-Ichor Transfusion and Martyr's Shroud, Hollis's Predator
  Reticle Pulse, Dunstan's Molten Marrow Eruption, Elyse's Pressurized Vitriol Siphon and Spore-Bloom); the Marrow
  Horror, the Inquisitor Proctor and the Guild Carver as units (templates only; Act 2 bestiary, Q-76); Act 2 recruits'
  base trees.
- Any system: CAST1 only fills the fields of HOOKS1, ROSTER1, CAMPAIGN1 and the specs in section 4. The lore parts no
  spec covers (auras, execute riders, barriers, terrain change, multi-cell units, teleports, per-target conditions,
  the CAST1_GAPS open rows) stay deferred.
- Art generation: CAST1 picks existing placeholders only; new art waits for the user's approval per item.
- Dialogue, vignettes, quests and the board itself (the encounter rows name the flags they set or read; the
  `recruit` effects for Dunstan and Elyse are the world's dialogue data).
- Ashlands rows, the RULESET1 `tl_` test kit (CAST1 adds beside it; replacing the test kit in the golden traces is a
  RULESET1-X decision, CQ12).

## 6. Tasks (rows for the planner; status `spec`; spec `games/emberwake/docs/specs/CAST1.md`)

| Id | Title | Layer | Lane | Depends | Touches |
| --- | --- | --- | --- | --- | --- |
| CAST1-S | Weapon type options `censer`, `bone_saw`, `sprayer` (items `weapon_type`, the gear token list, proficiency keys), as RULESET1-S did for `polearm` and `scourge`; data_source strip and HISTORY row | kit | logic | KIT1-N, RULESET1-S | `kits/rpg/schema.json`; `kits/rpg/crates/rpgkit_data/src/lib.rs` (gear tokens); `kits/rpg/crates/rpgkit_content/src/check/item.rs`; `games/emberwake/crates/emberwake_world/tests/data_source.rs` |
| CAST1-A1 | Families and base items: `scourge`, `censer`, `bone_saw`, `sprayer` family rows; the gear, consumable and material items, relics and keepsakes (gear 1-5, 7) | game (data) | asset | CAST1-S, RULESET1-A (`polearm`), HOOKS1-S (the pavise's trigger) | `data/catalog/{item_families,items}.json` |
| CAST1-A2 | Passives: proficiency drills, founder passives, keystones, traits, enemy traits (founders 2-7, gear 3, bestiary, bosses) | game (data) | asset | RULESET1-S, DERIVE1-D, HOOKS1-S, HOOKS1-A, CAST1-R | `data/catalog/passives.json` |
| CAST1-A3 | Skills: founders', enemies', bosses' rows with intents, timeline and hook fields | game (data) | asset | RULESET1-S, CAST1-A1, HOOKS1-S, HOOKS1-A, CAST1-R | `data/catalog/skills.json` |
| CAST1-A4 | Units: founder base units, enemy and boss units with decks, `poise`, `armor`, `tactics`, loot; palette files | game (data) | asset | CAST1-A2, CAST1-A3, CAST1-A7 (palettes), HOOKS1-S, HOOKS1-A, CAST1-R | `data/catalog/units.json` |
| CAST1-A5 | `sunk_ward` world rows: characters (founders with `keepsake`, enemies, bosses, captives), disciplines, named items, loot rows | game (data) | asset | CAST1-A4, M0 world skeleton, ROSTER1-L (`keepsake` key), HOOKS1-A (`classes`) | `data/worlds/sunk_ward/{characters,disciplines,items,loot}.json` |
| CAST1-A6 | Encounters, the slice rows (0a, 0b, 0c, the dilemma job, M3, M5) | game (data) | asset | CAST1-A5, ENCOUNTER2-T, -H, -K, -W, HOOKS1-S, HOOKS1-Z, HOOKS1-A, ROSTER1-L, CAST1-R | `data/worlds/sunk_ward/encounters.json` |
| CAST1-A6b | Encounters, the M2b rows (M1, M2, M4, the interlude) | game (data) | asset | CAST1-A6, HOOKS1-Q | `data/worlds/sunk_ward/encounters.json` |
| CAST1-A7 | Placeholder art: unit palette files on existing rig bodies, copied icon placeholders (assets file); no new art | game (data) | asset | - | `games/emberwake/assets/units/*/*/palettes/*.json`, `games/emberwake/assets/{skills,items}/...` |
| CAST1-B | Act 1 feel and balance runs: founders vs each encounter at the listed levels, the Q-49 armour scale, break rates, intent variety (OI1), hook metrics (T12-T14) | game | balance | CAST1-A6, RULESET1-B, HOOKS1-AI, HOOKS1-UI | `games/emberwake/docs/BALANCE_REPORT.md` |

Order: CAST1-S, then A1, A2, A3, A7 (A7 any time), A4, A5, A6, B; A6b after the slice playtest (Q-86) and HOOKS1-Q.
The data rows pass `content check` once HOOKS1-S and HOOKS1-A land; the hooks act in battle only as their logic
tasks land (HOOKS1 order: S, F, G, Z, E, K, M, H, U, AI), so CAST1-B waits for the last of them.

## 7. Content tests (the data must pass; asset lane runs them, the logic lane adds none)

| # | Check | Expectation from |
| --- | --- | --- |
| T1 | `tools/fc.sh content check` after each task | clean (no error); warnings only those listed in T9 |
| T2 | every CAST1 skill row has `round_intent` set and allowed for its kind | RULESET1 IN4 |
| T3 | every enemy-used skill with `blocks + recovery_blocks <= 6`; no party-only row carries recovery | RULESET1 T2, T4 |
| T4 | every `sure_break` row (Winch Sunder) has `stagger >= 1` and AP 1 above its twin | RULESET1 P14 |
| T5 | no CAST1 row sets `armor_piercing` except `sanctified_vivisection` (Quell) | RULESET1 A3; CQ7 |
| T6 | no CAST1 row names Strain, Calcification or a founder's taint (`strain_cost` absent, descriptions free of them) | RULESET1 C6; Q-58 |
| T7 | every unit's `deck` counts fit `deck_slots` (3, 2, 1) and every `order` entry is in `active` or the basics of that intent; every boss capstone in `extra_skills` and a phase `deck` with a free active slot | RULESET1 1.5; HOOKS1 DU2 |
| T8 | every encounter: at most 12 enemy-side units with phase waves; at most 4 boss phases plus one arrival; `rows` 9 x 12; `zone` a `zones` row | encounter checks; HOOKS1 ZN2 |
| T9 | warnings allowed: a Maneuver ATTACK with `dash_tiles` 0 (none expected), gear budget over band on relics flagged `special`; no HOOKS1 warning (Tripwire Snare carries no status, stagger or move; `hp_cost` 10 is under half the Flagellant's HP) | RULESET1 1.3; M2d; HOOKS1_TASKS 1.1 |
| T10 | `aegis sim feel enc=enc_pro_0a seeds=10` with Kaelen: at least one windup break by a Pommel Strike in 9 of 10 seeds | section 1 (the first lesson) |
| T11 | the slice encounters load in the Combat Arena under timeline and finish within `max_rounds` on autoplay in most seeds | Q-41 |
| T12 | `sim feel enc=enc_pro_0b seeds=10`: triggers fired (the intercept) and hazards placed (smoke) above 0 in most seeds | HOOKS1 4.2 metrics |
| T13 | `sim feel enc=enc_pro_0c seeds=10`: traps sprung above 0 in most seeds | HOOKS1 4.2 metrics |
| T14 | `sim feel enc=enc_evt_03 seeds=10`: arena strikes at blocks 3 and 6 of every round; Grendel's `deck` phase fires and the Pulverizer lands in most seeds; pushes and collisions above 0 | HOOKS1 7, 10.1, 4.2 |

## 8. Docs delta (for the planner)

- `games/emberwake/docs/GAME_BRIEF.md`: a short "Act 1 cast (CAST1)" note: where the founders' kits, families and
  encounter rows live, and that they use the HOOKS1 hooks.
- Lore: `systems/classes_and_skill_trees.md` 4 and the class files point to CAST1 for the Act 1 data and the
  deferred lore parts; `systems/items_gear_and_progression.md` 2 and gap 6 (families exist);
  `bestiary_encounters_act1.md` (rosters decided, Q-73 a); `bestiary_iron_inquisition.md` (Nemeth's Crucible kit).
- ROADMAP row wanted: CAST1, proposed: the Act 1 combat cast as data (founders, bestiary, bosses, encounters).

## 9. Open questions (recommended option first; CQ1-CQ16 are planner picks while the user is away)

CQ17 and CQ18 are new with CAST1-R. "Planner pick, user away" = OPEN_QUESTIONS 4 (CAST1 row): it stands until the
user says otherwise.

| Id | Question | Options | Recommendation |
| --- | --- | --- | --- |
| CQ1 | How a world re-themes a kit class (Q-53) when the founders need proficiencies the shared class lacks (polearm, censer, sprayer) | (a) proficiency passives granted by a founder discipline (data only), the display name and path from HOOKS1 12; (b) grow the shared kit classes (appends to Ashlands rows: a parity strip) | a: planner pick, user away (HQ6 a keeps the drills) |
| CQ2 | When the second passive and the keystone arrive | (a) all three owned at join; (b) T2 passive and keystone as mission rewards | a: planner pick, user away |
| CQ3 | Skill unlock levels | (a) T1 and T2 at level 1, T3 at 2, Kaelen's signature at 3; (b) one tier per level | a: planner pick, user away |
| CQ4 | Lore traits whose effect needed a missing hook (Penance, Reflex, Copyist, Spite, Volatile Catalyst) | (a) ship them; (b) leave them out until the hook lands | a with the pick's change: planner pick, user away; HOOKS1 covers all but Volatile Catalyst, which ships as INT +2 (CAST1_GAPS G4b) |
| CQ5 | Captain Vane after M1 (Q-24) | (a) he withdraws at 25% HP and his pavise is the objective reward; (b) he dies | a: planner pick, user away |
| CQ6 | The M2 captives (tone rule) | (a) Unsealed adults 18-24 on the board, the cull a phase line and `kill`; (b) captives off the board | a: planner pick, user away |
| CQ7 | Quell's Vivisection pierces armour in Act 1 | (a) yes, the one Act 1 piercing attack; (b) no pierce in Act 1 | a: planner pick, user away |
| CQ8 | The Bile Phial (Q-61 b) | (a) the counter through the per-unit meter (HOOKS1 9, Bile Coat); (b) hold Maren's trait | a: planner pick, user away (HQ3 a, HQ4 a) |
| CQ9 | The slice's dilemma job (Q-85) | (a) Corvo's gate: bribe (quiet) or force (`enc_job_corvo_gate`, loud); (b) another board job | a: planner pick, user away |
| CQ10 | Vesper's vendetta (Q-28) | (a) key loot on Orlov's kill: Flensing Talon and Orlov's Retribution; (b) "Vesper lands the kill" (a new hook) | a: planner pick, user away; b only if playtests want it |
| CQ11 | Keepsake names for Vesper, Maren, Dunstan and Elyse (Q-35) | (a) the names in CAST1_GEAR 7; (b) the user names them | a: planner pick, user away; any may change |
| CQ12 | The RULESET1 `tl_` test kit once CAST1 lands | (a) keep it as the golden-trace fixture and sim CAST1 beside it; (b) re-record the goldens on CAST1 data | a: planner pick, user away |
| CQ13 | Footmen reach | (a) a one-handed hookbill in the axe family (reach 1) with a heater shield; (b) the polearm family (reach 2) | a: planner pick, user away |
| CQ14 | Mid-act founders' level (Dunstan at M3, Elyse at M4) | (a) the cell's average level (ROSTER1 BK5 `average`); (b) level 1 with a catch-up XP share | a: planner pick, user away |
| CQ15 | The Calcified Amalgam before BIGUNIT1 | (a) M2b with a 1-cell stand-in, the 2 x 2 body later; (b) hold the interlude | a: planner pick, user away |
| CQ16 | Kaelen's signature plants a tower shield, but he wields a two-handed billhook | (a) the Bulwark needs no shield; (b) a one-handed weapon and a shield from the start | a: planner pick, user away |
| CQ17 | Which boss fights are `loud` (CAMPAIGN1_TASKS 4.2 says the main missions' boss fights) | (a) as written: the prologue untagged, the job and M1-M4 loud, M5 and the interlude not (the Crucible ends the act; the fissure is under the cell's own hub); (b) every main boss fight loud, M5 and the interlude included | a: a loud Crucible win could trip the Act 1 Breach at the climax |
| CQ18 | Kaelen's two `ally_hit` redirects (Penance 35, the Hook's intercept 100; HOOKS1 8.4 allows one a hit) | (a) a running skill's trigger goes before the unit's passives (the Hook intercepts; Penance covers the hits the Hook does not); (b) as written (passives first: Penance takes the first hit of each round) | a: the chosen Guard should do what it says; a one-line HOOKS1 8.4 amendment (HOOKS1-E) |

### As built: UNITDRILL1 (user chose the kit fix over CQ1 b)

- A unit's gear passes when its class or the drills of a world character on it (any world) train it. A character needs
  its own drills. A member (`can_wear`) reads its `passives`, set from its character by `hero_from_character`.
- `wears_untrained` on a unit row (default empty): item ids of its own gear excused from training. Refused when the item
  is not worn, or the class or a drill trains it. Only `inquisitorial_purifier` needs it: `["warded_apron"]`.
- A unit whose gear only a drill trains needs its character first, and a character needs its unit first. So the seed
  upserts the unit without that gear, then the character, then the full unit.
- The party founders hold their drill in the character's `passives`: a party hero may not carry `disciplines` yet
  (`party_character_errors`, RECRUIT1), against CAST1_FOUNDERS. `start_party` lists the characters (`kaelen`, ...): a
  member made from a unit row has no character and no drills, and the manifest check now refuses that.
- No code removes a passive. A member that did lose a drill keeps its gear. Gear equipped over the default then fails
  validation ("no training", no save, no build) until it comes off; base gear from the unit row validates and builds,
  untrained: lower power, and `min_proficiency` skills gated off. A data update between save and load is the one way
  there, and it can refuse a load (DRILLSAVE1, proposed).
