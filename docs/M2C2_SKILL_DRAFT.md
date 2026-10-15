# M2c-2 skill content draft (status: proposed, not approved)

Scope: skill paths (step level 1-10) for warrior, ranger, priest, brawler, rogue, mage. All numbers are STARTING VALUES for the balance pass. Nothing here changes code or data; a builder would apply it through `ContentCatalog.upsert`. HISTORY.md checked: nothing below revives a deprecated idea (no mastery, no per-world tiers, no passive-from-class).

## Rules used
- `skill_path` entries are `{level: step level, skill: id}`. Skill level L needs step >= 2L-1 (L1 steps 1-2, L2 steps 3-4, L3 steps 5-6, L4 steps 7-8, L5 steps 9-10). AP cost = skill level.
- Today the catalog tops out at skill level 3; every L4/L5 skill below is NEW. If you prefer to keep the cap at 3 for M2c, move the L4/L5 skills to L3 (they stay unlocked later in the path) and drop their power ~10%.
- Existing `classes.json` still holds the OLD six classes; this draft is the content the reworked roster would use. Existing ids are reused unchanged. `learned_only` skills (mend, rally, taunt_shout, lunge, bleeding_cut, riposte) reach a unit through its class path (DECISIONS: support skills).
- Reach columns use the existing `reach_min/reach_max` (0/0 = weapon default). Ranged values are copied from `volley` (2-3) / `firebomb` (2-2); builder should confirm against bow/staff default reach.
- Weapon types seen in items: sword, axe, mace, hammer, fist, dagger, bow, crossbow, staff, wand, orb, shield (armor: plate, leather, cloth).

## Existing skills (reference)
strike L1 1.0x/2b; power_strike L2 1.5x; heavy_blow L3 2.4x/3b; offhand_jab L1 0.5x/1b; shield_bash L2 0.8x stagger70; hamstring L2 0.8x root80; shockwave L2 0.9x LINE; volley L2 0.7x PLUS reach2-3; firebomb L3 0.6x BURST reach2; guard L1; brace L2; fortress L3; focus L2; deep_focus L3; sunder L2 1.2x stagger50 (axe/mace p2); reckless_swing L2 1.7x (axe/sword p2); aimed_shot L2 1.4x (bow p2); pinning_shot L2 0.8x root80 (bow p1); arcane_bolt L1 1.1x (staff/wand p1); hex L2 0.6x stagger90 (staff/wand p2); haymaker L3 2.0x stun50 (fist/mace p2); body_blow L2 1.0x stagger50 (fist/mace p1); quick_stab L1 0.6x/1b (dagger p1); mend L2 SUPPORT ally-or-self heal 8+1.5xKNOW; rally L1 SUPPORT ally haste; taunt_shout L1 SUPPORT self taunt 2r; lunge L2 1.2x dash3; bleeding_cut L2 0.8x bleed70 2r; riposte L2 DEFEND riposte 2r.

## NEW skills needed (all start value, all `learned_only` unless noted)
| id | Name | Kind | Lv | Blocks | Power / heal | Area | Reach | Status (potency, rounds) | Weapon gating | Identity |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| shield_wall | Shield Wall | DEFEND | 4 | 3 | guard strong (above fortress, tune ~0.55) | - | self | grant taunt, 1r | shield p2 | Tank anchor: heavy mitigation and forces enemies onto the warrior |
| barbed_arrow | Barbed Arrow | ATTACK | 3 | 2 | 0.7x | NONE | bow default | bleed 60, 2r | bow p2 | Ranger DoT, single target |
| deadeye | Deadeye | ATTACK | 4 | 3 | 2.6x | NONE | bow default | none | bow p3 | Ranger burst on one target (the "main" nuke) |
| smite | Smite | ATTACK | 1 | 2 | 0.8x | NONE | 2-3 | none | staff/mace/wand p1 | Priest light ranged attack, below arcane_bolt |
| cleanse | Cleanse | SUPPORT | 2 | 1 | heal 0 | - | ally 0-3 | removes bleed/root/stagger/stun (ENGINE GAP) | none | Priest status removal |
| greater_mend | Greater Mend | SUPPORT | 3 | 3 | 12 + 2.0xKNOW | - | ally 0-3 | none | none | Priest stronger single heal |
| judgement | Judgement | ATTACK | 4 | 3 | 1.4x | NONE | 2-3 | stagger 40, 1r | staff/mace p2 | Priest finisher, deliberately weak |
| renewal | Renewal | SUPPORT | 4 | 2 | 6 + 1.0xKNOW, grant haste | - | ally 0-3 | haste, 1r | none | Priest heal plus tempo (uses existing haste) |
| shoulder_charge | Shoulder Charge | ATTACK | 4 | 3 | 1.6x, dash 4 | NONE | 1-1 | stagger 60, 1r | fist/mace/axe p2 | Brawler long gap-close with impact |
| uppercut | Uppercut | ATTACK | 5 | 3 | 3.0x | NONE | 0 | stun 60, 1r | fist/mace p3 | Brawler capstone single hit |
| pounce | Pounce | ATTACK | 3 | 2 | 0.9x, dash 4 | NONE | 1-1 | bleed 50, 2r | dagger p2 | Rogue gap-close that opens a wound |
| lacerate | Lacerate | ATTACK | 4 | 2 | 1.2x | NONE | 0 | bleed 90, 2r | dagger p2 | Rogue reliable DoT |
| assassinate | Assassinate | ATTACK | 5 | 3 | 2.6x | NONE | 0 | none | dagger p3 | Rogue capstone, no stance trick |
| arc_lance | Arc Lance | ATTACK | 2 | 2 | 0.9x | LINE | 2-4 | none | staff/wand/orb p1 | Mage line AoE |
| ice_burst | Ice Burst | ATTACK | 3 | 3 | 0.7x | BURST | 2-3 | root 60, 1r | staff/orb p2 | Mage control AoE |
| inferno | Inferno | ATTACK | 4 | 3 | 1.0x | BURST | 2-3 | bleed 50 (burn), 2r | staff/orb p2 | Mage AoE damage over time |
| meteor | Meteor | ATTACK | 5 | 4 | 1.5x | BURST | 3-3 | stun 40, 1r | staff/orb p3 | Mage capstone burst, long cast |

Engine notes: `cleanse` is the only skill needing a new effect (remove statuses from an ally). No skill needs new statuses beyond stagger/stun/root/bleed/haste/taunt/riposte. There is no "protect ally" status, so protection comes from taunt + self mitigation only. Backward dash (ranger disengage) is NOT in the engine (`dash_tiles` moves toward the target).

## Class paths (step: skill [level])
Each class also keeps the free utility ids (move, switch_target, rest, status skills).

### Warrior (tank; 8 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | guard [1] | reuse |
| 1 | taunt_shout [1] | reuse |
| 3 | riposte [2] | reuse |
| 3 | shield_bash [2] | reuse |
| 5 | fortress [3] | reuse |
| 7 | shield_wall [4] | NEW |
| 9 | sunder [2] | reuse (axe/mace) |
Offense stays low (no power_strike). Reactive damage via riposte; control via shield_bash/sunder stagger.

### Ranger (single-target ranged; 8 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | guard [1] | reuse |
| 3 | aimed_shot [2] | reuse |
| 3 | pinning_shot [2] | reuse |
| 5 | barbed_arrow [3] | NEW |
| 5 | volley [2] | reuse (chip AoE only, 0.7x) |
| 7 | deadeye [4] | NEW |
| 9 | deep_focus [3] | reuse |
Mobility is the ranger's speed stat (about 35 at level 5), not a skill; root via pinning_shot keeps enemies off.

### Priest (support; 9 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | guard [1] | reuse |
| 1 | smite [1] | NEW |
| 1 | rally [1] | reuse (ally haste) |
| 3 | mend [2] | reuse (heal) |
| 3 | cleanse [2] | NEW (engine gap) |
| 5 | greater_mend [3] | NEW |
| 5 | hex [2] | reuse (stagger debuff) |
| 7 | renewal [4] | NEW |
| 9 | judgement [4] | NEW |
Ten rows because step 1 holds four cheap skills; if 9 is too many, drop strike (smite replaces it).

### Brawler (bruiser, gap-closer; 9 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | offhand_jab [1] | reuse |
| 1 | guard [1] | reuse |
| 3 | body_blow [2] | reuse (stagger) |
| 3 | lunge [2] | reuse (dash 3) |
| 5 | haymaker [3] | reuse (stun) |
| 7 | shoulder_charge [4] | NEW (dash 4) |
| 9 | uppercut [5] | NEW |
Protection: guard only. Add `brace` at step 5 if the bruiser needs more survivability (balance call).

### Rogue (gap-closer, bleed, quick strikes; 9 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | quick_stab [1] | reuse (1 block, quick) |
| 1 | guard [1] | reuse |
| 3 | bleeding_cut [2] | reuse (bleed) |
| 3 | lunge [2] | reuse (dash 3) |
| 5 | pounce [3] | NEW (dash 4 + bleed) |
| 5 | hamstring [2] | reuse (root) |
| 7 | lacerate [4] | NEW |
| 9 | assassinate [5] | NEW |
Rogue lunge and brawler lunge share one skill; if the two gap-closers feel the same, give the rogue pounce earlier and drop its lunge.

### Mage (INT ranged AoE, weak single target; 8 skills)
| Step | Skill | New? |
| --- | --- | --- |
| 1 | strike [1] | reuse |
| 1 | arcane_bolt [1] | reuse (weak single) |
| 1 | guard [1] | reuse |
| 3 | volley [2] | reuse (PLUS) |
| 3 | arc_lance [2] | NEW (LINE) |
| 5 | firebomb [3] | reuse (BURST) |
| 5 | ice_burst [3] | NEW (BURST root) |
| 7 | inferno [4] | NEW (BURST DoT) |
| 9 | meteor [5] | NEW |
Mage ranges lean on the cheap AoE skills; high AP cost keeps burst turns rare.

## Coverage after the draft (skills per area, new + reused)
| Class | Heal/ally support | Gap-close/mobility | Debuff (stagger/root/stun) | DoT | Protection | Attack |
| --- | --- | --- | --- | --- | --- | --- |
| Warrior | taunt (1) | - | shield_bash, sunder | - | guard, riposte, fortress, shield_wall (4) | 1 (strike) |
| Ranger | - | - | pinning_shot | barbed_arrow | guard | aimed, deadeye, volley (chip) |
| Priest | rally, mend, cleanse, greater_mend, renewal (5) | - | hex, judgement | - | guard | smite, judgement |
| Brawler | - | lunge, shoulder_charge | body_blow, haymaker, uppercut | - | guard | strike, jab, haymaker, uppercut |
| Rogue | - | lunge, pounce | hamstring | bleeding_cut, pounce, lacerate | guard | quick_stab, assassinate |
| Mage | - | - | ice_burst | inferno | guard | volley, arc_lance, firebomb, inferno, meteor |
Residual gaps: only the priest heals (by design, the support identity), only brawler/rogue gap-close, only the rogue and mage damage over time besides ranger's single arrow. Warrior has no way to close distance; accepted (low mobility is the tank design).

## Open questions for the user
1. Raise the skill-level cap from 3 to 5 (L4/L5 skills are new), or keep it at 3 and compress the capstones to L3?
2. Priest `cleanse` needs a small engine addition (remove ally statuses). Approve that, or give the priest only heal/haste/hex?
3. Should the priest have an ally-protection buff (needs a new status, e.g. a damage-reduction ward), or is heal + haste + cleanse enough? Likewise "shield wall" only protects the warrior itself plus taunt.
4. Should brawler and rogue share `lunge`, or should the rogue get its own dash skill earlier (pounce at step 3)?
5. Ranger disengage (backward dash) is not in the engine; wanted, or does ranger speed cover it?
6. Is the "chip only" ranger AoE (volley 0.7x PLUS) low enough, and should the mage lose volley to the ranger entirely (mage keeps arc_lance/firebomb)?
7. Which weapon types for the priest (mace + staff + wand?) and mage (staff + wand + orb?), and do you want a priest shield proficiency?
8. Accept 8-9 skills per class (path above), or fewer for the first campaign?
