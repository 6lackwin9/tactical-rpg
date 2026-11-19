# Seeds

Idempotent data edits that author game data, applied with the engine's writer (since 2026-10-19):
`aegis content apply data/seeds/<task>.json` (or `tools/fc.sh content apply ...`). A seed is
`{"steps": [{"op": "upsert", "kind": "<kind>", "entry": {...}, "world": "<id>"?, "replace": "<old id>"?}, {"op": "remove", "kind": "<kind>", "id": "<id>"}]}`,
steps in dependency order; format, kinds and rules: skill `patch-and-seed`. Rule: a seed leaves this folder in the
cleanup pass after its task is merged (git history keeps it).

The Godot-era seeds (`ContentCatalog.upsert` GDScript, `.gd`) ran on the Godot build; their effect is in the data, and
the files are frozen in `legacy/data/seeds/` (and in git history). The tables below index them as they were.

## Engine-era seeds

### Stale seeds in `archive/` (ENGINE-22, SEED-ARCHIVE1, 2026-11-19)

These seeds no longer apply to today's rules: each one, applied alone to the shipped data, is refused with the reason
shown. Their effect is in the data. They moved to `archive/` so the seed order check (`tools/fc.sh seeds`) and a
re-run of the folder only meet seeds that still apply. The engine's ruleset switch tests still read some of them
from there. `cast1a3.json` stays here: its `bile_coat` step now names its `fields`, so a re-run keeps the shipped
wording and status that `empower1.json` set.

| Seed | Last commit | Refused with |
| --- | --- | --- |
| `cln45.json` | `767ef4e` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `attacklock1.json` | `391fafe` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `cast1a4.json` | `391fafe` | units:founder_hollis: Deck order assault: 'tripwire_snare' is of intent field. |
| `cast1a4_blocked.json` | `391fafe` | units:founder_kaelen: Deck order maneuver: 'flank_cut' is neither an active of the deck nor a basic. |
| `cast1a4b.json` | `391fafe` | units:founder_kaelen: Deck order maneuver: 'flank_cut' is neither an active of the deck nor a basic. |
| `hooks1a.json` | `391fafe` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `u22s3.json` | `391fafe` | classes:warrior: Proficiency: unknown gear type 'mace'. |
| `u32s1.json` | `391fafe` | item_families:plate: A HEAD or BODY armour family names its armour weight (heavy, medium, light) under the ... |
| `u32s3.json` | `391fafe` | units:inquisitorial_purifier: Deck order assault: 'pyre_nozzle_sweep' is of intent field. |
| `u32s4.json` | `391fafe` | item 'wpn_glass_wand' has no known family ('wand'); a version 2 file stores family + modifiers only |
| `s22s3.json` | `cb08793` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `a12s1.json` | `3d0ea5b` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `c02a.json` | `3d0ea5b` | classes:ranger: A two-handed weapon can't be paired with an off-hand item. |
| `p12s1.json` | `3d0ea5b` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `r1bal.json` | `3d0ea5b` | units:founder_hollis: Deck order: 'field' is not an intent (one of assault, guard, support, maneuver). |
| `w24k1b.json` | `3d0ea5b` | units:founder_hollis: Deck order maneuver: 'flank_cut' is neither an active of the deck nor a basic. |
| `w24s4.json` | `3d0ea5b` | classes:warrior: Proficiency: unknown gear type 'mace'. |
| `classbasics.json` | `4ed71e4` | units:brawler: Loadout: the unit does not own the skill family 'body_blow'. |
| `layerdown2.json` | `0f688ea` | catalog/dojo_presets.json: Dojo preset 'claymo |
| `pk1s3fix.json` | `0f688ea` | units:inoculated_hero: Deck: 'marrow' is not one of active, passive, keystone, order, follow_ups, channeled. |
| `pk1s3fix_remove.json` | `0f688ea` | No dojo_presets entry 'channel_duel'. |
| `weaponroster1.json` | `0f688ea` | classes:warrior: Proficiency: unknown gear type 'mace'. |
| `weaponroster1_loadouts.json` | `0f688ea` | classes:warrior: Proficiency: unknown gear type 'mace'. |
| `weaponroster1_rulings.json` | `0f688ea` | items:wpn_iron_billhook: Hand 'main_2h' is not allowed by family 'polearm' (allowed: main_1h). |
| `pk2s6.json` | `b2224b2` | classes:ranger: A two-handed weapon can't be paired with an off-hand item. |
| `layerup3.json` | `3a54d6b` | item_families:fist: Technique 1: unknown skill 'tq_fist_breaker'. |
| `lore1.json` | `635b90b` | items:tl_billhook: Hand 'main_1h' is not allowed by family 'polearm' (allowed: main_2h). |
| `canon1.json` | `3ded9b4` | items:keepsake_kaelen: An accessory carries at most 2 bonus modifiers (requirements aside) or 1 passive hoo... |
| `act1fix1.json` | `d16deae` | items:notched_billhook: Hand 'main_2h' is not allowed by family 'polearm' (allowed: main_1h). |
| `gearrework1.json` | `30415b9` | classes:mage: Skill path: glacial_burst is an advanced skill and needs unit level 5 or higher (listed at 2). |
| `gearrework1_after.json` | `30415b9` | classes:ranger: Skill 'aimed_shot' is in both the class path and a technique (family 'aimed_shot'); a unit ... |
| `pr5_57.json` | `3d9ca3c` | item_families:sprayer: Technique 1: unknown skill 'tq_sprayer_jet'. |
| `classpass1_data.json` | `a1b1efb` | item_families:sprayer: Technique 1: unknown skill 'tq_sprayer_jet'. |

### Archived seeds (CLN-49, 2026-11-01)

Every engine-era seed below belonged to a finished task and left this folder in cleanup pass 3 (CLN-49); its
effect is in the data. Restore one with `git show <hash>:games/emberwake/data/seeds/<file>`.

| Seed | Last commit |
| --- | --- |
| `assetorg2.json` | `297e11ae` |
| `cast1_a1.json` | `6f4e4d5b` |
| `cast1_a1_alt.json` | `6f4e4d5b` |
| `cast1_a1_pavise_keepsake.json` | `81f23506` |
| `cast1_a1_talon.json` | `81f23506` |
| `derive1_d.json` | `83542956` |
| `encounter2_d.json` | `695c0599` |
| `input1_ma.json` | `7e3a09d0` |
| `material_stacks.json` | `81f23506` |
| `phasefold.json` | `691f2ee6` |
| `portraitfx1.json` | `8535fe0f` |
| `props1_ashlands.json` | `ecc9d640` |
| `rare_materials.json` | `be221e5a` |
| `reward1.json` | `6cca4c7e` |
| `ruleset1_a.json` | `512e0def` |
| `ruleset1_a_interim.json` | `512e0def` |
| `ruleset1a2.json` | `235a5486` |
| `ruleset1a3.json` | `77343bd2` |
| `sprite2.json` | `57c560d6` |
| `war2h1_gear.json` | `f58532f7` |

### Engine-era seed index

| Seed | Task | Note |
| --- | --- | --- |
| `encounter2_d.json` | ENCOUNTER2-D (kit samples `kit_winch_escape` hold escape and `kit_culling` phase kill) | full entries, as above |
| `reward1.json` | REWARD1-A (loot tables, Ashlands purses, key loot, objective reward) | full entries: regenerate it before a re-run on data other tasks changed since |
| `assetorg2.json` | ASSETORG2-D (body keys, portrait tiers, item icons) | full entries, as above |
| `phasefold.json` | PHASEFOLD-A (arrival phases for goblin_ambush, pass_ambush, ashmaw_stand; rewrites the catalog, ashlands and tidewrack encounters without the wave_* keys) | full entries, as above |
| `props1_ashlands.json` | PROPS1-A1 (`props` on the four Ashlands map templates: 63 props, spec 6.3) | full template entries, as above |
| `war2h1_gear.json` | WAR2H1 slice 2 (gear behind the looks: world items `<name>_hatchet` and `<name>_buckler` as the `named_items` of Hest and Brask (ashlands) and Orsa (iron_reaches); `goblin_warlord` `main_hand` `wpn_battle_axe` (2H), no off hand) | world item and character entries, the catalog unit |
| `sprite2.json` | SPRITE2 (art and palette keys: 4 named Horde elites drop their own art folder for the goblin body + own palette, gruk_smokecaller and 5 ash_* grunts get palettes, orsa_tallow gets art warrior/orsa_tallow) | full entries, as above |
| `u22s2.json` | U2.2-S2 (`training_dummy` gets `class_id` warrior with every slot and the pouch `none`, so it builds as before; a fixed unit with no class is now a content error) | full entries, as above |
| `ruleset1a2.json` | RULESET1-A2 (tl_spiker SKIRMISHER; Guard active on every tl_ enemy: tl_brace_counter / tl_fade; tl_crushing_judgment power_mult 2.2 to 1.5; encounter tl_flank_elite) | full entries, as above |
| `ruleset1a3.json` | RULESET1-A3 (tl_crushing_judgment power_mult 1.5 to 1.8; tl_footman, tl_footman_b, tl_crossbowman level 2 with CONS spend for about +20 % HP; tl_footman deck swaps Snap Jab for tl_hook_thrust; encounter tl_skirmish max_rounds 8 to 10; kit_culling gains a goblin_warlord) | full entries, as above |
| `u32s1.json` | U3.2-S1 (gear `block_cost` retired: the 22 item families and 2 items that set it re-saved without it; no other field moves) | full entries, as above |

## Godot-era seeds

### Archived seeds (CLN-2)

Restore with `git show <hash>:data/seeds/<file>`.

| Seed | Task | Merge commit |
| --- | --- | --- |
| `c1a1.gd` | C1-A1 (skill gating, priest proficiency, tome retype) | `11ffe67` |
| `m2c2b_a.gd` | M2C2B (class skill families) | `b7c4016` |
| `m2c2b_b.gd` | M2C2B | `b7c4016` |
| `m2c2b_c.gd` | M2C2B | `b7c4016` |
| `m2c2b_d.gd` | M2C2B | `b7c4016` |
| `m2c2b_lib.gd` | M2C2B (shared helpers for m2c2b_a..d) | `b7c4016` |
| `sk2.gd` | SK2-A (data step) | `9686a36` |
| `sk2_schema.gd` | SK2-L (schema step) | `7bed659` |
| `skill1_ranks.gd` | SKILL1 (ranked skills v2 migration) | `745962a` |
| `item1_migrate.gd` | ITEM1-A (item families, items v2 migration) | pending merge |
| `item1_tokens.gd` | ITEM1-A (number-quoting item descriptions to tokens) | pending merge |
| `class1_migrate.gd` | CLASS1-M (class skill_path list to dict, units heal_below default) | pending merge |

Example: `git show 11ffe67:data/seeds/c1a1.gd`

### Seeds active when the Godot build was frozen (applied; files in `legacy/data/seeds/`)

| Seed | Task |
| --- | --- |
| `m4a.gd` | M4-A (quest templates, Ashlands loot, linked quests) |
| `exp1a.gd` | EXP1-A (map templates, Ashlands exploration data) |
| `trav1a.gd` | TRAV1-A (Ashlands old/high road, Iron Reaches south road, two world_map exit markers) |
| `dlg1a.gd` | DLG1-A (service lines, Ashlands dialogues, keep shop, gold) |
| `rep1a.gd` | REP1-A (quest reputation deltas, Hest rep-gated choice, ashlands:ember_patrol) |
| `s14_a1.gd` | S14-A1 (first manifest rows: goblin_brawler alias, four sample file overrides; goblin_brawler art key) |
| `ui1_start_party.gd` | UI1-A (Ashlands start party: warrior, ranger, rogue, mage; tidewrack inherits it) |
| `s14_a2.gd` | S14-A2 (13 placeholder SFX blips + their manifest rows) |
| `s12t.gd` | S12T-A (the 17 terrain rows) |
| `s12s.gd` | S12S-A (crippling_shot, rend_armor, envenom, aegis, rejuvenate on class paths at level 6) |
| `s12c.gd` | S12C-A (feint, opportunist on the rogue path at levels 6-7; crushing_blow on the brawler path at level 6) |
| `bal3.gd` | BAL3-A (data levers: sweep_kick power 0.8/1.0/1.15; other levers gated off) |
| `camp1_unit.gd` | CAMP1-A1 (goblin_warlord boss unit for the Ashlands finale; run before camp1_story) |
| `camp1_story.gd` | CAMP1-A2 (Ashlands story: steel_for_the_gate, keep_one_fire_lit, Ashmaw Camp, wastes_track, wagon raid, Grakka and Orsa dialogues, steel choice, epilogue) |
| `trav2a.gd` | TRAV2-A (Ashlands supplies: manifest start_supplies 10, hest_stores supply_price 2, chest supplies on stores / ruin_cache / camp_cache) |
| `art2_a.gd` | ART2-A (manifest rows for intent chips, badge_foresight, chip_slay, chip_guest, overlay_objective_v0, sfx miss; run `tools/gen_art.py chips overlays` first) |
| `itemuse1_a.gd` | ITEMUSE1-A (use_reach: con_heal_potion 1, con_focus_elixir 0, con_fire_bomb 3) |
| `itemuse1_a2.gd` | ITEMUSE1-A2 (empty `pouch` on warrior, ranger, priest, mage classes; no free class potion) |
| `pr7_40.json` | PR7-40 enemy `short_name` (plate names) |
