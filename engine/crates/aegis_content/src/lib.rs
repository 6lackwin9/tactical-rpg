//! Engine content layer, game-agnostic (docs/specs/SPLIT1.md section 2): the data file format Godot reads and writes
//! (`file`), the schema-driven normalize, storage form and checks (`schema`), the content store and the world graph
//! (`world`: inheritance, merged entries, overrides, qualified ids) and the writer's upsert / remove / commit with the
//! game's hooks (`store`). The game supplies its schema, its kinds and the rules of its own kinds (for Emberwake:
//! skill ranks, item families and modifiers, game-data files).

pub mod file;
pub mod schema;
pub mod store;
pub mod world;

pub use world::{split_qid, Catalogs, World};
