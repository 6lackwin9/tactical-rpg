//! Engine content layer, game-agnostic (docs/specs/SPLIT1.md section 2): the data file format Godot reads and writes
//! (`file`), the schema-driven normalize, storage form and checks (`schema`), the content store and the world graph
//! (`world`: inheritance, merged entries, overrides, qualified ids), the writer's upsert / remove / commit with the
//! game's hooks (`store`), the game manifest (`manifest`, game.json) and the content fingerprint of a data folder
//! (`fingerprint`, which data a playtest recording was played on). The game supplies its schema, its kinds and
//! the rules of its own kinds (for example skill ranks, item families and modifiers, game-data files).

pub mod file;
pub mod fingerprint;
pub mod manifest;
pub mod schema;
pub mod store;
pub mod world;

pub use world::{split_qid, Catalogs, World};
