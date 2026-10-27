//! Engine UI, game-agnostic (docs/specs/SPLIT1.md section 2): sound and music (`audio`), the canvas and painter (pixel art scaled with nearest
//! sampling, the UI drawn smooth at native resolution), the widget kit (`ui`; its text editor and clipboard in
//! `edit`), the art loader, live tuning files, the
//! agent link (`bridge`), keyboard focus (`focus`), playtest recordings (`playtest`: the hub that keeps a run's
//! sessions, trail and notes; `recordings`: the browser that plays them back), the screen stack with its shared
//! context (`screen`: the engine half, plus the game's half through `Cx<G>`) and the window shell and event loop
//! (`shell`), a feet-sorted draw list (`depth`) and the vignette player (`vignette`: still cards with fades, a
//! typewriter, skip and cues, and its full screen) and the effects kit (`fx`, FX1-E: the effect clock with hit-stop,
//! drawn primitives, a live list and the camera shake) and the portrait effects (`portrait_fx`, PORTRAITFX1-E: base
//! life, overlays, states and reactions over a portrait, by size tier, cached and budgeted). A game supplies its screens, its context and the agent replies about its own state.

pub mod art;
pub mod audio;
pub mod bridge;
pub mod canvas;
pub mod depth;
pub mod edit;
pub mod focus;
pub mod fx;
pub mod jpeg;
pub mod notes;
pub mod painter;
pub mod playtest;
pub mod portrait_fx;
pub mod preview;
pub mod recordings;
pub mod screen;
pub mod shell;
pub mod specimens;
pub mod timeline;
pub mod text_audit;
pub mod tuning;
pub mod ui;
pub mod vignette;
pub mod window;

pub use screen::{Context, Cx, Held, Key, Nav, Screen};
