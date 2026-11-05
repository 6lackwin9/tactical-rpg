# Session U2: class and loadout

MASTERPLAN1 sub-block U2. Interactive design session with the user, 2026-11-06. Status: design closed (user 2026-11-06); build waits for the rest of Unit.

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

## Decided (user 2026-11-06)

- **Static units use templates, not classes.** For a hero the class is the limit on the unit's path and decides its progression; a static unit neither changes nor grows, so it shows little class-like behaviour. Generic static units act as templates for named units, splitting behaviour from class where it applies. Heroes may be reused as a base.
- **Mob growth** follows a world stage (act or campaign progress, flag-gated), not the party's level.
- **Tiers** scale the budget, with pool modifiers through data where budget scaling is too weak a knob, one consistent approach for all units rather than unit-specific behaviour. The flat 60 in the HP formula scales with tier too, so stats stay budgeted for the unit.
- **One budget per level for every class**; classes differ in shares. The first goal is that every base stat matters, so its weight sits near 1 and no stat feels wasted; balance then works around each stat's natural weight, so an INT-heavy class at weight 0.9 gets enough INT to match a DEX-heavy class at 1.1.
- **Playable growth** has its own, more robust curve on the same machinery: level-driven for stats and skills, complemented by gear and region access as gameplay progression. Mobs grow on their gated, flagged curve.

## Design result, for the user's approval: what is fixed and what is a parameter

| Item | Fixed (kit behaviour) | Parameter (data) |
| --- | --- | --- |
| Unit types | Growth units on a class path; static units from a static template, named units as a template plus overrides | Which template each named unit uses; its overrides |
| Budget | Budget = sum of stat x weight; one budget per level for every class | Stat weights (start 1); the playable curve by level; the mob curve by world stage |
| Class identity | Base stats and growth from the class's share of the budget per stat; largest-remainder rounding on a running total, so each level hits the budget exactly; free allocation separate | Class shares per stat; the free allocation pool by level |
| Proficiency | Four levels: 0 none, 1 standard, 2 lower requirement, 3 bonus effect | Per class and gear family: level and scaling weight in the weapon power rating; the level 3 effect per family, from the effect library |
| Skill path and deck | A path per class; generic access by unlock condition; a deck of picked skills plus base skills every unit has | Paths, unlock levels and conditions; the base skill list; deck slots |
| Loadout | Templates that units reference; named units may hold unique items | Template contents; unique items |
| Tiers | Tier scales the budget and is an input to derived formulas (the HP base); optional pool modifiers through the damage pipeline | Tier budget factors; tier terms in formulas; tier modifiers |

Moved to other sub-blocks: AI roles as weighted choice with randomness (AI, I1). Build note: tier as a formula input is one added input for the derived formulas, made once in the build.

**Approved (user 2026-11-06).** The core is one consistent unit base for every unit in the game: once a unit is defined, its behaviour follows and nothing is guessed about how it plays. An NPC put in a fight follows its loadout and AI; the weapons and skills it can use and the damage it deals come from the same core. Acceptance test for the build: any static or growth unit, NPC included, enters a fight with no unit-specific code.
