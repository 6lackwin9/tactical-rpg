//! Emberwake's content rules on the engine content layer (`aegis_content`): the data loader (`source`: skill ranks
//! and SkillText, item families and modifiers, the game-data lists and the XP curve) and the writer's hooks and
//! storage rules (`store`: skills, items, game-data files, the folder check).

pub mod art;
pub mod check;
pub mod source;
pub mod store;
