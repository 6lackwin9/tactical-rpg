# Session U1: stats and derived values

MASTERPLAN1 sub-block U1. Interactive design session with the user, 2026-11-06. Status: in progress; nothing final until the user closes it.

## The user's purpose for each derived value (2026-11-06)

1. **Max HP**: total health. Damage lowers it (min 0), heals raise it (max HP cap). At 0 the unit is knocked: a separate derived flag.
2. **Weapon power**: attack damage from the weapon; a formula by weapon, the scaling stat by weapon type; a fixed value from weapon and unit stat.
3. **Unarmed power**: damage with no weapon; rethink whether a separate value is needed.
4. **Speed**: today both initiative and move rate; consider splitting into move rate (how far a unit moves each move block) and initiative (which action takes precedence on a tick, the order of resolution).
5. **Armour**: mitigation with diminishing returns; survivability; helps classes specialise. Derives a mitigation value only, and conflicts with other mitigation sources.
6. **Crit chance**: adds crits to hits; variability and non-determinism; another damage amplifier.
7. **Resist**: counter to status effects; a chance on status application; reduces deterministic outcomes.
8. **Max poise**: the meter that holds command execution; adds or prevents interrupts to the command chain; needs simplifying so it is intuitive to play and track.
9. **Composure**: no behaviour today. See whether a morale system can use it, else remove; it should link to an anti-corruption mechanic.
10. **Ranged reach +1**: belongs to the skill, not the unit, as a unit can have melee and ranged abilities; rethink.
11. **Heal power**: strength of healing; belongs to the skill, not the unit; rethink.
12. **Unlock early**: unlocks skills sooner; more a passive than a derived value.
13. **Carry**: total weight allowed for the stash, a penalty for hoarding; adds little beyond inventory management.
14. **Item requirements**: minimum stats for a weapon's full power; a soft cap on when a weapon can be used; for proficient units the cost can drop, giving variance and preference in weapons.
15. **Dialogue check**: a threshold unlocks other options; flavour. Instead of a flat threshold, a roll plus bonus against it, to succeed or fail.

## The user's proposal

Treat every derived value as a meter: a value in a range (min, max), with behaviour on a threshold breach, on a change (+delta, -delta), or at a meter level.

## Decided (user 2026-11-06)

- **One derived class, two modes.** A **pool** holds a current value that moves in play, with behaviour on bands and on change (+/- delta): HP (knocked at 0 as a band effect), poise (5 pips as bands), AP. A **rating** is read when needed and changes only with its inputs (gear, buffs, level): initiative, move rate, armour, crit, resist, weapon power. Both: formula, clamp to (min, max), optional bands with effects. Reuses DERIVE1 rows and unit meters.
- **Speed splits** into move rate and initiative, both from DEX by default; a pack may separate them.
- **Unarmed folds in**: fists are an implicit weapon item, so no weapon is a weapon.
- **Moved out of U1**: ranged reach to the skill; heal value to the skill, with heal scaling as a base rule; unlock early to passives (U4); item requirements to gear (U3), proficiency lowering their cost; the dialogue check to story, as a roll plus bonus against a threshold; carry kept as an option, off by default.
- **Composure** parked as a named pool with no behaviour, a candidate for morale tied to anti-corruption, until its gameplay case.
- **The mitigation stack** is one rule covering armour, guard, cover and any others, settled in this session.
- **AP** counts as a U1 pool, the same shape as poise.
