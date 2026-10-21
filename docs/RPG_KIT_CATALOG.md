# RPG kit catalog: what Emberwake has that any RPG can reuse

Status: review copy for the user (2026-10-21), written from the game data by `tools/worldbook.py`. ASSETORG2 builds the
RPG kit layer (`kits/rpg/`) for assets; this list is the content and art behind it. **Kit** means generic: a class,
skill, item family, status or unit template any fantasy RPG could ship. **Game** means named or bound to Emberwake's
worlds (Ember-, Ash-, Grakka, Wardens). A kit row's art is reusable with it; game rows stay in `games/emberwake/`.
Code follows SPLIT1 principle 2 (extract when a second game needs it); this catalog only marks what would move.

Counts: 6 classes, 57 skills, 15 statuses, 13 passives, 20 item families, 32 items, 17 terrain types, 18 unit templates.

## Classes

| Class | Role | Weapons (proficiency) | Skill path | Art | Layer |
| --- | --- | --- | --- | --- | --- |
| Warrior | Tank: huge constitution, kite shield and plate; a wall that holds the line and hits lightly. | sword 3, mace 2, axe 2, hammer 1, shield 3, plate 3, leather 1 | 7 skills | body and clips on the sprite rig; generic portrait | kit |
| Ranger | Ranged skirmisher: dexterity and perception, bows and light armor. | bow 3, crossbow 2, dagger 2, sword 1, leather 3, cloth 1 | 8 skills | body and clips on the sprite rig; generic portrait | kit |
| Brawler | Bruiser: bare fists and a thick hide; leaps into a crowd and fights several foes at once. | fist 3, mace 3, hammer 2, leather 3, axe 1, plate 1, shield 1, sword 1 | 8 skills | body and clips on the sprite rig; generic portrait | kit |
| Priest | Support: heals and rallies allies; knowledge, maces and cloth. | mace 3, staff 2, dagger 1, cloth 3, leather 1, tome 3, orb 2 | 10 skills | body and clips on the sprite rig; generic portrait | kit |
| Rogue | Fast skirmisher: daggers and speed; dashes in, cuts and slips away. | dagger 3, leather 3, bow 2, cloth 2, sword 2, fist 1 | 10 skills | body and clips on the sprite rig; generic portrait | kit |
| Mage | Glass caster: intelligence and area damage from range. | staff 3, wand 3, orb 3, tome 2, dagger 1, cloth 3 | 7 skills | body and clips on the sprite rig; generic portrait | kit |

## Skills

| Skill | Kind | Classes (unlock level) | What it does | Icon | Layer |
| --- | --- | --- | --- | --- | --- |
| Strike (`strike`) | ATTACK | everyone | A measured blow. | kind icon | kit |
| Off-hand Jab (`offhand_jab`) | ATTACK | everyone | Melee jab with the off-hand weapon (or bare hands) at distance 1: ranged units' point-blank answer. | yes | kit |
| Shield Bash (`shield_bash`) | ATTACK | warrior L3 | Shield Bash: N power, N to Stagger for N. | yes | kit |
| Hamstring (`hamstring`) | ATTACK | brawler L4, rogue L5 | Hamstring: N power, N to Root for N. | yes | kit |
| Shockwave (`shockwave`) | ATTACK | everyone | Hits the 3 tiles in a line from you (aim at an adjacent tile). Friendly fire! | yes | kit |
| Volley (`volley`) | ATTACK | ranger L5 | Volley: N power. Plus area, hits allies too. Needs N training N. | yes | kit |
| Firebomb (`firebomb`) | ATTACK | mage L5 | Firebomb: N power. Burst area, hits allies too. Needs N training N. Hits a 3x3 area around a tile N away. | yes | kit |
| Guard (`guard`) | DEFEND | everyone | Raise a light guard: -N damage. | kind icon | kit |
| Focus (`focus`) | FOCUS | ranger L5, mage L4 | Focus: +50% attack for N. | kind icon | kit |
| Move (`move`) | MOVE | everyone | Move up to 2 blocks (speed-based tiles per block), once per round: 1 AP per tile. Keeps your stance. | kind icon | kit |
| Switch Target (`switch_target`) | TARGET | everyone | Lock onto a different enemy (no AP). Added automatically when an attack changes target. | kind icon | kit |
| Staggered (`staggered`) | STATUS | everyone | Block lost to Stagger. | kind icon | kit |
| Stunned (`stunned`) | STATUS | everyone | Block lost to Stun. | yes | kit |
| Rest (`rest`) | REST | everyone | Catch your breath: recover N and return to Neutral stance, free. | kind icon | kit |
| Sunder (`sunder`) | ATTACK | warrior L5 | Sunder: N power, N to Stagger for N. Needs N training N. Armor-breaking chop. | yes | kit |
| Reckless Swing (`reckless_swing`) | ATTACK | everyone | All-in swing: N power. Needs N training N. | yes | kit |
| Aimed Shot (`aimed_shot`) | ATTACK | ranger L1 | Aimed Shot: N power. Needs N training N. A patient shot. | yes | kit |
| Pinning Shot (`pinning_shot`) | ATTACK | ranger L3 | Pinning Shot: N power, N to Root for N. Needs N training N. | yes | kit |
| Arcane Bolt (`arcane_bolt`) | ATTACK | mage L1 | Arcane Bolt: N power. Needs N training N. A bolt of raw force. | yes | kit |
| Hex (`hex`) | ATTACK | priest L5, mage L3 | Hex: N power, N to Stagger for N. Needs N training N. | yes | kit |
| Haymaker (`haymaker`) | ATTACK | brawler L5 | Haymaker: N power, N to Stun for N. Needs N training N. | yes | kit |
| Body Blow (`body_blow`) | ATTACK | brawler L1 | Body Blow: N power, N to Stagger for N. Needs N training N. | yes | kit |
| Quick Stab (`quick_stab`) | ATTACK | rogue L1 | Quick Stab: N power. Needs N training N. A fast thrust. | yes | kit |
| Mend (`mend`) | SUPPORT | priest L3 | Mend: heals N. A friendly unit within 3 tiles, or yourself. | kind icon | kit |
| Rally (`rally`) | SUPPORT | priest L1 | Rally: grants haste for N starting next round. +1 tile per move block. | yes | kit |
| Taunt Shout (`taunt_shout`) | SUPPORT | warrior L1 | Taunt Shout: grants taunt for N starting next round. Foes that can reach you must target you. | yes | kit |
| Lunge (`lunge`) | ATTACK | brawler L3 | Lunge: N power. Dashes up to N. Gap-close: the dash is your move for the round. | yes | kit |
| Bleeding Cut (`bleeding_cut`) | ATTACK | rogue L3 | Bleeding Cut: N power, N to Bleed for N. | yes | kit |
| Riposte (`riposte`) | DEFEND | warrior L3 | Riposte: -N damage; grants riposte for N. Adjacent melee attackers you survive are countered for 50% of your attack power (once per attacker per round). Any stance change ends it; moving does not. | yes | kit |
| Purify (`purify`) | SUPPORT | priest L3 | Purify: cleanses stun. Active or pending debuffs on a friendly unit within 3 tiles, or yourself. Buffs stay. | yes | kit |
| Backflip (`backflip`) | DISENGAGE | rogue L4 | Backflip: jump up to N away from the nearest enemy (or a chosen one) to break melee contact. Your move for the round: it comes first, leaps over units but not walls, and a rooted unit can't. | kind icon | kit |
| Sweep Kick (`sweep_kick`) | ATTACK | brawler L3 | Sweep Kick: N power. Sweep area, hits allies too. Aim at a diagonal neighbour to pick the quadrant. Bare hands or feet. | yes | kit |
| Barbed Arrow (`barbed_arrow`) | ATTACK | ranger L5 | Barbed Arrow: N power, N to Bleed for N. Needs N training N. | yes | kit |
| Pounce (`pounce`) | ATTACK | rogue L3 | Pounce: N power. Dashes up to N. Needs N training N. Gap-close: the dash is your move for the round. | yes | kit |
| Smite (`smite`) | ATTACK | priest L1 | Smite: N power. Needs N training N. Holy force from a distance. | yes | kit |
| Deadeye (`deadeye`) | ATTACK | ranger L5 | Deadeye: N power, N to Stagger for N. Needs N training N. | yes | kit |
| Judgement (`judgement`) | ATTACK | priest L7 | Judgement: N power, N to Stagger for N. Needs N training N. | yes | kit |
| Charge In (`charge_in`) | ATTACK | brawler L5 | Charge In: N power, N to Stagger for N. Dashes up to N. Gap-close: the dash is your move for the round. | yes | kit |
| Lacerate (`lacerate`) | ATTACK | rogue L5 | Lacerate: N power, N to Bleed for N. Needs N training N. | yes | kit |
| Assassinate (`assassinate`) | ATTACK | rogue L7 | Assassinate: N power. Needs N training N. | yes | kit |
| Greater Mend (`greater_mend`) | SUPPORT | priest L5 | Greater Mend: heals 10 + 1.8 x KNOW; cleanses stun. Within 3 tiles, or yourself. | yes | kit |
| Renewal (`renewal`) | SUPPORT | priest L5 | Renewal: heals N; grants haste for N starting next round. Within 3 tiles. | yes | kit |
| Leap Back (`leap_back`) | DISENGAGE | ranger L3 | Leap Back: jump up to N away from the nearest enemy (or a chosen one) to break melee contact. Your move for the round: it comes first, leaps over units but not walls, and a rooted unit can't. | yes | kit |
| Arc Lance (`arc_lance`) | ATTACK | mage L3 | Arc Lance: N power. Line area, hits allies too. Needs N training N. Aim at an adjacent tile; hits the 3 tiles in a line. | yes | kit |
| Intimidate (`intimidate`) | ATTACK | warrior L4 | Intimidate: no damage, N to Weaken (-N outgoing power) for N. Roar at a foe. | yes | kit |
| Cyclone Kick (`cyclone_kick`) | ATTACK | brawler L5 | Cyclone Kick: N power, N to Stagger for N on every enemy hit. Sweep area, hits allies too. Bare hands or feet. | yes | kit |
| Inferno (`inferno`) | ATTACK | mage L7 | Inferno: N power, N to Bleed for N on every enemy hit. Burst area, hits allies too. Needs N training N. | yes | kit |
| Glacial Burst (`glacial_burst`) | ATTACK | mage L5 | Glacial Burst: N power, N to Root for N on every enemy hit. Burst area, hits allies too. Needs N training N. | yes | kit |
| Shield Wall (`shield_wall`) | DEFEND | warrior L5 | Shield Wall: -N damage; grants taunt for N and N. Foes that can reach you must target you, starting next round. | yes | kit |
| Crippling Shot (`crippling_shot`) | ATTACK | ranger L6 | Crippling Shot: N power, N to Slow for N. Needs N training N. | yes | kit |
| Rend Armor (`rend_armor`) | ATTACK | warrior L6 | Rend Armor: N power, N to Expose (-N) for N. Needs N training N. | yes | kit |
| Envenom (`envenom`) | ATTACK | rogue L6 | Envenom: N power, N to Poison (N per round, stacks) for N. Needs N training N. | yes | kit |
| Aegis (`aegis`) | SUPPORT | priest L6 | Aegis: grants Fortify (+N) for N starting next round. | yes | kit |
| Rejuvenate (`rejuvenate`) | SUPPORT | priest L6 | Rejuvenate: grants Regen (N per round) for N starting next round. | yes | kit |
| Feint (`feint`) | ATTACK | rogue L6 | Feint: N power, N. Needs N training N. | yes | kit |
| Crushing Blow (`crushing_blow`) | ATTACK | brawler L6 | Crushing Blow: N power, N. Needs N training N. | yes | kit |
| Opportunist (`opportunist`) | ATTACK | rogue L7 | Opportunist: N power, N. Needs N training N. | yes | kit |

Fallback icons per kind (from the ART2 sheets, in `art_src/skills_4.png`): any attack, ranged attack, spell, support, defence, disengage, status.

## Statuses

| Status | Type | Shown as | Effect | Icon | Layer |
| --- | --- | --- | --- | --- | --- |
| `stagger` | debuff | Staggered | loses a block | yes | kit |
| `stun` | debuff | Stunned | loses blocks | yes | kit |
| `root` | debuff | Rooted | cannot move | yes | kit |
| `bleed` | debuff | Bleeding | damage over time | yes | kit |
| `weaken` | debuff | Weakened | less outgoing power | yes | kit |
| `slow` | debuff | Slowed | fewer move tiles | yes | kit |
| `expose` | debuff | Exposed | less armour | yes | kit |
| `poison` | debuff | Poisoned | stacking damage over time | yes | kit |
| `burn` | debuff | Burning | damage over time | yes | kit |
| `haste` | buff | Hasted | more move tiles | yes | kit |
| `taunt` | buff | Taunting | foes in reach must target this unit | yes | kit |
| `riposte` | buff | Riposte | counters adjacent melee | yes | kit |
| `fortify` | buff | Fortified | more armour | yes | kit |
| `regen` | buff | Regenerating | heals each round | yes | kit |
| `empower` | buff | Empowered | more outgoing power | no | kit |

## Passives

| Passive | Effect | Layer |
| --- | --- | --- |
| Sword Training (`sword_training`) | Drilled with blades: +1 sword proficiency. | kit |
| Bow Training (`bow_training`) | Raised on the longbow: +1 bow proficiency. | kit |
| Axe Training (`axe_training`) | Trained with axes: +1 axe proficiency. | kit |
| Hardy (`hardy`) | Tough constitution: +2 CONS. | kit |
| Keen Senses (`keen_senses`) | Watchful: +2 PERC. | kit |
| Ruin Lore (`ruin_lore`) | Knows old stones: +3 KNOW in ruins. | kit |
| Steady Nerves (`steady_nerves`) | Hard to rattle: 25% resistance to Stagger and Stun. | kit |
| Second Wind (`second_wind`) | Recovers quickly: +1 AP each round. | kit |
| Dirty Tricks (`dirty_tricks`) | Street-taught: knows Quick Stab. | kit |
| Field Medic (`field_medic`) | Opens healing families one level earlier. | kit |
| Arcane Study (`arcane_study`) | Opens spell families one level earlier. | kit |
| Riposte Drill (`riposte_drill`) | Opens Riposte two levels earlier (level 1 for a warrior). | kit |
| Warden's Eye (`wardens_eye`) | Reads the foe coming for you: shows the whole round plan of every enemy that targets this hero. | game |

## Item families

| Family | Hands | Icon | Layer |
| --- | --- | --- | --- |
| Sword (`sword`) | main_1h, main_2h | yes | kit |
| Axe (`axe`) | main_1h, main_2h | yes | kit |
| Mace (`mace`) | main_1h | yes | kit |
| Hammer (`hammer`) | main_2h | yes | kit |
| Fist (`fist`) | main_1h | yes | kit |
| Dagger (`dagger`) | main_1h | yes | kit |
| Bow (`bow`) | main_2h | yes | kit |
| Staff (`staff`) | main_2h, main_1h | yes | kit |
| Wand (`wand`) | main_1h | yes | kit |
| Orb (`orb`) | off | yes | kit |
| Tome (`tome`) | off | yes | kit |
| Shield (`shield`) | off | yes | kit |
| Plate (`plate`) | - | yes | kit |
| Leather (`leather`) | - | yes | kit |
| Cloth (`cloth`) | - | yes | kit |
| Crossbow (`crossbow`) | main_2h | yes | kit |
| Accessory (`accessory`) | - | yes | kit |
| Consumable (`consumable`) | - | yes | kit |
| Material (`material`) | - | yes | kit |
| Quest (`quest`) | - | yes | kit |

## Items

| Item | Family | What it is | Layer |
| --- | --- | --- | --- |
| Iron Claymore (`wpn_claymore`) | sword | A heavy double-edged greatsword. Gains +N Power when wielded with N+ STR. | kit |
| Titan Warhammer (`wpn_warhammer`) | hammer | Colossal warhammer requiring N STR. Inflicts severe deficit penalty if under-statted. | kit |
| Oak Shortbow (`wpn_shortbow`) | bow | Supple curved bow capable of ranged volleys up to 3 tiles. | kit |
| Steel Broadsword (`wpn_broadsword`) | sword | Reliable military broadsword with modest strength requirements. | kit |
| Iron Shortsword (`wpn_shortsword`) | sword | Agile 1-handed short blade wieldable in main or off-hand (STR scaling). | kit |
| Spiked Knuckles (`wpn_knuckles`) | fist | Iron knuckle dusters for ruthless close-quarters off-hand strikes (STR scaling). | kit |
| Shadow Dagger (`wpn_dagger`) | dagger | Featherweight stiletto primed for rapid off-hand strikes (DEX scaling). | kit |
| Mystic Spellbook (`wpn_tome`) | tome | A bound book of rites held in the off hand. Strengthens healing and the grip of curses. | kit |
| Runed Staff (`wpn_staff`) | staff | Carved ash stave engraved with mystic glyphs. Scales with intelligence. | kit |
| Iron Kite Shield (`shd_kite`) | shield | Sturdy teardrop shield granting +N flat damage mitigation. | kit |
| Duelist Buckler (`shd_buckler`) | shield | Light metal fist-shield providing N mitigation without slowing down. | kit |
| Steel Plate Cuirass (`arm_plate`) | plate | Interlocking steel plates providing N mitigation with N burden. | kit |
| Leather Brigandine (`arm_leather`) | leather | Riveted hardened leather protecting vital organs while keeping mobility. | kit |
| Ring of the Bear (`acc_bear_ring`) | accessory | Chiseled bone band infusing the wearer with ursine fortitude (+2 STR, +2 CONS). | kit |
| Amulet of Swiftness (`acc_swift_amulet`) | accessory | Windstone pendant enhancing timeline reflexes (+3 DEX). | kit |
| Healing Draught (`con_heal_potion`) | consumable | Herbal restorative that instantly patches wounds for +90 HP. | kit |
| Elixir of Focus (`con_focus_elixir`) | consumable | Distilled ginseng elixir imparting +50% attack power for 2 rounds. | kit |
| Throwing Fire Bomb (`con_fire_bomb`) | consumable | Clay vessel of volatile pitch exploding on impact for 45 damage. | kit |
| Hatchet (`wpn_hand_axe`) | axe | A balanced one-handed axe. | kit |
| Battle Axe (`wpn_battle_axe`) | axe | Two-handed axe that bites deep. Heavy. | kit |
| Flanged Mace (`wpn_flanged_mace`) | mace | A one-handed mace for cracking guards. | kit |
| Stiletto (`wpn_stiletto`) | dagger | A slim main-hand blade made for quick thrusts. | kit |
| Plain Robe (`arm_robe`) | cloth | Simple cloth robe. Light as air, barely any protection. | kit |
| Cleric Vestment (`arm_cleric_vestment`) | cloth | Blessed cloth vestments that sharpen the wearer's devotion. | kit |
| Mage Robe (`arm_mage_robe`) | cloth | Embroidered robe that focuses arcane power. | kit |
| Tower Shield (`shd_tower`) | shield | A wall of iron. Superb protection, but heavy and slow to raise. | kit |
| Ash Wand (`wpn_wand`) | wand | A slender one-handed wand that bolts arcane energy up to 3 tiles. | kit |
| Mage Staff (`wpn_mage_staff`) | staff | A two-handed war staff. More power than a plain staff, at the price of weight. | kit |
| Cleric Mace (`wpn_cleric_mace`) | mace | A consecrated mace for those who fight to protect. Grants a touch of insight. | kit |
| Glass Wand (`wpn_glass_wand`) | wand | A brittle crystal wand that hums with raw power. | kit |
| Focus Orb (`wpn_orb`) | orb | A glass sphere held in the off hand. Sharpens spells and lets them reach one tile further. | kit |
| Ember-Scale Brigandine (`ember_brigandine`, world ashlands) | - | Leather studded with cooled slag scales. | game |

## Terrain

| Terrain | Move cost | Cover | Hazard | Tile art | Layer |
| --- | --- | --- | --- | --- | --- |
| `floor` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `grass` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `dirt` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `stone` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `sand` | 2 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `snow` | 2 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `shallow_water` | 2 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `deep_water` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `mud` | 2 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `lava` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `wall` | 1 | 0.3 | - | yes | kit (engine baseline for the common set) |
| `tree` | 1 | 0.15 | - | yes | kit (engine baseline for the common set) |
| `bridge` | 1 | 0.0 | - | yes | kit (engine baseline for the common set) |
| `rubble` | 2 | 0.15 | - | yes | kit (engine baseline for the common set) |
| `burning_ground` | 1 | 0.0 | burn (both) | yes | kit (engine baseline for the common set) |
| `toxic_bog` | 2 | 0.0 | poison (both) | yes | kit (engine baseline for the common set) |
| `ice` | 1 | 0.0 | slow (enter) | yes | kit (engine baseline for the common set) |

## Unit templates

| Unit | Side | Class | Tier | Level | Layer |
| --- | --- | --- | --- | --- | --- |
| Warrior (`warrior`) | party | warrior | hero | 5 | kit |
| Ranger (`ranger`) | party | ranger | hero | 5 | kit |
| Warrior (`dojo_warrior`) | party | warrior | hero | 2 | kit |
| Ranger (`dojo_ranger`) | party | ranger | hero | 1 | kit |
| Goblin Brawler (`goblin_brawler`) | enemy | brawler | minion | 3 | kit |
| Goblin Skirmisher (`goblin_skirmisher`) | enemy | rogue | minion | 3 | kit |
| Goblin Archer (`goblin_archer`) | enemy | ranger | minion | 3 | kit |
| Goblin Shaman (`goblin_shaman`) | enemy | priest | minion | 3 | kit |
| Goblin Berserker (`goblin_berserker`) | enemy | brawler | minion | 3 | kit |
| Enemy Brawler (`enemy_brawler`) | enemy | brawler | minion | 3 | kit |
| Fleeing Scout (`fleeing_scout`) | enemy | rogue | minion | 3 | kit |
| Goblin Berserker (`ambush_berserker`) | enemy | brawler | standard | 4 | kit |
| Training Dummy (`training_dummy`) | neutral |  | hero | 1 | kit |
| Rogue (`rogue`) | party | rogue | hero | 5 | kit |
| Mage (`mage`) | party | mage | hero | 5 | kit |
| Goblin Warlord (`goblin_warlord`) | enemy | warrior | boss | 6 | kit |
| Priest (`priest`) | party | priest | hero | 5 | kit |
| Brawler (`brawler`) | party | brawler | hero | 5 | kit |
