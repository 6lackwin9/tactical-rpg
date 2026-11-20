//! The RPG kit's UI on the engine's `aegis_ui` (KIT-UI1, ENGINE-8 KU1; ENGINE-PURITY-3, ENGINE-5 part 2): what a
//! character-driven RPG draws that a game of another genre would not. Names no game.
//!
//! - `portrait_fx` (PORTRAITFX1-E): base life, overlays, states and reactions over a portrait, by size tier, cached and
//!   budgeted; `portrait_overlays` (ASSETORG4): the overlay layer a game mounts and the player's switch.
//! - `pips` (PIPS1): the pip gauge (AP and the like): pips in a row, planned blocks, ticks.
//! - `vignette` (STORYUI1-E): the vignette player: still cards with fades, a typewriter, cues and a line card's living
//!   portrait.
//! - `shop` (PLAYUI1-P, KU2): the shop as a trade, hosted by a game through `shop::ShopGame`; `shop_view` (UISPLIT1-G):
//!   its view-model and actions.
//! - `portrait_page`: the Kit widgets' Portraits page (`aegis_ui::specimens` page hook) on the code-drawn sample face.

pub mod banner;
pub mod pips;
pub mod portrait_fx;
pub mod portrait_page;
pub mod portrait_overlays;
pub mod shop;
pub mod shop_view;
pub mod vignette;
