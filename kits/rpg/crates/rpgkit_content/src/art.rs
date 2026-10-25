//! The game's art lookup chains (ASSETORG2 section 4) over the engine's asset layers (`aegis_core::assets`): which
//! body, portrait, item icon, skill icon, status icon, tile set, background and UI file an entry shows, as relative
//! paths or body keys tried in order. The RPG chains live in the game until a second game needs them (SPLIT1
//! principle 2). Pure: they read data entries; the screens and the checks resolve them against the layers.

use aegis_core::json::{self, Dict};
use rpgkit_data::ItemDef;
use rpgkit_rules::gear::{Inventory, Slot};

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

/// What a unit's off hand holds, for its look (WAR2H1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OffHand {
    Empty,
    /// A shield that weighs on its bearer (burden above `BUCKLER_MAX_BURDEN`: the kite and tower shields).
    Shield,
    /// A light shield (burden at most `BUCKLER_MAX_BURDEN`: the buckler, a fist-shield that does not slow you down).
    Buckler,
    /// A weapon (dual wield, or the spare under a two-handed main weapon) or a focus (orb, tome).
    Other,
}

/// The burden at or under which a shield is drawn as a buckler (WAR2H1): read from the item's resolved `burden` (its
/// family base plus its modifiers), so a light shield gets the buckler look with no list of item ids.
pub const BUCKLER_MAX_BURDEN: i32 = 0;

/// The gear a body's look follows (WAR2H1): the main hand's item family ("" = no weapon), whether that weapon is
/// two-handed, and what the off hand holds.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Grip {
    pub main_family: String,
    pub two_handed: bool,
    pub off_hand: OffHand,
}

/// The family whose 2H look stands in for a family without its own (a mace or a dagger held with no shield): the
/// greatsword, the warrior sheet's own weapon.
pub const LOOK_FALLBACK_FAMILY: &str = "sword";

impl Grip {
    /// From the equipped items (resolved item rows: `item_type`, `weapon_type` = the item's family, `two_handed`,
    /// `burden`; a shield is `item_type` SHIELD or `off_hand_category` shield, a buckler a shield of burden at most
    /// `BUCKLER_MAX_BURDEN`). None = the slot is empty.
    pub fn of(main: Option<&ItemDef>, off: Option<&ItemDef>) -> Grip {
        let weapon = main.filter(|m| m.item_type == "WEAPON");
        let off_hand = match off {
            None => OffHand::Empty,
            Some(o) if o.item_type == "SHIELD" || o.off_hand_category == "shield" => {
                if o.burden <= BUCKLER_MAX_BURDEN {
                    OffHand::Buckler
                } else {
                    OffHand::Shield
                }
            }
            Some(_) => OffHand::Other,
        };
        Grip { main_family: weapon.map_or(String::new(), |m| m.weapon_type.clone()), two_handed: weapon.is_some_and(|m| m.two_handed), off_hand }
    }

    /// From a unit's equipped gear (its main and off hand; the class default items included, as the build gives them).
    pub fn of_inventory(inv: &Inventory) -> Grip {
        Grip::of(inv.get(Slot::MainHand).map(|e| &e.item), inv.get(Slot::OffHand).map(|e| &e.item))
    }

    /// The look's variant (WAR2H1, user 2026-10-23): `2h_<family>` when the main hand holds a two-handed weapon, or
    /// a weapon with no shield beside it (an empty off hand or one holding a weapon or a focus); `1h_buckler` with a
    /// buckler; None = the body's own sheet (a heavier shield: 1H + kite shield; no weapon and no buckler).
    pub fn look(&self) -> Option<String> {
        let two = Some(format!("2h_{}", self.main_family));
        if self.main_family.is_empty() && self.off_hand != OffHand::Buckler {
            return None;
        }
        match self.off_hand {
            _ if self.two_handed => two,
            OffHand::Shield => None,
            OffHand::Buckler => Some(BUCKLER_LOOK.into()),
            OffHand::Empty | OffHand::Other => two,
        }
    }
}

/// The 1H + buckler look's variant.
pub const BUCKLER_LOOK: &str = "1h_buckler";

/// The bodies to try for a unit wearing body `key` with gear `grip` (WAR2H1; the battle's token will call this, then
/// `Art::first_body`, in WAR2H1's battle slice): a class base (`<class>`) whose grip has a look tries `<class>/<look>` (`1h_buckler` or
/// `2h_<family>`), for a 2H look then the fallback family's `<class>/2h_sword`, then the class base itself (a class
/// without looks shows its sheet; once another class gets a `2h_sword` look, a family without its own look, such as
/// a bow, would show that greatsword: give the fallback per class then); any other body, a named or goblin variant, keeps its own art key: `body_chain(key)`.
/// The looks share the base's palette slots and palette files (DIR4 L3), so the unit's palette applies unchanged.
pub fn gear_body_chain(key: &str, grip: &Grip) -> Vec<String> {
    let (class, variant) = split_body(key);
    let Some(look) = grip.look().filter(|_| variant == "base") else { return body_chain(key) };
    let mut out = vec![format!("{class}/{look}")];
    let fallback = format!("{class}/2h_{LOOK_FALLBACK_FAMILY}");
    if look.starts_with("2h_") && !out.contains(&fallback) {
        out.push(fallback);
    }
    out.push(class.to_string());
    out
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

/// The folders of a map prop's art (PROPS1 3.1): `props/<set>/<art>` over the region sets (the world lineage child
/// first, then `common`); the first holding `v0.png` in any layer gives the prop.
pub fn prop_dirs(lineage_child_first: &[String], art: &str) -> Vec<String> {
    region_sets(lineage_child_first).into_iter().map(|set| format!("props/{set}/{art}")).collect()
}

/// The pixel size a map prop's PNG may have (PROPS1 3.1, 32 px of art per tile): ((min w, max w), (min h, max h)). A
/// standing (`sort`) prop is up to one tile wider than its footprint and up to four tiles taller (roof, chimney,
/// canopy); a ground prop is exactly its footprint.
pub fn prop_png_range(w: i64, h: i64, layer: &str) -> ((i64, i64), (i64, i64)) {
    if layer == "ground" {
        ((32 * w, 32 * w), (32 * h, 32 * h))
    } else {
        ((32 * w, 32 * w + 32), (32 * h, 32 * h + 128))
    }
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
        let mob = d(json!({"id": "mob_brawler", "art": "", "role": "enemy"}));
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
        assert_eq!(portrait_chain("common", "mob_brawler", "brawler/goblin"), ["portraits/common/brawler_goblin.png", "portraits/common/brawler_base.png"]);
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
        assert_eq!(region_sets(&["world_a".into(), "world_b".into()]), ["world_a", "world_b", "common"]);
    }
}
