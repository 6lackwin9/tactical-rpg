# Roadmap (agent-facing summary; the README roadmap has the long form)

## Done
- Planner/resolver/AI combat, 9x9 zones, replay, modular combat UI
- Content catalog + Content Forge; World Engine + World Forge (inheritance, lore layer)
- Class/unit rules: six classes, equal budget, spend pool, tiers, proficiency dict, passives, skill paths

## Next (in order, confirm with the user before starting any)
1. Balance pass: the sim exists (`tools/fc.sh sim`); baseline shows warrior dominant and shaman unable to win (see docs/WORKFLOW.md). Decide targets with the user (design-brainstorm agent), then tune tier multipliers, class stats, shaman damage output.
2. Remaining base content: more skills per class path at levels 7-10, armor for cloth classes (shaman has no body armor), a few named passives per source type.
3. Party/progression layer: saving unit state (spend, learned skills, passives), level-up UI to spend the pool.
4. World layer: quests/lore granting passives and skills at runtime, then regions/travel/overworld (README systems 1, 2, 9).
5. Dojo/forge polish: Forge preview of tier-scaled numbers; Dojo shows passives.

## Open questions for the user
- Should enemies in worlds be able to define their own tier tables, or stay on the one base curve? (current decision: one base curve)
- How should passives be earned/lost in play (permanent vs. temporary)?
