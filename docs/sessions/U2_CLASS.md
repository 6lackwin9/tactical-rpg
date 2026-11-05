# Session U2: class and loadout

MASTERPLAN1 sub-block U2. Interactive design session with the user, 2026-11-06. Status: in progress.

## The user's purpose for each item (2026-11-06)

1. **C1 Base stats**: the class's foundation. Instead of a static weight of 1 for each stat in the unit budget, each stat gets a weight (starting at 1) that sets the unit's budget, and a stat budget column weights the unit budget for each stat as the class's stat identity.
2. **C2 Growth**: the same logic as base stats, with rounding so the budget at each level follows the same curve as the starting allocation. Free stat allocation stays independent, for flavour and to reach specific checkpoints.
3. **C3 Gear proficiency**: what gear a unit can equip. Level 0 cannot equip; 1 standard; 2 lowers the requirement; 3 adds a bonus effect. Scaling: see whether it can use the derived logic, class-specific.
4. **C4 Skill path**: a unique path per class, defining its role and aligned with its stat balance. For a generic unit the template is access when an unlock condition holds.
5. **C5 Default loadout**: the class's base gear and pouch; completely data and used only for testing. Marked to a specific unit that can be duplicated rather than maintained by hand, especially for mobs. Named units may start with unique items, unavailable or much rarer in the world.
6. **C6 AI roles**: the behaviour of NPC and enemy units, reacting to the situation; weights on the possible actions by situation, then a pick by weight and randomness.
7. **C7 Tiers**: the unit's relevance to the game, as data scaling the base budget. Named enemies and NPCs can have no growth and a fixed budget, so a level 5 boss over budget for a level 5 or lower party becomes under budget as the party levels up.
8. **C8 Deck**: the unit's selected move list, its range of options; switching between acquired skills; base skills always available to all units.

## Unit types (the user, 2026-11-06)

- **Static units**: NPCs and named enemies; a fixed budget, skill set and loadout, saved as data, that does not change.
- **Growth units**: follow a class path and upgrade over time. Mobs may follow a growth path of their own, different from playable units', that scales as the game progresses (Act 1 against Act 2 mobs), so the same mob reference serves random encounters, gaining skills and budget in Act 2 rather than building stronger mobs and disabling weaker ones.
