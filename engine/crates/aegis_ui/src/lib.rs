//! Engine UI, game-agnostic (docs/specs/SPLIT1.md section 2): sound and music (`audio`), the canvas and painter (pixel art scaled with nearest
//! sampling, the UI drawn smooth at native resolution), the widget kit (`ui`), the art loader, live tuning files, the
//! agent link (`bridge`), keyboard focus (`focus`), the screen stack with its shared context (`screen`: the engine half, plus the game's half
//! through `Cx<G>`) and the window shell and event loop (`shell`). A game supplies its screens, its context and the
//! agent replies about its own state.

pub mod art;
pub mod audio;
pub mod bridge;
pub mod canvas;
pub mod focus;
pub mod painter;
pub mod screen;
pub mod shell;
pub mod timeline;
pub mod tuning;
pub mod ui;
pub mod window;

pub use screen::{Context, Cx, Key, Nav, Screen};
