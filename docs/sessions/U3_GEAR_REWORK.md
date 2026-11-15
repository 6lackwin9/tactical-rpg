# Session U3.3: Asymmetric Gear, Strict Proficiency & Off-Hand Rework (`GEAR-REWORK1`)

Status: **design closed / `[LOCKED]`** (interactive design walkthrough with user, 2026-11-14; supersedes the soft-untrained, weapon-switch, and `plate/leather/cloth` clauses of `docs/sessions/U3_GEAR.md`).
Layers: kit (`rpgkit_data`, `rpgkit_rules`, `rpgkit_battle`, `rpgkit_content`, `rpgkit_world`), game (`emberwake_gui` battle off-hand button & party screen), data (`item_families.json`, `classes.json`, `items.json`, `skills.json`, `passives.json`, `units.json`, `worlds/sunk_ward/*`).

## 1. Core Equipment & Proficiency Rules (`[LOCKED]`)

- **Retire `Switch Weapon` (`2H` + stowed `1H` backup)**: Remove `Inventory::toggle_active_weapon`, `needs_weapon_switch`, and `stowed_slot` rules. A unit's equipped `main_hand` and `off_hand` are always live together if legal (`pairs_with` / `1H`), or mutually exclusive (`2H Solo` locks `off_hand` empty).
- **Strict Proficiency (`0..3`) & `required_stats` Hard Lock**:
  - **`Proficiency 0` (Untrained)**: **Hard equip lock** (`can_wear` refuses untrained weapons, off-hands, and armor/helms; empty `proficiency` map on classless/fixture units remains unrestricted).
  - **Unmet `required_stats`**: **Hard equip lock** (`can_wear` refuses items whose `required_stats` exceed the unit's unscaled stats after proficiency discount `level * 2`; unusable drops feed Hearth salvage per `P-PROG-04`). Retires soft `untrained_penalty` (`0.75x` + `+1` burden) and `unfit_penalty`.
  - **`Proficiency 1` (`minor` weapon / `sub` off-hand / `sub` armor)**: Equippable at **`0.80x`** multiplier on weapon power or flat armor mitigation; **cannot** earn weapon Mastery XP or unlock family mastery bonuses.
  - **`Proficiency 2` (`secondary` & `third` weapons)**: **`1.00x`** baseline weapon power; **unlocks** weapon Mastery XP (`1.0x` rate) and Mastery 1-3 techniques, plus family `level2_bonus`.
  - **`Proficiency 3` (`main` weapon / `main` off-hand / `main` armor)**: **`1.25x`** multiplier on weapon power or flat armor/off-hand mitigation, family `level2_bonus` + `level3_passive`, and **`1.5x`** weapon Mastery XP rate on main-hand weapons.
- **Main-Hand Only Mastery XP**: Only `main_hand` weapon families at `Proficiency >= 2` earn Mastery XP, hold `techniques` / `keystone`, or use `MASTERY_BOOK` / `drill`. Off-hands and armor have **no** Mastery XP bar.
- **Seeded Class Roles (`proficiency.<fam>.role`)**: Seed `main`, `secondary`, `third`, and `minor` on `classes.json` (and add a `check/recipe.rs` content warning if a `train_proficiency` recipe names a `role` that matches `0` classes in the catalog). Enemy units wielding off-class gear receive explicit `*_drill` passives (`Proficiency::with_drills`).
- **Active Skill Sources (`CLASSPASS1` alignment)**: Keep the duplicate-skill guard in kit checks; in Emberwake, heroes acquire active skills strictly from class progression (`3 + 1` keystone) and weapon mastery / grafts (`3 + 1` keystone), with no free-floating learned active skills.

## 2. Weapon Hand Categories (`15` Soot-Gothic Main-Hand Families)

Every weapon family has **one** fixed hand category in `item_families.json` (`hands` + `hand_kind` + optional `pairs_with` list of allowed `off_only` families; `OFF_CATEGORIES` in `check/item.rs` derives dynamically from `item_families.json`). Retires `axe`, `mace`, and `wand` in Sunk Ward.

| Hand Category | Count | Families & Identity | Hand & Off-Hand Rule |
| --- | --- | --- | --- |
| **`2H Solo`** | `3` | `greatsword` *(Claymores, Executioner Blades — merges 2H sword/axe)*, `hammer` *(Foundry Sledges, Piston-Mauls)*, `staff` *(Igniter Staves, Prosector Crooks — 2H Heavy INT)* | Uses both hands (`hands: 2`, `pairs_with: []`); `off_hand` locked empty; high base power (`24-34`) |
| **`2H + Rig`** | `2` | `crossbow` *(Winch Arbalests, range `2-5`)*, `bow` *(Shortbows/Ash-Bows, range `2-4`)* | `hands: 2`, `pairs_with: ["ammo", "cloak"]` (hip/shoulder rig worn in `off_hand`) |
| **`1H Heavy`** | `7` | `polearm` *(Phalanx Spear / Garrison Short-Bill, `1H` Reach `1-2`, budgeted `~12-13` base power to pair fairly with `shield`)*, `gauntlet` *(Piston-Gauntlets, Penal Shackles — absorbs `penal_shackles`)*, `sprayer` *(1H Brass Vitriol Spray-Nozzle, ranged `2-4`)*, `sword` *(Broadswords, Sabers, Hookbills)*, `censer` *(Bronze Thuribles, Flanged Maces)*, `bone_saw` *(Chirurgeon Saws, Flensing Hooks)*, `scourge` *(Penitent Scourges, Cinder-Lashes, Reach `1-2`)* | `main_only` (`hands: 1`); pairs freely with any trained `off_only` item or `1H Light` weapon |
| **`1H Light`** | `3` | `dagger` *(Curfew Bodkins, Stilettos)*, `knife` *(Hunting/Throwing Knives, Scalpels — absorbs `boning_knife`, `orlovs_flensing_talon`; gains ranged `2-3` Throw when paired with `ammo`)*, `fist` *(Cestuses, Knuckles + unarmed fallback)* | `main_or_off` (`hands: 1`); in `off_hand`, grants item stats/effects and enables 1-block `Off-hand Jab` at the weapon's melee or ranged reach (lifting the legacy `attack_range <= 1` cap in `single_attacks`) |

*(Note: Dual-wield specialization bonuses when `rogue` mixes two `1H Light` weapons are deferred to `rogue` class passives in `CLASSPASS1`.)*

## 3. Locked 6-Class Main-Hand Weapon Matrix (`24` Slots: `6` Unique Mains + `9` Shared Pairs)

Every class holds `main (3)`, `secondary (2)`, `third (2)`, and `minor (1)`. Each of the `6` `main` families is unique (`P-PROG-01`), and each of the remaining `9` families is shared by `2` classes (`6 + 18 = 24` slots).

| Class | Sunk Ward Archetype | `main` (`Prof 3`, `1.25x`) | `secondary` (`Prof 2`, `1.0x`) | `third` (`Prof 2`, `1.0x`) | `minor` (`Prof 1`, `0.8x`, no Mastery) |
| --- | --- | --- | --- | --- | --- |
| **`warrior`** | Vanguard (`Kaelen`) | `polearm` *(1H Spear/Bill)* | `greatsword` | `sword` | `hammer` |
| **`brawler`** | Slag-Brute (`Dunstan`) | `gauntlet` | `fist` | `hammer` | `greatsword` |
| **`ranger`** | Arbalestier (`Hollis`) | `crossbow` | `bow` | `knife` | `sword` |
| **`rogue`** | Gutter-Stalker (`Vesper`) | `dagger` | `knife` | `bow` | `fist` |
| **`priest`** | Heretic Acolyte (`Maren`) | `censer` | `bone_saw` | `scourge` | `staff` |
| **`mage`** | Ichor Alchemist (`Elyse`) | `sprayer` *(1H Nozzle)* | `staff` | `bone_saw` | `scourge` |

## 4. Dedicated Off-Hands (`7` `off_only` Families) & Combat Off-Hand Button

- **Combat UI Off-Hand Skill Button**: The battle planning bar exposes a dedicated **Off-Hand button** dynamically bound to the equipped `off_hand` item:
  - Holding a `1H Light` weapon -> `Off-hand Jab` (`1` block, melee or ranged).
  - Holding a dedicated `off_only` item -> that off-hand family/item's signature `gear_skill` (`Shield Bash`, `Riposte`, `Brace`, `Reload`, `Fade-Step`, `Catalyst Pulse`, `Cite Passage`).
  - Empty `off_hand` (`2H Solo` or unequipped) -> disabled/hidden cleanly.

| Off-Hand Family | Stats & Passive Hook | Signature Off-Hand Button Skill |
| --- | --- | --- |
| **`shield`** | High flat mitigation + Poise / forced-move anchor (`burden` cost) | `Shield Bash` (1H shove / stagger check) |
| **`buckler`** | Low/`0` mitigation, `0` burden, `+Initiative` / `+Crit` | `Parry / Riposte` (reactive melee counter) |
| **`braces`** | Forearm recoil/stagger guard + brawl/poise bonus | `Brace` (short self-mitigation / poise steady) |
| **`ammo`** | Quiver / Pressurized Fuel Canister / Throwing Bandolier (`unit_meters` ammo pips, e.g. `3/3`): basic attacks still fire at `0` pips; spending `1` pip **empowers** ranged shots (`crossbow`, `bow`, `sprayer`) and unlocks ranged `2-3` Throw on `main_hand` `knife` | `Reload` (`1` block, refills ammo pips) |
| **`cloak`** | Ash-Cloak / Stalker's Mantle (`0` mit, `0` burden, `+Initiative` + ranged/glancing defense; pairs with `1H` and `crossbow`/`bow`) | `Fade-Step` (`1` block disengage step) |
| **`orb`** | Alchemical/Marrow Focus: **injects `INT` scaling** into non-`INT` `1H` weapon Attack Power (letting `priest`/`mage` swing `bone_saw`, `scourge`, `censer` off `INT`) | `Catalyst Pulse` (short range/status pulse) |
| **`tome`** | Psalter / Codex: class-neutral skill & status potency (`skill_potency` / `status_potency`, `skill_reach`) | `Cite Passage` (ally/self focus or ward pulse) |

### Locked 6-Class Off-Hand Proficiency Table (`Main [3] / Sub [1]`)

| Class | `Main` Off-Hand (`Prof 3`, `1.25x` mit) | `Sub` Off-Hand (`Prof 1`, `0.8x` mit) | Additional Off-Hand Access |
| --- | --- | --- | --- |
| **`warrior`** | `shield (3)` | `buckler (1)` | - |
| **`brawler`** | `braces (3)` | `shield (1)` | `fist (2)` (`1H Light` dual-wield) |
| **`ranger`** | `ammo (3)` | `cloak (1)` | `knife (2)` (`1H Light` off-hand) |
| **`rogue`** | `cloak (3)` | `braces (1)` | `dagger (3)`, `knife (2)`, `fist (1)` (`1H Light` dual-wield) |
| **`priest`** | `tome (3)` | `orb (1)` | - |
| **`mage`** | `orb (3)` | `ammo (1)` | - |

## 5. Armor & Helm Weights (`BODY` & `HEAD` — All `6` Permutations) & Accessories

- **Rename `plate / leather / cloth` -> `heavy / medium / light`** across both `BODY` and `HEAD` slots (sharing a single proficiency key per weight):
  - **`heavy`**: High flat mitigation & Poise, highest `burden`; helms add Poise / crit-ward (`Dunstan`'s `penal_foundry_apron` moves to `heavy`).
  - **`medium`**: Balanced mitigation & low `burden`; helms/hoods add `PERC` / Crit / foresight.
  - **`light`**: Low mitigation, `0` `burden`, status/elemental resists; cowls/masks add `INT` / `KNOW` / status ward.
- **Locked 6-Class Armor & Helm Matrix (`Main [3] / Sub [1]` — Exact `6` Ordered Permutations)**:
  - **`warrior`**: `heavy (3)` / `medium (1)`
  - **`brawler`**: `heavy (3)` / `light (1)`
  - **`ranger`**: `medium (3)` / `heavy (1)`
  - **`rogue`**: `medium (3)` / `light (1)`
  - **`priest`**: `light (3)` / `heavy (1)`
  - **`mage`**: `light (3)` / `medium (1)`
- **Accessories (`ACCESSORY_1`, `ACCESSORY_2`)**: Proficiency-free (`item_type == "ACCESSORY"`), `ALLOW_DUPLICATE_ACCESSORY = false`, bonus effects/stats only, with optional `required_stats` on high-tier/named relics (hard-locked if unmet).

## 6. Execution Split (`PR5-51a` -> `PR5-51b`)

1. **`PR5-51a` (`GEAR-REWORK1-KIT`)**: Pure kit + GUI mechanics (keep existing catalog data green via defaults/aliases where needed until `PR5-51b` seeds the new families, or gate new strict check behind family metadata): retire `Switch Weapon`, strict `Prof 0` & `required_stats` `can_wear` lock, `0.80x / 1.00x / 1.25x` proficiency power/mitigation multipliers, `pairs_with` & dynamic `OFF_CATEGORIES`, `main_or_off` ranged/melee `Off-hand Jab`, `orb` `INT` weapon-power hybridizer, `ammo` Empower + `Reload` meter hook, `cloak` `Fade-Step` hook, and the dedicated Combat UI Off-Hand button.
2. **`PR5-51b` (`GEAR-REWORK1-DATA`)**: Seed the `15` weapon families, `7` `off_only` families, `heavy`/`medium`/`light` armor & helms, `6`-class weapon/off-hand/armor tables with `role` tags, Founder & catalog item migrations (`notched_billhook` `1H` + shield on Kaelen, `penal_shackles -> gauntlet`, `penal_foundry_apron -> heavy`, `boning_knife` & `orlovs_flensing_talon -> knife`, `brass_vitriol_sprayer -> 1H sprayer` + `ammo`/`orb`, retiring `axe`/`mace`/`wand`), enemy `*_drill` passives, placeholder icons/body looks (reported in `FEEDBACK.md` for `ART_UI_REQUESTS.md`), and named golden re-records.

## 7. Implementation Notes & Provisional Ratifications (`PR5-51b`, `FB-267`)

- **Ratified `PR5-51b` (`30415b91`) provisional numbers & renames (`[PROVISIONAL]` for Playtest Review)**:
  - Off-hand skills & ammo: `reload` (`FOCUS`, `0` AP, `1` block); `fade_step` (jump `1`); `cite_passage` (`fortify +3`, `2` rounds, range `0-2`); ammo `empower_next` (`ammo_pips`, cost `1`, `power_mult: 1.25`); `buckler` (`mitigation 4`, `burden 0`); `braces` (`mitigation 3`, `burden 1`); new default items `hip_quiver`, `fuel_canister`, `ash_cloak`, `forearm_braces`; `penal_foundry_apron` moved to `heavy` (`mitigation 10`, `burden 4`); `knife` family band `1-2` + `knife_throw` (`1` block, `1` AP, reach `2-3`, `weapon_types: ["knife"]`, `min_proficiency: 1`); keepsakes trimmed to `2` bonuses (`keepsake_kaelen` drops stun resist, `keepsake_vesper` drops root resist).
  - §2–3 matrix confirmed over stale `PR5-51b` summary wording (`warrior` `hammer 1`, `rogue` `bow 2` / `fist 1`, `1H` `sprayer`, retired `axe`/`mace`/`wand`, no `flail`).
- **`§4` Off-Hand Button guards & `PR5-57` (`GEAR-REWORK1-FOLLOWUP`)**:
  - `shield`, `buckler`, and `braces` provisionally carry no `off_hand_skill` in Act 1 because `shield_bash` / `riposte` / `brace` already sit in class paths (revisited in `CLASSPASS1` `READY.md` §3).
  - `orb` `Catalyst Pulse` is queued in `PR5-57` (`GEAR-REWORK1-FOLLOWUP`), along with retiring the remaining soft-penalty fallback via unit-level drills on catalog enemies (`calcified_husk`, `guild_master_orlov`, `overseer_grendel`, `mortuary_flagellant`, `proctor_quell`), verifying `sprayer` spell-sourcing, fixing the `6` pinned illegal warrior NPC loadouts & `4` Dojo presets (`emberwake_world/tests/strict_loadouts.rs`), and adding strict `wear_lock` checks + `counts=on` `FB-268` telemetry.
  - `deserters_kettle_helm` and `nemeths_warded_cassock` belong to `ACTS23` (`CANON.md` §5.1) and stay unseeded in Act 1.
