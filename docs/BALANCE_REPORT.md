# Balance report (written by the balance agent; read by the brainstorm agent)

## BAL2-R clean rerun 2026-10-17 (cloud thread, branch claude/project-thread-w34eos at main d6f716c, after PLAN1 fix ca1d094)

Scope: the BAL2 rerun repeated on the fixed planner, at the user's request (ROADMAP S2/S4 are not promoted to `balance-ready`). Method: `tools/fc.sh sim party|sweep|role|duel|tier` as in README "Balance rules and sim v2", cap 10, smart_skills on, 3 seeds, spend class, preset balanced (role: spend class/cons/primary x preset balanced/aggressive), no code or data edits. Levels 3 and 5. About 2,300 fights in total: **0 `PLAN FAIL` and 0 `METRIC FAIL` in every run**. X = outside the decided target. Rechecked after merging main 7e1fdb6 (CLN-16 removed the `smart_skills = false` path; S12S-L status rows; UI1/UI2 screens; S14-L2/L3 presentation and audio): party L3/L5, sweep L3 and duel L5 print identical output. The previous BAL2 rerun (abort-contaminated shipped-sim numbers plus a scratch repaired runner) is archived, marked superseded, in `docs/archive/BALANCE_REPORT_2026-10.md`.

### Headline

1. **The plan-abort defect is gone** (I1 resolved by PLAN1): no fight ends on a refused plan. The clean numbers match the old repaired-runner picture closely, so its conclusions stand; differences are noted per table.
2. **Party encounters are too easy**: `line`, `elite` and `boss` clear 100% for the default party at L3 and L5, and for every one of the 15 compositions at L5; `boss` stays 100% at elevel +2, `elite=mage` and `boss=warrior`. Only `wave` bites (L5 default party 0%, elevel +1 33%).
3. **Casters die before they act**: at L5 the mage deals 0 damage in the default party's `mob` and `wave` fights (184 damage taken = its full HP), and in `aoe_swarm` it never hits more than 1 foe per cast. The priest heals 93-256 per party fight but 0 in the support limit test.
4. **Role scenarios are mis-scaled**: `aoe_swarm` 0% clear for every class at L5, L3 `striker_priority` 0% for every class (the priority target is never downed), `ranged_dive` 100% for the ranger and the warrior. Only the striker (L5) and ranged KPIs rank their primary first.
5. **Tank taunts more but still does not protect**: warrior taunt active in 37% of alive blocks (was 8%), rank 1 at L5, yet its reference allies end at 1% HP (band 40).
6. Composition swap deltas are within +-10 at L5; at L3 rogue (+14.7) and brawler (-15.3) are out. Duels unchanged: warrior too strong, priest too weak (mage too weak at L3).

### 1. Party mode, default comp warrior,ranger,priest,mage (3 fights per encounter, clear %)

| Encounter | Target | L3 default | L3 loadout=role | L5 default | L5 loadout=role | L5 elevel +1 | L5 elevel +2 |
| --- | --- | --- | --- | --- | --- | --- | --- |
| mob | >= 90 | 100 (surv 2.0/4) | 100 (2.0/4) | 100 (3.0/4) | 100 (2.3/4) | 100 | 100 |
| line | 70-90 | 100 X | 100 X | 100 X | 100 X | 100 X | 100 X |
| wave | 55-80 | 100 X (2.7/4) | 100 X | 0 X (0/4) | 0 X | 33 X (0.3/4) | 0 X |
| elite | 50-75 | 100 X | 100 X | 100 X (4.0/4, hp 0.94) | 100 X | 100 X | 100 X |
| boss | 30-50 | 100 X (r 2.0) | 100 X | 100 X (2.7/4, hp 0.48) | 100 X | 100 X | 100 X |

`elite=mage boss=warrior` at L5: elite 100 (2.0 rounds, 4/4 survive), boss 100 (3.7/4, hp 0.87). Per-unit L5 default: the mage deals 0 in `mob` and `wave` and 99-268 elsewhere; the priest heals 256 (`mob`), 233 (`line`), 93 (`wave`), 139 (`elite`), 0 (`boss`). Cells are 0 or 100 because the three seeds give near-identical fights. Versus the old repaired runner: the same pattern (L5 loadout=role `mob` and `elite` now 100 on the shipped sim).

### 2. Composition sweep (15 distinct 4-class comps, 3 fights per cell, mean clear % over comps)

| Level | mob (>= 90) | line (70-90) | wave (55-80) | elite (50-75) | boss (30-50) |
| --- | --- | --- | --- | --- | --- |
| L3 | 96 | 100 X | 76 | 78 X | 100 X |
| L5 | 96 | 100 X | 69 | 100 X | 100 X |

| Class | L3 swap delta / top-5 count | L5 swap delta / top-5 count |
| --- | --- | --- |
| warrior | +6.7 / 5 | +1.3 / 5 |
| ranger | +4.7 / 3 | -2.7 / 3 |
| priest | -5.3 / 2 | -2.7 / 2 |
| brawler | -15.3 X / 2 | -4.7 / 2 |
| rogue | +14.7 X / 5 | +5.3 / 5 |
| mage | -5.3 / 3 | +3.3 / 3 |

No comp is below 50% on `line` (all 100). Every class is in at least 2 of the top 5 (priest and brawler just meet it).

- L5 ranking: 8 comps at 100 (top: warrior,brawler,rogue,mage hp 0.66; warrior,ranger,priest,rogue 0.66); 5 at 87 (each loses one `wave`, or `mob` for ranger,brawler,rogue,mage); bottom warrior,ranger,priest,mage and warrior,ranger,priest,brawler at 80 (`wave` 0). The spread is `wave` only.
- L3 ranking: 9 comps at 100 (top warrior,priest,rogue,mage 0.76); bottom ranger,priest,brawler,mage 53, warrior,ranger,brawler,mage 67, warrior,priest,brawler,rogue 73 (`wave` and `elite` losses). The six weakest comps all contain brawler.
- Versus the old repaired runner: L3 elite 78 (was 89), L5 wave 69 (was 56), L3 brawler delta -15.3 (was -12).

### 3. Role limit tests, L5 (spend x preset grid, 3 seeds = 18 fights per cell; mean [floor..ceiling]; P primary, S secondary)

| Role (scenario) | Primary result | Bands | Other classes |
| --- | --- | --- | --- |
| Tank (tank_hold) | warrior clear 67, rank 1 | dmg_taken share 43 ok; atk_rcv share 36 [26..46] (band 45, ceiling reaches); taunt/alive 37; allies hp_end 1 X (band 40); survival 67 ok | brawler S 43 share, clear 0; every other class clear 0-22 |
| Ranged (ranged_dive) | ranger clear 100 X (too easy), rank 1 | nearest_foe 2.23 [1.44..3.00] (ceiling reaches 2.5); ranged dmg 100 ok; atk_rcv/rnd 2.29 [1.33..3.00] X (band <= 1.0) | warrior also clears 100; brawler 67; mage S 17 |
| AoE (aoe_swarm) | mage clear 0 X, rank 1; brawler clear 0 X, rank 3 X | foes/cast mage 1.00 [1.00..1.00] X, brawler 0.04 X (band 2.0); allies/cast 0 ok; swarm clear 0 X (band 70) | every class clears 0; ranger 0.50 [0..3.00] foes/cast |
| Support (support_attrition) | priest clear 39 [0..100] X, rank 3 X | heal 0 X, saves 0 X, survivors vs brawler -1.39 X (baseline 2.00) | ranger and brawler clear 100, rogue and mage 89 |
| Striker (striker_priority) | rogue clear 11 [0..67] X, rank 1 | priority down in round 1 in every variant (down by r5 100, prio dmg 67) ok; rogue survival 0 | brawler S clear 78 (down round 2.8); others 0-33 |

Versus the old run: Tank changed (taunt 8 -> 37, rank 2 -> 1, allies hp_end 10 -> 1); AoE mage foes/cast 0.72 -> 1.00 with a 0 floor gone (it now casts, but hits one foe); Ranged, Support and Striker are unchanged.

### 4. Role limit tests, L3 (same grid and bands)

| Role / class | clear (50-85) | First KPI | Other band checks |
| --- | --- | --- | --- |
| Tank / warrior | 50 [0..100] | dmg_taken share 38 [18..53], rank 4 X (rogue 50, priest 45 and brawler 41 rank above) | atk_rcv share 28 X, taunt 31, allies hp_end 17 X, survival 50 |
| Ranged / ranger | 94 [67..100] X | nearest_foe 1.61 [1.17..1.92] X | ranged dmg 93 ok, atk_rcv/rnd 2.24 X; warrior also 94 |
| AoE / mage | 33 [0..100] X | foes/cast 0.17 [0..1.00] X | swarm clear 33 X |
| AoE / brawler | 0 X | foes/cast 0.36 [0..1.00] X | swarm clear 0 X |
| Support / priest | 0 X | heal 0 X, saves 0 X | survivors 0.00 vs brawler baseline 0.50 (-0.50 X); only the rogue reaches 50 (56) |
| Striker / rogue | 0 X | priority never downed (round 11) X | every class clears 0 and none downs the priority target |

### 5. Diagnostics

| Class | Duel L5 (bin/hp, band 30-70) | Duel L3 | Tier L5 elite 1v1 (bin/hp) | Tier L5 boss 1v1 |
| --- | --- | --- | --- | --- |
| warrior | 90/89 X | 93/88 X | 0/0 | 0/0 |
| ranger | 53/53 | 67/64 | 0/0 | 0/0 |
| priest | 3/4 X | 10/17 X | 0/36 | 0/33 |
| brawler | 57/57 | 50/51 | 0/0 | 0/0 |
| rogue | 60/60 | 60/60 | 0/15 | 0/13 |
| mage | 37/37 | 20/20 X | 0/0 | 0/0 |

Duel L5 and tier L5 are identical to the old run (no data change since). Tier L5: minion and standard 1v1 100 for every class; 1v3 0 except priest vs minions 17/26 and rogue vs minions 50/50. Solo elite (target ~50%) and boss (10-20%) still read 0.

### 6. Targets that fail (summary)

| Target | Result | Gap |
| --- | --- | --- |
| Party line 70-90 | 100 at L3 and L5, every comp | +10 |
| Party elite 50-75 | sweep L3 78, L5 100 | +3 / +25 |
| Party boss 30-50 | 100 everywhere, up to elevel +2 | +50 |
| Party wave 55-80 | sweep ok (76 / 69); default comp L5 0 | comp-dependent |
| Sweep swap delta +-10 | L3 rogue +14.7, brawler -15.3; L5 ok | +-5 |
| Tank bands | allies hp_end 1 (L5) / 17 (L3) vs 40; atk_rcv share 36 / 28 vs 45 | -39 / -9 |
| Ranged bands | atk_rcv/rnd 2.29 vs <= 1.0; scenario clear 100 vs <= 85 | +1.3 / +15 |
| AoE bands | foes/cast 1.00 (mage) vs 2.0; swarm clear 0 vs 70 | -1.0 / -70 |
| Support bands | heal 0, saves 0, survivors -1.39 | all |
| Striker bands | L5 KPIs ok, clear 11; L3 nothing downed, clear 0 | -39 / all |
| Duel 30-70 | warrior 90 / 93, priest 3 / 10, mage 20 (L3) | +20 / -27 / -10 |
| Solo elite ~50, boss 10-20 | 0 for every class | -50 / -10 |

### Issues for the brainstorm agent (input to BAL3)

Numbering kept from the archived BAL2 rerun so the BAL3 row (I2-I14) still resolves.

- I1: **Resolved** by PLAN1 (ca1d094): 0 `PLAN FAIL` over every run here; the sim's PLAN FAIL guard stays as the check.
- I2: **Party encounters too easy**: `line`/`elite`/`boss` 100% for every comp at L5, boss 100% at elevel +2. Levers: encounter compositions (more or higher-tier enemies, a boss with real adds), tier multipliers, enemy gear budget, party-size scaling; decide whether the boss 30-50% target is at equal level. `wave` is the only encounter that bites.
- I3: **Squishy classes die in round 1** (mage 0 damage in L5 `mob`/`wave`, taking its full 184 HP; priest dies early in the role tests). Levers: caster HP/CONS growth, caster armor, enemy ranger Aimed Shot or rogue dash reach, start distance.
- I4: **Role scenario scale**: `aoe_swarm` 0% for every class at L5, L3 `striker_priority` 0% for every class, `ranged_dive` 100% even for the warrior. Constants in `tools/sim/sim_scenarios.gd`; target 50-85% clear.
- I5: **Tank**: taunt now active 37% of alive blocks and rank 1, but reference allies end at 1% HP and attacks received share is 36 (band 45). Levers: taunt reach/duration, shield_wall/intimidate, peel for the mage/ranger references.
- I6: **Brawler as AoE primary**: foes/cast 0.04 (L5) / 0.36 (L3); sweep_kick/cyclone_kick geometry or multipliers.
- I7: **Priest/Support**: heal 0 in the limit test at both levels; party-mode heals work (93-256 per fight). The problem is survival and plan-time targeting (I14).
- I8: **Solo tiers**: elite and boss 1v1 0% for every class; elite multipliers look too strong for a ~50% solo elite.
- I9: **Duel**: warrior 90 (L5) / 93 (L3) and priest 3 / 10 out of 30-70; mage 20 at L3. Levers: warrior EHP, priest mitigation, mage early damage.
- I10: **Bands**: now measured on a clean run; recalibration can start once the I4 scenario scale is settled (unchanged until then, per the BAL3 row).
- I11 (AI option B, Focus-skip under pressure), I12 (C, faster tier when likely to die), I13 (D, wider taunt trigger) and I14 (re-evaluate targets by block; reserve a heal block at plan time): the evidence above still applies (mage 0 damage in two L5 party fights, priest heal 0 in the support test, allies hp_end 1 under the tank). I13 is partly met (taunt 8 -> 37%) without fixing the protection gap.

Notes: seeds still give near-identical fights (cells mostly 0 or 100); a seeded start-position or initiative jitter would make bands measurable. Cloud run times (4 cores, 4 runs in parallel): role L5 905 s, role L3 500 s, sweep L5 358 s, sweep L3 185 s, party/duel/tier 13-51 s each.

---

Older runs (superseded BAL2 rerun 2026-10-17, BAL2 baseline 2026-10-17, BAL1 2026-10-16, baseline 2026-10-15): `docs/archive/BALANCE_REPORT_2026-10.md`.
