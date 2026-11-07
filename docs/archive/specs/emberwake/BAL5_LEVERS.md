# BAL5_LEVERS: design levers for the BAL3-R gaps

Status update (2026-10-20): approved (LEVERS1 rows, user 2026-10-18); BAL5 rows done, LEVERS1 rows approved and queued. The status line below is the original.

Status: proposed (spec 2026-10-18; input: BAL3-R final rerun and CAMP1-B, report `BALANCE_REPORT_BAL3-R.md`, sections cited as "R §n")
Depends on: user answers Q1-Q6 below; BAL4-ATTR (shares `tools/sim/*`); BOSS1-L for lever B2
Lane: balance, logic, asset (split in section 9)
Touches: `tools/sim/*`, `bal2_rules_test_suite`; `scripts/combat/ai/tactical_ai.gd`, `ai_skills_tactics_test_suite`; skill schema (`scripts/content/skill_fields.gd`, `SkillValidator`, `SupportRules` grant path) only if lever T3 fires; `data/catalog/{skills,classes}.json` and `scripts/core/specs/stat_rules.gd` `TIERS` only if their triggers fire (seed `data/seeds/bal5.gd`)

## Goal

BAL3-R left six gaps outside the decided bands (`games/emberwake/docs/decisions/balance.md`). For each one this spec names the cause found in the code or the report, says what approved work already covers, offers 2-3 levers with a trigger and a sim check, and recommends one. Levers marked **spec** are numbers or rules judged by sims; levers marked **option-build** are new feel mechanics judged by playing, for the first design review (`docs/workflow/design-review.md`). DR1 (3) "do not tune battles much" applies: sim-method levers come first, AI rules second, data last, and every data lever has a trigger.

## 0. Cross-cutting causes (affect every gap)

| Id | Cause | Evidence |
| --- | --- | --- |
| X1 | **Party size.** Every party, sweep and CAMP1-B number uses 4 heroes; the game now plays a trio (warrior + two recruits, RECRUIT1 section 3, decision world-campaign "DR1" (4)). BAL3 Q2 ("party of 4") predates RECRUIT1; RECRUIT1 section 4 already asks the planner to raise this. | `tools/sim/sim_args.gd:40` (`DEFAULT_COMP`, 4 classes); BAL3 3.3 threat table: party 4.00 vs boss archetype 3.25, a trio is 3.00 |
| X2 | **Cell granularity.** Party cells are 5 fights, so a cell moves in 20-point steps and the boss band 30-50 is hit only by exactly 2 of 5. The non-monotonic elevel boss row (80 / 60 / 80, R §1) is inside that noise. | R §1 header ("5 seeds") |
| X3 | **Sim archetype vs authored encounters.** The generated boss is a plain brawler boss with no phases (`sim_args.gd:45`, `sim_scenarios.gd:26-27`); authored bosses will get phases (BOSS1). The sim cannot reach world encounters (`enc=` checks `ContentCatalog` only, `sim_args.gd:159-170`; R "Sim tooling"). | |

Recommended for all gaps: re-measure at a trio with 10 seeds per cell before any data lever (task BAL5-R1). That needs Q1.

## 1. Boss clears 80-100 (band 30-50)

Already covered: BOSS1 adds the Crown of Nails phase (fortify 5, regen 4 for 3 rounds, reckless, smokecallers heal at 90%) to the two Ashmaw stands only (BOSS1 4.1-4.2); BOSS1-B probes them with trios. Neither touches the generated `boss` archetype that the party band is judged on. BAL3 3.3 raised boss bodies (threat 2.13 -> 3.25) and left tier multipliers alone by design.

Cause: X1 (4 heroes focus one 1.5 HP body; the boss is 1.65 of a party's 4.00 threat), X2, X3. Boss tier `{"hp": 1.5, "power": 1.1}` (`scripts/core/specs/stat_rules.gd:70`) is unchanged since BAL2.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| B1 trio re-measure | spec (sim) | party and sweep at 3 heroes, 10 seeds (BAL5-R1) | no game change; may close the gap alone (threat ratio 1.08 instead of 0.81) |
| B2 sim boss phase | spec (sim, needs BOSS1-L) | the generated `boss` archetype carries the Crown of Nails effects as a generic phase (table B2) | the sim boss fights like authored bosses; "a boss has a phase" becomes the authoring rule; no tier change |
| B3 boss tier HP | spec (data) | boss HP 1.5 -> 1.8 (power 1.1 kept); second step 2.0 | every boss in every world gets longer; changes decision "BAL2 answers" (7), needs the user |
| B4 boss double action | option-build | a boss tier gets a second, short plan (e.g. +4 AP regen and acts in blocks 1-6 twice) | a different boss feel; new rule in planning; judged by playing |

Table B2 (sim phase on the generated boss; fields as BOSS1 1.2-1.4):

| id | trigger | effects |
| --- | --- | --- |
| sim_boss_phase | hp_below (the boss unit id) 50 | status fortify rounds 3 magnitude 5; status regen rounds 3 magnitude 4; ai (the boss) reckless true, target_pref WEAKEST |

Rules: B2 is on by default for `enc=boss`; `phase=off` restores the BAL3 archetype for A/B. B3 fires only if, after B1 and B2, the trio sweep boss mean is above 60 at L5; apply 1.8, rerun, then 2.0 only if still above 60.

Recommendation: B1, then B2; B3 only on its trigger. B4 goes to the design review.

Sim check: `tools/fc.sh sim party <3|5|7> 10 comp=warrior,<a>,<b> enc=boss` for the 10 warrior trios plus the trio sweep (BAL5-SIM K2): sweep boss mean in 30-50 at L3, L5 and L7; no trio below 20; 0 PLAN FAIL, 0 METRIC FAIL.

## 2. Tank leaves allies at about 6% HP (band 40)

Already covered: BAL3-AI lever D (taunt pull) and BAL3-A (riposte out of the sim tank loadout) moved nothing (R "What each BAL3 step changed"). The fortify grant was blocked by the one-grant schema (R "Issues": shield_wall grants taunt, `data/catalog/skills.json` shield_wall `grant_status` taunt).

Cause (measured plus code):

- The metric mixes losses in: tank clear is 33 at L5 (R §3), so two of three fights end with allies at 0. `allies hp_end` is the mean over all fights (`tools/sim/sim_roles.gd:148`).
- Taunt reaches the foes late. The warrior casts taunt only when an ally is within 3 tiles of a foe (`scripts/combat/ai/tactical_ai.gd:733-745`), and it starts next round (taunt_shout "starting next round"). Rogue divers (pounce) reach the ranger or mage in round 1 (`tools/sim/sim_scenarios.gd:35-36`: 2 brawlers, 2 rogues).
- Once taunt is up every foe that has a route is forced onto the warrior (`tactical_ai.gd:496-497`); survival 33 says he then falls and the allies follow.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| T1 won-fight split | spec (sim) | print `allies hp_end` on won fights next to the all-fight mean; band >= 40 is read on won fights, the all-fight floor is the R proposal >= 15 | measures the tank, not the loss rate; no game change |
| T2 early taunt | spec (AI) | the warrior also casts taunt when any foe would pick an ally this round (`_likely_target`, used by `incoming_damage`, `tactical_ai.gd:1049-1062`); the 3-tile rule stays as an OR | taunt goes up in round 1 against divers; affects every taunting unit, enemies included (symmetric, as decision "BAL3-AI lever C scope") |
| T3 second grant | spec (schema + data) | skill field `extra_grants` (below); shield_wall ranks grant taunt plus fortify 4 / 6 / 8 | the warrior lasts longer under a forced focus; a schema change in skills |
| T4 bodyguard | option-build | a warrior stance or skill: hits on an adjacent ally go to the warrior | a new tank feel; judged by playing |

T3 schema (only if its trigger fires):

| Field | Type | Default | Validation (message) |
| --- | --- | --- | --- |
| extra_grants | json array of {status, rounds, magnitude} | [] | DEFEND or SUPPORT only ("extra_grants needs a DEFEND or SUPPORT skill."); at most 1 entry ("A skill holds at most one extra grant."); status is a registry buff and not the skill's `grant_status` ("Extra grant '%s' must be another buff."); rounds 1..3; magnitude inside the status row's range (fortify 1..20) |

Grant timing is the same as the skill's own grant (immediate for DEFEND riposte-style, else next round). Ranks may override the whole list.

Triggers: T2 when the trio re-measure (BAL5-R1) shows won-fight allies hp_end below 40 or tank survival below 35 at L5; T3 only if still failing after T2.

Recommendation: T1 + T2; T3 on trigger; T4 to the design review.

Sim check: `tools/fc.sh sim role <3|5|7> 3 role=tank`: won-fight allies hp_end >= 40, all-fight >= 15, survival >= 35, warrior rank 1 on dmg_taken share, warrior clear 30-85; new AI rule covered in `ai_skills_tactics_test_suite` (a fixed board where a rogue can pounce the mage in round 1: the warrior drafts taunt_shout).

## 3. Priest healing about 1% of party HP (band 20)

Already covered: BAL3-AI heal reservation and BAL3-A mend 10/12/14 plus renewal regen were tried with no effect (R "What each BAL3 step changed"); do not re-propose heal power. HEAL1 (approved, spec pending BAL4-ATTR) changes healing between battles, not inside them. In party mode the priest does heal (cell mean >= 23 HP, R §1), so the gap is the `support_attrition` scenario.

Cause (code plus one inference to confirm):

- `heal/party hp%` = heal_done / party max HP (`tools/sim/sim_roles.gd:173-174`); 1% is about 6 HP per fight at L5, so the priest almost never casts.
- The AI heals only an ally at or below `heal_below` 0.7 (`tactical_ai.gd:569`, `ai_profile.gd:32`).
- Inference: the priest is downed before any ally reaches 70%. Foes are two rangers, a mage and a rogue (`sim_scenarios.gd:41-42`), all ranged or diving, and non-aggressors target the weakest (`tactical_ai.gd:479-489`); the priest has the lowest HP of its trio (CONS 12 vs warrior 17, brawler 15, `data/catalog/classes.json`). "surv vs brawler -1.00" (R §3) fits: a brawler in the slot survives where the priest does not. BAL5-SIM K3 prints the priest's down round to confirm.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| S1 healer self-care | spec (AI) | a SUPPORT unit whose `incoming_damage` this round (by block 6) is >= 50% of its current HP heals itself first with the lowest heal tier that fits (counts toward `MAX_HEALS`, `tactical_ai.gd:40`); with no self-heal it retreats one step (`_retreat_dir`) before its cast | recruited priests stop dying first in the campaign too; enemy shamans also last longer |
| S2 attrition scenario | spec (sim) | `support_attrition` enemies: ranger, brawler, mage, rogue standard (one ranger becomes a brawler) | tests sustained heal instead of priest burst survival; sim only |
| S3 ward | option-build | a priest skill that grants an absorb shield (new status `ward`) | pre-emptive support feel; a new status; judged by playing |

Triggers: S1 if BAL5-SIM K3 shows the priest down before round 3 in >= 50% of L5 support fights; S2 only if heal share is still under 5 (R interim) after S1.

Recommendation: S1 (fixes real recruit behaviour), then S2 on trigger; drop the "surv vs brawler" band as R proposes (Q5). S3 to the design review.

Sim check: `tools/fc.sh sim role <3|5|7> 3 role=support`: heal share >= 20 (interim >= 5), priest alive at round 3 >= 60%, priest clear 30-85; AI test: a priest projected to lose 60% of its HP drafts Mend on itself.

## 4. AoE swarm clears 0-22 (band 70)

Already covered: BAL3-S (2x3 block, third brawler minion), BAL3-A sweep_kick 0.8 / 1.0 / 1.15 (brawler foes/cast 0.1 -> 1.6) and firebomb reach 3 (no effect, gated off) (R §3, "What each BAL3 step changed").

Cause:

- Two heroes against six minions (`sim_scenarios.gd:39-40`): six attackers focus the weakest; `mob` (5 minions) is cleared 100 by four heroes (R §1), so the game handles swarms in a party.
- The AI refuses any area cast that would hit an ally (`tactical_ai.gd:993-997`), and the only ally is a melee warrior inside the block.
- The mage has no area skill at L3: firebomb and glacial_burst sit at path level 5 (`data/catalog/classes.json` mage `skill_path`), so the L3 mage AoE cell (0) tests nothing.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| A1 trio swarm | spec (sim) | `aoe_swarm` allies ["warrior"] -> ["warrior", "priest"] | matches the trio; the priest adds sustain, not kills, so the KPI stays the AoE unit's |
| A2 friendly-fire tolerance | spec (AI) | an area cast may hit one ally when it hits >= 3 foes and the estimated damage on that ally is < 25% of its current HP (`area_min_foes` 2 still applies to clean casts) | more AoE casts in the game, some friendly fire; enemies too |
| A3 mage AoE at L3 | spec (data, needs Q4) | mage `skill_path` firebomb 5 -> 3, arc_lance 3 -> 5 | a recruited mage has an area skill from L3; changes the mage's early kit |
| A4 self-centred cone | option-build | a mage cone from its own tile (no reach) | a close-range caster feel; judged by playing |

Triggers: A1 with Q1 (trio). A2 if brawler foes/cast < 1.5 or mage < 1.0 (R proposed bands) after A1. A3 only on Q4 yes. The L3 mage AoE cell is reported as "n/a (no area skill)" unless A3 lands.

Recommendation: A1 + A2; A3 is a user question. A4 to the design review.

Sim check: `tools/fc.sh sim role <3|5|7> 3 role=aoe`: swarm clear >= 70 (keep the band, R proposal), brawler rank 1 on foes/cast and >= 1.5, mage >= 1.0 at L5 and L7, allies/cast <= 0.3 (`sim_targets.gd:32`); AI test: a burst on 3 foes and one ally at 90% HP is cast, the same burst with 2 foes is not.

## 5. Warrior wins 93% of duels; solo elite 0% for five classes

Already covered: BAL3-A elite power 1.1 -> 1.0 (no effect) and kite shield mitigation (no effect) (decision balance "BAL3-A elite power applied", R §5). RECRUIT1-B probes the solo warrior against the early fights (mob band >= 90).

Cause:

- Tier mode is a mirror: a hero against its own class at the tier (`tools/balance_sim.gd:136-143`). An elite is the same class with 1.25 HP and equal power (`stat_rules.gd:69`), so the hero loses nearly every time with jitter as the only variance; 0% is arithmetic, not a class problem.
- The warrior has the most HP: 60 + 17 x 8 + 13 x 2 = 222 at base stats against mage 136, rogue 142, ranger 148 (Max HP rule, `games/emberwake/docs/GAME_BRIEF.md`; `classes.json`). DR1 (4) now needs that: the warrior starts alone and must solo low-level fights.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| D1 accept with reasons | spec (decision) | warrior and priest exempt from the duel band (solo-start hero; support); duels stay a diagnostic | no game change; RECRUIT1-B becomes the solo check |
| D2 tier vs a fixed foe | spec (sim) | tier mode key `tier_foe=<class>` (default `mirror`); the solo check runs every class against a brawler elite and boss | a solo number that compares classes; "Solo difficulty" (boss 10-20%) becomes measurable |
| D3 warrior damage | spec (data) | warrior growth STR 2 -> 1, PERC 1 -> 2 | duel nearer 70, but the solo phase gets harder; RECRUIT1-B must rerun |

Recommendation: D1 + D2; replace the solo elite ~50 band with "solo vs the RECRUIT1 solo fights >= 90" and keep solo boss 10-20% measured with D2 (Q3). D3 not recommended (conflicts with DR1 (4)).

Sim check: `tools/fc.sh sim tier 5 3 tier_foe=brawler`: solo boss 10-20 per class at equal level (warrior may exceed, flagged); RECRUIT1-B solo cells >= 90.

## 6. Ashmaw finale far too easy (100% clear, CAMP1-B)

Already covered: BOSS1-A (phase on both stands), BOSS1-B (trio probe before and after), HEAL1 (no full heal between battles, so the party arrives hurt), ASHPACE1 (light pace pass with BAL4-ATTR numbers), RECRUIT1 (trio). BOSS1 5 already fixes the fallback order: magnitudes first, never stats.

Cause: CAMP1-B used 4 heroes at full HP (X1); the authored foes are a level 6 boss with level 3 minions (`data/catalog/units.json` goblin_*: level 3 minion, goblin_warlord level 6 boss), threat 2.13 barred and 2.85 with flankers (BAL3 3.3 values) against a party of 4.00; no phase yet.

| Lever | Kind | Change | Consequence |
| --- | --- | --- | --- |
| F1 measure the real finale | spec (sim) | BOSS1-B runs trios at the BAL4-ATTR arrival HP, via the world-encounter sim path (BAL5-SIM K1) | the first honest number; no data change |
| F2 guard at party level | spec (data, inside ASHPACE1) | ash_shaman and ash_archer at the finale's party level (world character `level`) | a harder guard without touching Grakka or tiers |
| F3 more smokecallers | spec (data) | BOSS1 Q2 option B (two more smokecallers in the unbarred stand) | the user chose A; re-proposed only because CAMP1-B shows 100% with 0.37 party HP lost, and only if F1 + F2 still clear above 50 |

Recommendation: F1 first; F2 as an ASHPACE1 row on trigger (trio clear > 50 after BOSS1-A); F3 only after both, and the user decides again.

Sim check: `tools/fc.sh sim party <campaign level> 10 world=ashlands enc=ashmaw_stand comp=warrior,<a>,<b>` for the 10 trios, before and after BOSS1-A: stand clear 30-50, barred above stand (barred is the reward for the steel choice), 0 PLAN FAIL.

## 7. Party line and elite at L5 default (100, bands 70-90 / 50-75)

No new lever. The sweep is inside both bands (L5 88 / 72, R §2); only the default comp is above. The trio re-measure (B1) decides it.

## 8. Sim tooling needed (BAL5-SIM)

| Id | Change | For |
| --- | --- | --- |
| K1 | `world=<id>`: `enc=` also finds that world's encounters (make the CAMP1-B scratch wrapper permanent; story characters spawn through `WorldFactory`, tuning applied) | F1, BOSS1-B |
| K2 | trio comps: `sweep size=3 lead=warrior` (the 10 warrior trios); `seeds` 10 default for party cells | B1, all bands |
| K3 | won-fight split for `allies hp_end`; priest down round and heal casts attempted per fight | T1; S1 trigger |
| K4 | `tier_foe=<class>` | D2 |
| K5 | scenario edits A1, S2 (S2 only on its trigger); B2 boss phase after BOSS1-L | A1, S2, B2 |
| K6 | PLAN FAIL from jitter (unreachable target) counts as a skipped fight with its own line, not a fail (R "Issues") | gate noise |

Acceptance: `bal2_rules_test_suite` covers each key; `jitter=off phase=off` with 4 heroes reproduces BAL3-R cells within seed noise.

## 9. Task split (rows for the planner)

| Id | Title | Lane | Depends on | Status | Spec | Touches |
| --- | --- | --- | --- | --- | --- | --- |
| BAL5-SIM | Sim tooling: world encounters, trio sweep, won-fight and healer metrics, tier_foe, scenario edits, jitter skip | logic | BAL4-ATTR (same files), Q1 | spec | games/emberwake/docs/specs/BAL5_LEVERS.md section 8 | tools/sim/*, tools/balance_sim.gd, bal2_rules_test_suite |
| BAL5-R1 | Trio re-measure at 10 seeds (party, trio sweep, role, tier_foe) on current data; marks which BAL5 triggers fire | balance | BAL5-SIM | spec | games/emberwake/docs/specs/BAL5_LEVERS.md sections 1-7 | games/emberwake/docs/BALANCE_REPORT.md |
| BAL5-AI | AI levers on trigger: early taunt (T2), healer self-care (S1), friendly-fire tolerance (A2) | logic | BAL5-R1 | spec | games/emberwake/docs/specs/BAL5_LEVERS.md sections 2-4 | scripts/combat/ai/tactical_ai.gd, ai_skills_tactics_test_suite |
| BAL5-PH | Generated boss archetype carries the sim boss phase (B2) | logic | BOSS1-L, BAL5-SIM | spec | games/emberwake/docs/specs/BAL5_LEVERS.md section 1 | tools/sim/sim_scenarios.gd, bal2_rules_test_suite |
| BAL5-L | `extra_grants` skill field (T3), only if T3 fires | logic | BAL5-AI rerun | spec | games/emberwake/docs/specs/BAL5_LEVERS.md section 2 | skill_fields.gd, SkillValidator, StatusEffects grant path, skill tests |
| BAL5-A | Data levers on trigger: shield_wall fortify (T3), boss HP 1.8 / 2.0 (B3), mage firebomb at L3 (A3, Q4) | asset | BAL5-AI, BAL5-PH, BAL5-L as needed | spec | games/emberwake/docs/specs/BAL5_LEVERS.md sections 1-4 | data/catalog/{skills,classes}.json, stat_rules.gd TIERS (B3 is a one-line logic edit), data/seeds/bal5.gd |
| BAL5-R | Final rerun and band confirmation | balance | BAL5-A | spec | games/emberwake/docs/specs/BAL5_LEVERS.md | games/emberwake/docs/BALANCE_REPORT.md |

Ashmaw: no new row; BOSS1-B uses K1 (add `BAL5-SIM` to its Depends on), and F2 is a candidate line for the ASHPACE1 spec. Option-build candidates B4, T4, S3, A4 go to the first design review list.

## Out of scope

Damage formula changes; heal power changes (tried, R); kite shield mitigation and firebomb reach (tried, R); injuries or loss penalties (HISTORY 2026-10-18); per-block re-targeting (deferred in BAL3 4; revisit only if S1 fails); retuning shipped world encounters outside ASHPACE1; Grakka's stats (BOSS1 5).

## Open questions

1. **Q1. Party targets at a trio?** Supersede BAL3 Q2 ("party of 4") with "warrior + two recruits, 10 trio comps" for the Ashlands bands. Rec: yes.
2. **Q2. Boss tier HP 1.5 -> 1.8 allowed as a last step** (after trio and sim phase, trigger boss > 60)? It changes "BAL2 answers" (7). Rec: yes, on trigger only.
3. **Q3. Solo bands.** Replace solo elite ~50 with "solo warrior >= 90 on the RECRUIT1 solo fights"; keep solo boss 10-20 measured against a fixed brawler (D2); warrior and priest exempt from the duel band. Rec: yes.
4. **Q4. Mage gets Firebomb at L3** (path swap with Arc Lance)? Changes the recruit mage's early kit. Rec: yes only if A1 + A2 leave mage foes/cast under 1.0.
5. **Q5. Band recalibration** from R ("Proposed band recalibration"): accept all except AoE swarm clear (kept at 70) and party boss (kept 30-50). Rec: yes, with tank allies hp_end read on won fights (T1).
6. **Q6. AI levers apply to enemies too** (early taunt, healer self-care, friendly-fire tolerance), as decided for lever C? Rec: yes.
