# ASSETORG3: Three-Layer Template Promotion, UI Consolidation, and Sunk Ward Asset Separation

Status: approved (user 2026-11-08: Decisions A1, A2, A3)
Depends on: [ASSETORG2.md](ASSETORG2.md), [LOOK1.md](LOOK1.md)
Layer: engine (`engine/assets/**`, `aegis_ui`), kit (`kits/rpg/assets/**`, `rpgkit_content`), game (`games/emberwake/assets/**`, `emberwake_content`)
Companion registry: [SCREEN_ATLAS.md](../../games/emberwake/docs/specs/SCREEN_ATLAS.md)

## 1. Goal & Scope

While `ASSETORG2` established the 3-layer lookup chain (`game -> kit -> engine`), assets grew scattered across layers during the transition from the legacy *Ashlands* prototype to *Emberwake: A Scarlet Lullaby* (*The Sunk Ward*):

1. **Missing Engine UI Base Templates**: `engine/crates/aegis_ui/src/theme.rs` loads `ui/theme.json` and 9-slice frames from `ui/frames/*.{png,json}` across all three layers, yet `engine/assets/ui/` only held 5 glyph icons (`chevron`, `delete`, `help`, `lock`, `unlock`) and no base 9-slice frame templates or monospace font (`JetBrainsMono-Bold.ttf`).
2. **Duplicated & Undocumented Kit UI Folders**: `kits/rpg/assets/` duplicated the exact same 10 map marker PNGs in both `kits/rpg/assets/map/markers/*.png` and `kits/rpg/assets/ui/markers/*.png`, while `kits/rpg/assets/ui/{hud,services}/*.png` were added outside the `ASSETORG2` Section 3 folder tree.
3. **Ashlands Prototype vs. Sunk Ward Mixing in `games/emberwake/assets/`**: `games/emberwake/assets/` mixed 6 Sunk Ward Founders and Act I bosses/NPCs with 8 retired Ashlands heroes (`alda_wren`, `bryn_hale`, `dagny_coal`, `kit_vane`, `mara_voss`, `oren_dusk`, `sella_marsh`, `tev_quill`), 7 Ashlands goblin/bandit bosses (`grakka_ashmaw`, `skarn_ironjaw`, `bolg_redhand`, `vex_cutpurse`, `nok_flamebow`, `gruk_smokecaller`, `ash_warchief`), and Ashlands-only unit sheets/palettes.

`ASSETORG3` resolves all three issues and pairs with [SCREEN_ATLAS.md](../../games/emberwake/docs/specs/SCREEN_ATLAS.md) so both the Rust GUI (`emberwake_gui`) and Design Session interactive HTML mockups source every asset from a single canonical directory tree.

---

## 2. Updated Three-Layer Folder Tree

```text
engine/assets/                               # LAYER 1: Game-agnostic toolkit baseline
  fonts/
    OpenSans-SemiBold.ttf, OpenSans-OFL.txt  # Default body font
    JetBrainsMono-Bold.ttf, JetBrainsMono-OFL.txt  # Default monospace / numbers / debug font
  ui/
    chevron.png, delete.png, help.png, lock.png, unlock.png
    frames/                                  # Base 9-slice frame templates (neutral slate/iron)
      panel.{png,json}                       # 24x24 (slice 8)
      ornate.{png,json}, ornate_full.{png,json}
      card.{png,json}, card_hot.{png,json}, card_on.{png,json}
      button.{png,json}, button_hot.{png,json}, button_down.{png,json}, button_off.{png,json}
      inset.{png,json}, tab.{png,json}, tab_on.{png,json}, tip.{png,json}, parchment.{png,json}
  tilesets/
    common/<terrain>/v<n>.png                # 32x32 square fallback terrain tiles
    common_stage/<terrain>/v<n>.png          # 64x48 (4:3 tilted) battle stage fallback tiles
  audio/
    sfx/ui_{back,click,confirm,error}.wav

kits/rpg/assets/                             # LAYER 2: Reusable tactical RPG genre templates
  units/
    <class>/base/{sheet.png,clips.json}      # 6 core classes + dummy + beast + crawler
    warrior/{1h_buckler,2h_axe,2h_hammer,2h_sword}/{sheet.png,clips.json}
  portraits/
    common/<class>_base.png                  # Generic class silhouettes for unnamed mobs/NPCs
  items/
    <family>/base.png, <family>/<variant>.png
    any/{base,weapon,armor,accessory,consumable,material,quest}.png
  skills/
    kinds/{attack,defend,focus,support,disengage,rest,move,target,status}.png
  statuses/
    <status>.png                             # Core RPG status icons (bleed, burn, poison, etc.)
  props/
    common/<prop>/v<n>.png                   # Reusable rocks, crates, banners, trees
  ui/                                        # Consolidated RPG UI icon sets (no map/markers duplicate)
    hud/{gold,supplies,party,journal,stash,menu}.png
    pins/{here,town,fortress,gate,ruin,camp,wild}.png
    markers/{npc,encounter,roamer,chest,chest_open,exit,exit_locked,lore,point,poi,objective,world_map}.png
    services/{shop,recruit}.png
    badges/{quest,ready,foresight}.png
    chips/{guest,slay}.png
    intents/{act,area,arriving,attack,flee,focus,guard,move,rest,retreat,stunned,support,switch}.png
  audio/
    sfx/<combat_key>.wav
    music/<mood_key>.ogg

games/emberwake/assets/                      # LAYER 3: Emberwake (The Sunk Ward) game-specific art
  fonts/
    AlegreyaSC-Bold.ttf, AlegreyaSC-OFL.txt  # Gothic display serif
  ui/
    theme.json                               # Tallow & Iron theme palette + font bindings
    logo_title.png, logo_crest.png, banner_bar.png, banner_full.png
    frames/                                  # Tallow & Iron 9-slice frame overrides
      *.{png,json}
  backgrounds/
    menu/{title,main_menu}.png
    title/sunk_ward.png
    world_map/sunk_ward.png
    story/{intro_1,intro_2,intro_3,intro_4,hearth,meat_market}.png
    battle/{alley,rooftops,canal,foundry,tallow_vats,undercroft,watchpost}.png
    location/{sluice_undercroft,corvos_gate,fourth_kiln,crucible_watchpost}.png
  portraits/
    heroes/{kaelen,vesper,maren,hollis,dunstan,elyse}.{png,face.json}
    heroes/faces/<hero_id>/{neutral,angry,determined,hurt,afraid,sad,happy,surprised}.{png,face.json}
    npcs/{aline,bram,tess,wenn_tallow}.{png,face.json}
    npcs/faces/<npc_id>/{neutral,angry,hurt,afraid}.{png,face.json}
    elites/{inquisitorial_purifier,calcified_husk}.{png,face.json}
    bosses/{captain_vane,proctor_quell,guild_master_orlov,inquisitor_prelate_nemeth}.{png,face.json}
    bosses/faces/<boss_id>/{neutral,angry,hurt,afraid}.{png,face.json}
    common/{beast_render_beast,crawler_ash_crawler}.{png,face.json}
  units/
    beast/render_beast/{sheet.png,clips.json}
    crawler/ash_crawler/{sheet.png,clips.json}
    ranger/wenn_tallow/{sheet.png,clips.json}
    <class>/<variant>/palettes/<sunk_ward_palette_id>.json
  skills/
    <skill_family_id>.png                    # 100 skill family icons
  items/
    {polearm,scourge,censer,bone_saw,sprayer}/base.png
    hammer/titan_warhammer.png, leather/ember_brigandine.png, staff/runed_staff.png
  statuses/
    {steadfast,stealth}.png
  tilesets/
    sunk_ward/<tile>/v<n>.png                # Sunk Ward gothic stone, canal water, slag grates
  props/
    sunk_ward/<prop>/v<n>.png                # Braziers, sluice winch, toll gate, bone-mill wheel
  audio/
    music/<sunk_ward_track>.ogg
```

---

## 3. Concrete Consolidation & Cleanup Rules (Post-R4 Build Handoff)

### 3.1 Promote Base 9-Slice Frames & Monospace Font to `engine/assets/` (`ASSETORG3-E`)

- Copy `games/emberwake/assets/fonts/JetBrainsMono-{Bold.ttf,OFL.txt}` to `engine/assets/fonts/` so the engine provides both `OpenSans-SemiBold.ttf` (`body`) and `JetBrainsMono-Bold.ttf` (`mono`) out of the box.
- Seed `engine/assets/ui/frames/` with neutral baseline 9-slice frames (`panel`, `ornate`, `ornate_full`, `card`, `card_hot`, `card_on`, `button`, `button_hot`, `button_down`, `button_off`, `inset`, `tab`, `tab_on`, `tip`, `parchment`).
- Because `aegis_ui::theme::load` merges layers bottom-up (`engine -> kit -> game`), `games/emberwake/assets/ui/frames/` continues to override every role with the Tallow & Iron palette (`emberwake_gui/tests/theme.rs` stays green), and `engine/crates/aegis_ui/tests/theme.rs` (which runs on `Theme::default()` / isolated scratch layers) stays untouched.

### 3.2 Consolidate Duplicate Marker Icons in `kits/rpg/assets/ui/` (`ASSETORG3-K`)

- Today, `kits/rpg/assets/map/markers/*.png` (10 files) and `kits/rpg/assets/ui/markers/*.png` (9 files) are duplicate copies of the same marker icons (`chest`, `chest_open`, `encounter`, `exit`, `lore`, `npc`, `point`, `roamer`, `world_map`).
- Update `rpgkit_content::art::marker(id)` (and `emberwake_gui/src/region.rs` line 2036) from `"map/markers/{id}.png"` to `"ui/markers/{id}.png"`, move `exit_locked.png`, `poi.png`, and `objective.png` into `kits/rpg/assets/ui/markers/`, and delete `kits/rpg/assets/map/`.
- All RPG kit UI icons now live under one root: `kits/rpg/assets/ui/{hud,pins,markers,services,badges,chips,intents}/`.

### 3.3 Archive Ashlands-Only Prototype Assets Out of `games/emberwake/assets/` (`ASSETORG3-G`)

- Once `games/emberwake/game.json` sets `"start_worlds": ["sunk_ward"]` (Decision A1) and the *Ashlands* prototype is either retired or moved to `legacy/assets/ashlands/`, remove or archive the Ashlands-only files from `games/emberwake/assets/`:
  - **Ashlands Heroes (8)**: `portraits/heroes/{alda_wren,bryn_hale,dagny_coal,kit_vane,mara_voss,oren_dusk,sella_marsh,tev_quill}.{png,face.json}`.
  - **Ashlands Elites/Bosses/NPCs (9)**: `portraits/elites/{ash_warchief,bolg_redhand,gruk_smokecaller,nok_flamebow,skarn_ironjaw,vex_cutpurse}.{png,face.json}`, `portraits/bosses/grakka_ashmaw.{png,face.json}`, `portraits/npcs/{brask_toller,hest_the_quartermaster}.{png,face.json}`.
  - **Ashlands Bodies & Palettes**: `units/brawler/ash_warchief/`, `units/warrior/{brask_toller,grakka_ashmaw,hest_the_quartermaster,orsa_tallow}/`, and `units/*/{goblin,goblin_scout,goblin_berserker}/palettes/*.json`.
  - **Ashlands Backgrounds/Props/Tilesets**: `backgrounds/{battle/desert_outpost,title/ashlands,world_map/ashlands,location/ashmaw_camp,location/cinder_gate,location/ruined_courtyard,location/warden_keep}.png`, `props/ashlands/`, `tilesets/ashlands/`.
- **Note on Temporary Placeholder References (`CAST1_ASSETS.md`)**: Where a Sunk Ward character or boss previously pointed at an Ashlands portrait as a fallback in `CAST1_ASSETS.md` (e.g., Captain Vane using `brask_toller.png` before `portraits/bosses/captain_vane.png` was generated), update `CAST1_ASSETS.md` to reference the dedicated Sunk Ward portrait directly (`portraits/bosses/captain_vane.png`, `portraits/heroes/{kaelen,vesper,maren,hollis,dunstan,elyse}.png`, etc. — all of which already exist on disk!).
