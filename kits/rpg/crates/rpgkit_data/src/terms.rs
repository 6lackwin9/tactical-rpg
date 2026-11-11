//! The term register (TERMS1 TM1-TM2, TM5, TM7; engine-only, no Godot original): the game-system words kit code names
//! only by a neutral id, each with today's text as its default. A `world` term is named per world by an existing world
//! block (the manifest's `purse`, CAMPAIGN1 P1); a `game` term by a row of the game's `terms` table (catalog kind
//! TERMS_KIND). The register is append-only: a term joins it with the task that first needs it. The resolvers are
//! `rpgkit_world::terms::term` (with a world) and `Catalog::term` (inside a battle); nothing else spells a term.

use aegis_core::value::*;

/// The catalog kind of the game's term rows (file `data/catalog/terms.json`; a missing file is an empty table).
pub const TERMS_KIND: &str = "terms";

/// The term ids.
pub const CURRENCY: &str = "currency";
pub const UPKEEP: &str = "upkeep";
pub const INTENT_ASSAULT: &str = "intent_assault";
pub const INTENT_GUARD: &str = "intent_guard";
pub const INTENT_SUPPORT: &str = "intent_support";
pub const INTENT_MANEUVER: &str = "intent_maneuver";
pub const INTENT_CHANNEL: &str = "intent_channel";
/// INTENT2 IR2 (A4.2-K1b): the Field intent's term (a tile target; rule `field_intent`).
pub const INTENT_FIELD: &str = "intent_field";
/// PK1-S2b (PK1 4.3 item 6): the mark a channeled version's name carries in the tells ("Cleave (Channeled)"; rule `channel`).
pub const CHANNELED_MARK: &str = "channeled_mark";
/// PK1-S2c-2 (PK1 4.3, FB-180 (b)): the name of a round that is not a Channel round (the Channel toggle's card and the
/// planner's words: "Standard round").
pub const STANDARD_MARK: &str = "standard_mark";

/// TM1: who names a term.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermScope {
    /// Named per world by an existing world block (the purse); the `terms` table refuses it.
    World,
    /// Named once per game by a `terms` row.
    Game,
}

/// TM1: one register row: the id, today's texts (title name, short form, in-sentence word, tip) and the scope. Every
/// default `short` equals its `name` (TM5: an empty short is the name).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TermDef {
    pub id: &'static str,
    pub name: &'static str,
    pub short: &'static str,
    pub word: &'static str,
    pub tip: &'static str,
    pub scope: TermScope,
}

const fn game(id: &'static str, name: &'static str, word: &'static str) -> TermDef {
    TermDef { id, name, short: name, word, tip: "", scope: TermScope::Game }
}

/// TM1, TM2: the register (spec section 3). The purse terms point at the purse defaults (CURRENCY_NAME, UPKEEP_NAME
/// and their tips).
pub const REGISTER: [TermDef; 10] = [
    TermDef { id: CURRENCY, name: crate::CURRENCY_NAME, short: crate::CURRENCY_NAME, word: "gold", tip: crate::CURRENCY_TIP, scope: TermScope::World },
    TermDef { id: UPKEEP, name: crate::UPKEEP_NAME, short: crate::UPKEEP_NAME, word: "supplies", tip: crate::UPKEEP_TIP, scope: TermScope::World },
    game(INTENT_ASSAULT, "Assault", "assault"),
    game(INTENT_GUARD, "Guard", "guard"),
    game(INTENT_SUPPORT, "Support", "support"),
    game(INTENT_MANEUVER, "Maneuver", "maneuver"),
    game(INTENT_CHANNEL, "Channel", "channel"),
    game(INTENT_FIELD, "Field", "field"),
    game(CHANNELED_MARK, "Channeled", "channeled"),
    game(STANDARD_MARK, "Standard", "standard"),
];

/// The register row of `id`, if any.
pub fn def(id: &str) -> Option<&'static TermDef> {
    REGISTER.iter().find(|t| t.id == id)
}

/// The ids of one scope, in register order.
pub fn ids(scope: TermScope) -> Vec<&'static str> {
    REGISTER.iter().filter(|t| t.scope == scope).map(|t| t.id).collect()
}

/// The default title name of `id` (TM7: the id itself for an unknown id, a debug assertion in debug builds).
pub fn default_name(id: &str) -> &str {
    match def(id) {
        Some(t) => t.name,
        None => {
            debug_assert!(false, "unknown term id '{id}'");
            id
        }
    }
}

/// TM3: a resolved term: the title name, the short form (HUD, labels, Results lines), the word inside a sentence and
/// the tip.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Term {
    pub name: String,
    pub short: String,
    pub word: String,
    pub tip: String,
}

impl Term {
    /// TM4, TM5: term `id` with the overrides `name`, `short`, `word` and `tip` ("" = not set): the name else the
    /// default name; `short` empty = the name; the word when set, else the default word while the name is the default
    /// name, else the short form as written ("gold" stays lower case, "Coin" stays "Coin"); the tip else the default
    /// tip. TM7: an unknown id is its own text (a debug assertion in debug builds).
    pub fn resolve(id: &str, name: &str, short: &str, word: &str, tip: &str) -> Term {
        let Some(t) = def(id) else {
            debug_assert!(false, "unknown term id '{id}'");
            return Term { name: id.into(), short: id.into(), word: id.into(), tip: String::new() };
        };
        let name = if name.is_empty() { t.name } else { name };
        let short = if short.is_empty() { name } else { short };
        let word = if !word.is_empty() {
            word.to_string()
        } else if name == t.name {
            t.word.to_string()
        } else {
            short.to_string()
        };
        Term { name: name.into(), short: short.into(), word, tip: if tip.is_empty() { t.tip } else { tip }.into() }
    }

    /// TM2: the default texts of `id`.
    pub fn default_of(id: &str) -> Term {
        Self::resolve(id, "", "", "", "")
    }
}

/// One row of the game's `terms` table (spec 5.1): the id and its overrides ("" = the default).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TermRow {
    pub id: String,
    pub name: String,
    pub short: String,
    pub word: String,
    pub tip: String,
}

impl TermRow {
    pub fn from_obj(o: &Obj) -> Self {
        Self { id: string(o, "id", ""), name: string(o, "name", ""), short: string(o, "short", ""), word: string(o, "word", ""), tip: string(o, "tip", "") }
    }

    /// The row's term (TM5 over its overrides).
    pub fn term(&self) -> Term {
        Term::resolve(&self.id, &self.name, &self.short, &self.word, &self.tip)
    }
}
