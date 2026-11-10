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

## Feasibility resolutions (`FB-10` to `FB-15`, accepted 2026-11-07)

1. **Two share vectors per class (`FB-10`)**: Each class defines `base_shares` (for the level-1 budget of 62) and `growth_shares` (for the +5 per level budget), using largest-remainder rounding on the running total (ties broken by largest fraction, then stat order). This reproduces all 7 existing classes at levels 1-10 identically so `rules_golden` holds.
2. **Tier scaling & role-based generic Unit templates (`FB-11`, `FB-78`, `FB-144`, `FB-145`, `PR4-2`, user 2026-11-08 & 2026-11-09)**: Merge `tier_hp` and `tier_power` into a single `tier_mult` (`rpgkit_rules/src/tiers.json` / `Catalog.tiers` rows `{id, mult, skill_cap}`: `hero: 1.0`, `minion: 0.4`, `standard: 0.825`, `elite: 1.25`, `boss: 2.0`; formula input `tier_mult`, with `tier_hp` and `tier_power` aliases removed in `PR4-2` `c49194b2`). In `U2.3-TIERS` (`R4-1`, `858eaab`), non-hero derived non-party units (`tier != "hero"`) take their reference base stats (`base_shares` and `growth_shares`) from `data/catalog/unit_templates.json` (`AGGRESSOR` base `13/9/7/15/9/9` + growth `2/1/0/2/0/0`, `SKIRMISHER` `9/16/9/8/12/8` + `1/2/0/1/1/0`, `SNIPER` `7/14/11/9/13/8` + `1/2/0/1/1/0`, `SUPPORT` `6/8/11/12/10/15` + `0/0/1/1/1/2` — delinked from playable hero `classes.json`, seeded in `r41s2.json`), or from an optional per-unit `template` override on `units.json` (`""` = by `ai_role`, `"class"` = keep class shares, or a role/template id; `calcified_amalgam` and `inquisitorial_purifier` use `"class"` so caster foes keep caster shares). Scale those reference core stats at level (before gear modifiers, which stay unscaled) by `tier_mult` (`max(6, roundi(stat * tier_mult))`), compute templated HP as `roundi(60.0 * tier_mult) + CONS * 8 + STR * 2` (other units keep `(60 + CONS*8 + STR*2) * tier_mult`), check gear requirements (`Equipment::meets_requirements` / `wield_penalty`) and `KNOW` skill unlocks against the **unscaled** stats at level, apply `tier_mult` in attack power to the weapon part only (not the already-scaled stat part) for templated units, apply the `6..18` stat check to base rows (`unit_templates.json` and `classes.json`), and keep the AI skill cap on tier.
3. **One unified unit builder (`FB-12`)**: Define a static unit as a derived unit at a fixed level and budget, reusing `base_unit`, `tuning`, `named_items`, and `passives` as the override layer instead of a second override system. Migrate `training_dummy` off the legacy classless path and route all unit creation (heroes, NPCs, Dojo, sim) through one builder function.
4. **World stage on `WorldRoster` and XP weight (`FB-13`, `FB-139`)**: Pass `stage` on `WorldRoster` (derived from `campaign.act` in campaign worlds, `0` in non-campaign worlds, host, and sim), and have `Roster::entry` and `roster()` compute enemy level (`base_level + stage * growth_per_stage`, where `growth_per_stage` is a per-unit `int` field on `units.json`, default `0`, applied to enemies only — `tier != "hero"`, no party guests) and XP weight using that `stage`. `WORLDFIGHT1` (`C18`) lands in `W2-6`.
5. **Proficiency slice ordering (`FB-14`)**: Build the proficiency bonus slice (`C3`) after `U3.1` and `U4.1` close so Level 1-3 rules (requirement discount, crit/burden, Level 3 effect) are implemented once.
6. **Module naming, schema shares & `auto_spend` (`FB-15`, `FB-44`, `FB-46`)**: Name the module `unit_budget` (distinct from gear `budget.rs`), keep `base_stats` and `growth` as a computed compatibility view so `party.rs`, `forge.rs`, `sim`, and goldens hold without churn, and move class JSON rows to stored float shares (`base_shares`, `growth_shares` with a float-stats field type in `aegis_content` and `auto_spend` reading `growth_shares`) together with the stat-weights schema slice (`FB-44`, `FB-46`). Keep loadout templates as an optional final slice.
