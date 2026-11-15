# art_src/ARCHIVE_INDEX: Active Sources vs. Retired Probes & Negative Examples

Maintained by `/art-ui-session` (`CTX-CLEANUP1-ARTUI`, 2026-11-15). Retired probes and negative examples stay at their historical filenames below because tools (`tween.py`, `render_map.py`, `cut_props.py`, `mock_ui`) or experiment records reference paths directly — do not move or delete files without grepping `tools/` and `art_src/`.

## 1. Active Production Pipelines (`art_src/`)

| Path | Active Role | Primary Script(s) |
| --- | --- | --- |
| `art_src/prompts/` | Prompt specs (`sunk_ward_props.md`, `sunk_ward_walls.md`, `sunk_ward_icons.md`), `ASSET_PLAN.md`, cutters | `cut_props.py`, `cut_ai_sheets.py`, `queue_sunk_ward.py` |
| `art_src/region_maps/*.json` | Act 1 Sunk Ward layout JSONs (`sluice_undercroft`, `drowned_runoff_grate`, streets) & `render_map.py` | `build_layouts.py`, `render_map.py` |
| `art_src/sunk_ward/sheets/` | Picked queue sheets (`families_1`, `gear_1`, `hud_chips_2`, `keepsakes_1`, `services_hud_1`, `props_sunk_*`) | `cut_ai_sheets.py`, `cut_props.py` |
| `art_src/sunk_ward/tiles/` | Ground & boundary masters (`ground_*.png`, `boundary_*.png`, `walls_interior_*.png`) | `tools/tiles/{cut,seam,flatten,groundcheck}.py` |
| `art_src/sunk_ward/props_cut/` | 81 cut prop PNGs (`v0.png`) staged for verification & `render_map.py` preview | `cut_props.py`, `render_map.py` |
| `art_src/music/` | Flow Music import JSON records (`menu_theme`, `sunk_ward_town`, `sunk_ward_combat*`) & `prompts/` | `tools/mock_ui/music.py` |
| `art_src/vn_stage/` | Signed-off VN backdrops, full-body cast, Kaelen video loop/action sheets, `prompts/` | `tools/mock_ui/anim_lab/tween.py` |

## 2. Retired Probes, Superseded Takes & Negative Examples (Do Not Delete While Referenced)

| Path Pattern (`art_src/`) | Category | Superseded By / Reason Kept |
| --- | --- | --- |
| `ground_probe_{dirt_road,flagstone,scrub}_gen*` | Tile probe (`REGIONGROUND1`) | Early generation parameter probes (`gen1793628367`..`383`), superseded by `art_src/sunk_ward/tiles/` |
| `title_backdrop_mist_v2_gen1794152884_1.*` | Menu backdrop probe | Exported & cropped as `art_src/vn_stage/title_sunk_ward_mist_v2.jpg` |
| `props_{buildings,items,kit}_gen*` / `tiles_*_gen*` | Ashlands / Kit early queue masters | Cut into Ashlands/Kit `assets/`; `props_buildings_gen1793556098_1.png` still used as front-on view reference |
| `vn_stage/exp1_{optionA_3views,optionB_exploded,optionC_3poses}_sheet.jpg` | Experiment 1 comparisons | Historical comparison records documented in `ART_UI_EXPERIMENTS_LOG.md` `3.1` |
| `vn_stage/exp1_rejected_black_hair_drift.jpg` | Negative example | Black-hair drift example kept to validate hair-color lock rule |
| `vn_stage/kaelen_drawn_kf_halfdraw.{png,json}` | Superseded breakdown | Copied guard stance; replaced by `kaelen_drawn_kf_halfdraw_r2.{png,json}` |
| `vn_stage/kaelen_draw_sheet_lift.{png,json}` | Superseded breakdown | Copied frame 2; replaced by `kaelen_draw_sheet_lift_r2.{png,json}` |
| `vn_stage/kaelen_walk_sheet_8frame.{png,json}` | Superseded sheet | No leg swap (shuffle); replaced by `kaelen_walk_sheet_8frame_r2.{png,json}` |
| `vn_stage/kaelen_sneak_sheet_6frame.{png,json}` | Superseded sheet | Six repeats; partially replaced by `kaelen_sneak_sheet_6frame_r2.*` then Veo `kaelen_sneak_loop_sheet.png` |
| `vn_stage/kaelen_poke_sheet_6frame.{png,json}` | Superseded sheet | Lunge overshoot; superseded by `kaelen_poke_thrust_r2.*` and Veo `kaelen_poke_action_sheet.png` |
| `sunk_ward/sheets/props_sunk_{undercroft,drains,works,streets,interiors}_*_2.png` | Alternate / angled takes | Take 1 or S12–S16 front-on sheets superseded angled/halo cells; retained per `cut_props.py` `PICKS` |
| `sunk_ward/sheets/props_sunk_side_q1794690874_*.png` | Partial S15 sheet | 7 floor-furniture cells rotated side-on cleanly; 9 wall-mounted cells came back front-on (skipped by `cut_props.py`) |
