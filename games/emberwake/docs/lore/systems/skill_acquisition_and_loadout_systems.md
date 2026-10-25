# Skill acquisition and the combat deck

Status: canon for the Cold Steel path and the deck shape (Act 1); proposal for grafting and the Inoculated path (Act 2
on).

Numbers follow [CANON](../CANON.md) section 2.1; the trees are in [classes and skill trees](classes_and_skill_trees.md);
today's skill rules are in the game's [GAME_BRIEF](../../GAME_BRIEF.md) ("Skill unlock rule (SK2)", "Skill families
and loadouts") and the kit's skills in [RPG_KIT_CATALOG](../../../../../docs/RPG_KIT_CATALOG.md). Layer tags as in
[combat and the timeline](combat_and_timeline.md).

## 1. Two paths

| | Cold Steel | The Bitter Inoculation |
| --- | --- | --- |
| Source | levels and weapon training | harvested Aberrant Organs and surgery |
| Where | the Tinker's Hearth (Bram) | the Apothecary's Slab (Sister Aline) |
| When | Act 1 on | Act 2 on (locked until the Crucible) |
| Cost | none lasting | Strain per use, a higher dormant floor, Calcification for Tier III |

### 1.1 Cold Steel

- **Tiers by level** `[kit hook]` (exists): a class's `skill_path` owns families with an unlock level; tier t of a family is
  usable at unlock + 0 / 2 / 4 (SK2), across levels 1-10. The lore's Tier I (levels 1-3), Tier II (4-6) and Tier III
  (7+) are filled as `[data]` on that rule.
- **Weapon gates** `[kit hook]` (exists): a skill's `weapon_types` and `min_proficiency` decide who can use it (a sword skill
  needs a sword and the training level).
- **Training drills at the Tinker's Hearth** `[kit hook]` hub facility: the lore has Tier II learned through drills
  paid with skill points; the engine has no skill points (tiers open by level). What drills do is open (see gaps).
- **Keystones** `[data]`: after Bram's quest "Cold Slag, Black Iron" (its act is open, CANON section 6); a Cold Steel
  keystone may require masterwork forged gear (a weapon or item gate).

### 1.2 The Inoculation (Act 2 on)

1. **Harvest** `[kit hook]` (exists) loot tables (REWARD1), `[data]` organ items: elite aberrations and Inquisitors drop Aberrant
   Organs (e.g. Stalker Optic Nerve, Caustic Gall-Bladder; Apex organs such as the Titan Marrow Core are Tier III).
2. **Surgery** `[kit hook]` exchange recipe at a hub facility, `[data]` recipes: the operative and the organ go to the
   Slab; Sister Aline grafts it.
3. **Result**: the matching Inoculated skill is taught through the engine's `learn_skill` (`learned_only` skills reach
   a unit only that way) `[kit hook]` (exists); the dormant Strain floor rises (+10 for a Tier I graft; keystones lock it at 30)
   `[kit hook]`; a Tier III graft also advances Calcification by one stage. The operative starts the next mission
   bandaged (flavour; no rule).
4. **Graft slots** (Act 2 proposal): four, Cranial, Thoracic, Forearms, Visceral (CANON 2.1).

No graft surgery on anyone under 18; every playable operative is 18-24 (CANON 2.4).

## 2. The combat deck

### 2.1 The canon deck (8 slots) `[kit hook]`

| Slot | Count | Rule |
| --- | --- | --- |
| Base moves | 2, fixed | Move (tiles per block from speed) and Active Guard (50% damage reduction, stagger halved) |
| Class actives | 3 | from unlocked tiers; Cold Steel and Inoculated may mix; weapon gates apply |
| Passives | 2 | from unlocked passives; an Inoculated passive raises the floor |
| Keystone | 1 | one capstone |

The deck is set in the hub and locked when a battle starts (as `set_loadout` is today).

### 2.2 How it sits on today's engine

- The engine's loadout is 5 class families + 3 extra families (`MAX_CLASS_SLOTS`, `MAX_EXTRA_SLOTS`), locked at
  battle start; an empty loadout means unrestricted.
- The basics (strike, guard, move, rest, switch_target, offhand_jab) are always active and never take a slot; in the
  timeline ruleset Active Guard takes the `guard` basic's place.
- Passives are granted only by sources today (world character, faction grants, discipline passives, named item
  grants), not chosen in a loadout.

So CANON's 8 slots match the engine's 5 class + 3 extra in count only (CANON 2.1 says so since the 2026-10-25 final check): the two base moves are engine basics, the
three actives use class slots, and passives and the keystone need a slot type the engine does not have. See gaps.

### 2.3 No stances (timeline ruleset)

The four stances (Neutral, Offensive, Defensive, Tactical) are off in the Low Sleep (DEC-LORE-11); classic keeps them
for the Ashlands.

- Defence is an action: Active Guard over chosen blocks (50% damage reduction, stagger halved).
- Vulnerability is timing: a heavy maneuver's windup blocks are when it can be broken.
- Offence scales with Strain (Flesh-Tension +25% damage, Act 2 on), not with a stance.
- Speed replaces the "tactical" stance: Snap (1 block) lands before a Heavy (3-4 blocks) started in the same block.

## 3. Reusing the kit's skills `[data]`

The kit ships 57 skills (`games/emberwake/data/catalog/skills.json`); the Cold Steel branches reuse them as families
and add the new skills of the trees.

| Class (engine class) | New Cold Steel skills | Kit skills reused |
| --- | --- | --- |
| Vanguard (`warrior`) | Pommel Strike, Braced Poise, Intercepting Hook | `shield_bash`, `guard`, `sunder`, `rend_armor`, `shield_wall`, `riposte`, `taunt_shout` |
| Gutter-Stalker (`rogue`) | Hamstring Sever, Ash-Cloud and Fade, Throat-Carver's Art | `quick_stab`, `hamstring`, `bleeding_cut`, `backflip` (disengage), `envenom`, `assassinate` |
| Arbalestier (`ranger`) | Notched Quarrel, Tripwire Snare, Heavy Winch Sunder | `aimed_shot`, `pinning_shot`, `barbed_arrow`, `deadeye`, `leap_back`, `crippling_shot` |
| Slag-Brute (`brawler`) | Jaw-Breaker, Slag Slam and Throw, Foundry Shatter | `body_blow`, `haymaker`, `lunge`, `sweep_kick`, `cyclone_kick`, `crushing_blow` |
| Heretic Acolyte (`priest`) | Brimstone Cauterize, Tourniquet and Tonic, Resilience Ward | `mend`, `greater_mend`, `purify`, `smite`, `renewal`, `aegis` |
| Ichor Alchemist (`mage`) | Phosphorus Pot, Magnesium Flash-Bomb, Suffocating Choke-Gas | `firebomb`, `arc_lance`, `arcane_bolt`, `glacial_burst`, `inferno` |

The `riposte` kit skill is tied to stances in classic (a stance change removes it); under timeline it needs a review.
The skill fields the trees need (`ap_cost`, `recovery_blocks`, `stagger`, `armor_piercing`, `strain_cost`) are listed
once, in [rules engine architecture](rules_engine_architecture.md) section 3.1.

## 4. An operative's progression

1. Levels and weapon training open Cold Steel tiers; the Tinker's Hearth forges the gear keystones need.
2. From Act 2: harvest organs, graft at the Slab, learn the Inoculated skill, raise the floor; Tier III adds a
   Calcification stage.
3. Before each mission: build the deck (2 base moves + 3 actives + 2 passives + 1 keystone).

## 5. Gaps (open)

| Gap | Options | Recommendation |
| --- | --- | --- |
| Passive and keystone slots (the engine grants passives from sources, with no loadout choice) | A: a kit loadout slot type for passives and a keystone; B: passives stay source-granted, the deck is 2 base + 3 actives + keystone | A; it is the deck's whole choice of build |
| How the 8 slots map onto 5 class + 3 extra | A: 3 actives in class slots, 2 passives + 1 keystone in the extra slots (needs slot kinds); B: actives may use all 8 | A (CANON 2.1 now says the base moves are basics outside the slots; slot kinds: Q-48) |
| Skill points and drills (the lore's Tier II) | A: drop skill points, tiers open by level (SK2) and drills give a small Hearth bonus; B: add a skill-point currency | A (no new currency; the purse already has four resources) |
| Inoculated skills in a class's family list or graft-only | A: `learned_only` skills taught by the graft recipe; B: a second `skill_path` per class | A (exists today) |
| Graft slot limits against the deck's 3 actives | A: a graft slot holds one organ and teaches one skill; B: slots are cosmetic | A, Act 2 spec |
