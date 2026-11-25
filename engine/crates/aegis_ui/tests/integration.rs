//! Consolidated integration test harness for `aegis_ui`.
//!
//! Grouping the stateless/isolated test suites under one binary cuts 51 redundant
//! link steps down to 1 while keeping every test identical. Tests that mutate
//! process-global engine state (`crashlog.rs`, `crash_safe.rs`, `live_reload.rs`)
//! intentionally remain standalone top-level test binaries.

#[path = "suite/ask.rs"]
mod ask;
#[path = "suite/audio_focus.rs"]
mod audio_focus;
#[path = "suite/audio_gesture.rs"]
mod audio_gesture;
#[path = "suite/autotile.rs"]
mod autotile;
#[path = "suite/blend_rows.rs"]
mod blend_rows;
#[path = "suite/canvas_policy.rs"]
mod canvas_policy;
#[path = "suite/clip_views.rs"]
mod clip_views;
#[path = "suite/clips.rs"]
mod clips;
#[path = "suite/controls.rs"]
mod controls;
#[path = "suite/draw_pieces.rs"]
mod draw_pieces;
#[path = "suite/feel.rs"]
mod feel;
#[path = "suite/frame_tuner.rs"]
mod frame_tuner;
#[path = "suite/fx.rs"]
mod fx;
#[path = "suite/fx_loops.rs"]
mod fx_loops;
#[path = "suite/ground.rs"]
mod ground;
#[path = "suite/headless.rs"]
mod headless;
#[path = "suite/kit_widgets.rs"]
mod kit_widgets;
#[path = "suite/layout_audit.rs"]
mod layout_audit;
#[path = "suite/layout_editor.rs"]
mod layout_editor;
#[path = "suite/layout_grid.rs"]
mod layout_grid;
#[path = "suite/layout_kit2.rs"]
mod layout_kit2;
#[path = "suite/layout_kit3.rs"]
mod layout_kit3;
#[path = "suite/metrics.rs"]
mod metrics;
#[path = "suite/music_loops.rs"]
mod music_loops;
#[path = "suite/music_variants.rs"]
mod music_variants;
#[path = "suite/pacing.rs"]
mod pacing;
#[path = "suite/painted_bodies.rs"]
mod painted_bodies;
#[path = "suite/palette_ramps.rs"]
mod palette_ramps;
#[path = "suite/panel_fill.rs"]
mod panel_fill;
#[path = "suite/panes.rs"]
mod panes;
#[path = "suite/panes_native.rs"]
mod panes_native;
#[path = "suite/recordings_keys.rs"]
mod recordings_keys;
#[path = "suite/screen_keys.rs"]
mod screen_keys;
#[path = "suite/screen_layout.rs"]
mod screen_layout;
#[path = "suite/settings_store.rs"]
mod settings_store;
#[path = "suite/shared_blocks.rs"]
mod shared_blocks;
#[path = "suite/shot_diff.rs"]
mod shot_diff;
#[path = "suite/sprite_fit.rs"]
mod sprite_fit;
#[path = "suite/stage_pieces.rs"]
mod stage_pieces;
#[path = "suite/start.rs"]
mod start;
#[path = "suite/strings.rs"]
mod strings;
#[path = "suite/text_audit.rs"]
mod text_audit;
#[path = "suite/text_edit.rs"]
mod text_edit;
#[path = "suite/theme.rs"]
mod theme;
#[path = "suite/theme_panes.rs"]
mod theme_panes;
#[path = "suite/ui_bot.rs"]
mod ui_bot;
#[path = "suite/web_display.rs"]
mod web_display;
#[path = "suite/widgets_native.rs"]
mod widgets_native;
#[path = "suite/window_icon.rs"]
mod window_icon;
#[path = "suite/window_settings.rs"]
mod window_settings;
#[path = "suite/world_prims.rs"]
mod world_prims;
