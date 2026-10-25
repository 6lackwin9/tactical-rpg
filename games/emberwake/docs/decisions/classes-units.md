# Decisions: classes and units

Settled; do not re-litigate without asking the user. Scope: class budget and roster, class identities, units, passives, enemy scaling. Rows moved unchanged from `docs/DECISIONS.md` on 2026-10-17 (CLN-10), in their original order; append new rows at the end of the table. Index and global rules: `docs/DECISIONS.md`; superseded rows: `docs/decisions/archive.md`.

| Decision | Choice | Why |
| --- | --- | --- |
| Class balance | Equal stat budget (62) and growth (5/level) enforced by the catalog | Classes differ only by distribution, so balance is comparable |
| Unit variance | Authored spend pool (4 + level-1), exact | Same-class units differ without breaking budget |
| Mastery | Removed. Gear = class Proficiency dict. Traits = Passives granted by sources | Mastery was coupled to units; lore should grant traits |
| Passives | Generic lore-free templates in base catalog; sources reference them | Worlds stay modular |
| Enemy scaling | One base curve (class + tier + AI). Worlds set level and tuning deltas; "goblin" is lore | Same class behaves the same across worlds |
| Authored `power` | Absolute (tiers do not scale it) | Predictable for fixed/special units |
| Passives lifetime | Permanent by default; scripted fights/events may grant temporary ones for that fight/event | User 2026-10-15 |
| Class roster (six) | Warrior (tank: high defense, less offense and mobility), Ranger, Priest (support/heal), Brawler (gap closer), Rogue (reworked Scout, gap closer), Mage (ranged damage). Replaces Shaman, Berserker, Scout | User 2026-10-15: add a support class, keep at least two ranged offensive classes, avoid a melee-heavy start |
| Mage / Ranger split | Mage: INT-heavy ranged damage with AoE focus. Ranger: single-target ranged damage, AoE only for chip damage | User 2026-10-15 |
| Warrior tank | Highest HP/armor, taunt to protect allies, mostly reactive damage (counter-style) rather than proactive | User 2026-10-15 |
| Berserker behaviour | Goblin berserkers become Brawler minions with an aggressive AI profile (never flee, charge) | User 2026-10-15 |
| M2c-1 roster data | warrior CONS17 STR13 tank with hand axe + special tower shield + plate (loadout 40, 15 of it special; armor 31, HP 370 at Lv5, offense lowest of the melee classes); priest SUPPORT (cleric mace + vestment); mage SNIPER role with wand + mage robe; rogue keeps scout stats; brawler/ranger unchanged. Goblin shaman -> melee Priest minion; goblin berserkers -> Brawler minions with `reckless`; goblin skirmisher/scout -> Rogue. Old art keys (shaman/berserker/scout) kept for now. Skill paths are TEMPORARY (existing skills only); M2c-2 replaces them | M2c-1 2026-10-15 |
| Priest | Heal/support focus, no ally-protection buff; weapons: mace, staff, tome, orb allowed; NO shields (paladin territory) | User 2026-10-15 |
| Rogue | Positioning and mobility class: both an engage (gap-close) and a disengage skill | User 2026-10-15 |
| Ranger | Gets a disengage (jump back out of melee range); single-target ranged | User 2026-10-15 |
| Brawler | Jump into a hostile area and fight several melee enemies with L-shaped sweeps (hits (1,0), (1,1), (0,1) from origin (0,0), rotating with aim); power from STR+DEX hybrid because it fights bare-handed/knuckles | User 2026-10-15 |
| Mage | Volley removed for the mage; elemental damage vs the ranger's physical is parked on the backburner; AoE cross-pattern may be shared later via different skills | User 2026-10-15 |
| Brawler unarmed (user 2026-10-15) | Brawler stays unarmed (no knuckles item for now); unarmed combat mechanics are explored through the class (STR+DEX hybrid) | User 2026-10-15 |
| CLASS1 answers (user via front desk 2026-10-17) | (a) The new soft warnings on shipped classes (more than 5 families; a tier 3 capped past level 10) stay as warnings for the balance pass. (b) The unit duplicate-accessory check counts only the unit's own slots. (c) The legacy list-path converter (`ClassUnitFields.LIST_PATH_CONVERTER`) is switched off in the same merge as the CLASS1-M data migration | User 2026-10-17 |
| LEVERS1 role levers (user 2026-10-18 14:26, "all recs" on the three LEVERS1 questions) | `games/emberwake/docs/specs/LEVERS1.md` approved: the priest gets Ward, a shield on herself or one ally (Q1 A); the brawler's Cyclone Kick becomes a spin around the caster (Q2 A) that hits friends like every area attack (Q3 A); Bodyguard deferred behind BAL5 T3; boss acts twice not built | BAL5-R1b: priest down before round 3 in 83%, brawler 0.79 foes per cast; one rule for all area attacks |
| Non-human monster bodies (user 2026-10-25, CAST1_ASSETS N1/N2) | Code creature rigs: `tools/sprites/rig.py` gains new skeletons (a four-legged one, a segmented crawler) drawn in code like every other body, with every clip and palette recolours; the image API makes only their portraits (asked first), which can serve as the visual reference | An image-API picture cannot drive the code rig; rotating API parts blurs 48 px pixel art, and API frame sheets drift between frames. Built by CREATURE1 (2026-10-25). |
| Animated, reactive portraits (user 2026-10-25) | (A) now: code effects on every portrait as a reusable engine piece with a UI Gallery page (idle breath, rim-light flicker, blink, sinister overlays, reactions to hits, wounds, corruption, Composure and stealth); (B) then 2x2 expression sheets from the image API for the founders, hub cast and bosses (each run asked first); (C) layered puppets left for later. Surfaces in order: dialogue and vignettes, party and roster screens, then the battle unit panel, then the turn-order timeline | The base sprite is too small to show a reaction; effects over the portrait make it read as sinister and reactive without new art, and expression frames add true faces for the main cast |

## Partly superseded rows (text unchanged; the newer row wins on the named part)

- "Class roster (six)": the six-class limit is relaxed for user classes (content-forge.md "FORGE1 class answers": the Forge may create more classes); the shipped roster is unchanged.
- "M2c-1 roster data": the TEMPORARY skill paths are replaced by skills.md "M2C2B content (2026-10-16)" and "SK2-A data"; the old art keys are mapped in CLN-15 (balance.md "Small answers").
- "Priest" (class identities): "no ally-protection buff" is relaxed by "LEVERS1 role levers" (Ward on one ally); heal and support focus and the weapon list are unchanged.
