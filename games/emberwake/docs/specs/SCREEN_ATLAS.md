# SCREEN_ATLAS: Emberwake Screen Build Tracker & Canonical Art Sourcing Registry

Status: active (user 2026-11-08: Decisions A1–A4 & Asset Organization A1–A3)
Depends on: [ASSETORG3.md](../../../../docs/specs/ASSETORG3.md), [CAST1_ASSETS.md](CAST1_ASSETS.md), [LOOK1.md](../../../../docs/specs/LOOK1.md), [STAGE1.md](../../../../docs/specs/STAGE1.md), [STORYUI1.md](../../../../docs/specs/STORYUI1.md)
Mockup Artifact: `emberwake_screen_flow.html` (generated via `scratch/build_playable_preview.py` + `scratch/generate_html.py`)

## 1. Purpose

This registry is the single source of truth for:

1. **Screen Build Tracking**: What is already built in `games/emberwake/crates/emberwake_gui/src/*.rs`, what lands in the `R2–R4` engine/kit queue (`docs/READY.md`), and what lands in `Post-R4` Sunk Ward game setup.
2. **Canonical Art Sourcing**: Which exact files from **Layer 1 (`engine/assets/`)**, **Layer 2 (`kits/rpg/assets/`)**, and **Layer 3 (`games/emberwake/assets/`)** feed each screen in both the native Rust build and Design Session HTML mockups.

---

## 2. Screen Build Status & Architecture Overview (`640×360` Logical Canvas @ `2×` = `1280×720`)

| # | Screen / Overlay | Rust Source (`emberwake_gui/src/`) | Logical Geometry (`640×360`) | Code Status | `R2–R4` / `Post-R4` Remaining Work |
| --- | --- | --- | --- | --- | --- |
| **1a** | `MainMenu` | `menus.rs` (`MainMenu`, `draw_peek`) | Full `640×360`; `logo_room` top-right (`x: 388, y: 14, w: 235, h: 62`); `MenuStyle::Text` right column (`x: 422, y: 150`); `peek_room` left of menu (`x: 244, y: 150, w: 166, h: 68`) | **Built** | **Post-R4 (`A1`)**: Set `"start_worlds": ["sunk_ward"]` in `games/emberwake/game.json` so clicking `New game` skips `NewGameMenu` world-picker and boots `sunk_ward` directly. |
| **1b** | `SceneRunner` (`Tools`) | `runner.rs` (`SceneRunner`, `DebugFight`) | Full-screen modal (`640×360`) with `Encounters`, `Locations`, `Stories` columns | **Built** | **Post-R4 (`A1`)**: Keep legacy `ashlands` world accessible only here (or retire once Sunk Ward slice is complete). |
| **2** | `StoryStage` / `VignetteScreen` | `story.rs` (`StoryStage`) | Full `640×360` backdrop + bottom ornate caption card (`x: 40..600, y: 252..344`) with `56×56` speaker portrait | **Built** (`W3-1` adds `T1.2` seeded skill-roll steps) | **R3 (`W3-1`)**: Seeded `d20 + Stat >= DC` roll steps.<br>**Post-R4 (`A2`)**: Author `sequences.json` `"new_game"` chaining `0a -> 0b -> 0c -> hearth -> enter: sluice_undercroft`. |
| **3a** | `hud_bar` (Top Bar) | `hud.rs` (`hud_bar`, `PurseWord`) | `Rect(0, 0, 640, 20)` (`HUD_H = 20`) | **Built** (`W2-4` adds Alert & Red Tithing chips) | **R2 (`W2-4`)**: Wire `Alert` tier chip and `Red Tithing` macro deadline chip into `hud_bar`. |
| **3b** | `RegionMap` + `TalkBox` | `region.rs` + `talk.rs` (`TalkBox`) | Map `Rect(0, 20, 640, 340)`; top-left `"Here (N)"` legend; top-right log ticker; bottom-center `TalkBox` (`TALK_W = 400`, `TALK_LEFT = 76`, `PORTRAIT = 64`) | **Built** (`W3-1` adds inline roll result banner) | **Post-R4 (`A3`, `A4`)**: Seed `sluice_undercroft` (Aline, Bram, Tess, 6 Bunks, World Map exit) and `corvos_gate` street parley map (`Gate-Sergeant Corvo`). |
| **4a** | `BoardPane` (Notice Board) | `board.rs` (`BoardPane`) | Centered modal over `DIM_FILL`; left `240px` section list (`Main`, `Jobs`, `Under way`, `Closed` with `#e11d48` deadlines) + right detail pane | **Built** | **Post-R4 (`A3`)**: Seed Tess's Notice Board contracts (`Corvo's Gate`, `Bone-Mill Raid`, `Crucible Watchpost`, `Drowned Runoff Grate`). |
| **4b** | `ShopPane` (Trade / Forge) | `shop.rs` (`ShopPane`) | Centered modal over `DIM_FILL`; stock list + detail/buy pane | **Built** | **Post-R4 (`A3`)**: Seed Sister Aline's Apothecary stock and Bram's Slag-Heap Anvil stock. |
| **5** | `WorldMap` (District Map) | `worldmap.rs` (`WorldMap`) | Top `hud_bar` + `MAP = Rect(0, 22, 500, 316)` left map + right panel (`x: 506..636`) with `Travel`, `Enter`, `Camp`, `Mode` (`Stealth`/`Fast`), `Back` | **Built** (`W2-4` adds `Stealth`/`Fast` & Alert coupling) | **R2 (`W2-4`)**: `Stealth` vs `Fast` travel mode toggle.<br>**Post-R4 (`A4`)**: Seed 6 Sunk Ward nodes (Parley nodes open `RegionMap`; Raid nodes launch `BattleScreen`). |
| **6** | `Character` (Party / Roster / Gear) | `party.rs` (`Character`) | Modal popup; `48×48` portrait, member tabs (`Q/E`), `Active` toggle, 4 sub-tabs (`Stats`, `Level up`, `Skills`, `Gear`) | **Built** (`W2-5` & `W2-6` add 5-pip Belt + Wounds + 4-of-6 Bunks) | **R2 (`W2-5`, `W2-6`)**: 5-pip visual Belt Pouch in `Gear` tab + `Wounded` status badge & Active 4-of-6 toggle. |
| **7** | `BattleScreen` + `Stage` + `SkillMenu` + `TimelineView` | `battle/draw.rs`, `stage.rs`, `battle/skill_menu.rs`, `timeline_view.rs` | Top Strip `Rect(2, 2, 636, 24)`; Left Party `Rect(2, 30, 122, 288)`; Center Stage `Rect(128, 30, 384, 288)` (`BACKDROP` `384×52`, `BOARD` `12×9` cells of `32×24`, `FRONT_FACE` `384×12`); Right Foes/Intel `Rect(516, 30, 122, 288)`; Bottom Bar `Rect(2, 322, 636, 36)` | **Built** (`W2-1..W2-3` add `RING`/`ARC`, Dual-Intent, Boss Phase 2) | **R2 (`W2-1..W2-3`)**: `RING`/`ARC` stage overlays, `IR21` dual-intent strip icons, Boss Phase 2 transition.<br>**Post-R4**: Seed `0a`, `0b`, `0c`, `M2`, `M3`, `M5` encounters. |

---

## 3. Canonical Screen-by-Screen Art Sourcing Table

Every screen below lists the exact relative paths resolved across **Engine (`E` = `engine/assets/`)**, **Kit (`K` = `kits/rpg/assets/`)**, and **Game (`G` = `games/emberwake/assets/`)**.

### 3.1 Global Theme, Fonts & 9-Slice Chrome (All Screens)

| Role | Layer | Relative Path | Notes |
| --- | --- | --- | --- |
| Theme Manifest | `G` (over `E`) | `ui/theme.json` | 18-color Tallow & Iron palette (`#0a0b10` bg, `#d97706` focus, `#991b1b` frame_line, `#fbbf24` gold, `#d9cbb4` text) |
| Display Font | `G` | `fonts/AlegreyaSC-Bold.ttf` | `TITLE` / `H1` / `MENU` headers |
| Body Font | `E` | `fonts/OpenSans-SemiBold.ttf` | `BODY` / `SMALL` copy |
| Monospace Font | `E` (promoted) | `fonts/JetBrainsMono-Bold.ttf` | `MONO` counters, AP/Block numbers, roll breakdowns |
| 9-Slice Frames | `G` (over `E`) | `ui/frames/{ornate,ornate_full,panel,card,card_hot,card_on,button,button_hot,button_down,button_off,inset,tab,tab_on,tip,parchment}.{png,json}` | `E` holds neutral baseline templates; `G` holds Tallow & Iron overrides |

### 3.2 Screen 1: `MainMenu` (`menus.rs`)

| Element | Layer | Relative Path | Size / Placement |
| --- | --- | --- | --- |
| Full-Screen Backdrop | `G` | `backgrounds/menu/title.png` (fallback `backgrounds/menu/main_menu.png`) | `640×360` with `MENU_VIGNETTE = 0.4` |
| Game Title Logo | `G` | `ui/logo_title.png` | `logo_room` top-right (`235×62` logical) |
| Save Peek Frame | `G` | `ui/frames/ornate.png` | `peek_room` (`166×68` logical, left of menu column) |
| Peek Party Portraits | `G` | `portraits/heroes/{kaelen,vesper,maren,hollis}.png` | Up to 4 `32×32` portraits with `PORTRAITFX1` breathing & Tallow Amber rim |

### 3.3 Screen 2: `StoryStage` (`story.rs`)

| Vignette Beat | Layer | Backdrop Path (`640×360`) | Speaker Portrait (`64×64`) |
| --- | --- | --- | --- |
| Prologue 0a Intro (The Alley) | `G` | `backgrounds/story/intro_1.png` | `portraits/heroes/kaelen.png` |
| Prologue 0b Intro (The Rooftop) | `G` | `backgrounds/story/intro_2.png` | `portraits/heroes/vesper.png` |
| Prologue 0c Intro (Canal Bridge) | `G` | `backgrounds/story/meat_market.png` | `portraits/heroes/maren.png`, `portraits/bosses/captain_vane.png` |
| Undercroft Arrival (Tallow Hearth) | `G` | `backgrounds/story/hearth.png` | `portraits/npcs/aline.png` |

### 3.4 Screen 3 & 4: `RegionMap`, `hud_bar`, `TalkBox`, `BoardPane` & `ShopPane`

| Element | Layer | Relative Path | Used In |
| --- | --- | --- | --- |
| HUD Currency & Supply Icons | `K` | `ui/hud/{gold,supplies}.png` | `hud.rs` (`Scrap`, `Tallow` chips) |
| HUD Right Action Buttons | `K` / `E` | `ui/hud/{party,journal,stash,menu}.png`, `E: ui/help.png` | `hud.rs` (5 `18×18` icon buttons) |
| Region Legend & Map Markers | `K` | `ui/markers/{npc,encounter,roamer,chest,chest_open,exit,exit_locked,lore,poi,objective,world_map}.png` | `region.rs` `"Here (N)"` list & map markers (consolidated per `ASSETORG3-K`) |
| NPC Service Badges | `K` | `ui/services/{shop,recruit}.png`, `ui/badges/{quest,ready}.png` | `region.rs` overhead & legend row badges |
| Hub NPC Portraits | `G` | `portraits/npcs/{aline,bram,tess}.png` | `talk.rs` `64×64` left column, `board.rs`, `shop.rs` |
| Hub NPC Map Sprites | `K` + `G` | `K: units/priest/base` + `G: palettes/maren.json` (Aline)<br>`K: units/warrior/2h_hammer` + `G: palettes/grendel.json` (Bram)<br>`K: units/rogue/base` + `G: palettes/captive.json` (Tess) | `region.rs` standing character tokens |
| Region Tileset | `G` (over `E`) | `G: tilesets/sunk_ward/{floor,wall,shallow_water}/v<n>.png` (fallback `E: tilesets/common/...`) | `region.rs` ground & boundary walls |

### 3.5 Screen 5: `WorldMap` (`worldmap.rs` — Sunk Ward District Map)

| Element | Layer | Relative Path | Notes |
| --- | --- | --- | --- |
| District Map Canvas | `G` | `backgrounds/title/sunk_ward.png` (and `backgrounds/world_map/sunk_ward.png`) | Drawn in `MAP = Rect(0, 22, 500, 316)` |
| Location Pin Icons | `K` | `ui/pins/{here,town,fortress,gate,ruin,camp,wild}.png` | Drawn at each Sunk Ward node (`00`..`06`) |
| Quest Marker Badge | `K` | `ui/badges/{quest,ready}.png` | Drawn on nodes with active Notice Board contracts |

### 3.6 Screen 6 & 7: `Character` (`party.rs`) & `BattleScreen` (`battle/draw.rs`, `stage.rs`, `skill_menu.rs`)

#### A. Playable Founders (6)

| Founder | Class (`Role`) | Portrait (`G`) | Expression Faces (`G`) | Body Sheet (`K` / `G`) | Palette (`G`) |
| --- | --- | --- | --- | --- | --- |
| **Kaelen** | `warrior` (Vanguard) | `portraits/heroes/kaelen.png` | `portraits/heroes/faces/kaelen/*.png` | `K: units/warrior/2h_axe` | `units/warrior/2h_axe/palettes/kaelen.json` |
| **Vesper** | `rogue` (Flanker) | `portraits/heroes/vesper.png` | `portraits/heroes/faces/vesper/*.png` | `K: units/rogue/base` | `units/rogue/base/palettes/vesper.json` |
| **Maren** | `priest` (Support/DoT) | `portraits/heroes/maren.png` | `portraits/heroes/faces/maren/*.png` | `K: units/priest/base` | `units/priest/base/palettes/maren.json` |
| **Hollis** | `ranger` (Control/Ranged) | `portraits/heroes/hollis.png` | `portraits/heroes/faces/hollis/*.png` | `K: units/ranger/base` | `units/ranger/base/palettes/hollis.json` |
| **Dunstan** | `brawler` (Poise Breaker) | `portraits/heroes/dunstan.png` | `portraits/heroes/faces/dunstan/*.png` | `K: units/brawler/base` | `units/brawler/base/palettes/dunstan.json` |
| **Elyse** | `mage` (Zone/Conduit) | `portraits/heroes/elyse.png` | `portraits/heroes/faces/elyse/*.png` | `G: units/ranger/wenn_tallow` | `units/ranger/wenn_tallow/palettes/elyse.json` |

#### B. Sunk Ward Bestiary & Act I Bosses

| Foe / Boss | Tier | Portrait (`G` / `K`) | Body Sheet (`K` / `G`) | Palette (`G`) |
| --- | --- | --- | --- | --- |
| **Castellan Watch Halberdier** | Common | `K: portraits/common/warrior_base.png` | `K: units/warrior/1h_buckler` | `units/warrior/1h_buckler/palettes/castellan_watch.json` |
| **Castellan Watch Arbalest** | Common | `K: portraits/common/ranger_base.png` | `K: units/ranger/base` | `units/ranger/base/palettes/castellan_watch.json` |
| **Castellan Bailiff / Sgt. Corvo** | Elite | `K: portraits/common/warrior_base.png` | `K: units/warrior/2h_hammer` | `units/warrior/2h_hammer/palettes/bailiff.json` |
| **Penitent Flagellant** | Common | `K: portraits/common/brawler_base.png` | `K: units/brawler/base` | `units/brawler/base/palettes/flagellant.json` |
| **Tallow Guild Dredger / Flayer** | Common | `G: portraits/npcs/wenn_tallow.png` | `G: units/ranger/wenn_tallow` | `units/ranger/wenn_tallow/palettes/tallow_guild.json` |
| **Inquisitorial Purifier** | Elite | `G: portraits/elites/inquisitorial_purifier.png` | `K: units/priest/base` | `units/priest/base/palettes/purifier.json` |
| **Calcified Husk** | Elite | `G: portraits/elites/calcified_husk.png` | `K: units/brawler/base` | `units/brawler/base/palettes/calcified.json` |
| **Feral Render-Beast** | Common | `G: portraits/common/beast_render_beast.png` | `G: units/beast/render_beast` | *(baked in sheet)* |
| **Ash-Crawler** | Common | `G: portraits/common/crawler_ash_crawler.png` | `G: units/crawler/ash_crawler` | *(baked in sheet)* |
| **Captain Vane** (`0c` Boss) | Boss | `G: portraits/bosses/captain_vane.png` | `K: units/warrior/2h_axe` | `units/warrior/2h_axe/palettes/vane.json` |
| **Foreman Grendel** (`M3` Boss) | Boss | `K: portraits/common/brawler_base.png` | `K: units/warrior/2h_hammer` | `units/warrior/2h_hammer/palettes/grendel.json` |
| **Guild Master Orlov** (`M4` Boss) | Boss | `G: portraits/bosses/guild_master_orlov.png` | `K: units/brawler/base` | `units/brawler/base/palettes/orlov.json` |
| **Proctor Quell** (`M5` Sub-Boss) | Boss | `G: portraits/bosses/proctor_quell.png` | `K: units/warrior/2h_sword` | `units/warrior/2h_sword/palettes/quell.json` |
| **Prelate Nemeth** (`M5` Climax) | Boss | `G: portraits/bosses/inquisitor_prelate_nemeth.png` | `K: units/warrior/2h_sword` | `units/warrior/2h_sword/palettes/nemeth.json` |

#### C. Battle Stage Backdrops (`384×52` logical, `backgrounds/battle/<zone>.png`)

| Encounter | Zone ID | Backdrop Path (`G`) | Stage Tileset (`G` over `E`) |
| --- | --- | --- | --- |
| `enc_pro_0a` (The Alley) | `alley` | `backgrounds/battle/alley.png` | `E: tilesets/common_stage/{stone,wall,wall_face,rubble}` |
| `enc_pro_0b` (The Rooftop) | `rooftops` | `backgrounds/battle/rooftops.png` | `E: tilesets/common_stage/{stone,wall,wall_face,burning_ground}` |
| `enc_pro_0c` (Canal Bridge) | `canal` | `backgrounds/battle/canal.png` | `E: tilesets/common_stage/{bridge,deep_water,wall,wall_face}` |
| `enc_m2_corvo` (Corvo's Gate) | `alley` | `backgrounds/battle/alley.png` | `E: tilesets/common_stage/{stone,wall,wall_face,shallow_water}` |
| `enc_m3_bonemill` (Fourth Kiln) | `foundry` | `backgrounds/battle/foundry.png` | `E: tilesets/common_stage/{stone,lava,wall,wall_face}` |
| `enc_m5_crucible` (Watchpost) | `watchpost` | `backgrounds/battle/watchpost.png` | `E: tilesets/common_stage/{stone,bridge,deep_water,burning_ground,wall}` |

---

## 4. Tracked Art & Placeholder Deltas for `CAST1_ASSETS.md`

1. **Already Generated on Disk (Update `CAST1_ASSETS.md` to mark complete)**:
   - All 6 Founder portraits (`portraits/heroes/{kaelen,vesper,maren,hollis,dunstan,elyse}.png`) and their 8-expression face sheets (`portraits/heroes/faces/<id>/*.png`) are present in `games/emberwake/assets/` (`CAST1_ASSETS.md` Section 1 still listed old Ashlands hero placeholders `bryn_hale`, `kit_vane`, `alda_wren`, `tev_quill`, `dagny_coal`, `sella_marsh`).
   - All 3 Hub NPC portraits (`portraits/npcs/{aline,bram,tess}.png`) and their 4-expression face sheets are present on disk.
   - `portraits/bosses/captain_vane.png` (+ expression faces) is present on disk (`CAST1_ASSETS.md` Section 3 still listed `brask_toller.png`).
   - All 7 battle zone backdrops (`backgrounds/battle/{alley,rooftops,canal,foundry,tallow_vats,undercroft,watchpost}.png`) and 6 story backdrops (`backgrounds/story/{intro_1..4,hearth,meat_market}.png`) are present on disk.
2. **Remaining Optional Art Upgrades (Tracked for `Post-R4`)**:
   - **Common Watch / Bailiff / Flagellant / Grendel Portraits**: Currently share `kits/rpg/assets/portraits/common/{warrior,ranger,brawler}_base.png`. Optional `ART2` pass if unique portraits are desired for `Foreman Grendel` and `Gate-Sergeant Corvo`.
   - **Sunk Ward Custom Tileset (`tilesets/sunk_ward/`)**: Currently uses `engine/assets/tilesets/common` and `common_stage` (which work cleanly out of the box).
