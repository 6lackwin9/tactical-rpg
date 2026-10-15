# M2c-2b skill content draft v2 (status: proposed, not approved)

Scope: skill families, class paths and loadouts for warrior, ranger, priest, brawler, rogue, mage. All numbers are STARTING VALUES for the later balance pass; no sims were run. Nothing here changes code or data; a builder would apply it through `ContentCatalog.upsert`. Supersedes draft v1 (skill levels 4-5, one cleanse/"protect" question set). HISTORY.md checked: nothing below revives a deprecated idea (no mastery, no per-world tiers, no passive-from-class, no ally-protection status).

## Rules used

- Skill model v2: a FAMILY has tiers 1-3 (basics strike and guard run to tier 5; offhand_jab has one tier). The tier is chosen at planning time. A tier's catalog `level` field is its AP cost. Unit skill cap = `ceil(unit level / 2)` (1-5), so AP 3 needs unit level 5+, AP 4 needs 7+, AP 5 needs 9+.
- Standard class-family scaling (used unless a row says otherwise): T2 = T1 AP +1, power x1.25; T3 = T1 AP +2, power x1.5 and +1 block (SUPERSEDED by Revision 3: no extra block at T3). Status potency +5 per tier, status rounds +0 or +1 where noted.
- ADVANCED families are separate families with their own tiers 1-3, base AP 3 (T1 AP 3 / T2 AP 4 / T3 AP 5), and extra effects that improve a starter skill in ONE slot, so a unit can drop the starter(s). No class-skill level 4-5 singles.
- Active loadout: max 5 class families + 3 extra slots; basics are free. A class path may own more than 5 families (unlocked by unit level); the default loadout is the FIRST 5 families of the path. Others are optional swaps set outside battle.
- New tier ids: `<id>_2`, `<id>_3` (existing ids stay T1, or keep their own ids where already tiered: power_strike, heavy_blow, brace, fortress, deep_focus). Existing skills still have `family: ""` for most ids, so migration sets `family` and `tier` on them.
- Reach 0/0 = weapon default. Ranged reach copied from existing volley (2-3) and firebomb (2-2).

## 1. Family list

Tier cells read T1 / T2 / T3. "Exist" = already in `skills.json` (re-tiered/reused); NEW = to author.

### Basics (free, everyone) (SUPERSEDED by Revision 3 section R1: basics keep only base versions)

| Family | Tiers (id) | AP | Blocks | Power | Notes | Exist/NEW |
| --- | --- | --- | --- | --- | --- | --- |
| strike | strike / power_strike / heavy_blow / strike_4 / strike_5 | 1 / 2 / 3 / 4 / 5 | 2 / 2 / 3 / 3 / 4 | 1.0 / 1.5 / 2.4 / 3.0 / 3.6 | no gating | T1-T3 exist; T4-T5 NEW |
| guard | guard / brace / fortress / guard_4 / guard_5 | 1 / 2 / 3 / 4 / 5 | as existing | mitigation above fortress at T4 (+15%) and T5 (+30%) | DEFEND | T1-T3 exist; T4-T5 NEW |
| offhand_jab | offhand_jab | 1 | 1 | 0.5 | single tier | exists |

### Class families

| Family | Tiers (id) | AP | Blocks | Power / heal | Area, reach | Status | Weapon gating | Exist/NEW |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| taunt_shout | taunt_shout / `_2` / `_3` | 1 / 2 / 3 | 1 | - | SELF | taunt 2 / 3 / 3 rounds | none | T1 exists |
| riposte | riposte / `_2` / `_3` | 2 / 3 / 4 | 2 | guard 0.15 / 0.22 / 0.30 | SELF | riposte 2 / 3 / 3 rounds | none | T1 exists |
| shield_bash | shield_bash / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.8 / 1.0 / 1.2 | NONE, 0 | stagger 70 / 75 / 80 | none (DEFENSIVE stance) | T1 exists |
| hamstring | hamstring / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.8 / 1.0 / 1.2 | NONE, 0 | root 80 / 85 / 90 | none | T1 exists |
| sunder | sunder / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 1.2 / 1.5 / 1.8 | NONE, 0 | stagger 50 / 55 / 60 | axe/mace p2 | T1 exists |
| aimed_shot | aimed_shot / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 1.4 / 1.75 / 2.1 | NONE, bow default | none | bow p2 | T1 exists |
| pinning_shot | pinning_shot / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.8 / 1.0 / 1.2 | NONE, bow default | root 80 / 85 / 90 | bow p1 | T1 exists |
| barbed_arrow | barbed_arrow / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.7 / 0.9 / 1.05 | NONE, bow default | bleed 60 / 65 / 70, 2 rounds | bow p2 | NEW |
| volley (ranger chip AoE) | volley / `_2` / `_3` | 2 / 3 / 4 | 2 | 0.7 / 0.8 / 0.9 | PLUS, 2-3 | none | bow p1 | T1 exists; ranger only |
| leap_back (ranger disengage) | leap_back / `_2` / `_3` | 2 / 3 / 4 | 1 | - | DISENGAGE, jump 2 / 3 / 3 | none | none | NEW |
| focus | focus / deep_focus / focus_3 | 2 / 3 / 4 | as existing | existing effect, +1 step per tier | SELF | - | none | T1-T2 exist; T3 NEW |
| smite (priest) | smite / `_2` / `_3` | 1 / 2 / 3 | 2 | 0.8 / 1.0 / 1.2 | NONE, 2-3 | none | staff/mace/orb p1 | NEW |
| mend | mend / `_2` / `_3` | 2 / 3 / 4 | 2 | heal 8 + 1.5 x KNOW / 10 + 1.8 / 12 + 2.1 | ally-or-self, 0-3 | none | none | T1 exists |
| purify | purify / `_2` / `_3` | 2 / 3 / 4 | 1 | removes 1 debuff type / 2 types / all | ally-or-self, 0-3 | cleanse | none | T1 exists |
| rally | rally / `_2` / `_3` | 1 / 2 / 3 | 1 | - | ally, 0-3 | haste 1 / 2 / 2 rounds | none | T1 exists |
| hex | hex / `_2` / `_3` | 2 / 3 / 4 | 2 | 0.6 / 0.75 / 0.9 | NONE, 2-3 | stagger 90 / 90 / 95 | staff/wand/orb p2 | T1 exists |
| lunge | lunge / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 1.2 / 1.5 / 1.8 | NONE, 1-1, dash 3 / 3 / 4 | none | none | T1 exists |
| sweep_kick | sweep_kick / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.7 / 0.9 / 1.05 | SWEEP | none | none (unarmed) | T1 exists |
| body_blow | body_blow / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 1.0 / 1.25 / 1.5 | NONE, 0 | stagger 50 / 55 / 60 | none (unarmed OK, see Q1) | T1 exists |
| haymaker | haymaker / `_2` / `_3` | 3 / 4 / 5 | 3 / 3 / 4 | 2.0 / 2.5 / 3.0 | NONE, 0 | stun 50 / 55 / 60 | none (unarmed OK) | T1 exists |
| pounce (rogue engage) | pounce / `_2` / `_3` | 2 / 3 / 4 | 2 | 0.8 / 1.0 / 1.2 | NONE, 1-1, dash 3 / 3 / 4 | none | dagger p1 | NEW |
| backflip (rogue disengage) | backflip / `_2` / `_3` | 2 / 3 / 4 | 1 | - | DISENGAGE, jump 3 / 3 / 4 | none | none | T1 exists |
| bleeding_cut | bleeding_cut / `_2` / `_3` | 2 / 3 / 4 | 2 | 0.8 / 1.0 / 1.2 | NONE, 0 | bleed 70 / 75 / 80, 2 rounds | none | T1 exists |
| quick_stab | quick_stab / `_2` / `_3` | 1 / 2 / 3 | 1 | 0.6 / 0.75 / 0.9 | NONE, 0 | none | dagger p1 | T1 exists |
| arcane_bolt | arcane_bolt / `_2` / `_3` | 1 / 2 / 3 | 2 | 1.1 / 1.4 / 1.65 | NONE, 2-3 | none | staff/wand/orb p1 | T1 exists |
| arc_lance | arc_lance / `_2` / `_3` | 2 / 3 / 4 | 2 / 2 / 3 | 0.9 / 1.1 / 1.35 | LINE, 2-4 | none | staff/wand/orb p1 | NEW |
| firebomb | firebomb / `_2` / `_3` | 3 / 4 / 5 | 3 | 0.6 / 0.75 / 0.9 | BURST, 2 | none | staff/orb p1 | T1 exists |

### Advanced families (base AP 3; unit level 5+ for T1)

| Family | Improves | Tiers (id) | AP | Blocks | Power / heal | Area, reach | Extra effect (what the second slot buys) | Weapon gating | Exist/NEW |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| shield_wall | taunt_shout (+ guard) | shield_wall / `_2` / `_3` | 3 / 4 / 5 | 3 | guard 0.35 / 0.45 / 0.55 | SELF | grants taunt 1 / 2 / 3 rounds with heavy self mitigation in one slot | shield p2 | NEW |
| deadeye | aimed_shot | deadeye / `_2` / `_3` | 3 / 4 / 5 | 3 | 2.0 / 2.5 / 3.0 | NONE, bow default | stagger 50 / 55 / 60 on the shot | bow p3 | NEW |
| greater_mend | mend + purify | greater_mend / `_2` / `_3` | 3 / 4 / 5 | 3 | heal 10 + 1.8 x KNOW / 14 + 2.0 / 18 + 2.2 | ally-or-self, 0-3 | cleanse 1 type / all / all + haste 1 round | none | NEW |
| renewal | mend + rally | renewal / `_2` / `_3` | 3 / 4 / 5 | 2 | heal 6 + 1.0 x KNOW / 8 + 1.2 / 10 + 1.4 | ally, 0-3 | grants haste 1 / 1 / 2 rounds with the heal | none | NEW |
| judgement | smite | judgement / `_2` / `_3` | 3 / 4 / 5 | 3 | 1.2 / 1.5 / 1.8 | NONE, 2-3 | stagger 40 / 45 / 50 on the hit | staff/mace p2 | NEW |
| charge_in | lunge | charge_in / `_2` / `_3` | 3 / 4 / 5 | 3 | 1.4 / 1.75 / 2.1 | NONE, 1-1, dash 4 | stagger 60 / 65 / 70 on arrival | none (unarmed) | NEW |
| cyclone_kick | sweep_kick | cyclone_kick / `_2` / `_3` | 3 / 4 / 5 | 3 | 0.9 / 1.1 / 1.3 | SWEEP | stagger 50 / 55 / 60 on every enemy hit | none (unarmed) | NEW |
| lacerate | bleeding_cut | lacerate / `_2` / `_3` | 3 / 4 / 5 | 2 | 1.2 / 1.5 / 1.8 | NONE, 0 | bleed 90 / 95 / 100 for 2 rounds (stronger wound than bleeding_cut, with real damage) | dagger p2 | NEW |
| assassinate | quick_stab | assassinate / `_2` / `_3` | 3 / 4 / 5 | 3 | 2.0 / 2.5 / 3.0 | NONE, 0 | single huge hit (no stance trick); the "extra" is power per slot | dagger p3 | NEW |
| inferno | firebomb | inferno / `_2` / `_3` | 4 / 5 / 5 | 3 | 0.8 / 1.0 / 1.2 | BURST, 2-3 | bleed 50 / 60 / 70 (burn), 2 rounds in the same cast | staff/orb p2 | NEW |
| glacial_burst | hex | glacial_burst / `_2` / `_3` | 3 / 4 / 5 | 3 | 0.6 / 0.75 / 0.9 | BURST, 2-3 | root 60 / 70 / 80 on all hit (AoE hex) | staff/orb p2 | NEW |

Note: inferno starts at AP 4 (stronger AoE payload than other advanced T1) and needs unit level 7.

## 2. Class loadouts

Path rows are `{unit level: family}`; default = first 5 rows. Advanced families unlock at level 5 (AP 3) and swap for the starters they improve. All classes also keep free basics and utility ids (move, switch_target, rest).

| Class | Default 5 (unit level) | Later / advanced (unit level) | Swap rule |
| --- | --- | --- | --- |
| Warrior (tank, taunt, self mitigation, riposte, low offense) | taunt_shout (1), riposte (3), shield_bash (3), hamstring (4), sunder (5) (SUPERSEDED: hamstring becomes intimidate, see R4) | shield_wall (5), shockwave line family (7, optional chip, not re-tiered here) | shield_wall replaces taunt_shout and frees the slot for sunder/a 3rd extra |
| Ranger (single-target ranged, chip AoE, disengage) | aimed_shot (1), pinning_shot (3), leap_back (3), barbed_arrow (5), volley (5) | deadeye (5), focus (5) | deadeye replaces aimed_shot; focus swaps in for volley when chip AoE is not needed |
| Priest (heal, buffs, cleanse, light smite; mace/staff/orb, no shield) | smite (1), rally (1), mend (3), purify (3), hex (5) | greater_mend (5), renewal (5), judgement (7) | greater_mend replaces mend + purify (2 slots freed); renewal replaces mend + rally; judgement replaces smite |
| Brawler (unarmed STR+DEX bruiser) | lunge (1), body_blow (3), sweep_kick (3), hamstring (4), haymaker (5) | charge_in (5), cyclone_kick (5) | charge_in replaces lunge; cyclone_kick replaces sweep_kick; hamstring is the first to drop |
| Rogue (engage + disengage, bleed, quick strikes) | quick_stab (1), pounce (3), bleeding_cut (3), backflip (4), hamstring (5) | lacerate (5), assassinate (7) | lacerate replaces bleeding_cut; assassinate replaces quick_stab |
| Mage (INT ranged AoE, weak single; volley removed) | arcane_bolt (1), arc_lance (3), hex (3), focus (4), firebomb (5) | glacial_burst (5), inferno (7) | glacial_burst replaces hex; inferno replaces firebomb; the PLUS pattern is unused for now |

Why each default is coherent: warrior = taunt + reactive + control; ranger = shot, root, escape, DoT, chip; priest = one attack, one buff, one heal, one cleanse, one debuff (a unit can fill the 3 extra slots with a second cast of the same role); brawler = jump in, sweep, stagger, root, stun; rogue = engage, disengage, bleed, quick strike, root; mage = bolt, line, debuff, charge, burst.

## 3. Coverage by role

| Role | Warrior | Ranger | Priest | Brawler | Rogue | Mage |
| --- | --- | --- | --- | --- | --- | --- |
| Heal | - | - | mend, greater_mend, renewal | - | - | - |
| Cleanse | - | - | purify, greater_mend | - | - | - |
| Buff (ally) | - | - | rally, renewal | - | - | - |
| Taunt / protect | taunt_shout, shield_wall, riposte, guard line | - | - | guard | guard | guard |
| Gap-close | - | - | - | lunge, charge_in | pounce | - |
| Disengage | - | leap_back | - | - | backflip | - |
| Single-target | sunder | aimed_shot, deadeye, barbed_arrow | smite, judgement | body_blow, haymaker | quick_stab, assassinate | arcane_bolt (weak) |
| AoE | - | volley (chip) | - | sweep_kick, cyclone_kick | - | arc_lance, firebomb, inferno, glacial_burst |
| DoT | - | barbed_arrow | - | - | bleeding_cut, lacerate | inferno |
| Debuff / control | shield_bash, hamstring, sunder | pinning_shot | hex | body_blow, haymaker, hamstring | hamstring | hex, glacial_burst |

Residual gaps (accepted): only the priest heals/cleanses/buffs; the warrior and mage cannot close or leave melee; no ally-protection status exists, so "protect" = taunt + self mitigation.

## 4. Migration notes

- `classes.json` paths and loadouts: rewrite warrior (currently strike, guard, taunt, riposte, shield_bash, brace, fortress), ranger (volley/focus/offhand_jab path), brawler (lunge, body_blow, hamstring, haymaker, brace), priest (rally, mend, focus), rogue (lunge, bleeding_cut, hamstring), mage (volley, focus, hex, firebomb). Brace/fortress move from class paths to the free guard family.
- Set `family`/`tier` on existing ids (most are `family: ""`); set the T2/T3 stats above. Existing fixed-skill hand-built units keep the empty (unrestricted) loadout.
- Brawler `body_blow`/`haymaker` are gated to fist/mace (haymaker also hammer) and the brawler is unarmed with no knuckles item: the gating must be cleared or an "unarmed" token added, otherwise the brawler cannot use its own skills.
- Unit skill cap `ceil(level / 2)`: level-3 goblin minions (brawler, skirmisher, archer, shaman, berserker) cap at AP 2, so they only use T1 AP 1-2 families; level 5 units reach advanced T1. Derived minions get the class default 5 families: goblin_shaman (priest) gets smite/rally/mend/purify/hex, goblin_brawler/berserker/enemy_brawler/ambush_berserker (brawler) get lunge/body_blow/sweep_kick (cap 2 omits hamstring/haymaker until level 5), goblin_skirmisher/fleeing_scout (rogue) get quick_stab/pounce/bleeding_cut, goblin_archer (ranger) gets aimed_shot/pinning_shot/leap_back/volley at AP <= 2.
- Heroes: hero ranger loses focus/deep_focus and offhand_jab-style defaults from the path (basics stay free); dojo warrior/ranger rely on the new defaults, so dojo/arena win rates will shift and need a balance rerun.
- Tests needing rule-derived updates (read counts and ids from the catalog/class rules, do not hard-code): `class_rules_test_suite.gd` (path unlock levels, 5-family default), `skill_family_test_suite.gd` (family/tier/AP and planner grouping), `content_test_suite.gd` (skill count, validation), `engine_pieces_test_suite.gd` (purify/backflip/sweep_kick become T1 of families), `combat_extensions_test_suite.gd` (mend, rally, lunge, taunt values), `riposte_test_suite.gd` (T1 values unchanged), `gear_budget_test_suite.gd` (brawler unarmed), `arena_test_suite.gd` and `runner_test_suite.gd` (skills now loadout-driven), `world_test_suite.gd` (minion kits).

## 5. AI follow-ups

- Disengage (leap_back, backflip): kite logic exists for ranged units but the AI never uses DISENGAGE; it should jump when an enemy is adjacent and the unit has ranged skills, then attack from range.
- SWEEP (sweep_kick, cyclone_kick): AI must pick a diagonal aim tile that maximises enemy hits and avoids allies (friendly fire).
- Cleanse-only skills (purify) and cleanse+heal (greater_mend): AI should cast when an ally has a debuff, even at full HP; current SUPPORT AI only heals allies below 70% HP.
- Buff support (rally, renewal): decide a trigger (ally about to attack or low HP) and a per-round cap.
- Taunt/shield_wall as DEFEND with a taunt grant: AI warrior should use it when allies are in danger.
- Tier choice: the AI must choose a tier within its AP and cap (e.g. highest affordable, or by situation); minions mostly get T1.
- Advanced families with extra statuses are normal ATTACK/SUPPORT and need no new AI beyond the above.
- Possible engine follow-up: per-tier riposte counter percent (counter is fixed 50%; tiers above only raise mitigation and duration).

## 6. Open questions

Questions 1, 2, 3, 5, 6 and 7 were answered on 2026-10-15 (see DECISIONS.md, M2c-2b design answers) and are covered by Revision 3 below.

1. Brawler gating: clear `weapon_types` on body_blow/haymaker (unarmed allowed), or add an "unarmed" weapon type token?
2. Rogue and brawler gap-closers: confirm brawler keeps `lunge` (and charge_in) while the rogue gets its own dagger `pounce`, instead of sharing one skill.
3. Hamstring appears in warrior, brawler and rogue defaults. Acceptable overlap, or should the brawler/warrior get a distinct 5th family?
4. Mage default no longer has any PLUS pattern; fine for now, or add a mage-only PLUS family later (parked)?
5. Priest weapons: the catalog has no tome item (mace, staff, orb, wand exist). Add a tome item type in M2b scope or leave it out?
6. Tier scaling: use the uniform +25%/+50% rule above, or let a balance pass tune each family?
7. Per-tier riposte counter percent needs a small engine change; do it in M2c-2b or leave the counter at 50%?

## Revision 3 proposals (status: proposed, not approved)

Honours the 2026-10-15 "M2c-2b design answers" row. HISTORY.md checked: no deprecated idea is revived (removing tier variants of basics is new; the tier model itself is unchanged for class families). No sims were run; every number is a starting value for the balance pass.

### R1. Basic skill set

Migration cost (grep of `scripts/` and `scenes/`, 37 hits in 8 files; `world_schema.gd` is a false positive, it is the "fortress" map kind):

- `arena_test_suite.gd`: 13 hits (AP-cost checks around lines 743-748, switch cost 763-766, planner chain 776-811, locked plan 847, 974).
- `skill_family_test_suite.gd`: 9 hits (family lists lines 51-53, 65, 237, 295-336).
- `dojo_assertions.gd`: 7 hits (catalog check, power_strike damage probe, brace, lines 696-745).
- `core_rules_test_suite.gd`: 3 hits (brace mitigation, lines 118-144).
- `content_test_suite.gd`: 2 hits (line 111 "too advanced" uses heavy_blow; line 184 uses brace).
- `class_rules_test_suite.gd`: 1 hit (line 172).
- `content_schema.gd`: 1 hit (help text only).
- Data: `skills.json` rows power_strike, heavy_blow, brace, fortress, deep_focus, plus `skill_path` entries in `classes.json` (warrior, brawler).

| Option | Basic set | Effect | Cost and risk |
| --- | --- | --- | --- |
| A (minimal) | strike, guard, move, rest, switch_target, offhand_jab | Six ids. Strike is ONE skill; power comes from weapon and level, AP stays 1 and 2 blocks. Guard is one skill. | Lowest. Risk: no AP sink for basics, so AP above 3 only matters for class skills. Tests that pivot on power_strike/brace move to class-skill equivalents. |
| B (A plus shove) | A plus a new 1 AP `shove` (push 1 tile, no damage, melee) | Every unit gets a positional answer and a cheap peel. | Needs a new push engine piece (not in the brief); defer. |
| C (keep an AP sink) | A, but the planner lets the player add 1-2 AP to a strike block for +25% / +50% power | Preserves AP-spending choice without separate ids. | Re-creates the removed variants in disguise and touches planner UI; against the spirit of the decision. |

Per-id disposition:

| Existing id | Becomes | Note |
| --- | --- | --- |
| strike, guard, move, rest, switch_target, offhand_jab | basic, kept | single tier each; `family` is its own id |
| power_strike, heavy_blow, brace, fortress | REMOVED | delete rows; fix `skill_path` references and the tests above |
| deep_focus | REMOVED | focus is a class family (ranger, mage), tiers focus / focus_2 / focus_3 |

Ranged is covered because strike uses the weapon reach (bow, wand); a basic buff is not needed, focus stays class-owned.

Recommendation: A. Strike has no selectable tiers and scales through weapon power and unit level as today.

### R2. Non-attack tier scaling baseline

AP is always T1 + 0 / +1 / +2. Blocks never grow at T3 (decision); blocks are per-skill and fixed across tiers.

| Type | Magnitude | Duration or potency | Reach or area | Sanity range |
| --- | --- | --- | --- | --- |
| heal (mend) | whole heal x1.25 (T2), x1.5 (T3) | n/a | +0 | T3 heal at most 60% of target max HP at level 9 |
| ally buff (haste) | effect level fixed | rounds 1 / 2 / 2 | +1 reach at T3 | at most 2 rounds |
| self buff (focus) | existing effect +1 step per tier | duration +0 | SELF | at most 3 steps; total buff <= +50% |
| defend (guard, riposte, shield wall) | mitigation +0.05 per tier | rounds +0 / +1 / +1 | SELF | total mitigation at most 0.60 (check against current fortress) |
| taunt | none | rounds 2 / 3 / 3 | radius +0 | at most 3 rounds |
| cleanse | NOT scaled by magnitude | T1 one type, T2 two types, T3 all | +0 | AP rises per the standard rule |
| debuff (weaken, hamstring, stagger) | potency +5 per tier | rounds +0 / +0 / +1 | +0 | potency 50-95, rounds at most 3 |
| gap-close (lunge, pounce) | dash 3 / 3 / 4 | n/a | n/a | dash at most 4 |
| disengage (leap_back, backflip) | jump 2 / 3 / 3 | n/a | n/a | jump at most 3 |
| DoT (bleed) | potency +5 per tier | 2 rounds all tiers | +0 | potency 60-100 |
| move, rest, switch_target | NOT scaled | n/a | n/a | basics, single tier |

Should not scale by tier: cleanse magnitude, move, rest, switch_target, offhand_jab. Flag: sim guard/riposte at T3 before locking the 0.60 ceiling.

### R3. Weapon types pass

| weapon_type | Slot | Item today | Class proficiency (level) |
| --- | --- | --- | --- |
| sword | main | wpn_claymore, wpn_broadsword, wpn_shortsword | warrior 3, rogue 2, ranger 1, brawler 1 |
| axe | main | wpn_hand_axe, wpn_battle_axe | warrior 2, brawler 1 |
| mace | main | wpn_flanged_mace, wpn_cleric_mace | brawler 3, priest 3, warrior 2 |
| hammer | main | wpn_warhammer | brawler 2, warrior 1 |
| fist | main | wpn_knuckles | brawler 3, rogue 1 |
| dagger | main (off-hand via `off_hand_category`) | wpn_dagger, wpn_stiletto | rogue 3, ranger 2, priest 1, mage 1 |
| bow | main | wpn_shortbow | ranger 3, rogue 2 |
| crossbow | main | NONE | ranger 2 |
| staff | main | wpn_staff, wpn_mage_staff | mage 3, priest 2 |
| wand | main | wpn_wand, wpn_glass_wand | mage 3 |
| orb | off-hand (schema `off_hand_category`) | wpn_tome (mis-typed: weapon_type orb, slot MAIN_HAND) | mage 3 |
| tome | off-hand | NONE | mage 2 |
| shield | off-hand | shd_kite, shd_buckler, shd_tower | warrior 3, priest 1, brawler 1 |
| plate, leather, cloth | body | arm_plate, arm_leather, three robes | by class |

Gaps: crossbow and tome have no item; wpn_tome is a main-hand typed orb; the mage and priest have no off-hand item at all. Every class has a valid main hand. Two-handed flags live on the item (`two_handed`), not the type.

| Option | Orb (off-hand) | Tome (off-hand) | Cost and risk |
| --- | --- | --- | --- |
| A (power vs utility) | spell damage: +damage on spells, +1 spell reach | support: +heal power, +5 debuff potency, no reach | Clear identity; mage leans orb, priest leans tome. Two new stat blocks and gear budget entries. |
| B (stat flavour only) | +magic stat | +INT or KNOW stat | Cheap, but both feel the same. |

Recommendation: A. Priest = mace and staff main, tome and orb off-hand, NO shield (drop shield 1; tome 3, orb 2). Mage = staff and wand main, orb 3 and tome 2 off-hand. Re-type wpn_tome to `slot OFF_HAND`, `weapon_type tome`; author one new `wpn_orb`. Caster skill gating that names "orb" in v2 (arcane_bolt, arc_lance, hex, firebomb, inferno, glacial_burst, smite) must read staff/wand (main hand) only, since an off-hand orb cannot be the casting weapon; crossbow stays a gap.

### R4. Weaken status, intimidate and riposte

Weaken (engine status):

| Field | Proposal |
| --- | --- |
| Effect | afflicted unit's outgoing power multiplier x(1 - magnitude): 15% / 20% / 25% by tier |
| Duration | 2 rounds (T3: 3), ticks at round end like other statuses |
| Application | chance = potency 70 / 75 / 80, reduced by `debuff_resistance` and `status_resist["weaken"]` (100 = immune), same path as stagger and root |
| Resist key | `weaken` |
| Hook | NOT in `DamageCalculator.compute`; the code that builds the `power_mult` input multiplies it by (1 - magnitude), so formula tests stay untouched |
| Stacking | none; reapplying refreshes duration. Magnitude capped at 25% |

Intimidate family (warrior 4th family, replaces hamstring in the default path):

| Tier | AP | Blocks | Reach | Status |
| --- | --- | --- | --- | --- |
| intimidate | 1 | 1 | 0-2, single target, no damage | weaken 15%, potency 70, 2 rounds |
| intimidate_2 | 2 | 1 | 0-2 | weaken 20%, potency 75, 2 rounds |
| intimidate_3 | 3 | 1 | 0-3 | weaken 25%, potency 80, 3 rounds |

Riposte tweak (counter stays 50%):

| Option | AP T1 / T2 / T3 | Mitigation | Effect |
| --- | --- | --- | --- |
| A (AP cut) | 1 / 2 / 3 (was 2 / 3 / 4) | 0.15 / 0.20 / 0.25 | Cheap reactive stance, combinable with another skill. Risk: overlap with guard; compare in a sim. |
| B (mitigation raise) | 2 / 3 / 4 | 0.25 / 0.30 / 0.35 | Deliberate 2 AP commitment that pays defensively. |
| C (both) | 1 / 2 / 3 | 0.20 / 0.25 / 0.30 | Likely too strong; sim first. |

Recommendation: A, rounds stay 2 / 3 / 3. `riposte_test_suite.gd` T1 AP expectation changes from 2 to 1.

### R5. Open questions and implementation order

Open questions:

1. Strike without tiers: accept no basic AP sink (R1 option A), or keep a charged strike (option C)?
2. Focus: stay class-owned (ranger, mage) or become the single basic buff?
3. Confirm orb and tome are off-hand only and caster skills gate on main-hand staff/wand.
4. Priest loses shield proficiency: confirm (any priest hero or minion carrying a shield changes).
5. Riposte: option A or B? Guard T1 mitigation must be read at build time to confirm no overlap.
6. Weaken: cap 25% and no stacking acceptable?

Proposed slice order (feature branches, engine before content):

1. Basic-skill cleanup: remove power_strike, heavy_blow, brace, fortress, deep_focus, fix `skill_path`, update the 37 references; nothing new is built on removed ids.
2. Unarmed token: skill `weapon_types` accepts `unarmed`; empty main hand counts as unarmed; fix the brawler gear budget test.
3. Orb/tome typing: re-type wpn_tome, add wpn_orb, proficiency edits, off-hand validation per class.
4. `weaken` status with the `power_mult` hook and test, then the intimidate family.
5. Riposte AP/mitigation change and test.
6. Optional ContentCatalog warning when a family's tiers break the R2 formulas.
7. M2c-2b content (families, paths, minions) and a balance rerun against the 40-60% duel bracket.
