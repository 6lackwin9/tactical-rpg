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
