# Magic and corruption: Strain and Calcification

Status: proposal (Strain, the Aberration Crisis, Calcification and grafting switch on in Act 2); the Act 1 rule that
no founder has Strain or Calcification is canon.

The numbers follow [CANON](../CANON.md) sections 2.2-2.3; where this file and CANON differ, CANON wins. Related:
[combat and the timeline](combat_and_timeline.md), [risk and reward](risk_reward_and_corruption_balance.md), [skill
acquisition](skill_acquisition_and_loadout_systems.md), [safehouse economy](safehouse_economy_and_triage.md). Layer tags
as in the combat doc.

## 1. Two branches

| | Cold Steel | Inoculated |
| --- | --- | --- |
| AP | 2-4 per action (`ap_cost`) | 1-2 per action, plus Strain |
| Strain | none | `strain_cost` 15-35 per action |
| Against armour | sunders (expose) and breaks poise | pierces armour (binary) |
| Lasting cost | none | Calcification through Flesh Breach and Tier III grafts |
| When | Act 1 on | Act 2 on (grafts start after the Crucible) |

Act 1 founders carry visible taints (Kaelen's bone spurs, Hollis's eye graft, Maren's black veins) as backstory and
Act 2 hooks only: 0 Strain, no Calcification, no countdown.

## 2. Strain in combat `[kit hook]` per-unit meter

Strain measures the toll of carrying titan marrow: Inoculated skills, dark ground and eldritch attacks raise it.

| Strain | Bracket `[data]` | Effect (engine scale) | Read |
| --- | --- | --- | --- |
| 0-24 | Dormant | none | baseline human |
| 25-49 | Acute Resonance | +15% crit chance, +1 tile per move block, -25% healing received | pupils dilate; healing starts to fail |
| 50-74 | Flesh-Tension | +25% damage, +4 armour rating, stagger taken x2 | muscle hardens to cord; easy to break |
| 75-99 | Aberrant Edge | +50% damage, recoil 10% of damage dealt | the Low Sleep's whispers drown out the squad |
| 100 | Rupture | crisis roll at round end | the graft overrides the host |

- Gain: `strain_cost` when the action starts (so the bracket can change before the same action lands); hazards and
  eldritch attacks add more.
- Decay: at round end -10 if the unit used no Inoculated action that round, a further -10 if it held Active Guard;
  never below the dormant floor. After battle Strain resets to the floor.
- Floor: grafts raise a unit's dormant floor (0-50); a floor of 25 or more starts every battle in Acute Resonance.

### 2.1 The Aberration Crisis `[game code]` (unless the kit meter's triggers can express it)

At round end, a unit at Strain 100 rolls 1d3 (equal odds, seeded):

1. **Unchecked Spasm**: a full-power strike on the nearest unit, ally or foe, then stunned next round.
2. **Biological Collapse**: 0 AP next round, Bleed for 3 rounds, armour 0 for the rest of the battle.
3. **Flesh Breach**: +1 Calcification stage; bone spurs break the skin and stay.

Strain resets to `50` (the bottom of Flesh-Tension) after the crisis resolves (`PK1`, `post_crisis_strain: 50`). The
crisis waits for round end: a unit that reaches 100 mid-round finishes its planned blocks.

## 3. Calcification `[kit hook]` per-unit stage track (out of combat, permanent)

Calcification advances only through a Flesh Breach and through Tier III graft surgery at the Apothecary's Slab. Ending
a battle in the Aberrant Edge does not advance it (that rule of the older drafts is retired).

| Stage | Name `[data]` | Look | Effect |
| --- | --- | --- | --- |
| I | Vein-Blackening | ink-black streaks from the graft site, chalky skin | -5 Composure (section 3.1), +5% debuff resistance |
| II | Calloused Chitin | black bone plates through shoulders, forearms or ribs | +2 armour rating, -1 tile per move block |
| III | Sensory Rot | milky cataracts, ears weeping black fluid | immune to fear and morale debuffs; -15% ranged damage beyond 3 tiles |
| IV | The Shakes | tremors as muscle fights petrified tendon | cannot equip two-handed weapons (bows and arbalests included); melee and bare-handed attacks pierce armour; sleeps in the Isolation Crypts |
| V | Terminal Drift | the self is gone | lost: walks into the catacombs or turns hostile |

A Stage IV Arbalestier loses the class weapon: the stage is a role change, not only a penalty.

### 3.1 Composure, the corruption resist `[kit hook]` derived stat

Composure is a derived stat, not a seventh base stat (user 2026-10-25, Q-47 and Q-47a; [CANON](../CANON.md) 2.1): the
corruption resist, computed by its own formula from the base stats plus gear, status and Calcification modifiers, and
shown on the unit panel. Every corruption effect reads it: Calcification I (-5), the High Bastion's curfew chimes and
belltower chimes, the Act 3 continental shockwave, the nightmares, the Living Martyr mercy reward and recruit quirks.
The formula and the numbers come from the derived-stat spec (`docs/specs/DERIVE1.md`: an `[engine]` formula evaluator
and `[kit hook]` data-defined derived stats, with Composure as a `[data]` row) and the balance lane. It plays no part in
Act 1, which has no corruption.

## 4. The Slab and the Hearth (hub side)

- **Graft surgery** `[kit hook]` exchange recipe, `[data]` recipes: at the Apothecary's Slab, Sister Aline installs a
  harvested Aberrant Organ; the operative learns the matching Inoculated skill and the dormant floor rises. Tier III
  (Apex organs) also advances Calcification by one stage. Details in [skill
  acquisition](skill_acquisition_and_loadout_systems.md).
- **Flushes** `[data]` (`PK2`): Slab flushes spend Rock Salt to lower the operative's dormant Strain floor by `10`
  (`strain_floor_delta: -10`, clamped at `0`) for the next battle only.
- **Bone-scraping** `[kit hook]`: the only undo, unlocked by Sister Aline's quest ("Anatomy of a Living Saint"): -1
  Calcification stage and -10% max HP for good.
- **Tincture of Still Blood** `[data]` pouch item on a kit status (also unlocked by Sister Aline's quest): holds the
  user's Strain at its current value for one round, so a burst cannot cross into Rupture that round.

## 5. Cut / later

- The d100 crisis table (Flesh Mastery, Marrow Rupture, Predatory Frenzy, Spontaneous Calcification, Irreversible
  Metamorphosis) and its Composure term: cut, replaced by the 1d3 table (Composure itself stays, section 3.1).
- "Ending a battle in the Aberrant Edge adds Calcification": cut (CANON 2.3).
- Aberrant Edge's bonus bleed and stagger and its "100% armour pierce": dropped (Inoculated attacks pierce anyway).
- "-20 Stance Stability" in Flesh-Tension: replaced by stagger taken x2 (stances are off).
- Calcification "points": stages only.

## 6. Gaps (open)

| Gap | Options | Recommendation |
| --- | --- | --- |
| Composure (Stage I) is not an engine stat (STR DEX INT CONS PERC KNOW) | A: replace with an existing quantity, e.g. -5% debuff resistance against fear and morale, or -1 KNOW; B: add Composure as a new kit stat | Decided (2026-10-25, user, Q-47a; shipped `DERIVE1`): a derived stat, the corruption resist (`KNOW * 2 + round(CONS / 2)`) |
| Strain after a crisis | A: set to 75 (the Edge); B: reset to the floor; C: set to 50 (Flesh-Tension) | Decided (`PK1`): set to `50` (`post_crisis_strain: 50`), keeping Flesh-Tension risk live without immediate re-rupture |
| What a Rock Salt flush does | A: -10 dormant floor for the next battle; B: -Strain gain for one battle | Decided (`PK2`): `-10` dormant floor for the next battle (`strain_floor_delta: -10`) |
| Stage III "-15% ranged damage beyond 3 tiles" replaces an accuracy roll (planner pick) | keep, or swap for -1 range | keep (`PK2` Stage III applies `-1` max weapon range above 1 plus fear/terror immunity) |
