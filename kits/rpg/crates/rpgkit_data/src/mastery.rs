//! Weapon mastery and techniques (PK2-S4, `docs/sessions/PK2_GRAFTS.md` section 5; engine-only, no Godot original).
//! Mastery is a member's own earned XP per main-hand weapon family (a bar toward levels 1-3), separate from the class
//! proficiency, which gates and scales it: only a main-hand weapon family the member has at proficiency 2 or more gains
//! mastery, at x1 (proficiency 2) or x1.5 (proficiency 3). It fills through use (each battle the member attacked with the
//! family), books (item type `MASTERY_BOOK`) and drills (the recipe effect `drill`), from the act the game's manifest
//! names (`durability.from_act`; no key = never). Each weapon family row may list up to 3 `techniques` (a skill each,
//! with an optional `grip` tag): mastery 1, 2 and 3 open technique 1, 2 and 3 while a weapon of the family is wielded in
//! the main hand, usable only in standard rounds (not Channel). A weapon row's own `technique` is its masterwork
//! technique (open while a masterwork instance of it is wielded; weapon instances, PR5-13a). A family row's `keystone`
//! is its standard-path keystone skill: open at mastery 3 once the member's forge rite is done, taking the deck's one
//! keystone slot unless a keystone graft holds it. The family rows ship without techniques until the asset lane fills
//! them, so no member gains or uses any.

use aegis_core::value::*;

/// The item type of a mastery book (a stash item read out of battle: `mastery_family` gains `mastery_xp`).
pub const MASTERY_BOOK: &str = "MASTERY_BOOK";
/// The book's family and XP keys.
pub const BOOK_FAMILY_KEY: &str = "mastery_family";
pub const BOOK_XP_KEY: &str = "mastery_xp";
/// PK2-FOLLOWUP (PR5-36): a book's XP when its row names none (the schema's default of `mastery_xp`).
pub const BOOK_XP_DEFAULT: i32 = 150;
/// The family row's technique list key, its keystone key, and the weapon row's own (masterwork) technique key.
pub const TECHNIQUES_KEY: &str = "techniques";
pub const KEYSTONE_KEY: &str = "keystone";
pub const TECHNIQUE_KEY: &str = "technique";
/// The resolved item row's keys that carry its family's techniques and keystone (written by the loader; the writer
/// leaves them out of the stored form).
pub const ITEM_FAMILY_TECHNIQUES: &str = "family_techniques";
pub const ITEM_FAMILY_KEYSTONE: &str = "family_keystone";
/// The grip tags a technique may carry: "" (either grip), `1h` (a one-handed weapon only), `2h` (a two-handed only).
pub const GRIPS: [&str; 3] = ["", GRIP_1H, GRIP_2H];
pub const GRIP_1H: &str = "1h";
pub const GRIP_2H: &str = "2h";
/// The reasons a technique is greyed when the wielded grip does not fit.
pub const NEEDS_1H: &str = "Needs a one-handed grip";
pub const NEEDS_2H: &str = "Needs a two-handed grip";
/// The top mastery level (also the most techniques a family row lists).
pub const MASTERY_MAX: i32 = 3;
/// The total mastery XP each level needs (level 1, 2, 3).
pub const MASTERY_LEVEL_XP: [i64; 3] = [100, 300, 600];
/// The XP one battle's use gives (each battle the member attacked with the family), before the proficiency rate.
pub const USE_XP: i64 = 10;
/// The proficiency a family needs to gain mastery, and the one with the accelerated rate.
pub const MASTERY_PROF_MIN: i32 = 2;
pub const MASTERY_PROF_FAST: i32 = 3;
/// The rate at `MASTERY_PROF_MIN` and at `MASTERY_PROF_FAST`, in percent.
pub const RATE_PCT: (i64, i64) = (100, 150);
/// A book's XP range (`mastery_xp`) and a drill's (`xp`).
pub const MASTERY_XP_RANGE: (i64, i64) = (1, 999);
/// The most masterable families (main-hand weapon families at proficiency 2 or more) a class has at baseline; the class
/// check warns past it.
pub const MASTERABLE_MAX: usize = 3;
/// The hands of a main-hand weapon family (an item family row's `hands`).
pub const MAIN_HANDS: [&str; 2] = ["main_1h", "main_2h"];

/// One technique: its skill and grip tag (`GRIPS`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Technique {
    pub skill: String,
    pub grip: String,
}

impl Technique {
    pub fn from_obj(o: &Obj) -> Self {
        Self { skill: string(o, "skill", ""), grip: string(o, "grip", "") }
    }

    /// The technique list of a row's `key` (objects with a skill; anything else is skipped).
    pub fn list(o: &Obj, key: &str) -> Vec<Self> {
        array(o, key).iter().filter_map(serde_json::Value::as_object).map(Self::from_obj).filter(|t| !t.skill.is_empty()).collect()
    }

    /// The row's single technique at `key` (None: no key, or no skill).
    pub fn one(o: &Obj, key: &str) -> Option<Self> {
        obj(o, key).map(Self::from_obj).filter(|t| !t.skill.is_empty())
    }

    /// Why the technique may not be used with a weapon of grip `two_handed` (None: the grip fits).
    pub fn grip_problem(&self, two_handed: bool) -> Option<&'static str> {
        match self.grip.as_str() {
            GRIP_1H if two_handed => Some(NEEDS_1H),
            GRIP_2H if !two_handed => Some(NEEDS_2H),
            _ => None,
        }
    }
}

/// A mastery book's family and XP (`ItemDef::book`).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MasteryBook {
    pub family: String,
    pub xp: i64,
}

impl MasteryBook {
    pub fn from_obj(o: &Obj) -> Self {
        Self { family: string(o, BOOK_FAMILY_KEY, ""), xp: i64::from(int(o, BOOK_XP_KEY, BOOK_XP_DEFAULT)) }
    }
}

/// The mastery level of `xp` total XP (0-3).
pub fn level_of(xp: i64) -> i32 {
    MASTERY_LEVEL_XP.iter().filter(|need| xp >= **need).count() as i32
}

/// The XP gain rate at proficiency `level`, in percent (0: the family gains no mastery).
pub fn rate_pct(level: i32) -> i64 {
    if level >= MASTERY_PROF_FAST {
        RATE_PCT.1
    } else if level >= MASTERY_PROF_MIN {
        RATE_PCT.0
    } else {
        0
    }
}

/// `base` XP at proficiency `level`: `base x rate`, rounded half away from zero (0 below proficiency 2).
pub fn scaled(base: i64, level: i32) -> i64 {
    let raw = (base.max(0) * rate_pct(level)) as f64 / 100.0;
    raw.round() as i64
}

/// Whether an item family row is a main-hand weapon family (item type WEAPON, a main hand among its `hands`): the only
/// families that gain mastery (never armour, shields or off-hand foci).
pub fn main_hand_family(row: &Obj) -> bool {
    string(row, "item_type", "WEAPON") == "WEAPON" && strings(row, "hands").iter().any(|h| MAIN_HANDS.contains(&h.as_str()))
}
