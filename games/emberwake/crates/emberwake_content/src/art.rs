//! The game's art lookup chains (ASSETORG2 section 4) over the engine's asset layers (`aegis_core::assets`): which
//! body, portrait, item icon, skill icon, status icon, tile set, background and UI file an entry shows, as relative
//! paths or body keys tried in order. The RPG chains live in the game until a second game needs them (SPLIT1
//! principle 2). Pure: they read data entries; the screens and the checks resolve them against the layers.

use aegis_core::json::{self, Dict};

/// A body key's class and variant (`<class>` = `<class>/base`).
pub fn split_body(key: &str) -> (&str, &str) {
    key.split_once('/').unwrap_or((key, "base"))
}

/// A unit's body key (4.1): the character's `art`, else its base unit's, else its class's, else the class id.
/// `character` is empty for a catalog unit; `class` is the unit's class entry (empty for a prop like the dummy).
pub fn body_of(character: &Dict, unit: &Dict, class: &Dict) -> String {
    for e in [character, unit, class] {
        let art = json::s(e, "art");
        if !art.is_empty() {
            return art;
        }
    }
    json::s(unit, "class_id")
}

/// The bodies to try for `key`: the key, then its class's base (a variant missing in every layer falls back).
pub fn body_chain(key: &str) -> Vec<String> {
    let (class, variant) = split_body(key);
    if variant == "base" {
        vec![class.to_string()]
    } else {
        vec![key.to_string(), class.to_string()]
    }
}

/// The palette a unit wears: the character's `palette`, else its base unit's ("" = none).
pub fn palette_of(character: &Dict, unit: &Dict) -> String {
    let own = json::s(character, "palette");
    if own.is_empty() {
        json::s(unit, "palette")
    } else {
        own
    }
}

/// The portrait tier (4.2): `portrait_tier` when set, else auto. A character: role party = hero, npc = npc, enemy =
/// by its base unit's tier (boss, elite, else common); a catalog unit: boss, elite, else common.
pub fn portrait_tier(character: &Dict, unit: &Dict) -> String {
    for e in [character, unit] {
        let t = json::s(e, "portrait_tier");
        if !t.is_empty() {
            return t;
        }
    }
    let by_tier = || match json::s(unit, "tier").as_str() {
        "boss" => "boss",
        "elite" => "elite",
        _ => "common",
    };
    if character.is_empty() {
        return by_tier().into();
    }
    match json::s(character, "role").as_str() {
        "party" => "hero".into(),
        "npc" => "npc".into(),
        _ => by_tier().into(),
    }
}

/// The folder of a unique portrait tier (hero -> heroes, ...); None for common.
pub fn tier_folder(tier: &str) -> Option<&'static str> {
    match tier {
        "hero" => Some("heroes"),
        "npc" => Some("npcs"),
        "elite" => Some("elites"),
        "boss" => Some("bosses"),
        _ => None,
    }
}

/// The common portrait of body `key`: portraits/common/<class>_<variant>.png.
pub fn common_portrait(key: &str) -> String {
    let (class, variant) = split_body(key);
    format!("portraits/common/{class}_{variant}.png")
}

/// The portraits to try (4.2): a unique tier's own file (`id` = the character id, else the unit id), then the common
/// portrait of the body, then of its class's base.
pub fn portrait_chain(tier: &str, id: &str, body: &str) -> Vec<String> {
    let mut out = Vec::new();
    if let Some(folder) = tier_folder(tier) {
        out.push(format!("portraits/{folder}/{id}.png"));
    }
    out.push(common_portrait(body));
    let base = format!("portraits/common/{}_base.png", split_body(body).0);
    if !out.contains(&base) {
        out.push(base);
    }
    out
}

/// The item icons to try (4.3): `items/<folder>/<icon_id>.png`, the family's base, then the type's and any base.
/// `family` is the item's family entry (its `art` names the folder, else the family id).
pub fn item_icon_chain(item: &Dict, family: &Dict) -> Vec<String> {
    let fam = json::s(item, "family");
    let folder = json::s_or(family, "art", &fam);
    let folder = if folder.is_empty() { fam } else { folder };
    let mut out = Vec::new();
    let icon = json::s(item, "icon_id");
    if !icon.is_empty() && !folder.is_empty() {
        out.push(format!("items/{folder}/{icon}.png"));
    }
    if !folder.is_empty() {
        out.push(format!("items/{folder}/base.png"));
    }
    let kind = json::s(item, "item_type").to_lowercase();
    if !kind.is_empty() {
        out.push(format!("items/any/{kind}.png"));
    }
    out.push("items/any/base.png".into());
    out
}

/// A skill's icons (4.4): its family's, then its kind's.
pub fn skill_icon_chain(family: &str, kind: &str) -> Vec<String> {
    vec![format!("skills/{family}.png"), format!("skills/kinds/{}.png", kind.to_lowercase())]
}

pub fn status_icon(status: &str) -> String {
    format!("statuses/{status}.png")
}

/// The region map's tile sets (4.5): the world lineage child first, then `common`.
pub fn region_sets(lineage_child_first: &[String]) -> Vec<String> {
    let mut out: Vec<String> = lineage_child_first.to_vec();
    out.push("common".into());
    out
}

/// The region map's tile of a map character: `.` floor, `#` wall, `~` shallow_water (others: the screen's colour).
pub fn region_tile(ch: char) -> Option<&'static str> {
    match ch {
        '.' => Some("floor"),
        '#' => Some("wall"),
        '~' => Some("shallow_water"),
        _ => None,
    }
}

/// Battle tile sets: `common` until a map names a set (ASSETORG2 open question 5).
pub const BATTLE_SETS: [&str; 1] = ["common"];

pub fn world_map_background(world: &str) -> String {
    format!("backgrounds/world_map/{world}.png")
}

pub fn title_background(world: &str) -> String {
    format!("backgrounds/title/{world}.png")
}

pub const MENU_BACKGROUND: &str = "backgrounds/menu/main_menu.png";

pub fn location_background(location: &str) -> String {
    format!("backgrounds/location/{location}.png")
}

pub fn pin(kind: &str) -> String {
    format!("ui/pins/{kind}.png")
}

/// An intent chip; `idle` shows `rest`.
pub fn intent(kind: &str) -> String {
    format!("ui/intents/{}.png", if kind == "idle" { "rest" } else { kind })
}

pub fn badge(id: &str) -> String {
    format!("ui/badges/{id}.png")
}

pub fn chip(id: &str) -> String {
    format!("ui/chips/{id}.png")
}

pub fn marker(id: &str) -> String {
    format!("map/markers/{id}.png")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn d(v: serde_json::Value) -> Dict {
        v.as_object().cloned().unwrap()
    }

    #[test]
    fn bodies_and_portraits_follow_the_chains() {
        let class = d(json!({"id": "brawler", "art": "brawler"}));
        let unit = d(json!({"id": "goblin_brawler", "class_id": "brawler", "art": "brawler/goblin", "tier": "minion"}));
        let skarn = d(json!({"id": "skarn_ironjaw", "art": "brawler/skarn_ironjaw", "role": "enemy", "portrait_tier": "elite"}));
        let mob = d(json!({"id": "ash_brawler", "art": "", "role": "enemy"}));
        assert_eq!(body_of(&skarn, &unit, &class), "brawler/skarn_ironjaw");
        assert_eq!(body_of(&mob, &unit, &class), "brawler/goblin", "a character without art wears its base unit's body");
        assert_eq!(body_of(&Dict::new(), &d(json!({"class_id": "mage"})), &Dict::new()), "mage");
        assert_eq!(body_chain("brawler/goblin"), ["brawler/goblin", "brawler"]);
        assert_eq!(body_chain("brawler"), ["brawler"]);
        assert_eq!(portrait_tier(&skarn, &unit), "elite");
        assert_eq!(portrait_tier(&mob, &unit), "common");
        assert_eq!(portrait_tier(&d(json!({"role": "party"})), &unit), "hero");
        assert_eq!(portrait_tier(&d(json!({"role": "npc"})), &unit), "npc");
        assert_eq!(portrait_tier(&Dict::new(), &d(json!({"tier": "boss"}))), "boss");
        assert_eq!(portrait_chain("elite", "skarn_ironjaw", "brawler/skarn_ironjaw"), ["portraits/elites/skarn_ironjaw.png", "portraits/common/brawler_skarn_ironjaw.png", "portraits/common/brawler_base.png"]);
        assert_eq!(portrait_chain("common", "ash_brawler", "brawler/goblin"), ["portraits/common/brawler_goblin.png", "portraits/common/brawler_base.png"]);
        assert_eq!(portrait_chain("common", "x", "mage"), ["portraits/common/mage_base.png"]);
    }

    #[test]
    fn icons_fall_back_to_the_family_kind_and_any() {
        let fam = d(json!({"id": "sword", "art": ""}));
        let claymore = d(json!({"id": "wpn_claymore", "family": "sword", "icon_id": "greatsword", "item_type": "WEAPON"}));
        assert_eq!(item_icon_chain(&claymore, &fam), ["items/sword/greatsword.png", "items/sword/base.png", "items/any/weapon.png", "items/any/base.png"]);
        let oddity = d(json!({"id": "x", "family": "mace", "item_type": "WEAPON"}));
        assert_eq!(item_icon_chain(&oddity, &d(json!({"art": "hammer"})))[0], "items/hammer/base.png", "the family's art names the folder");
        assert_eq!(skill_icon_chain("shield_bash", "ATTACK"), ["skills/shield_bash.png", "skills/kinds/attack.png"]);
        assert_eq!(intent("idle"), "ui/intents/rest.png");
        assert_eq!(region_sets(&["tidewrack".into(), "ashlands".into()]), ["tidewrack", "ashlands", "common"]);
    }
}
