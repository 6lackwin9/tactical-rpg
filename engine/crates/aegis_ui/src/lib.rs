//! Engine UI, game-agnostic (docs/specs/SPLIT1.md section 2): sound and music (`audio`), the canvas and painter (pixel art scaled with nearest
//! sampling, the UI drawn smooth at native resolution), the widget kit (`ui`; its text editor and clipboard in
//! `edit`), the art loader, live tuning files, the
//! agent link (`bridge`), keyboard focus (`focus`), playtest recordings (`playtest`: the hub that keeps a run's
//! sessions, trail and notes; `recordings`: the browser that plays them back), the screen stack with its shared
//! context (`screen`: the engine half, plus the game's half through `Cx<G>`) and the window shell and event loop
//! (`shell`), a feet-sorted draw list (`depth`) and the effects kit (`fx`, FX1-E: the effect clock with hit-stop,
//! drawn primitives, a live list and the camera shake) and the
//! runtime theme (`theme`, LOOK1-E1: colour roles, faces, layout values and nine-slice frames over the asset layers;
//! the default is the look before it) and the menu panes (`panes`, LOOK1-ME: backdrop, menu column, tab rail, slot
//! card and the themed ask, standalone pieces on the theme) and the tile masks (`autotile`, LOOK1-E2: corner masks for
//! transition pieces, a block's open edges) and the ground composer (`ground`, REGIONGROUND1-E1: a cell grid to one
//! opaque image with generated edges, cached and blitted) and two shots compared with a tolerance (`shot_diff`,
//! LAYOUT720-E2) and what a native screen does with its space, on any window (`layout_audit`, LAYOUT720-E5, and its
//! subject `layout_specimen`) and the layout editor (`layout_editor`, PR5-48: markers and polylines placed on a background
//! over a padded viewport, dragged, nudged, sized and undone; it returns the values and never writes). `Rect` lives in
//! `aegis_layout` and `canvas` re-exports it. A game supplies its screens, its context and the agent replies about its
//! own state.

pub mod art;
pub mod audio;
pub mod audio_out;
pub mod audiofeel;
pub mod autotile;
pub mod controls;
#[cfg(not(target_arch = "wasm32"))]
pub mod crashlog;
pub mod bridge;
pub mod canvas;
pub mod depth;
pub mod edit;
pub mod feel;
pub mod focus;
pub mod frame_tuner;
pub mod float_panel;
pub mod fx;
pub mod ground;
pub mod icon;
pub mod jpeg;
pub mod layout_audit;
pub mod layout_editor;
pub mod layout_gallery;
pub mod layout_specimen;
pub mod live;
pub mod map_fx;
pub mod notes;
pub mod pacing;
pub mod painter;
pub mod perf;
pub mod panes;
pub mod playtest;
pub mod preview;
pub mod reach;
pub mod recordings;
pub mod screen;
pub mod screen_layout;
pub mod shell;
pub mod shot_diff;
pub mod specimens;
pub mod start;
pub mod strings;
pub mod text_audit;
pub mod theme;
pub mod tilemap;
pub mod timeline;
pub mod tuning;
pub mod ui;
pub mod walkfeel;
pub mod switchfeel;
pub mod window;
pub mod world_camera;

pub use screen::{Context, Cx, Held, Key, Nav, PointerPhase, Screen, UiCue, UiSound, UI_SOUND_KINDS};
