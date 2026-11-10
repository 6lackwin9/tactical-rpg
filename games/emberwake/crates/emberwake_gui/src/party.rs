//! The Character popup (party_model.gd with stats_model, level_up_model, loadout_model, gear_model): one member at a
//! time (Q / E cycle), tabs Stats, Level up, Skills and Gear. Drafts live here until Confirm writes them to the game
//! state (level-up points are permanent and asked about first); switching member or tab drops a draft after asking.

use aegis_ui::pacing::{FrameNeed, SLOW_LIFE};
use aegis_ui::theme::th;
use aegis_ui::canvas::{rgb, rgba, Rect};
use crate::screens::{show_errors, Cx, GameCx, Key, Nav, Screen};
use aegis_ui::ui::{tip, Gfx, Ui, Widget, BODY, H, SMALL, W};
use aegis_ui::widget_plumbing;
use aegis_core::json::{self, s_or};
use emberwake_data::{Catalog, Deck, DerivedStatDef, NO_ITEM, STATS};
use emberwake_rules::damage;
use emberwake_rules::derived::{self, Breakdown, Cap, InputKind};
use emberwake_rules::gear::TIER_SOURCE;
use emberwake_rules::unit::GEAR_EFFECT_SOURCE;
use emberwake_rules::Unit;
use emberwake_world::state::{off_hand_ok, stack_limit, UnitState, LOADOUT_SLOTS, POUCH_MAX};
use emberwake_world::{progression, Data};
use indexmap::IndexMap;
use serde_json::{json, Value};

/// PORTRAITFX1-R (6.2 row 2): the member's portrait side on the Character screen, at the frame's top-left (left of
/// the member picker, above every tab's content).
pub const PORTRAIT: i32 = 48;

const TABS: [(&str, &str); 4] = [("stats", "Stats"), ("level_up", "Level up"), ("skills", "Skills"), ("gear", "Gear")];

/// A Skills tab draft: class slots, extra slots, the chosen rank per slotted family.
type SkillDraft = (Vec<String>, Vec<String>, IndexMap<String, i64>);

/// SKILLUI1 R2 on the draft: no entry, the highest selectable rank; an entry `c`, the highest selectable rank <= `c`,
/// else the lowest; 0 when nothing is selectable.
fn draft_rank(selectable: &[i32], chosen: Option<i32>) -> i32 {
    match chosen {
        None => selectable.last().copied().unwrap_or(0),
        Some(c) => selectable.iter().rev().find(|&&r| r <= c).or(selectable.first()).copied().unwrap_or(0),
    }
}

#[derive(Debug, Clone)]
pub enum PAct {
    Member(i64),
    Tab(&'static str),
    Close,
    ToggleActive,
    Add(&'static str),
    Remove(&'static str),
    Auto,
    Revert,
    ConfirmLevel,
    Slot(String),
    Unslot(bool, usize),
    /// SKILLUI1: step a filled slot (class?, index) to the previous (-1) or next (+1) selectable rank.
    RankStep(bool, usize, i32),
    ClearAll,
    UseDefault,
    ResetSkills,
    ConfirmSkills,
    Unequip(String),
    Equip(String),
    PouchAdd(String),
    PouchRemove(usize),
    /// INTENT2-U (`pouch_stacks`): set pouch slot item's packed count (`UnitState::set_packed`, `1..min(5, max_stack)`).
    PouchPack(String, i64),
    ResetGear,
    ConfirmGear,
    /// RULESET1 9 D6 (deck rule): slot an owned (kind, id) in the first free slot of its bucket.
    DeckSlot(&'static str, String),
    /// Take slot (kind, index) out.
    DeckUnslot(&'static str, usize),
    /// The right column: "owned" or "order".
    DeckView(&'static str),
    /// Move entry (intent, index) one place earlier in that intent's order.
    OrderUp(String, usize),
    /// Every intent back to the auto order.
    OrderAuto,
    /// INPUT1 FL3: opens (or closes) the follow-up list of an opener family under its order entry.
    FollowOpen(String),
    /// Adds family `.1` at the end of opener `.0`'s follow-up list.
    FollowAdd(String, String),
    /// Takes place `.1` out of opener `.0`'s list (a right click).
    FollowRemove(String, usize),
    /// "Use the <Intent> order": empties opener `.0`'s list.
    FollowClear(String),
}

pub struct Character {
    member: String,
    tab: &'static str,
    draft: IndexMap<String, i64>,
    class_slots: Vec<String>,
    extra_slots: Vec<String>,
    /// SKILLUI1: the draft's chosen rank per slotted family (family -> rank).
    ranks: IndexMap<String, i64>,
    using_default: bool,
    /// The mouse at the last draw (a right click unslots the slot under it).
    mouse: (i32, i32),
    skills_loaded: bool,
    gear: Option<(UnitState, IndexMap<String, i64>)>,
    /// INTENT2-U: the gear draft's world plays `pouch_stacks` (the pouch holds stacks with packed counts; set with the
    /// draft).
    gear_stacks: bool,
    /// PR5-26 review F3: the played world the gear draft reads its class defaults in (set with the draft).
    gear_world: String,
    pending_switch: Option<(Option<String>, Option<&'static str>)>,
    /// First family row shown on the Skills tab (the wheel scrolls).
    scroll: usize,
    /// First stash row shown on the Gear tab (the wheel scrolls).
    gear_scroll: usize,
    detail: String,
    /// RULESET1 9 D6: under the deck rule, the member's deck terms (Some while the Skills tab is loaded under it) and
    /// the deck draft (empty = auto).
    terms: Option<DeckTerms>,
    deck: Deck,
    deck_view: &'static str,
    /// INPUT1 FL3: the opener family whose follow-up list is open in the order view.
    follow_open: Option<String>,
    /// PORTRAITFX1-R: the shown member's portrait (surface `roster`); cycling members drops the old one's.
    faces: crate::portraits::Faces,
    w: Vec<Widget<PAct>>,
}

/// One derived row as a screen lists it: the header, the value, the value as shown (with the row's suffix) and the
/// detail-pane and tooltip text.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct DRow {
    pub label: String,
    pub value: i64,
    pub shown: String,
    pub desc: String,
}

impl DRow {
    fn plain(label: &str, value: i64, desc: String) -> Self {
        Self { label: label.into(), value, shown: value.to_string(), desc }
    }
}

/// The ruleset the party screen reads derived rows by: the ruleset of the save's world (RULESET1 3.1.1).
fn party_ruleset(cx: &Cx) -> String {
    cx.data.ruleset_id(&cx.gs().world_id)
}

/// PartyPreview.derived: the derived rows of a member built without auto-spend, as it fights in `world` (its class path
/// and deck rule there; HOOKS1-UI slice 2).
fn derived(d: &Data, world: &str, ruleset: &str, m: &UnitState) -> Vec<DRow> {
    progression::build_with(d, m, world, false).map(|b| derived_of(&d.cat, ruleset, &b.unit)).unwrap_or_default()
}

/// The derived rows of a built unit: the built-in numbers, then the catalog's derived stats shown on the party screen
/// (DERIVE1 4.4; none in a ruleset without such rows, so the classic table is unchanged).
fn derived_of(cat: &Catalog, ruleset: &str, u: &Unit) -> Vec<DRow> {
    let mut rows: Vec<DRow> = builtin_of(u).into_iter().map(|(l, v)| DRow::plain(l, v, derived_desc(l))).collect();
    rows.extend(data_rows(cat, ruleset, u, "party"));
    rows
}

/// DERIVE1 4.4: the catalog derived-stat rows `screen` (`party`, `dojo`, `battle`) lists for `u` under `ruleset`, by
/// `order` (ties in catalog order); a row whose `rulesets` leave the ruleset out is absent.
pub(crate) fn data_rows(cat: &Catalog, ruleset: &str, u: &Unit, screen: &str) -> Vec<DRow> {
    let mut defs: Vec<&DerivedStatDef> = cat.derived_stats.values().filter(|d| d.shows_on(screen)).collect();
    defs.sort_by_key(|d| d.order);
    defs.into_iter()
        .filter_map(|d| {
            let b = derived::breakdown(cat, ruleset, u, &d.id)?;
            Some(DRow { label: d.header().to_string(), value: b.value as i64, shown: format!("{}{}", b.value, d.suffix), desc: data_desc(cat, d, &b) })
        })
        .collect()
}

/// A derived row's detail text: its description, its formula, then the breakdown, for example
/// `0 + KNOW 14 x 2 + CONS 12 x 1 = 40; Salt charm +3; = 43 (0-100)`.
fn data_desc(cat: &Catalog, d: &DerivedStatDef, b: &Breakdown) -> String {
    let cap = match b.cap {
        Some(Cap::Max) => "capped at the max; ",
        Some(Cap::Min) => "held at the min; ",
        None => "",
    };
    let end = format!("{}{} ({cap}{}-{})", b.value, d.suffix, b.min, b.max);
    let steps = if b.parts.is_empty() {
        format!("{} = {end}", worked(b))
    } else {
        let parts: Vec<String> = b.parts.iter().map(|p| format!("{} {}", source_name(cat, &p.source), part_text(p))).collect();
        format!("{} = {}; {}; = {end}", worked(b), num(b.raw), parts.join("; "))
    };
    let head = if d.description.is_empty() { String::new() } else { format!("{} ", d.description) };
    format!("{head}Formula: {}. {steps}.", b.formula)
}

/// The formula with its inputs filled in: a stat, `level` or another row as `NAME value`, a constant as its number,
/// `*` as `x` (function names and the rest as written).
fn worked(b: &Breakdown) -> String {
    let src: Vec<char> = b.formula.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < src.len() {
        let c = src[i];
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < src.len() && (src[i].is_ascii_alphanumeric() || src[i] == '_') {
                i += 1;
            }
            let name: String = src[start..i].iter().collect();
            let call = src[i..].iter().find(|c| !c.is_whitespace()) == Some(&'(');
            match b.inputs.iter().find(|x| x.name == name).filter(|_| !call) {
                Some(x) if matches!(x.kind, InputKind::Constant | InputKind::Unknown) => out.push_str(&num(x.value)),
                Some(x) => out.push_str(&format!("{name} {}", num(x.value))),
                None => out.push_str(&name),
            }
            continue;
        }
        out.push(if c == '*' { 'x' } else { c });
        i += 1;
    }
    out
}

/// A number as a breakdown shows it: whole numbers without decimals, others to at most two.
fn num(v: f64) -> String {
    if v.fract() == 0.0 {
        format!("{v:.0}")
    } else {
        let s = format!("{v:.2}");
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    }
}

/// A modifier part's change: `+3`, `-10%` or `+3, -10%`.
fn part_text(p: &derived::Part) -> String {
    let mut out: Vec<String> = Vec::new();
    if p.flat != 0 {
        out.push(format!("{:+}", p.flat));
    }
    if p.percent != 0.0 {
        let pct = p.percent * 100.0;
        out.push(format!("{}{}%", if pct > 0.0 { "+" } else { "-" }, num(pct.abs())));
    }
    out.join(", ")
}

/// The name of a modifier source (`derived::Part::source`): the item, passive or status it came from.
fn source_name(cat: &Catalog, src: &str) -> String {
    let item = |id: &str| cat.items.get(id).map(|i| i.name.clone()).unwrap_or_else(|| id.to_string());
    // a worn item's grant (`<item>`, `<item>:tier<n>`, `<item>:mastery`): the item, its tier or its family mastery
    let grant = |rest: &str| -> String {
        if let Some((id, n)) = rest.split_once(":tier") {
            return format!("{} {}", item(id), roman(n.parse().unwrap_or(0)));
        }
        match rest.strip_suffix(":mastery") {
            Some(id) => format!("{} mastery", item(id)),
            None => item(rest),
        }
    };
    if let Some(id) = src.strip_prefix(derived::STATUS_SOURCE).and_then(|s| s.strip_prefix(':')) {
        return emberwake_rules::status::display_name(id);
    }
    // AUDIT-05: a worn item's stat effect names its grant (`gear_effect:gear:<item>...`), never the raw kind
    if let Some(rest) = src.strip_prefix(GEAR_EFFECT_SOURCE).and_then(|s| s.strip_prefix(':')) {
        return grant(rest.strip_prefix("gear:").unwrap_or(rest));
    }
    if let Some(rest) = src.strip_prefix("passive:gear:") {
        return grant(rest);
    }
    if let Some(id) = src.strip_prefix("passive:") {
        return cat.passives.get(id).map(|p| p.name.clone()).unwrap_or_else(|| id.to_string());
    }
    if let Some(id) = src.strip_prefix(TIER_SOURCE).and_then(|s| s.strip_prefix(':')) {
        return format!("{} tier", item(id));
    }
    if let Some(id) = src.strip_prefix("gear:") {
        return item(id);
    }
    match src {
        "gear" => "Gear".into(),
        GEAR_EFFECT_SOURCE => "Worn item effects".into(),
        TIER_SOURCE => "Gear tiers".into(),
        _ => src.to_string(),
    }
}

/// The built-in derived values of a built unit (rules code; the catalog's rows follow them).
fn builtin_of(u: &Unit) -> Vec<(&'static str, i64)> {
    vec![
        ("Max HP", u.max_hp as i64),
        ("Initiative", u.initiative as i64),
        ("Tiles per move", u.move_per_block() as i64),
        ("Crit %", damage::crit_chance(u).round() as i64),
        ("Melee power", damage::attack_power(u, false, false).attack_power as i64),
        ("Ranged power", damage::attack_power(u, true, false).attack_power as i64),
        ("Armor", damage::armor_rating(u) as i64),
        ("Debuff resist %", u.debuff_resistance().round() as i64),
        ("AP", u.max_ap as i64),
    ]
}

/// Table columns at the current text size: each header keeps its position at normal size unless the previous header
/// would run into it (then it moves right by `gap`); (x, right edge), numbers are right-aligned to the right edge.
fn columns(ui: &mut Ui<PAct>, cols: &[(f32, &str)], gap: f32) -> Vec<(f32, f32)> {
    let mut out: Vec<(f32, f32)> = Vec::new();
    for &(x, h) in cols {
        let x = out.last().map_or(x, |p| x.max(p.1 + gap));
        out.push((x, x + ui.p.text_w(h, BODY + 1.0)));
    }
    out
}

/// What a core stat does, for the detail pane (numbers from the rules: StatRules, DamageCalculator).
fn stat_desc(st: &str) -> String {
    use emberwake_rules::damage::{CRIT_PER_PERC, CRIT_PERC_FLOOR};
    use emberwake_rules::stats::{HP_PER_CONS, KNOW_UNLOCK_BASE, KNOW_UNLOCK_STEP, MAX_UNLOCK_REDUCTION, RANGE_PERC_THRESHOLD};
    match st {
        "STR" => "Strength: power with STR weapons, +2 max HP per point, +1 armor per 4 points; unarmed hits use STR + DEX.".into(),
        "DEX" => "Dexterity: initiative (8 + DEX - gear burden) and move rate, power with DEX weapons and of shots without a ranged weapon.".into(),
        "INT" => "Intelligence: debuff resistance (2.5% per point) and power with INT weapons.".into(),
        "CONS" => format!("Constitution: +{HP_PER_CONS} max HP per point."),
        "PERC" => format!("Perception: crit chance +{CRIT_PER_PERC}% per point above {CRIT_PERC_FLOOR}; at {RANGE_PERC_THRESHOLD} or more, ranged weapons reach 1 tile further."),
        "KNOW" => format!("Knowledge: from {KNOW_UNLOCK_BASE}, every {KNOW_UNLOCK_STEP} points open skill families one level earlier (up to {MAX_UNLOCK_REDUCTION} levels)."),
        _ => String::new(),
    }
}

/// What a derived value (`derived` labels) means, for the detail pane.
fn derived_desc(label: &str) -> String {
    use emberwake_rules::damage::{ARMOR_K, CRIT_BASE, CRIT_PER_PERC, CRIT_PERC_FLOOR};
    use emberwake_rules::stats::HP_PER_CONS;
    match label {
        "Max HP" => format!("Health: 60 + {HP_PER_CONS} per CONS + 2 per STR."),
        "Initiative" => "Initiative: 8 + DEX - the burden of worn gear and carry overload (at least 1); units with more act earlier.".into(),
        "Tiles per move" => "Tiles one move covers (move rate): (8 + DEX + 2) / 5, from 1 to 4; burden and overload lower it only past initiative 1; haste adds 1, slow takes 1.".into(),
        "Crit %" => format!("Critical hit chance: {CRIT_BASE}% + {CRIT_PER_PERC}% per PERC above {CRIT_PERC_FLOOR}, plus the weapon proficiency bonus."),
        "Melee power" => "Melee hit power: the weapon's power (with tier and proficiency bonuses) + its scaling stat x your proficiency scaling.".into(),
        "Ranged power" => "Ranged hit power, the same way with a ranged weapon; without one, DEX alone.".into(),
        "Armor" => format!("Armor rating: gear mitigation + STR / 4; it cuts damage taken by rating / (rating + {ARMOR_K})."),
        "Debuff resist %" => "Chance to shake off a debuff: 2.5% per INT, plus the unit's own resistances.".into(),
        "AP" => "Action points each round; a skill costs its level in AP.".into(),
        _ => String::new(),
    }
}

fn fits(item: &emberwake_data::ItemDef, slot: &str) -> bool {
    emberwake_world::state::item_fits_slot(item, slot)
}

/// RULESET1 9 D6: the deck's slot kinds in bucket order: (kind, bucket header, what its slots take).
pub(crate) const DECK_KINDS: [(&str, &str, &str); 3] = [("active", "Actives", "active skills"), ("passive", "Passives", "passives"), ("keystone", "Keystone", "keystones")];

/// RULESET1 9 D1-D6 (rule `deck`; engine-only, no Godot original): what a hero may put in its deck: the owned actives
/// (never a basic) and slottable passives and keystones, the ruleset's basics (open to all, no slot) and intents, each
/// family's intent, the slot counts, and what an empty deck (D5 auto) or an empty active row stands for. Editing is pure
/// on these terms (every edit starts from the deck as it plays, so a first edit pins the automatic deck), shared by the
/// party screen and the Dojo.
#[derive(Debug, Clone, Default, PartialEq)]
pub(crate) struct DeckTerms {
    pub slots: emberwake_rules::ruleset::DeckSlots,
    pub intents: Vec<String>,
    pub basics: Vec<String>,
    pub actives: Vec<String>,
    pub passives: Vec<String>,
    pub keystones: Vec<String>,
    /// Skill family -> its intent (`round_intent`, else its kind's; actives and basics). A4.2-K1b: its primary row.
    pub intent_of: IndexMap<String, String>,
    /// A4.2-K1b (INTENT2 IR15, IR21): skill family -> every intent row it plays in under the ruleset (`intent::allowed`:
    /// the derived rows under `field_intent`, else the single intent); a family missing here has `intent_of`'s alone.
    pub rows_of: IndexMap<String, Vec<String>>,
    /// The deck an empty deck plays as (D5 auto: the base unit's deck, else the first class-path families).
    pub auto: Deck,
    /// The actives an empty `active` row plays as in a deck with other keys (the loadout's families).
    pub auto_actives: Vec<String>,
    /// TERMS1 TU5: intent id -> its name through the catalog's `terms` (`intent_names`; empty = the defaults).
    pub intent_names: IndexMap<String, String>,
}

/// A4.2-K1b: each family's primary intent and rows under `rs` (`intent::allowed`).
fn rows_of<'a>(cat: &emberwake_data::Catalog, rs: &emberwake_rules::ruleset::Ruleset, fams: impl Iterator<Item = &'a String>) -> (IndexMap<String, String>, IndexMap<String, Vec<String>>) {
    let mut primary = IndexMap::new();
    let mut rows = IndexMap::new();
    for f in fams {
        if let Some(s) = cat.tier_one(f) {
            let r: Vec<String> = emberwake_rules::intent::allowed(rs, &emberwake_rules::intent::IntentInput::of_def(s), &s.round_intent).into_iter().map(String::from).collect();
            primary.insert(f.clone(), r.first().cloned().unwrap_or_else(|| emberwake_battle::round_intent::skill_intent(s).to_string()));
            rows.insert(f.clone(), r);
        }
    }
    (primary, rows)
}

/// A4.2-K1b: the intents a deck order lists: the ruleset's, and Field under `field_intent`.
fn order_intents(rs: &emberwake_rules::ruleset::Ruleset) -> Vec<String> {
    let mut out = rs.numbers.intents.clone();
    if rs.has(emberwake_rules::ruleset::FIELD_INTENT) && !out.iter().any(|i| i == "field") {
        out.push("field".into());
    }
    out
}

impl DeckTerms {
    /// A4.2-K1b (IR15): the rows family `f` plays in (its `rows_of`, else its one intent).
    pub fn rows(&self, f: &str) -> Vec<String> {
        self.rows_of.get(f).cloned().unwrap_or_else(|| self.intent_of.get(f).cloned().into_iter().collect())
    }

    /// Whether families `a` and `b` share a row (a follow-up joins an opener of one of its rows).
    fn share_row(&self, a: &str, b: &str) -> bool {
        let rb = self.rows(b);
        self.rows(a).iter().any(|r| rb.contains(r))
    }

    /// A party member's terms under `rs`: None unless the ruleset has the deck rule (classic keeps today's tab).
    pub fn for_member(d: &Data, m: &UnitState, world: &str, rs: &emberwake_rules::ruleset::Ruleset) -> Option<Self> {
        // CLASS-BASICS: the member's class basics are open beside the ruleset's (its class in that world)
        let rule = emberwake_rules::DeckRule::of(rs)?.with_class(d.cat_in(world), &m.class_id(d));
        let cat = &d.cat;
        let basics: Vec<String> = rule.basics.iter().filter(|b| cat.tier_one(b).is_some()).cloned().collect();
        let actives: Vec<String> = m.owned_families(d, world).into_iter().filter(|f| cat.tier_one(f).is_some() && !cat.is_basic_family(f) && !rule.basics.contains(f) && !cat.is_system(f)).collect();
        // FB-63 (2): the re-gathered list, as the equip check and validate read it
        let owned_passives = progression::member_passives(d, m);
        let of_slot = |slot: &str| -> Vec<String> { owned_passives.iter().filter(|p| cat.passives.get(*p).is_some_and(|x| x.slot == slot)).cloned().collect() };
        let (passives, keystones) = (of_slot("passive"), of_slot("keystone"));
        let mut work = m.clone();
        work.deck = Deck::default();
        let auto_unit = progression::build_in(d, &work, world, Some(&rule))?.unit;
        let classic = progression::build_as(d, &work, world, false, None)?.unit;
        let auto_active = auto_unit.active_families();
        let auto = Deck {
            // PR5-26 (audit B4): the base unit's marrow marks among the automatic actives
            marrow: auto_unit.deck.marrow.iter().filter(|f| auto_active.contains(f)).cloned().collect(),
            active: auto_active,
            passive: auto_unit.deck.passive.iter().filter(|p| passives.contains(p)).cloned().collect(),
            keystone: auto_unit.deck.keystone.iter().filter(|p| keystones.contains(p)).cloned().collect(),
            order: auto_unit.deck.order.clone(),
            follow_ups: IndexMap::new(),
        };
        // apply_deck's auto pick for an empty active row: the loadout's families the member owns, the first `active`
        let mut auto_actives: Vec<String> = Vec::new();
        for f in classic.active_families() {
            if actives.contains(&f) && !auto_actives.contains(&f) {
                auto_actives.push(f);
            }
        }
        auto_actives.truncate(rule.slots.active.max(0) as usize);
        let (intent_of, rows_of) = rows_of(cat, rs, actives.iter().chain(&basics));
        Some(Self { slots: rule.slots, intents: order_intents(rs), basics, actives, passives, keystones, intent_of, rows_of, auto, auto_actives, intent_names: intent_names(cat) })
    }

    /// RULESET1-G (the Dojo's deck view): a catalog unit's terms under `rs` (None without the deck rule). `u` is the
    /// unit as it fights now; the actives are the families it owns (no basic); a catalog unit owns the slottable
    /// passives its deck names (D4), so any catalog passive or keystone may be named; the auto deck is the unit built
    /// with an empty deck.
    pub fn for_unit(cat: &emberwake_data::Catalog, rs: &emberwake_rules::ruleset::Ruleset, def: &emberwake_data::UnitDef, u: &emberwake_rules::Unit) -> Option<Self> {
        let rule = emberwake_rules::DeckRule::of(rs)?.with_class(cat, &def.class_id);
        let basics: Vec<String> = rule.basics.iter().filter(|b| cat.tier_one(b).is_some()).cloned().collect();
        let mut actives: Vec<String> = Vec::new();
        for s in cat.skills.iter().filter(|s| !s.system && !s.basic && !emberwake_battle::skill::is_item(s)) {
            let f = s.family_id().to_string();
            if !actives.contains(&f) && !rule.basics.contains(&f) && !cat.is_basic_family(&f) && !cat.is_system(&f) && cat.tier_one(&f).is_some() && u.owns_skill(cat, &s.id) {
                actives.push(f);
            }
        }
        let of_slot = |slot: &str| -> Vec<String> { cat.passives.values().filter(|p| p.slot == slot).map(|p| p.id.clone()).collect() };
        let (passives, keystones) = (of_slot("passive"), of_slot("keystone"));
        let mut empty = def.clone();
        empty.deck = Deck::default();
        let auto_unit = emberwake_rules::build_unit_in(cat, &empty, 0, Some(&rule));
        let classic = emberwake_rules::build_unit(cat, def, 0);
        let auto = Deck { active: auto_unit.active_families(), ..Deck::default() };
        let mut auto_actives: Vec<String> = Vec::new();
        for f in classic.active_families() {
            if actives.contains(&f) && !auto_actives.contains(&f) {
                auto_actives.push(f);
            }
        }
        auto_actives.truncate(rule.slots.active.max(0) as usize);
        let (intent_of, rows_of) = rows_of(cat, rs, actives.iter().chain(&basics));
        Some(Self { slots: rule.slots, intents: order_intents(rs), basics, actives, passives, keystones, intent_of, rows_of, auto, auto_actives, intent_names: intent_names(cat) })
    }

    /// The slots of `kind`.
    pub fn cap(&self, kind: &str) -> usize {
        (match kind {
            "active" => self.slots.active,
            "passive" => self.slots.passive,
            _ => self.slots.keystone,
        })
        .max(0) as usize
    }

    /// What the member owns of `kind`.
    pub fn owned(&self, kind: &str) -> &[String] {
        match kind {
            "active" => &self.actives,
            "passive" => &self.passives,
            _ => &self.keystones,
        }
    }

    pub fn list<'a>(deck: &'a Deck, kind: &str) -> &'a Vec<String> {
        match kind {
            "active" => &deck.active,
            "passive" => &deck.passive,
            _ => &deck.keystone,
        }
    }

    fn list_mut<'a>(deck: &'a mut Deck, kind: &str) -> &'a mut Vec<String> {
        match kind {
            "active" => &mut deck.active,
            "passive" => &mut deck.passive,
            _ => &mut deck.keystone,
        }
    }

    /// The deck as it plays: an empty deck is the auto deck; an empty active row the auto actives (D5).
    pub fn shown(&self, draft: &Deck) -> Deck {
        if draft.is_empty() {
            return self.auto.clone();
        }
        let mut d = draft.clone();
        if d.active.is_empty() {
            d.active = self.auto_actives.clone();
        }
        d
    }

    /// The deck an edit of `kind` starts from (user 2026-10-29): an actives edit (and the order editor) pins the deck as
    /// it plays (`shown`); a passive or keystone edit keeps the actives automatic (an empty `active` row stays empty, so
    /// later class-path actives still join), unless the deck as it plays carries an order or follow-up list (their
    /// entries must be in `active`: then it pins; INPUT1 FL3).
    fn base(&self, draft: &Deck, kind: &str) -> Deck {
        let shown = self.shown(draft);
        if kind == "active" || !draft.active.is_empty() || !shown.order.is_empty() || !shown.follow_ups.is_empty() {
            return shown;
        }
        // PR5-26 (audit B4): the actives marked marrow stay (they mark the automatic actives)
        Deck { active: Vec::new(), passive: shown.passive, keystone: shown.keystone, order: IndexMap::new(), follow_ups: IndexMap::new(), marrow: shown.marrow }
    }

    /// RULESET1_INTENT FU4: the order the planner fills `intent` by: the deck's `order` list for it (entries still in the
    /// deck), else the deck's actives of the intent in slot order, then the basics of the intent.
    pub fn order(&self, deck: &Deck, intent: &str) -> Vec<String> {
        let of = |f: &&String| self.intent_of.get(*f).is_some_and(|i| i == intent);
        // A4.2-K1b (IR15): the player's order may hold a skill in any of its rows; the auto order its primary row only
        let in_row = |f: &&String| self.rows(f).iter().any(|i| i == intent);
        match deck.order.get(intent).filter(|l| !l.is_empty()) {
            Some(l) => l.iter().filter(|f| in_row(f) && (deck.active.contains(f) || self.basics.contains(f))).cloned().collect(),
            None => deck.active.iter().chain(&self.basics).filter(of).cloned().collect(),
        }
    }

    /// A new active joins the end of its intent's written order (an auto order takes it by itself).
    fn ordered_in(&self, d: &mut Deck, fam: &str) {
        let Some(intent) = self.intent_of.get(fam) else { return };
        if let Some(l) = d.order.get_mut(intent).filter(|l| !l.is_empty()) {
            if !l.iter().any(|f| f == fam) {
                l.push(fam.to_string());
            }
        }
    }

    /// An active that leaves the deck leaves every order and follow-up list and drops its own follow-up list (an
    /// emptied list is the auto order again; INPUT1 FL3). PR5-26 (audit B4): and its marrow mark.
    fn unordered(d: &mut Deck, fam: &str) {
        d.marrow.retain(|f| f != fam);
        for l in d.order.values_mut().chain(d.follow_ups.values_mut()) {
            l.retain(|f| f != fam);
        }
        d.order.retain(|_, l| !l.is_empty());
        d.follow_ups.shift_remove(fam);
        d.follow_ups.retain(|_, l| !l.is_empty());
    }

    /// INPUT1 FL1: the follow-up list of opener `op` as it plays (entries still in the deck, of the opener's intent);
    /// empty when none is set (the fill reads the intent's order).
    pub fn follow_list(&self, deck: &Deck, op: &str) -> Vec<String> {
        if self.intent_of.get(op).is_none() {
            return Vec::new();
        }
        let ok = |f: &&String| self.share_row(op, f) && (deck.active.contains(f) || self.basics.contains(f));
        deck.follow_ups.get(op).map(|l| l.iter().filter(ok).cloned().collect()).unwrap_or_default()
    }

    /// FL3: the skills that can join `op`'s list: its intent's actives and basics in the deck (the order's entries),
    /// those already listed left out.
    pub fn follow_candidates(&self, deck: &Deck, op: &str) -> Vec<String> {
        let listed = self.follow_list(deck, op);
        let mut out: Vec<String> = Vec::new();
        // A4.2-K1b: the orders of each of the opener's rows
        for intent in self.rows(op) {
            for f in self.order(deck, &intent) {
                if !listed.contains(&f) && !out.contains(&f) {
                    out.push(f);
                }
            }
        }
        out
    }

    /// The deck a follow-up edit of `op` starts from (it pins the deck as it plays, as an order edit does) and the list
    /// as it plays; refused when `op` is not an opener of the deck (FL5).
    fn follow_base(&self, draft: &Deck, op: &str) -> Result<(Deck, Vec<String>), String> {
        let d = self.shown(draft);
        let opener = self.rows(op).iter().any(|i| self.intents.contains(i)) && (d.active.iter().any(|f| f == op) || self.basics.iter().any(|f| f == op));
        if !opener {
            return Err(format!("Follow-ups for {op}: not an opener in the deck."));
        }
        let list = self.follow_list(&d, op);
        Ok((d, list))
    }

    fn with_follow(mut d: Deck, op: &str, list: Vec<String>) -> Deck {
        if list.is_empty() {
            d.follow_ups.shift_remove(op);
        } else {
            d.follow_ups.insert(op.to_string(), list);
        }
        d
    }

    /// FL3: `fam` joins `op`'s list at place `at` (None: the end); a listed one moves there. Only skills of the
    /// opener's intent in the deck.
    pub fn follow_add(&self, draft: &Deck, op: &str, fam: &str, at: Option<usize>) -> Result<Deck, String> {
        let (d, mut list) = self.follow_base(draft, op)?;
        let intent = self.intent_of.get(op).cloned().unwrap_or_default();
        if !(d.active.iter().any(|f| f == fam) || self.basics.iter().any(|f| f == fam)) {
            return Err(format!("Follow-ups after {op}: {fam} is not in the deck."));
        }
        if !self.share_row(op, fam) {
            let has = self.intent_of.get(fam).map(|i| intent_title(&self.intent_names, i)).unwrap_or_else(|| "no intent".into());
            return Err(format!("Follow-ups after {op}: {fam} is {has}, not {}.", intent_title(&self.intent_names, &intent)));
        }
        let was = list.iter().position(|f| f == fam);
        if let Some(i) = was {
            list.remove(i);
        }
        let j = at.unwrap_or(list.len()).min(list.len());
        if was == Some(j) {
            return Err("Same place.".into());
        }
        list.insert(j, fam.to_string());
        Ok(Self::with_follow(d, op, list))
    }

    /// FL3: takes place `k` out of `op`'s list (the last one out: the intent's order again).
    pub fn follow_remove(&self, draft: &Deck, op: &str, k: usize) -> Result<Deck, String> {
        let (d, mut list) = self.follow_base(draft, op)?;
        if k >= list.len() {
            return Err("Nothing there.".into());
        }
        list.remove(k);
        Ok(Self::with_follow(d, op, list))
    }

    /// FL3 "Use the <Intent> order": empties `op`'s list.
    pub fn follow_clear(&self, draft: &Deck, op: &str) -> Result<Deck, String> {
        let (d, list) = self.follow_base(draft, op)?;
        if list.is_empty() {
            return Err("No follow-ups set: the intent's order fills.".into());
        }
        Ok(Self::with_follow(d, op, Vec::new()))
    }

    /// `follow.<op>.<k>`: place `k` of opener `op`'s follow-up list (`k` = the list's length: its end).
    fn follow_key(key: &str) -> Option<(&str, usize)> {
        let rest = key.strip_prefix("follow.").filter(|r| !r.starts_with("add."))?;
        let (op, k) = rest.rsplit_once('.')?;
        Some((op, k.parse().ok()?))
    }

    fn full(kind: &str) -> String {
        let label = DECK_KINDS.iter().find(|k| k.0 == kind).map(|k| k.1).unwrap_or("Slots");
        format!("{label} are full: drag onto a slot to replace one, or right click a slot to take it out.")
    }

    /// The click path: `id` (an owned `kind`) into the first free slot of its bucket.
    pub fn slot(&self, draft: &Deck, kind: &str, id: &str) -> Result<Deck, String> {
        if !self.owned(kind).iter().any(|x| x == id) {
            return Err(format!("'{id}' is not owned."));
        }
        let mut d = self.base(draft, kind);
        if Self::list(&d, kind).iter().any(|x| x == id) {
            return Err("Already slotted.".into());
        }
        if Self::list(&d, kind).len() >= self.cap(kind) {
            return Err(Self::full(kind));
        }
        Self::list_mut(&mut d, kind).push(id.to_string());
        if kind == "active" {
            self.ordered_in(&mut d, id);
        }
        Ok(d)
    }

    /// Takes slot `i` of `kind` out. The last active stays: an empty active row means the automatic pick (D5).
    pub fn unslot(&self, draft: &Deck, kind: &str, i: usize) -> Result<Deck, String> {
        let mut d = self.base(draft, kind);
        if i >= Self::list(&d, kind).len() {
            return Err("That slot is empty.".into());
        }
        if kind == "active" && d.active.len() <= 1 {
            return Err("Keep at least one active (an empty row is the automatic pick: Use default).".into());
        }
        let id = Self::list_mut(&mut d, kind).remove(i);
        Self::unordered(&mut d, &id);
        Ok(d)
    }

    /// `deck.<kind>.<i>`: a slot.
    fn slot_key(key: &str) -> Option<(&'static str, usize)> {
        let (k, i) = key.strip_prefix("deck.")?.split_once('.')?;
        Some((DECK_KINDS.iter().find(|x| x.0 == k)?.0, i.parse().ok()?))
    }

    /// `order.<intent>.<i>`: a place in an intent's order.
    fn order_key(key: &str) -> Option<(&str, usize)> {
        let (intent, i) = key.strip_prefix("order.")?.rsplit_once('.')?;
        Some((intent, i.parse().ok()?))
    }

    /// The deck after dropping payload `from` (`deck.own.<kind>.<id>` from the owned list, `deck.<kind>.<i>` from a slot,
    /// `order.<intent>.<i>` from the order editor) on `to` (`deck.<kind>.<i>`, `deck.list`, `order.<intent>.<i>`), or
    /// why it can't be done: the same rules as the clicks (a bucket takes its own kind; a filled slot swaps or is
    /// replaced; an order entry moves within its intent).
    pub fn after_drop(&self, draft: &Deck, from: &str, to: &str) -> Result<Deck, String> {
        let wrong = |kind: &str| {
            let k = DECK_KINDS.iter().find(|x| x.0 == kind).unwrap_or(&DECK_KINDS[0]);
            format!("{} slots take {} only.", k.1, k.2)
        };
        // INPUT1 FL3: a follow-up entry moves within its list; an intent skill dragged from the add row joins it there
        if let Some((op, fam)) = from.strip_prefix("follow.add.").and_then(|r| r.split_once('.')) {
            let (to_op, j) = Self::follow_key(to).ok_or("Drop it on a place in the same follow-up list.")?;
            if to_op != op {
                return Err("Each opener keeps its own follow-ups.".into());
            }
            return self.follow_add(draft, op, fam, Some(j));
        }
        if let Some((op, i)) = Self::follow_key(from) {
            let (to_op, j) = Self::follow_key(to).ok_or("Drop it on a place in the same follow-up list.")?;
            if to_op != op {
                return Err("Each opener keeps its own follow-ups.".into());
            }
            let list = self.follow_base(draft, op)?.1;
            let fam = list.get(i).cloned().ok_or("Nothing there.")?;
            return self.follow_add(draft, op, &fam, Some(j.min(list.len() - 1)));
        }
        if let Some((intent, i)) = Self::order_key(from) {
            let (to_intent, j) = Self::order_key(to).ok_or("Drop it on a place in the same intent's order.")?;
            if to_intent != intent {
                return Err("Each intent keeps its own order.".into());
            }
            let mut d = self.shown(draft);
            let mut list = self.order(&d, intent);
            if i >= list.len() {
                return Err("Nothing there.".into());
            }
            if i == j {
                return Err("Same place.".into());
            }
            let f = list.remove(i);
            list.insert(j.min(list.len()), f);
            d.order.insert(intent.to_string(), list);
            return Ok(d);
        }
        if let Some((kind, id)) = from.strip_prefix("deck.own.").and_then(|r| r.split_once('.')) {
            let (to_kind, i) = Self::slot_key(to).ok_or("Drop it on a slot.")?;
            let shown = self.shown(draft);
            // a slotted one moves from its own slot (swap or move, never a shifted index)
            if let Some(j) = Self::list(&shown, kind).iter().position(|x| x == id) {
                return self.after_drop(draft, &format!("deck.{kind}.{j}"), to);
            }
            if to_kind != kind {
                return Err(wrong(to_kind));
            }
            if !self.owned(kind).iter().any(|x| x == id) {
                return Err(format!("'{id}' is not owned."));
            }
            let mut d = self.base(draft, kind);
            match Self::list(&d, kind).get(i).cloned() {
                Some(old) => {
                    Self::list_mut(&mut d, kind)[i] = id.to_string();
                    Self::unordered(&mut d, &old);
                }
                None if Self::list(&d, kind).len() >= self.cap(kind) => return Err(Self::full(kind)),
                None => Self::list_mut(&mut d, kind).push(id.to_string()),
            }
            if kind == "active" {
                self.ordered_in(&mut d, id);
            }
            return Ok(d);
        }
        let (kind, i) = Self::slot_key(from).ok_or("That can't go there.")?;
        let mut d = self.base(draft, kind);
        if i >= Self::list(&d, kind).len() {
            return Err("That slot is empty.".into());
        }
        if to == "deck.list" {
            return self.unslot(draft, kind, i);
        }
        let (to_kind, j) = Self::slot_key(to).ok_or("That can't go there.")?;
        if to_kind != kind {
            return Err(wrong(to_kind));
        }
        if i == j {
            return Err("Same slot.".into());
        }
        let list = Self::list_mut(&mut d, kind);
        if j < list.len() {
            list.swap(i, j);
        } else {
            let f = list.remove(i);
            list.push(f);
        }
        Ok(d)
    }
}

impl Character {
    pub fn new(cx: &Cx, member: &str, tab: &str) -> Self {
        let gs = cx.gs();
        let first = gs.party.members.first().map(|m| m.id.clone()).unwrap_or_default();
        let member = if gs.party.member(member).is_some() { member.to_string() } else { first };
        let tab = TABS.iter().find(|t| t.0 == tab).map(|t| t.0).unwrap_or("stats");
        Self { member, tab, draft: IndexMap::new(), class_slots: Vec::new(), extra_slots: Vec::new(), ranks: IndexMap::new(), using_default: true, mouse: (0, 0), skills_loaded: false, gear: None, gear_stacks: false, gear_world: String::new(), pending_switch: None, scroll: 0, gear_scroll: 0, detail: String::new(), terms: None, deck: Deck::default(), deck_view: "owned", follow_open: None, faces: crate::portraits::Faces::default(), w: Vec::new() }
    }

    fn m<'a>(&self, cx: &'a Cx) -> &'a UnitState {
        cx.gs().party.member(&self.member).expect("member")
    }

    fn has_draft(&self, cx: &Cx) -> bool {
        match self.tab {
            "level_up" => self.draft.values().sum::<i64>() > 0,
            "skills" if self.terms.is_some() => self.skills_loaded && self.deck != self.m(cx).deck,
            "skills" => {
                let m = self.m(cx);
                let (c, e) = if self.using_default { (Vec::new(), Vec::new()) } else { (self.class_slots.clone(), self.extra_slots.clone()) };
                self.skills_loaded && (c != m.loadout_class || e != m.loadout_extra || self.ranks != m.loadout_ranks)
            }
            "gear" => self.gear.as_ref().is_some_and(|(w, st)| w != self.m(cx) || *st != cx.gs().party.stash),
            _ => false,
        }
    }

    fn drop_drafts(&mut self) {
        self.draft.clear();
        self.skills_loaded = false;
        self.gear = None;
    }

    fn load_skills(&mut self, cx: &Cx) {
        if self.skills_loaded {
            return;
        }
        let m = self.m(cx).clone();
        // RULESET1 9 D6: under the deck rule the tab edits the deck (classic: today's slots)
        self.terms = DeckTerms::for_member(&cx.data, &m, &cx.gs().world_id, &cx.data.ruleset(&cx.gs().world_id));
        self.deck = m.deck.clone();
        self.using_default = m.loadout_class.is_empty() && m.loadout_extra.is_empty();
        if self.using_default {
            self.fill_default(cx);
        } else {
            self.class_slots = m.loadout_class.clone();
            self.extra_slots = m.loadout_extra.clone();
        }
        self.ranks = m.loadout_ranks.clone();
        self.skills_loaded = true;
    }

    /// The member's class path families (class slots take only these, SKILLUI1 Q3 A); the played world's path (HOOKS1
    /// CN3, decision "HOOKS1-UI slice 1 answers").
    fn path_families(&self, cx: &Cx) -> Vec<String> {
        let d = &cx.data;
        let m = self.m(cx);
        d.cat_in(&cx.gs().world_id).classes.get(&m.class_id(d)).map(|a| emberwake_rules::unit::families_of(&d.cat, &a.path_steps().into_iter().map(|(_, s)| s).collect::<Vec<_>>())).unwrap_or_default()
    }

    /// The highest rank of `family` the member has unlocked (with confirmed gear): what a newly slotted skill stores.
    fn top_rank(&self, cx: &Cx, family: &str) -> Option<i64> {
        progression::build_with(&cx.data, self.m(cx), &cx.gs().world_id, false).and_then(|b| b.unit.selectable_ranks(&cx.data.cat, family).last().map(|&r| r as i64))
    }

    /// The draft after dropping payload `from` (`skill.<f>` from the list, `slot.<g>.<i>` from a slot) on target `to`
    /// (`slot.<g>.<i>` or `list`), or why it can't be done (SKILLUI1 section 4 drop table).
    fn skills_after_drop(&self, cx: &Cx, from: &str, to: &str) -> Result<SkillDraft, String> {
        let path = self.path_families(cx);
        let (mut class, mut extra, mut ranks) = (self.class_slots.clone(), self.extra_slots.clone(), self.ranks.clone());
        let slot_of = |key: &str| -> Option<(bool, usize)> {
            let rest = key.strip_prefix("slot.")?;
            let (g, i) = rest.split_once('.')?;
            Some((g == "class", i.parse().ok()?))
        };
        let fits = |fam: &str, class_group: bool| !class_group || path.iter().any(|p| p == fam);
        if let Some(fam) = from.strip_prefix("skill.") {
            let (to_class, i) = slot_of(to).ok_or("Drop a skill on a slot.")?;
            // A family already slotted moves from its own slot (the slot-to-slot rules: swap or move, never a shifted
            // index), whether it was dragged from the list or from its slot.
            if let Some(j) = class.iter().position(|f| f == fam) {
                return self.skills_after_drop(cx, &format!("slot.class.{j}"), to);
            }
            if let Some(j) = extra.iter().position(|f| f == fam) {
                return self.skills_after_drop(cx, &format!("slot.extra.{j}"), to);
            }
            if !fits(fam, to_class) {
                return Err("Class slots take class path skills only.".into());
            }
            let list = if to_class { &mut class } else { &mut extra };
            match list.get(i).cloned() {
                Some(old) => {
                    list[i] = fam.to_string();
                    ranks.shift_remove(&old);
                }
                None => {
                    let cap = if to_class { emberwake_rules::stats::MAX_CLASS_SLOTS } else { emberwake_rules::stats::MAX_EXTRA_SLOTS };
                    if list.len() >= cap {
                        return Err("That group is full.".into());
                    }
                    list.push(fam.to_string());
                }
            }
            // the user's Q2 answer: a newly slotted skill stores its highest unlocked rank now
            if let Some(r) = self.top_rank(cx, fam) {
                ranks.insert(fam.to_string(), r);
            }
            return Ok((class, extra, ranks));
        }
        let (from_class, i) = slot_of(from).ok_or("That can't go there.")?;
        let fam = (if from_class { &class } else { &extra }).get(i).cloned().ok_or("That slot is empty.")?;
        if to == "list" {
            (if from_class { &mut class } else { &mut extra }).remove(i);
            ranks.shift_remove(&fam);
            return Ok((class, extra, ranks));
        }
        let (to_class, j) = slot_of(to).ok_or("That can't go there.")?;
        if (from_class, i) == (to_class, j) {
            return Err("Same slot.".into());
        }
        let other = (if to_class { &class } else { &extra }).get(j).cloned();
        if !fits(&fam, to_class) || other.as_ref().is_some_and(|o| !fits(o, from_class)) {
            return Err("Class slots take class path skills only.".into());
        }
        match other {
            Some(o) => {
                (if from_class { &mut class } else { &mut extra })[i] = o;
                (if to_class { &mut class } else { &mut extra })[j] = fam;
            }
            None => {
                let cap = if to_class { emberwake_rules::stats::MAX_CLASS_SLOTS } else { emberwake_rules::stats::MAX_EXTRA_SLOTS };
                (if from_class { &mut class } else { &mut extra }).remove(i);
                let list = if to_class { &mut class } else { &mut extra };
                if list.len() >= cap {
                    return Err("That group is full.".into());
                }
                list.push(fam);
            }
        }
        Ok((class, extra, ranks))
    }

    fn fill_default(&mut self, cx: &Cx) {
        let mut work = self.m(cx).clone();
        work.loadout_class.clear();
        work.loadout_extra.clear();
        if let Some(b) = progression::build_with(&cx.data, &work, &cx.gs().world_id, false) {
            self.class_slots = b.unit.active_class_families.clone();
            self.extra_slots = b.unit.active_extra_families.clone();
        }
    }

    fn load_gear(&mut self, cx: &Cx) {
        if self.gear.is_none() {
            self.gear = Some((self.m(cx).clone(), cx.gs().party.stash.clone()));
            self.gear_stacks = cx.data.ruleset(&cx.gs().world_id).numbers.pouch_stacks;
            self.gear_world = cx.gs().world_id.clone();
        }
    }

    /// A member or tab switch; with a draft open it asks first (router.ask(DISCARD_QUESTION), No the default: ASKNO1) and applies on Yes.
    fn switch(&mut self, cx: &mut Cx, member: Option<String>, tab: Option<&'static str>) {
        if self.has_draft(cx) {
            self.pending_switch = Some((member, tab));
            cx.ask_tag_no("Discard the changes on this tab?", "character.discard");
        } else {
            self.apply_switch(member, tab);
        }
    }

    fn apply_switch(&mut self, member: Option<String>, tab: Option<&'static str>) {
        self.drop_drafts();
        if let Some(m) = member {
            self.member = m;
        }
        if let Some(t) = tab {
            self.tab = t;
        }
    }

    /// Answers to this screen's questions (taken on the frame after the question closes).
    fn take_answers(&mut self, cx: &mut Cx) {
        if cx.take_answer("character.discard") {
            if let Some((m, t)) = self.pending_switch.take() {
                self.apply_switch(m, t);
            }
        }
        if cx.take_answer("character.close") {
            cx.nav.push(Nav::Pop);
        }
        if cx.take_answer("character.spend") {
            let d = cx.data.clone();
            let draft: serde_json::Map<String, Value> = self.draft.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
            let id = self.member.clone();
            if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                let errors = progression::confirm(&d, m, &draft);
                if errors.is_empty() {
                    self.draft.clear();
                } else {
                    show_errors(cx, errors);
                }
            }
        }
    }

    fn act(&mut self, cx: &mut Cx, a: PAct) {
        let d = cx.data.clone();
        match a {
            PAct::Member(step) => {
                let ids: Vec<String> = cx.gs().party.members.iter().map(|m| m.id.clone()).collect();
                if ids.len() < 2 {
                    return;
                }
                let at = ids.iter().position(|i| *i == self.member).unwrap_or(0) as i64;
                let next = ids[(at + step).rem_euclid(ids.len() as i64) as usize].clone();
                self.switch(cx, Some(next), None);
            }
            PAct::Tab(t) => {
                if t != self.tab {
                    self.switch(cx, None, Some(t));
                }
            }
            PAct::Close => {
                if self.has_draft(cx) {
                    cx.ask_tag_no("Discard the changes on this tab?", "character.close");
                } else {
                    cx.nav.push(Nav::Pop);
                }
            }
            PAct::ToggleActive => {
                let id = self.member.clone();
                let party = &mut cx.gs_mut().party;
                if party.active_ids.is_empty() {
                    party.active_ids = party.members.iter().take(party.max_active as usize).map(|m| m.id.clone()).collect();
                }
                if let Some(i) = party.active_ids.iter().position(|x| *x == id) {
                    if party.active_ids.len() <= 1 {
                        show_errors(cx, vec!["Keep at least one active member.".into()]);
                    } else {
                        party.active_ids.remove(i);
                    }
                } else if party.active_ids.len() as i64 >= party.max_active {
                    let cap = party.max_active;
                    show_errors(cx, vec![format!("The active party holds at most {cap}.")]);
                } else {
                    party.active_ids.push(id);
                }
            }
            PAct::Add(stat) => {
                let left = self.m(cx).unspent(&d) - self.draft.values().sum::<i64>();
                if left > 0 {
                    *self.draft.entry(stat.into()).or_insert(0) += 1;
                }
            }
            PAct::Remove(stat) => {
                if let Some(v) = self.draft.get_mut(stat) {
                    *v -= 1;
                    if *v <= 0 {
                        self.draft.shift_remove(stat);
                    }
                }
            }
            PAct::Auto => {
                let m = self.m(cx);
                let left = m.unspent(&d) - self.draft.values().sum::<i64>();
                if let (Some(arch), true) = (d.cat.classes.get(&m.class_id(&d)), left > 0) {
                    for (k, v) in emberwake_rules::stats::auto_spend(&arch.unit_shares().growth, left as i32) {
                        if v > 0 {
                            *self.draft.entry(k).or_insert(0) += v as i64;
                        }
                    }
                }
            }
            PAct::Revert => self.draft.clear(),
            PAct::ConfirmLevel => {
                if self.draft.values().sum::<i64>() > 0 {
                    cx.ask_tag("Spend these points? This is permanent.", "character.spend");
                }
            }
            PAct::Slot(fam) => {
                // SKILLUI1 click path: the first free slot of the family's group (a class path family may use a free
                // extra slot when the class slots are full, Q3 A); a full group says how to make room.
                self.load_skills(cx);
                if self.class_slots.contains(&fam) || self.extra_slots.contains(&fam) {
                    return;
                }
                let on_path = self.path_families(cx).contains(&fam);
                let class_free = self.class_slots.len() < emberwake_rules::stats::MAX_CLASS_SLOTS;
                let extra_free = self.extra_slots.len() < emberwake_rules::stats::MAX_EXTRA_SLOTS;
                let to_class = if on_path && class_free {
                    true
                } else if extra_free {
                    false
                } else {
                    let group = if on_path { "Class" } else { "Extra" };
                    cx.notice.push(format!("{group} slots are full: drag onto a slot to replace one, or right click a slot to take it out."));
                    return;
                };
                self.using_default = false;
                if to_class {
                    self.class_slots.push(fam.clone());
                } else {
                    self.extra_slots.push(fam.clone());
                }
                // the user's Q2 answer: a newly slotted skill stores its highest unlocked rank now
                if let Some(r) = self.top_rank(cx, &fam) {
                    self.ranks.insert(fam, r);
                }
            }
            PAct::Unslot(class, i) => {
                self.load_skills(cx);
                self.using_default = false;
                let list = if class { &mut self.class_slots } else { &mut self.extra_slots };
                if i < list.len() {
                    let fam = list.remove(i);
                    self.ranks.shift_remove(&fam);
                }
            }
            PAct::RankStep(class, i, step) => {
                self.load_skills(cx);
                let Some(fam) = (if class { &self.class_slots } else { &self.extra_slots }).get(i).cloned() else { return };
                let Some(b) = progression::build_with(&d, self.m(cx), &cx.gs().world_id, false) else { return };
                let ranks = b.unit.selectable_ranks(&d.cat, &fam);
                let now = draft_rank(&ranks, self.ranks.get(&fam).map(|&r| r as i32));
                let at = ranks.iter().position(|&r| r == now).unwrap_or(ranks.len().saturating_sub(1)) as i32;
                if let Some(&next) = ranks.get((at + step).max(0) as usize).filter(|_| at + step >= 0) {
                    self.ranks.insert(fam, next as i64);
                }
            }
            PAct::ClearAll => {
                self.load_skills(cx);
                self.using_default = false;
                self.class_slots.clear();
                self.extra_slots.clear();
                self.ranks.clear();
            }
            PAct::DeckSlot(kind, id) => self.deck_edit(cx, |t, deck| t.slot(deck, kind, &id)),
            PAct::DeckUnslot(kind, i) => self.deck_edit(cx, |t, deck| t.unslot(deck, kind, i)),
            PAct::DeckView(v) => {
                self.deck_view = v;
                self.scroll = 0;
            }
            PAct::OrderUp(intent, i) if i > 0 => self.deck_edit(cx, |t, deck| t.after_drop(deck, &format!("order.{intent}.{i}"), &format!("order.{intent}.{}", i - 1))),
            PAct::OrderUp(..) => {}
            PAct::FollowOpen(op) => self.follow_open = if self.follow_open.as_deref() == Some(op.as_str()) { None } else { Some(op) },
            PAct::FollowAdd(op, fam) => self.deck_edit(cx, |t, deck| t.follow_add(deck, &op, &fam, None)),
            PAct::FollowRemove(op, k) => self.deck_edit(cx, |t, deck| t.follow_remove(deck, &op, k)),
            PAct::FollowClear(op) => self.deck_edit(cx, |t, deck| t.follow_clear(deck, &op)),
            PAct::OrderAuto => self.deck_edit(cx, |t, deck| {
                let mut d = t.shown(deck);
                d.order.clear();
                Ok(d)
            }),
            PAct::UseDefault if self.terms.is_some() => self.deck = Deck::default(),
            PAct::ConfirmSkills if self.terms.is_some() => {
                let mut work = self.m(cx).clone();
                work.deck = self.deck.clone();
                let errors = work.deck_errors(&d, &format!("Unit '{}'", work.id));
                if errors.is_empty() {
                    let id = self.member.clone();
                    if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                        m.deck = work.deck;
                    }
                    self.skills_loaded = false;
                } else {
                    show_errors(cx, errors);
                }
            }
            PAct::UseDefault => {
                self.using_default = true;
                self.fill_default(cx);
                let keep: Vec<String> = self.class_slots.iter().chain(&self.extra_slots).cloned().collect();
                self.ranks.retain(|f, _| keep.contains(f));
            }
            PAct::ResetSkills => self.skills_loaded = false,
            PAct::ConfirmSkills => {
                let mut work = self.m(cx).clone();
                if self.using_default {
                    work.loadout_class.clear();
                    work.loadout_extra.clear();
                } else {
                    work.loadout_class = self.class_slots.clone();
                    work.loadout_extra = self.extra_slots.clone();
                }
                // ranks for the slotted families only; every slotted family gets an entry (the user's Q2 answer)
                let slotted: Vec<String> = self.class_slots.iter().chain(&self.extra_slots).cloned().collect();
                work.loadout_ranks = self.ranks.iter().filter(|(f, _)| slotted.contains(f)).map(|(f, r)| (f.clone(), *r)).collect();
                let errors = work.loadout_errors(&d, &format!("Unit '{}'", work.id));
                if errors.is_empty() {
                    progression::pin_ranks(&d, &mut work, &cx.gs().world_id);
                    let id = self.member.clone();
                    if let Some(m) = cx.gs_mut().party.member_mut(&id) {
                        m.loadout_class = work.loadout_class;
                        m.loadout_extra = work.loadout_extra;
                        m.loadout_ranks = work.loadout_ranks;
                    }
                    self.skills_loaded = false;
                } else {
                    show_errors(cx, errors);
                }
            }
            PAct::Unequip(slot) => self.gear_move(cx, |d, world, work, stash| take_off(d, world, work, stash, &slot)),
            PAct::Equip(item) => self.gear_move(cx, |d, world, work, stash| equip(d, world, work, stash, &item, None)),
            PAct::PouchAdd(item) => {
                let stacks = self.stacks_on(cx);
                self.gear_move(cx, |d, _, work, stash| pouch_put(d, work, stash, &item, None, stacks))
            }
            PAct::PouchRemove(i) => {
                let stacks = self.stacks_on(cx);
                self.gear_move(cx, |_, _, work, stash| pouch_take(work, stash, i, stacks))
            }
            PAct::PouchPack(item, n) => self.gear_move(cx, |d, _, work, stash| work.pack_pouch(d, &item, n, stash).map_err(|e| named(d, &item, e))),
            PAct::ResetGear => self.gear = None,
            PAct::ConfirmGear => {
                let Some((work, stash)) = self.gear.clone() else { return };
                let errors = work.validate(&d);
                if !errors.is_empty() {
                    show_errors(cx, errors);
                    return;
                }
                let id = self.member.clone();
                let gs = cx.gs_mut();
                if let Some(m) = gs.party.member_mut(&id) {
                    m.equipment = work.equipment;
                    m.pouch = work.pouch;
                    m.pouch_packed = work.pouch_packed;
                }
                gs.party.stash = stash;
                self.gear = None;
            }
        }
    }
}

impl Character {
    /// RULESET1 9 D6: one deck edit on the draft (loaded first); a refused edit says why in a notice.
    fn deck_edit(&mut self, cx: &mut Cx, edit: impl FnOnce(&DeckTerms, &Deck) -> Result<Deck, String>) {
        self.load_skills(cx);
        let Some(t) = &self.terms else { return };
        match edit(t, &self.deck) {
            Ok(d) => self.deck = d,
            Err(e) => cx.notice.push(e),
        }
    }

    /// One gear move on the draft (loaded first); a refused move shows its reason. INTENT2-U: under `pouch_stacks` the
    /// pouch is not topped up here: only the kit's auto-fill at combat start and after a fight moves copies in.
    fn gear_move(&mut self, cx: &mut Cx, mv: impl FnOnce(&Data, &str, &mut UnitState, &mut Stash) -> Result<(), String>) {
        self.load_gear(cx);
        let d = cx.data.clone();
        let world = cx.gs().world_id.clone();
        let (work, stash) = self.gear.as_mut().unwrap();
        if let Err(e) = mv(&d, &world, work, stash) {
            show_errors(cx, vec![e]);
        }
    }

    /// INTENT2-U: whether the gear draft (loaded first) plays `pouch_stacks`.
    fn stacks_on(&mut self, cx: &Cx) -> bool {
        self.load_gear(cx);
        self.gear_stacks
    }

    /// The draft after the drop of payload `from` on target `to` (GEARUI1 drag and drop), or why it can't be done.
    fn after_drop(&self, d: &Data, from: &str, to: &str) -> Result<(UnitState, Stash), String> {
        let (mut work, mut stash) = self.gear.clone().ok_or("No gear draft.")?;
        drop_move(d, &self.gear_world, &mut work, &mut stash, from, to, self.gear_stacks)?;
        Ok((work, stash))
    }
}

/// The party stash: item id -> count.
type Stash = IndexMap<String, i64>;

fn take_one(stash: &mut Stash, item: &str) {
    let left = stash.get(item).copied().unwrap_or(0) - 1;
    if left <= 0 {
        stash.shift_remove(item);
    } else {
        stash.insert(item.to_string(), left);
    }
}

fn give_one(stash: &mut Stash, item: String) {
    *stash.entry(item).or_insert(0) += 1;
}

// ---- Gear moves on a draft (GearModel equip / unequip / pouch): clicks, drops and the stat preview share them ----

/// Wears `item` from the stash in `slot`, or (None) the first free slot it fits, else the first it fits; what the slot
/// held goes back to the stash.
fn equip(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, item: &str, slot: Option<&str>) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    let slot = match slot {
        Some(s) if fits(def, s) => s,
        Some(s) => return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(s).to_lowercase())),
        None => *LOADOUT_SLOTS.iter().find(|s| fits(def, s) && work.equipment.get(**s).is_none_or(|i| i == NO_ITEM)).or_else(|| LOADOUT_SLOTS.iter().find(|s| fits(def, s))).ok_or_else(|| format!("{} can't be worn.", def.name))?,
    };
    work.can_wear(d, item, slot)?;
    if slot == "off_hand" {
        if let Some(main) = main_weapon(d, world, work).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    // PR5-26 (audit B2): an emptied slot (`NO_ITEM`) gives nothing back
    if let Some(old) = work.equipment.insert(slot.to_string(), item.to_string()).filter(|o| o != NO_ITEM) {
        give_one(stash, old);
    }
    take_one(stash, item);
    if slot == "main_hand" && def.two_handed {
        // A shield or focus goes back to the stash (the Godot dojo's "stowed: ... is 2-Handed").
        if let Some(off) = work.equipment.get("off_hand").cloned().filter(|o| d.cat.items.get(o).is_some_and(|o| !off_hand_ok(Some(def), o))) {
            work.equipment.shift_remove("off_hand");
            give_one(stash, off);
        }
    }
    Ok(())
}

/// The weapon the main hand holds: the worn one, else the class default the build gives.
fn main_weapon(d: &Data, world: &str, work: &UnitState) -> Option<emberwake_data::ItemDef> {
    match work.equipment.get("main_hand") {
        Some(id) => d.cat.items.get(id).cloned(),
        None => progression::build_as(d, work, world, false, None).and_then(|b| b.unit.inventory.get(emberwake_rules::gear::Slot::MainHand).map(|e| e.item.clone())),
    }
}

/// Takes off what `slot` holds; when the main hand's class default that comes back is two-handed, a shield or focus
/// in the off hand goes to the stash too (the two-handed rule).
fn take_off(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    unequip(d, world, work, stash, slot)?;
    if slot == "main_hand" {
        let off = work.equipment.get("off_hand").and_then(|o| d.cat.items.get(o)).cloned();
        if let (Some(main), Some(off)) = (main_weapon(d, world, work), off) {
            if !off_hand_ok(Some(&main), &off) {
                unequip(d, world, work, stash, "off_hand")?;
            }
        }
    }
    Ok(())
}

/// Takes off what `slot` holds (the class default comes back). PR5-26 (audit B2): a slot left empty (`NO_ITEM`) stays
/// so; a worn item whose class default the member's `forbid_hands` refuses leaves the slot empty (`NO_ITEM`).
fn unequip(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, slot: &str) -> Result<(), String> {
    match work.equipment.get(slot) {
        None => return Err(format!("The {} slot already holds the class default.", slot.replace('_', " "))),
        Some(i) if i == NO_ITEM => return Err(format!("The {} slot is already empty.", slot.replace('_', " "))),
        Some(_) => {}
    }
    // PR5-26 review F3: the default of the played world (as the settle's `stash_forbidden` reads it)
    let empty = work.forbids_default(d, world, slot);
    let item = work.equipment.shift_remove(slot).unwrap_or_default();
    if empty {
        work.equipment.insert(slot.to_string(), NO_ITEM.to_string());
    }
    give_one(stash, item);
    Ok(())
}

/// Moves the item worn in `from` to `to` (it must fit there); what `to` held moves to `from` when it fits, else to the
/// stash.
fn move_slot(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str) -> Result<(), String> {
    let item = work.equipment.get(from).cloned().filter(|i| i != NO_ITEM).ok_or("That slot holds the class default.")?;
    let def = d.cat.items.get(&item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if from == to || !fits(def, to) {
        return Err(format!("{} doesn't go in the {} slot.", def.name, title_case(to).to_lowercase()));
    }
    // PR5-26 (audit B6): the member's `forbid_hands` refuses the item in its new slot too
    work.can_wear(d, &item, to)?;
    if to == "off_hand" {
        let mut rest = work.clone();
        rest.equipment.shift_remove(from);
        if let Some(main) = main_weapon(d, world, &rest).filter(|m| !off_hand_ok(Some(m), def)) {
            return Err(format!("{} needs both hands: the off hand can hold only a weapon with it.", main.name));
        }
    }
    work.equipment.shift_remove(from);
    // PR5-26 review F1: an emptied slot (`NO_ITEM`) gives nothing back; the slot left behind holds `NO_ITEM` when the
    // class default that would come back is one the member's `forbid_hands` refuses (as `unequip`)
    let old = work.equipment.insert(to.to_string(), item).filter(|o| o != NO_ITEM);
    match old {
        Some(old) if d.cat.items.get(&old).is_some_and(|o| fits(o, from)) => {
            work.equipment.insert(from.to_string(), old);
        }
        Some(old) => give_one(stash, old),
        None => {}
    }
    if !work.equipment.contains_key(from) && work.forbids_default(d, world, from) {
        work.equipment.insert(from.to_string(), NO_ITEM.to_string());
    }
    Ok(())
}

/// Puts a consumable from the stash in the pouch: in place of pouch slot `at` when it is filled (that item goes back),
/// else at the end while there is room. INTENT2-U: under `pouch_stacks` (`stacks`) it assigns the item a stack slot
/// instead (`stack_put`).
fn pouch_put(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, at: Option<usize>, stacks: bool) -> Result<(), String> {
    let def = d.cat.items.get(item).ok_or_else(|| format!("Unknown item '{item}'."))?;
    if def.item_type != "CONSUMABLE" {
        return Err(format!("{} can't go in the battle pouch.", def.name));
    }
    if !def.offered_in_battle() {
        return Err(format!("{} can't be used in battle.", def.name));
    }
    if stash.get(item).copied().unwrap_or(0) < 1 {
        return Err(format!("There is no {} in the stash.", def.name));
    }
    if stacks {
        return stack_put(d, work, stash, item, at);
    }
    match at.filter(|&i| i < work.pouch.len()) {
        Some(i) => {
            let old = std::mem::replace(&mut work.pouch[i], item.to_string());
            give_one(stash, old);
        }
        None if work.pouch.len() >= POUCH_MAX => return Err(format!("The pouch holds at most {POUCH_MAX} items.")),
        None => work.pouch.push(item.to_string()),
    }
    take_one(stash, item);
    Ok(())
}

fn pouch_take(work: &mut UnitState, stash: &mut Stash, i: usize, stacks: bool) -> Result<(), String> {
    if stacks {
        return work.take_pouch_slot(i, stash).map(|_| ());
    }
    if i >= work.pouch.len() {
        return Err("That pouch slot is empty.".into());
    }
    give_one(stash, work.pouch.remove(i));
    Ok(())
}

/// INTENT2-U ("Stacks & Auto-Fill"): gives `item` a stack slot (the kit's `assign_pouch`: packed to `min(5,
/// max_stack)`, a second slot of the same item refused), in place of stack slot `at` when that holds another item (its
/// copies go back to the stash) and moved to that place (`UnitState::move_pouch_slot`). Dropped on its own slot it is a
/// no-op. No copy moves now: the kit's auto-fill fills the slot at combat start and after a fight.
fn stack_put(d: &Data, work: &mut UnitState, stash: &mut Stash, item: &str, at: Option<usize>) -> Result<(), String> {
    let slots = work.pouch_slots();
    if at.is_some_and(|i| slots.get(i).is_some_and(|s| s == item)) {
        return Ok(());
    }
    if let Some(i) = at.filter(|&i| i < slots.len() && !slots.iter().any(|s| s == item)) {
        work.take_pouch_slot(i, stash)?;
    }
    work.assign_pouch(d, item).map_err(|e| named(d, item, e))?;
    // the new slot's place: not always the last (a key-less slot, copies with no packed target, sorts after it)
    if let (Some(i), Some(now)) = (at, work.pouch_slots().iter().position(|s| s == item)) {
        if i < now {
            work.move_pouch_slot(d, now, i)?;
        }
    }
    Ok(())
}

/// A kit refusal with the item's name for its id (`'con_x' already has a pouch slot.` reads `Fire Flask already ...`).
fn named(d: &Data, item: &str, e: String) -> String {
    e.replace(&format!("'{item}'"), &d.item_name(item))
}

/// INTENT2-U: one pip of a pouch stack on the Party Screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Pip {
    /// A copy in the pouch.
    Filled,
    /// Packed but missing (spent, or the stash is short).
    Hollow,
    /// Above the packed count, up to the stack limit.
    Dim,
}

/// INTENT2-U ("Stacks & Auto-Fill"): a stack slot's pips, visual only: one per copy the stack can hold (`limit`, capped at
/// 5), filled for the `cur` copies in the pouch, hollow up to the `packed` count, dimmed above it.
pub(crate) fn pouch_pips(cur: i64, packed: i64, limit: i64) -> Vec<Pip> {
    (0..limit.min(5)).map(|k| if k < cur { Pip::Filled } else if k < packed { Pip::Hollow } else { Pip::Dim }).collect()
}

/// A drop: payload `stash.<item>`, `equip.<slot>` or `pouch.<i>` on target `slot.<slot>`, `pouch.<i>` or `stash`
/// (INTENT2-U: under `pouch_stacks`, `stacks`, a pouch index names a stack slot).
fn drop_move(d: &Data, world: &str, work: &mut UnitState, stash: &mut Stash, from: &str, to: &str, stacks: bool) -> Result<(), String> {
    let (kind, what) = from.split_once('.').unwrap_or((from, ""));
    let (target, at) = to.split_once('.').unwrap_or((to, ""));
    let index = |s: &str| s.parse::<usize>().map_err(|_| format!("Bad pouch slot '{s}'."));
    match (kind, target) {
        ("stash", "slot") => equip(d, world, work, stash, what, Some(at)),
        ("stash", "pouch") => pouch_put(d, work, stash, what, Some(index(at)?), stacks),
        ("equip", "stash") => take_off(d, world, work, stash, what),
        ("equip", "slot") => move_slot(d, world, work, stash, what, at),
        ("pouch", "stash") => pouch_take(work, stash, index(what)?, stacks),
        ("pouch", "pouch") if stacks => work.move_pouch_slot(d, index(what)?, index(at)?),
        ("pouch", "pouch") => {
            let (i, j) = (index(what)?, index(at)?);
            if i == j || i >= work.pouch.len() {
                return Err("Nothing to move.".into());
            }
            let item = work.pouch.remove(i);
            let j = j.min(work.pouch.len());
            work.pouch.insert(j, item);
            Ok(())
        }
        _ => Err("That can't go there.".into()),
    }
}

impl Screen<GameCx> for Character {
    /// At rest only the portrait moves, and slowly.
    fn frame_need(&self, _cx: &Cx) -> FrameNeed {
        FrameNeed::Within(SLOW_LIFE)
    }
    fn id(&self) -> &'static str {
        "character"
    }
    fn overlay(&self) -> bool {
        true
    }

    fn draw(&mut self, cx: &mut Cx, g: &mut Gfx, mouse: (i32, i32), live: bool) {
        self.take_answers(cx);
        let d = cx.data.clone();
        if cx.gs().party.member(&self.member).is_none() {
            self.member = cx.gs().party.members.first().map(|m| m.id.clone()).unwrap_or_default();
        }
        let m = self.m(cx).clone();
        let mut ui: Ui<PAct> = Ui::new(g, mouse, live);
        ui.p.fill(Rect::new(0, 0, W, H), th().game("popup_veil", rgba(0, 0, 0, 110)));
        let fr = Rect::new(16, 16, 608, 328);
        ui.frame(fr, "Character", true);
        ui.close_x("character.close", Rect::new(fr.x + fr.w - 26, fr.y + 8, 19, 18), PAct::Close);
        let unspent = m.unspent(&d);
        let pick = format!("{}  Lv {}{}", crate::hub::member_name(cx, &m), m.level(&d), if unspent > 0 { format!("  +{unspent}") } else { String::new() });
        let pw = ui.p.text_w(&pick, BODY + 1.0) as i32 + 12;
        ui.p.text(320.0 - pw as f32 / 2.0 - 10.0, 57.0, "Q", BODY + 1.0, th().dim);
        ui.button_full("character.member", Rect::new(320 - pw / 2, 53, pw, 17), &pick, PAct::Member(1), true, true, BODY + 1.0, tip("Next member (E)", "Q / E cycle the party."));
        ui.p.text(320.0 + pw as f32 / 2.0 + 4.0, 57.0, "E", BODY + 1.0, th().dim);
        // PORTRAITFX1-R: the member's portrait (its carried HP and conditions hold the state rows; the initial without a
        // file)
        self.faces.begin();
        let pr = Rect::new(fr.x + 10, 44, PORTRAIT, PORTRAIT);
        ui.p.panel(pr, th().game("portrait_well", rgb(22, 30, 48)), th().edge, 2.0);
        let (who, look) = crate::hub::member_portrait(&d, cx.gs(), &m);
        self.faces.draw(&mut ui.p, cx, &who, &look, "roster", pr);
        ui.push("character.portrait", "portrait", crate::hub::member_name(cx, &m), pr, true, false, None, Vec::new());
        for (i, (id, label)) in TABS.iter().enumerate() {
            let label = if *id == "level_up" { format!("{label} ({unspent})") } else { label.to_string() };
            ui.tab(&format!("character.tab.{id}"), Rect::new(174 + i as i32 * 74, 78, 70, 18), &label, PAct::Tab(id), self.tab == *id);
        }
        self.detail.clear();
        self.mouse = mouse;
        match self.tab {
            "stats" => self.draw_stats(&mut ui, cx, &m),
            "level_up" => self.draw_level(&mut ui, cx, &m),
            "skills" => self.draw_skills(&mut ui, cx, &m),
            _ => self.draw_gear(&mut ui, cx, &m),
        }
        let dbox = Rect::new(25, 275, 590, 31);
        ui.p.panel(dbox, th().game("party_box", rgba(20, 28, 46, 255)), th().edge, 2.0);
        // while dragging on the Skills tab the pane says where the skill can go (SKILLUI1), not the hovered card
        let dragging_skill = self.tab == "skills" && ui.dragging().is_some() && !self.detail.is_empty();
        let hovered = ui.widgets.iter().rev().find(|w| !dragging_skill && w.rect.contains(mouse.0, mouse.1) && w.tip.len() > 1).map(|w| w.tip[1].0.clone());
        let text = hovered.unwrap_or_else(|| if self.detail.is_empty() { "Move to a row to read what it does.".into() } else { self.detail.clone() });
        ui.para(30, 279, 580, &text, BODY, th().text);
        self.w = ui.widgets;
    }

    /// PORTRAITFX1 4.5: the portrait's clock moves by the frame's wall time (at most 100 ms a frame).
    fn tick(&mut self, _cx: &mut Cx) {
        self.faces.tick();
    }

    fn key(&mut self, cx: &mut Cx, k: Key) {
        match k {
            Key::Char('q') => self.act(cx, PAct::Member(-1)),
            Key::Char('e') => self.act(cx, PAct::Member(1)),
            _ => {}
        }
    }

    /// Esc always closes (the right-click unslot of `back` is for the mouse only, SKILLUI1 Q4).
    fn escape(&mut self, cx: &mut Cx) {
        self.act(cx, PAct::Close);
    }

    /// A right click: on the Skills tab over a filled slot it takes the skill out (SKILLUI1 Q4); anywhere else it closes.
    fn back(&mut self, cx: &mut Cx) {
        if self.tab == "skills" && self.terms.is_some() {
            let (mx, my) = self.mouse;
            // INPUT1 FL3: a right click on a follow-up entry takes it out of the list
            let follow = self.w.iter().rev().find(|w| w.rect.contains(mx, my)).and_then(|w| {
                let (op, k) = w.id.strip_prefix("skills.follow.")?.rsplit_once('.')?;
                Some((op.to_string(), k.parse::<usize>().ok()?))
            });
            if let Some((op, k)) = follow.filter(|(op, _)| !op.starts_with("add.")) {
                if self.terms.as_ref().is_some_and(|t| k < t.follow_list(&t.shown(&self.deck), &op).len()) {
                    self.act(cx, PAct::FollowRemove(op, k));
                }
                return;
            }
            let over = self.w.iter().rev().find(|w| w.rect.contains(mx, my) && DECK_KINDS.iter().any(|k| w.id.starts_with(&format!("skills.{}.", k.0))));
            if let Some(w) = over {
                let kind = DECK_KINDS.iter().find(|k| w.id.starts_with(&format!("skills.{}.", k.0))).map(|k| k.0).unwrap_or("active");
                let i: usize = w.id.rsplit('.').next().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
                if self.terms.as_ref().is_some_and(|t| i < DeckTerms::list(&t.shown(&self.deck), kind).len()) {
                    self.act(cx, PAct::DeckUnslot(kind, i));
                }
                return;
            }
        } else if self.tab == "skills" {
            let (mx, my) = self.mouse;
            let over = self.w.iter().rev().find(|w| w.rect.contains(mx, my) && (w.id.starts_with("skills.class.") || w.id.starts_with("skills.extra.")));
            if let Some(w) = over {
                let class = w.id.starts_with("skills.class.");
                let i: usize = w.id.rsplit('.').next().and_then(|s| s.parse().ok()).unwrap_or(usize::MAX);
                if i < (if class { self.class_slots.len() } else { self.extra_slots.len() }) {
                    self.act(cx, PAct::Unslot(class, i));
                }
                return;
            }
        }
        self.act(cx, PAct::Close);
    }

    fn wheel(&mut self, _cx: &mut Cx, lines: i32) {
        let at = if self.tab == "gear" { &mut self.gear_scroll } else { &mut self.scroll };
        *at = (*at as i64 - lines as i64).max(0) as usize;
    }

    /// GEARUI1 and SKILLUI1: a drop is valid when the move it makes is (the same rules as the clicks).
    fn can_drop(&self, cx: &Cx, from: &str, to: &str) -> bool {
        match self.tab {
            "gear" => self.after_drop(&cx.data, from, to).is_ok(),
            "skills" if self.terms.is_some() => self.terms.as_ref().is_some_and(|t| t.after_drop(&self.deck, from, to).is_ok()),
            "skills" => self.skills_after_drop(cx, from, to).is_ok(),
            _ => false,
        }
    }

    fn accept_drop(&mut self, cx: &mut Cx, from: &str, to: &str) {
        if self.tab == "skills" {
            self.load_skills(cx);
            if let Some(t) = &self.terms {
                match t.after_drop(&self.deck, from, to) {
                    Ok(d) => self.deck = d,
                    Err(e) => show_errors(cx, vec![e]),
                }
                return;
            }
            match self.skills_after_drop(cx, from, to) {
                Ok((class, extra, ranks)) => {
                    (self.class_slots, self.extra_slots, self.ranks) = (class, extra, ranks);
                    self.using_default = false;
                }
                Err(e) => show_errors(cx, vec![e]),
            }
            return;
        }
        self.load_gear(cx);
        match self.after_drop(&cx.data, from, to) {
            Ok(draft) => self.gear = Some(draft),
            Err(e) => show_errors(cx, vec![e]),
        }
    }

    fn state(&self, cx: &Cx) -> Value {
        let mut out = json!({"member": self.member, "tab": self.tab, "draft": self.draft.iter().map(|(k, v)| (k.clone(), json!(v))).collect::<serde_json::Map<String, Value>>(), "has_draft": self.has_draft(cx), "portraits": self.faces.state()});
        if let (true, Some(t)) = (self.tab == "skills", &self.terms) {
            let shown = t.shown(&self.deck);
            out["deck"] = deck_json(&self.deck);
            out["deck_shown"] = deck_json(&shown);
            out["deck_auto"] = json!(self.deck.is_empty());
            out["deck_view"] = json!(self.deck_view);
            out["deck_order"] = Value::Object(t.intents.iter().map(|i| (i.clone(), json!(t.order(&shown, i)))).collect());
            out["basics"] = json!(t.basics);
        } else if self.tab == "skills" {
            out["class"] = json!(self.class_slots);
            out["extra"] = json!(self.extra_slots);
            out["ranks"] = Value::Object(self.ranks.iter().map(|(k, v)| (k.clone(), json!(v))).collect());
            out["using_default"] = json!(self.using_default);
        }
        out
    }

    widget_plumbing!();
}

impl Character {
    fn draw_stats(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        let d = &cx.data;
        let gs = cx.gs();
        let lvl = m.level(d);
        let end = ui.p.text(68.0, 104.0, &format!("Level {lvl}  ·  XP {} ({} to next)  ·  {} {}", m.total_xp, progression::xp_to_next(d, m.total_xp), crate::hud::term_short(cx, emberwake_world::terms::CURRENCY), gs.party.gold), BODY + 1.0, th().text);
        // ROSTER1-U W2: a wounded member's mark and the HP it takes the field with (the act's cut max)
        let (hp, max, wounded) = crate::hub::field_hp(cx, m);
        if wounded {
            let mark = "Wounded";
            let x = end + 12.0;
            let after = ui.p.text(x, 104.0, mark, BODY + 1.0, th().foe);
            ui.p.text(after + 5.0, 104.0, &format!("HP {hp} / {max}"), BODY + 1.0, th().dim);
            let w = ui.p.text_w(&format!("{mark} HP {hp} / {max}"), BODY + 1.0) + 5.0;
            let lh = (ui.p.line_h(BODY + 1.0) + 2.0).ceil() as i32;
            ui.push("stats.wounded", "row", format!("{mark} HP {hp} / {max}"), Rect::new(x as i32 - 2, 102, w as i32 + 4, lh), true, false, None, tip(mark, &crate::hub::wounded_tip(cx, m)));
        }
        let active = gs.party.active_members().iter().any(|a| a.id == m.id);
        let n = gs.party.active_members().len();
        ui.checkbox("character.active", 72, 122, &format!("In party ({n}/{})", gs.party.max_active), active, PAct::ToggleActive, true);
        let hint_end = ui.p.text(155.0, 122.0, "Keep at least one active member.", BODY + 1.0, th().dim);
        // ROSTER1-U BK1: the bunk chip in a world with a roster block
        if let Some(bunks) = crate::hub::bunks_text(gs) {
            let bw = ui.p.text_w(&bunks, BODY + 1.0);
            ui.p.text((600.0 - bw).max(hint_end + 12.0), 122.0, &bunks, BODY + 1.0, th().dim);
        }
        let Some(arch) = d.cat.classes.get(&m.class_id(d)) else { return };
        let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
        if spend.is_empty() {
            spend.insert(STATS[0].into(), 0);
        }
        let cs = emberwake_rules::stats::CharacterStats::new(arch, lvl as i32, &spend);
        let cols = columns(ui, &[(68.0, "Stat"), (112.0, "Base"), (148.0, "Growth"), (190.0, "Spent"), (226.0, "Total")], 10.0);
        for (&(x, _), h) in cols.iter().zip(["Stat", "Base", "Growth", "Spent", "Total"]) {
            ui.p.text(x, 143.0, h, BODY + 1.0, th().dim);
        }
        let table_right = cols[4].1;
        for (i, st) in STATS.iter().enumerate() {
            let y = 158.0 + i as f32 * 15.0;
            let base = arch.base_stats.get(*st).copied().unwrap_or(0);
            let sp = m.spend.get(*st).copied().unwrap_or(0) as i32;
            let total = cs.base(st);
            ui.p.text(68.0, y, st, BODY + 1.0, th().text);
            for (c, v) in cols[1..].iter().zip([base, total - base - sp, sp, total]) {
                ui.p.text_right(c.1, y, &v.to_string(), BODY + 1.0, th().text);
            }
            ui.push(format!("stats.{st}"), "row", format!("{st} {total}"), Rect::new(66, y as i32 - 2, (table_right - 62.0) as i32, 14), true, false, None, tip(st, &stat_desc(st)));
        }
        let rows = derived(d, &cx.gs().world_id, &party_ruleset(cx), m);
        let label_x = 250f32.max(table_right + 6.0);
        let value_x = 325f32.max(label_x + rows.iter().map(|r| ui.p.text_w(&r.label, BODY + 1.0)).fold(0.0, f32::max) + 8.0);
        // DERIVE1 4.4: the built-in rows keep their 15 px step; catalog rows past them close the step up to stay above
        // the detail pane (the last row starts at 263 at most)
        let step = 15f32.min(120.0 / (rows.len().max(2) - 1) as f32);
        for (i, r) in rows.iter().enumerate() {
            let y = 143.0 + i as f32 * step;
            ui.p.text(label_x, y, &r.label, BODY + 1.0, th().dim);
            ui.p.text(value_x, y, &r.shown, BODY + 1.0, th().text);
            ui.push(format!("stats.derived.{}", r.label), "row", format!("{} {}", r.label, r.shown), Rect::new(label_x as i32 - 2, y as i32 - 2, (value_x - label_x) as i32 + 26, step.min(14.0) as i32), true, false, None, tip(&r.label, &r.desc));
        }
        // the re-gathered list (FB-63 (2); a hero's temperament is not in the save)
        let owned = progression::member_passives(d, m);
        let names: Vec<String> = owned.iter().map(|p| s_or(d.entry("passives", p), "name", p)).collect();
        let line = format!("Passives: {}", if names.is_empty() { "none".into() } else { names.join(", ") });
        ui.p.text_fit(68.0, 258.0, &line, BODY + 1.0, label_x - 78.0, th().dim);
        let about: Vec<String> = owned.iter().map(|p| format!("{}: {}", s_or(d.entry("passives", p), "name", p), json::s(d.entry("passives", p), "description"))).collect();
        let line_h = (ui.p.line_h(BODY + 1.0) + 2.0).ceil().max(13.0) as i32;
        ui.push("stats.passives", "row", line, Rect::new(66, 256, (label_x - 74.0) as i32, line_h), true, false, None, tip("Passives", &if about.is_empty() { "None yet: passives come from the hero's background, faction, disciplines and named items.".into() } else { about.join(" ") }));
    }

    fn draw_level(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        let d = &cx.data;
        let drafted: i64 = self.draft.values().sum();
        let left = m.unspent(d) - drafted;
        ui.p.text(70.0, 104.0, &format!("Points left: {left}   (confirmed points are permanent)"), BODY + 1.0, th().dim);
        let Some(arch) = d.cat.classes.get(&m.class_id(d)) else { return };
        let mut spend: IndexMap<String, i32> = m.spend.iter().map(|(k, v)| (k.clone(), *v as i32)).collect();
        spend.entry(STATS[0].into()).or_insert(0);
        let cs = emberwake_rules::stats::CharacterStats::new(arch, m.level(d) as i32, &spend);
        let cols = columns(ui, &[(70.0, "Stat"), (105.0, "Now"), (132.0, "Added"), (165.0, "New")], 8.0);
        for (&(x, _), h) in cols.iter().zip(["Stat", "Now", "Added", "New"]) {
            ui.p.text(x, 122.0, h, BODY + 1.0, th().dim);
        }
        let minus_x = 195.max(cols[3].1 as i32 + 8);
        let auto_x = 266.max(minus_x + 71);
        for (i, st) in STATS.iter().enumerate() {
            let y = 136 + i as i32 * 21;
            let now = cs.base(st) as i64;
            let add = self.draft.get(*st).copied().unwrap_or(0);
            ui.p.text(70.0, y as f32 + 4.0, st, BODY + 1.0, th().text);
            ui.p.text_right(cols[1].1, y as f32 + 4.0, &now.to_string(), BODY + 1.0, th().text);
            ui.p.text_right(cols[2].1, y as f32 + 4.0, &add.to_string(), BODY + 1.0, if add > 0 { th().gold } else { th().dim });
            ui.p.text_right(cols[3].1, y as f32 + 4.0, &(now + add).to_string(), BODY + 1.0, th().text);
            ui.push(format!("level.{st}"), "row", format!("{st} {now} + {add}"), Rect::new(68, y, minus_x - 73, 17), true, false, None, tip(st, &stat_desc(st)));
            ui.button_tip(&format!("level.{st}.minus"), Rect::new(minus_x, y, 23, 17), "-", PAct::Remove(st), add > 0, tip(&format!("-1 {st}"), "Take back a drafted point. Nothing is spent until you Confirm."));
            ui.button_tip(&format!("level.{st}.plus"), Rect::new(minus_x + 31, y, 23, 17), "+", PAct::Add(st), left > 0, tip(&format!("+1 {st}"), &stat_desc(st)));
        }
        ui.button_tip("level.auto", Rect::new(auto_x, 121, 69, 17), "Auto", PAct::Auto, left > 0, tip("Auto", "Spend the remaining points by the class growth weights."));
        ui.button_tip("level.revert", Rect::new(auto_x, 145, 69, 17), "Revert", PAct::Revert, drafted > 0, tip("Revert", "Clear the drafted points. Nothing is spent until you Confirm."));
        let mut work = m.clone();
        for (k, v) in &self.draft {
            *work.spend.entry(k.clone()).or_insert(0) += v;
        }
        let ruleset = party_ruleset(cx);
        let world = &cx.gs().world_id;
        let (before, after) = (derived(d, world, &ruleset, m), derived(d, world, &ruleset, &work));
        let label_x = 352f32.max((auto_x + 86) as f32);
        let value_x = 427f32.max(label_x + after.iter().map(|r| ui.p.text_w(&r.label, BODY + 1.0)).fold(0.0, f32::max) + 8.0);
        // DERIVE1 4.4: as on the Stats tab, more rows close the step up (the last row starts at 257 at most)
        let step = 15f32.min(135.0 / (after.len().max(2) - 1) as f32);
        for (i, r) in after.iter().enumerate() {
            let y = 122.0 + i as f32 * step;
            let old = before.get(i).filter(|b| b.value != r.value);
            let text = old.map_or_else(|| r.shown.clone(), |b| format!("{} -> {}", b.shown, r.shown));
            ui.p.text(label_x, y, &r.label, BODY + 1.0, th().dim);
            ui.p.text(value_x, y, &text, BODY + 1.0, if old.is_some() { th().gold } else { th().text });
            ui.push(format!("level.derived.{}", r.label), "row", format!("{} {}", r.label, r.shown), Rect::new(label_x as i32 - 2, y as i32 - 2, (value_x - label_x) as i32 + 55, step.min(14.0) as i32), true, false, None, tip(&r.label, &r.desc));
        }
        ui.button("level.confirm", Rect::new(500, 312, 70, 18), "Confirm", PAct::ConfirmLevel, drafted > 0);
    }

    fn draw_skills(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_skills(cx);
        if let Some(t) = self.terms.clone() {
            return self.draw_deck(ui, cx, m, &t);
        }
        let d = &cx.data;
        let cat = &d.cat;
        let Some(b) = progression::build_with(d, m, &cx.gs().world_id, false) else { return };
        let u = &b.unit;
        // FB-107: skill text as the world's ruleset shows it (no stance tokens with stances off)
        let stances = d.ruleset(&cx.gs().world_id).stances();
        let fam_name = |f: &str| cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
        let row_of = |f: &str, rank: i32| cat.family_rows(f).into_iter().find(|s| s.tier == rank);
        let path = self.path_families(cx);
        let max_class = emberwake_rules::stats::MAX_CLASS_SLOTS;
        let max_extra = emberwake_rules::stats::MAX_EXTRA_SLOTS;
        // Left: the fixed slots, five class and three extra (SKILLUI1 section 4).
        ui.p.text(64.0, 104.0, &format!("Class slots {} / {max_class}", self.class_slots.len()), BODY + 1.0, th().info);
        ui.p.text(150.0, 104.0, if self.using_default { "Default loadout" } else { "Custom loadout" }, BODY, th().dim);
        ui.p.text(64.0, 205.0, &format!("Extra slots {} / {max_extra}", self.extra_slots.len()), BODY + 1.0, th().info);
        for (class, top_y, count) in [(true, 116, max_class), (false, 216, max_extra)] {
            let list = if class { self.class_slots.clone() } else { self.extra_slots.clone() };
            let g = if class { "class" } else { "extra" };
            for i in 0..count {
                let r = Rect::new(64, top_y + i as i32 * 17, 236, 15);
                let id = format!("skills.{g}.{i}");
                let Some(f) = list.get(i) else {
                    ui.p.panel(r, th().game("slot_empty", rgba(20, 28, 46, 120)), th().game("slot_empty_edge", rgba(58, 78, 116, 120)), 2.0);
                    ui.p.text(r.x as f32 + 4.0, r.y as f32 + 3.0, "Empty", BODY, th().dim);
                    let label = if class { "class" } else { "extra" };
                    ui.push(id, "row", "Empty", r, true, false, None, tip(&format!("Empty {label} slot"), "Drag a skill here, or click a skill in the list."));
                    ui.drop_target(format!("slot.{g}.{i}"));
                    continue;
                };
                let ranks = u.selectable_ranks(cat, f);
                let top = ranks.last().copied().unwrap_or(0);
                let chosen = self.ranks.get(f).map(|&r| r as i32);
                let rank = draft_rank(&ranks, chosen);
                let at = ranks.iter().position(|&x| x == rank).unwrap_or(0);
                let row = row_of(f, rank);
                let ap = row.map(|s| emberwake_battle::skill::ap_cost(s, Some(u))).unwrap_or(0);
                let desc = row.map(|s| emberwake_content::source::skill_description(&s.raw, stances)).unwrap_or_default();
                ui.p.panel(r, th().game("slot_filled", rgba(26, 34, 54, 255)), th().edge, 2.0);
                ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 3.0, &fam_name(f), BODY, 112.0, th().text);
                let card = vec![(fam_name(f), th().gold), (desc.clone(), th().text), ("Drag to another slot or back to the list. Right click takes it out.".into(), th().dim)];
                ui.push(id, "row", fam_name(f), r, true, false, None, card);
                ui.draggable(format!("slot.{g}.{i}"));
                ui.drop_target(format!("slot.{g}.{i}"));
                // < Rank >: step over the unlocked ranks; GOLD when kept below the highest unlocked rank
                let lines = emberwake_world::skill_text::rank_lines(cat, u, f);
                let step_tip = |to: Option<i32>| -> aegis_ui::ui::Tip {
                    let Some(t) = to else { return Vec::new() };
                    let line = lines.iter().find(|l| l.tier == t).map(|l| l.text.clone()).unwrap_or_default();
                    let to_ap = lines.iter().find(|l| l.tier == t).map(|l| l.ap).unwrap_or(ap);
                    let cost = match to_ap.cmp(&ap) {
                        std::cmp::Ordering::Less => format!("Cheaper: {to_ap} AP (now {ap})"),
                        std::cmp::Ordering::Greater => format!("Stronger: {to_ap} AP (now {ap})"),
                        std::cmp::Ordering::Equal => format!("Same cost: {ap} AP"),
                    };
                    vec![(format!("Rank {}", roman(t)), th().gold), (line, th().text), (cost, th().dim)]
                };
                let down = at.checked_sub(1).and_then(|k| ranks.get(k).copied());
                let up = ranks.get(at + 1).copied();
                ui.button_full(&format!("skills.rank.{g}.{i}.down"), Rect::new(r.x + 120, r.y + 1, 13, 13), "<", PAct::RankStep(class, i, -1), down.is_some(), false, BODY, step_tip(down));
                let mut num_tip = vec![(format!("{}: Rank {} of {}", fam_name(f), roman(rank), roman(top)), th().gold)];
                num_tip.extend(lines.iter().map(|l| (format!("{}: {}{}", roman(l.tier), l.text, if l.tier == rank { " (chosen)" } else { "" }), if l.unlocked { th().text } else { th().dim })));
                num_tip.push((if chosen.is_some() { "Stays at this rank when new ranks unlock; change it with < and >." } else { "Follows the top rank until you confirm; from then on it stays put." }.into(), th().dim));
                ui.p.text(r.x as f32 + 136.0, r.y as f32 + 3.0, roman(rank), BODY, if rank < top { th().gold } else { th().text });
                ui.push(format!("skills.rank.{g}.{i}"), "row", roman(rank), Rect::new(r.x + 134, r.y, 22, 15), true, false, None, num_tip);
                ui.button_full(&format!("skills.rank.{g}.{i}.up"), Rect::new(r.x + 157, r.y + 1, 13, 13), ">", PAct::RankStep(class, i, 1), up.is_some(), false, BODY, step_tip(up));
                ui.p.text(r.x as f32 + 176.0, r.y as f32 + 3.0, &format!("{ap} AP"), BODY, th().dim);
            }
        }
        // Right: the owned skills; the whole column takes slots dropped back (recorded under the rows).
        let list_area = Rect::new(310, 114, 294, 156);
        ui.p.text(310.0, 104.0, "Owned skills (drag onto a slot or click)", BODY + 1.0, th().info);
        ui.push("skills.list", "panel", "Owned skills", list_area, true, false, None, tip("Owned skills", "Drop a slot here to take the skill out."));
        ui.drop_target("list");
        let owned: Vec<String> = u.owned_families(cat).into_iter().filter(|f| !cat.is_basic_family(f)).collect();
        // AUDIT-05 (OFFHAND1): the worn items' skills close the list, read-only: open while worn, no slot
        let worn: Vec<String> = u.gear_skills.clone();
        let total = owned.len() + worn.len();
        const ROWS: usize = 6;
        self.scroll = self.scroll.min(total.saturating_sub(ROWS));
        if self.scroll > 0 || total > ROWS {
            ui.p.text_right(604.0, 104.0, &format!("{}-{} of {}", self.scroll + 1, (self.scroll + ROWS).min(total), total), SMALL, th().dim);
        }
        for (k, id) in worn.iter().enumerate().map(|(j, id)| (owned.len() + j, id)).filter(|(k, _)| (self.scroll..self.scroll + ROWS).contains(k)) {
            let y = 118 + (k - self.scroll) as i32 * 22;
            let f = cat.family_of(id).to_string();
            let top = u.selectable_ranks(cat, &f).last().copied().unwrap_or(1);
            let row = row_of(&f, top).or_else(|| cat.tier_one(&f));
            let ap = row.map(|s| emberwake_battle::skill::ap_cost(s, Some(u))).unwrap_or(0);
            let desc = row.map(|s| emberwake_content::source::skill_description(&s.raw, stances)).unwrap_or_default();
            ui.button(&format!("skills.slot.worn.{id}"), Rect::new(310, y, 50, 16), "Worn", PAct::Slot(f.clone()), false);
            ui.p.text_fit(366.0, y as f32 + 1.0, &format!("{}  {ap} AP  from worn gear", fam_name(&f)), BODY, 238.0, th().gold);
            ui.p.text_fit(366.0, y as f32 + 10.0, &desc, SMALL, 238.0, th().dim);
            let card = vec![(fam_name(&f), th().gold), (format!("Skill from worn gear, {ap} AP"), th().info), (desc, th().text), ("Open while the item is worn: it takes no slot, and taking the item off removes it.".into(), th().dim)];
            ui.push(format!("skills.worn.{id}"), "row", fam_name(&f), Rect::new(366, y, 238, 20), true, false, None, card);
        }
        for (k, f) in owned.iter().skip(self.scroll).take(ROWS).enumerate() {
            let y = 118 + k as i32 * 22;
            let top = u.selectable_ranks(cat, f).last().copied().unwrap_or(0);
            let row = row_of(f, top);
            let slotted = self.class_slots.contains(f) || self.extra_slots.contains(f);
            let on_path = path.contains(f);
            let full = !slotted && self.extra_slots.len() >= max_extra && (!on_path || self.class_slots.len() >= max_class);
            ui.button(&format!("skills.slot.{f}"), Rect::new(310, y, 50, 16), if slotted { "Slotted" } else if full { "Full" } else { "Slot" }, PAct::Slot(f.clone()), !slotted && !full);
            let grade: String = cat.family_rows(f).first().map(|s| { let mut c = s.grade.chars(); c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str()).unwrap_or_default() }).unwrap_or_default();
            let desc = row.map(|s| emberwake_content::source::skill_description(&s.raw, stances)).unwrap_or_default();
            ui.p.text_fit(366.0, y as f32 + 1.0, &format!("{}  [{}]  Rank {}", fam_name(f), grade, roman(top)), BODY, 238.0, th().text);
            ui.p.text_fit(366.0, y as f32 + 10.0, &desc, SMALL, 238.0, th().dim);
            ui.push(format!("skills.family.{f}"), "row", fam_name(f), Rect::new(366, y, 238, 20), true, slotted, Some(PAct::Slot(f.clone())), tip(&fam_name(f), &desc));
            ui.draggable(format!("skill.{f}"));
        }
        // While a skill is dragged the pane says where it can go.
        if let Some(from) = ui.dragging().map(String::from) {
            let (mx, my) = ui.mouse;
            let target = ui.widgets.iter().rev().find(|x| x.drop.is_some() && x.rect.contains(mx, my)).and_then(|x| x.drop.clone());
            self.detail = match target.as_deref().and_then(|t| t.strip_prefix("slot.")).and_then(|t| t.split_once('.')) {
                Some((g, i)) if self.skills_after_drop(cx, &from, &format!("slot.{g}.{i}")).is_ok() => {
                    let list = if g == "class" { &self.class_slots } else { &self.extra_slots };
                    match list.get(i.parse::<usize>().unwrap_or(usize::MAX)) {
                        Some(old) if from.strip_prefix("skill.").is_some_and(|f| !self.class_slots.iter().chain(&self.extra_slots).any(|x| x == f)) => format!("Replaces {}.", fam_name(old)),
                        Some(old) => format!("Swaps with {}.", fam_name(old)),
                        None => "Drop to slot it here.".into(),
                    }
                }
                _ => "Drop on a class slot (class path skills) or an extra slot; drop on the list to take it out.".into(),
            };
        }
        ui.button("skills.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetSkills, true);
        ui.button("skills.default", Rect::new(144, 312, 70, 18), "Use default", PAct::UseDefault, !self.using_default);
        ui.button_tip("skills.clear", Rect::new(220, 312, 70, 18), "Clear all", PAct::ClearAll, !self.class_slots.is_empty() || !self.extra_slots.is_empty(), tip("Clear all", "Empties every class and extra slot in the draft (an empty loadout means the class default)."));
        ui.button("skills.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmSkills, self.has_draft(cx));
    }

    /// RULESET1 9 D6: the Skills tab under the deck rule. Left: three buckets (Actives, Passives, Keystone; filled by
    /// click or drag like today's slots, a right click takes one out) and the read-only Basics row. Right: the owned
    /// list or the order editor (one short list per intent of its actives and basics; drag to reorder, a click moves an
    /// entry one place up).
    fn draw_deck(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState, t: &DeckTerms) {
        let d = &cx.data;
        let cat = &d.cat;
        let rs = d.ruleset(&cx.gs().world_id);
        let built = progression::build_with(d, m, &cx.gs().world_id, false).map(|b| b.unit);
        let shown = t.shown(&self.deck);
        let intent_name = |i: &str| emberwake_battle::round_intent::RoundIntent::parse(i).map(|x| x.label(cat)).unwrap_or_else(|| title_case(i));
        // (name, intent, AP, rank, description) of family `f` at the rank the member uses now (its top unlocked)
        let skill_of = |f: &str| -> (String, String, i32, i32, String) {
            let top = built.as_ref().and_then(|u| u.selectable_ranks(cat, f).last().copied()).unwrap_or(0);
            let row = cat.family_rows(f).into_iter().find(|s| s.tier == top).or_else(|| cat.tier_one(f));
            let name = cat.tier_one(f).map(|s| s.name.clone()).unwrap_or_else(|| f.to_string());
            let intent = intent_name(t.intent_of.get(f).map(String::as_str).unwrap_or(""));
            match row {
                Some(s) => (name, intent, emberwake_battle::skill::ap_cost_by(s, built.as_ref(), &rs), top, emberwake_content::source::skill_description(&s.raw, rs.stances())),
                None => (name, intent, 0, top, String::new()),
            }
        };
        let passive_of = |p: &str| (s_or(d.entry("passives", p), "name", p), json::s(d.entry("passives", p), "description"));
        let name_of = |kind: &str, id: &str| if kind == "active" { skill_of(id).0 } else { passive_of(id).0 };
        // Left: the buckets.
        let mut y = 104;
        for (kind, label, takes) in DECK_KINDS {
            let list = DeckTerms::list(&shown, kind);
            let cap = t.cap(kind);
            ui.p.text(64.0, y as f32, &format!("{label} {} / {cap}", list.len()), BODY + 1.0, th().info);
            if kind == "active" {
                ui.p.text(150.0, y as f32, if self.deck.is_empty() { "Automatic deck" } else { "Custom deck" }, BODY, th().dim);
            }
            y += 11;
            for i in 0..cap {
                let r = Rect::new(64, y, 236, 13);
                let (id, key) = (format!("skills.{kind}.{i}"), format!("deck.{kind}.{i}"));
                match list.get(i) {
                    None => {
                        ui.p.panel(r, th().game("slot_empty", rgba(20, 28, 46, 120)), th().game("slot_empty_edge", rgba(58, 78, 116, 120)), 2.0);
                        ui.p.text(r.x as f32 + 4.0, r.y as f32 + 2.0, "Empty", BODY, th().dim);
                        let help = if t.owned(kind).is_empty() && kind != "active" {
                            format!("Holds one of the hero's {takes}; none owned yet: they come from the hero's background, faction, disciplines and named items.")
                        } else {
                            format!("Holds one of the hero's {takes}: drag one here, or click it in the owned list.")
                        };
                        ui.push(id, "row", "Empty", r, true, false, None, tip(&format!("Empty {kind} slot"), &help));
                    }
                    Some(f) => {
                        ui.p.panel(r, th().game("slot_filled", rgba(26, 34, 54, 255)), th().edge, 2.0);
                        let card = if kind == "active" {
                            let (name, intent, ap, rank, desc) = skill_of(f);
                            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 2.0, &name, BODY, 140.0, th().text);
                            ui.p.text_fit(r.x as f32 + 148.0, r.y as f32 + 3.0, &format!("{intent} {}", roman(rank)), SMALL, 56.0, th().dim);
                            ui.p.text_right(r.x as f32 + 232.0, r.y as f32 + 2.0, &format!("{ap} AP"), BODY, th().dim);
                            vec![(name, th().gold), (format!("{intent} skill, Rank {}, {ap} AP", roman(rank)), th().info), (desc, th().text), ("Drag to another slot or back to the list. Right click takes it out.".into(), th().dim)]
                        } else {
                            let (name, desc) = passive_of(f);
                            ui.p.text_fit(r.x as f32 + 4.0, r.y as f32 + 2.0, &name, BODY, 228.0, th().text);
                            vec![(name, th().gold), (desc, th().text), ("Applies only while slotted. Drag to another slot or back to the list. Right click takes it out.".into(), th().dim)]
                        };
                        ui.push(id, "row", name_of(kind, f), r, true, false, None, card);
                        ui.draggable(key.clone());
                    }
                }
                ui.drop_target(key);
                y += 15;
            }
            y += 3;
        }
        // The basics: open to every unit, no slot, never removed (D1).
        ui.p.text(64.0, y as f32, "Basics", BODY + 1.0, th().info);
        ui.p.text(100.0, y as f32 + 1.0, "open to every unit, no slot", BODY, th().dim);
        for (k, f) in t.basics.iter().enumerate() {
            let r = Rect::new(64 + (k % 3) as i32 * 79, y + 11 + (k / 3) as i32 * 12, 76, 11);
            let (name, intent, ap, _, desc) = skill_of(f);
            ui.p.panel(r, th().game("slot_open", rgba(22, 30, 48, 255)), th().game("slot_open_edge", rgba(58, 78, 116, 200)), 2.0);
            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 2.0, &name, SMALL, 70.0, th().text);
            let line = if t.intent_of.get(f).is_some_and(|i| t.intents.contains(i)) { format!("{intent} basic, {ap} AP") } else { format!("Basic for any intent, {ap} AP") };
            ui.push(format!("skills.basic.{f}"), "row", name.clone(), r, true, false, None, vec![(name, th().gold), (line, th().info), (desc, th().text), ("Open to every unit: takes no slot and cannot be taken out.".into(), th().dim)]);
        }
        // Right: the owned list or the order editor.
        ui.tab("skills.view.owned", Rect::new(310, 100, 70, 14), "Owned", PAct::DeckView("owned"), self.deck_view == "owned");
        ui.tab("skills.view.order", Rect::new(384, 100, 100, 14), "Order by intent", PAct::DeckView("order"), self.deck_view == "order");
        if self.deck_view == "order" {
            let n = t.intents.len().max(1) as i32;
            let col_w = 294 / n;
            // INPUT1 FL3: the open follow-up list sits under its opener's entry across the pane; the places after that
            // row move down by its height (nothing is drawn below the pane's end, y 272)
            let open: Option<(String, String, usize)> = self.follow_open.clone().and_then(|op| t.intents.iter().find_map(|i| t.order(&shown, i).iter().position(|f| *f == op).map(|k| (op.clone(), i.clone(), k))));
            let (follow, adds) = open.as_ref().map(|(op, _, _)| (t.follow_list(&shown, op), t.follow_candidates(&shown, op))).unwrap_or_default();
            let lines = |n: usize| n.div_ceil(FOLLOW_PER) as i32;
            let ed_h = if open.is_some() { 16 + 15 * (lines(follow.len() + 1) + lines(adds.len())) + 2 } else { 0 };
            let after = open.as_ref().map(|o| o.2);
            let y_of = |k: usize| 132 + k as i32 * 15 + if after.is_some_and(|a| k > a) { ed_h } else { 0 };
            let mut bottom = 132;
            for (c, intent) in t.intents.iter().enumerate() {
                let x = 310 + c as i32 * col_w;
                let name = intent_name(intent);
                ui.p.text_fit(x as f32 + 2.0, 118.0, &name, BODY + 1.0, (col_w - 6) as f32, th().info);
                ui.push(format!("skills.order.{intent}"), "row", name.clone(), Rect::new(x, 116, col_w - 4, 15), true, false, None, tip(&name, &format!("In {name} the planner fills each round from the top of this list down (its actives and basics). Drag to reorder; a click moves a skill one place up.")));
                let list = t.order(&shown, intent);
                if list.is_empty() {
                    ui.p.text(x as f32 + 2.0, 134.0, "None", BODY, th().dim);
                }
                for (k, f) in list.iter().enumerate() {
                    let r = Rect::new(x, y_of(k), col_w - 4 - FOLLOW_TOGGLE - 1, 13);
                    if r.y + r.h > 272 {
                        break;
                    }
                    bottom = bottom.max(r.y + r.h);
                    let basic = t.basics.contains(f);
                    let set = !t.follow_list(&shown, f).is_empty();
                    let (fname, _, ap, _, desc) = skill_of(f);
                    ui.p.panel(r, if basic { th().game("slot_open", rgba(22, 30, 48, 255)) } else { th().game("slot_filled", rgba(26, 34, 54, 255)) }, th().edge, 2.0);
                    let mark = if set { 7 } else { 0 };
                    ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 2.0, &format!("{}. {fname}", k + 1), BODY, (r.w - 6 - mark) as f32, if basic { th().info } else { th().text });
                    if set {
                        // FL3: an entry with a set follow-up list shows a small "+"
                        ui.p.text((r.x + r.w - 7) as f32, r.y as f32 + 2.0, "+", BODY, th().gold);
                    }
                    let card = vec![(fname.clone(), th().gold), (format!("{} {name}, {ap} AP: place {} of {}", if basic { "Basic," } else { "Active," }, k + 1, list.len()), th().info), (desc, th().text), ("Drag to reorder; a click moves it one place up.".into(), th().dim)];
                    ui.push(format!("skills.order.{intent}.{k}"), "row", fname.clone(), r, true, false, (k > 0).then(|| PAct::OrderUp(intent.clone(), k)), card);
                    ui.draggable(format!("order.{intent}.{k}"));
                    ui.drop_target(format!("order.{intent}.{k}"));
                    let is_open = after == Some(k) && open.as_ref().is_some_and(|o| o.0 == *f);
                    let line = if set { format!("After {fname} the planner tries its follow-up list first, then the {name} order.") } else { format!("After {fname} the planner fills from the {name} order; set a follow-up list to try other skills first.") };
                    ui.button_tip(&format!("skills.follow.{f}"), Rect::new(r.x + r.w + 1, r.y, FOLLOW_TOGGLE, 13), if is_open { "v" } else { ">" }, PAct::FollowOpen(f.clone()), true, tip("Follow-ups", &line));
                }
            }
            if let Some((op, intent, k)) = &open {
                let y0 = 132 + (*k as i32 + 1) * 15;
                let iname = intent_name(intent);
                let oname = skill_of(op).0;
                ui.p.panel(Rect::new(310, y0 - 1, 294, (ed_h - 1).min(273 - y0)), th().game("editor_pane", rgba(16, 22, 38, 255)), th().edge, 2.0);
                ui.p.text_fit(314.0, y0 as f32 + 2.0, &format!("After {oname}:"), BODY, 168.0, th().gold);
                ui.button_tip(&format!("skills.follow.clear.{op}"), Rect::new(490, y0, 112, 13), &format!("Use the {iname} order"), PAct::FollowClear(op.clone()), !follow.is_empty(), tip("Use the order", &format!("Empties this list: after {oname} the planner fills from the {iname} order.")));
                let y = y0 + 16;
                for j in 0..=follow.len() {
                    let r = Rect::new(312 + (j % FOLLOW_PER) as i32 * 58, y + (j / FOLLOW_PER) as i32 * 15, 56, 13);
                    if r.y + r.h > 272 {
                        break;
                    }
                    bottom = bottom.max(r.y + r.h);
                    match follow.get(j) {
                        Some(f) => {
                            let (fname, _, ap, _, desc) = skill_of(f);
                            ui.p.panel(r, th().game("slot_filled", rgba(26, 34, 54, 255)), th().edge, 2.0);
                            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 2.0, &format!("{}. {fname}", j + 1), BODY, (r.w - 6) as f32, th().text);
                            let card = vec![(fname.clone(), th().gold), (format!("Follow-up {} of {} after {oname}, {ap} AP", j + 1, follow.len()), th().info), (desc, th().text), ("Drag to reorder. Right click takes it out.".into(), th().dim)];
                            ui.push(format!("skills.follow.{op}.{j}"), "row", fname, r, true, false, None, card);
                            ui.draggable(format!("follow.{op}.{j}"));
                        }
                        None => {
                            ui.p.panel(r, th().game("slot_empty", rgba(20, 28, 46, 120)), th().game("slot_empty_edge", rgba(58, 78, 116, 120)), 2.0);
                            let label = if follow.is_empty() { "None set" } else { "Drop here" };
                            ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 2.0, label, BODY, (r.w - 6) as f32, th().dim);
                            ui.push(format!("skills.follow.{op}.end"), "row", label, r, true, false, None, tip("Follow-ups", &format!("Drop one of the {iname} skills here to try it after {oname}; with no list the {iname} order fills.")));
                        }
                    }
                    ui.drop_target(format!("follow.{op}.{j}"));
                }
                let y = y + 15 * lines(follow.len() + 1);
                if !adds.is_empty() && y + 13 <= 272 {
                    ui.p.text(314.0, y as f32 + 2.0, "Add:", BODY, th().info);
                }
                for (j, f) in adds.iter().enumerate() {
                    let r = Rect::new(340 + (j % FOLLOW_PER) as i32 * 52, y + (j / FOLLOW_PER) as i32 * 15, 50, 13);
                    if r.y + r.h > 272 {
                        break;
                    }
                    bottom = bottom.max(r.y + r.h);
                    let (fname, _, ap, _, desc) = skill_of(f);
                    ui.p.panel(r, th().game("slot_open", rgba(22, 30, 48, 255)), th().game("slot_open_edge", rgba(58, 78, 116, 200)), 2.0);
                    ui.p.text_fit(r.x as f32 + 3.0, r.y as f32 + 2.0, &fname, BODY, (r.w - 6) as f32, th().info);
                    let card = vec![(fname.clone(), th().gold), (format!("{iname} skill, {ap} AP"), th().info), (desc, th().text), (format!("Click to try it after {oname} (at the end of the list), or drag it to a place."), th().dim)];
                    ui.push(format!("skills.follow.add.{op}.{f}"), "row", fname, r, true, false, Some(PAct::FollowAdd(op.clone(), f.clone())), card);
                    ui.draggable(format!("follow.add.{op}.{f}"));
                }
            }
            let auto_y = 254.max(bottom + 3);
            if auto_y + 15 <= 273 {
                ui.button_tip("skills.order.auto", Rect::new(310, auto_y, 70, 15), "Auto order", PAct::OrderAuto, !shown.order.is_empty(), tip("Auto order", "Every intent back to the automatic order: the deck's actives in slot order, then the basics."));
                ui.p.text(386.0, auto_y as f32 + 4.0, "Each round fills from the top.", SMALL, th().dim);
            }
        } else {
            let list_area = Rect::new(310, 114, 294, 158);
            ui.push("skills.list", "panel", "Owned", list_area, true, false, None, tip("Owned", "Drop a slot here to take it out."));
            ui.drop_target("deck.list");
            let owned: Vec<(&'static str, String)> = DECK_KINDS.iter().flat_map(|k| t.owned(k.0).iter().map(move |id| (k.0, id.clone()))).collect();
            // U3.2-S4 (OFFHAND1): the worn items' skills close the list, read-only: open while worn, no slot
            let worn: Vec<String> = built.as_ref().map(|u| u.gear_skills.clone()).unwrap_or_default();
            let total = owned.len() + worn.len();
            const ROWS: usize = 7;
            self.scroll = self.scroll.min(total.saturating_sub(ROWS));
            if self.scroll > 0 || total > ROWS {
                ui.p.text_right(604.0, 103.0, &format!("{}-{} of {}", self.scroll + 1, (self.scroll + ROWS).min(total), total), SMALL, th().dim);
            }
            for (k, id) in worn.iter().enumerate().map(|(j, id)| (owned.len() + j, id)).filter(|(k, _)| (self.scroll..self.scroll + ROWS).contains(k)) {
                let y = 118 + (k - self.scroll) as i32 * 22;
                let (name, intent, ap, _, desc) = skill_of(cat.family_of(id));
                ui.button(&format!("skills.slot.worn.{id}"), Rect::new(310, y, 50, 16), "Worn", PAct::DeckView("owned"), false);
                ui.p.text_fit(366.0, y as f32 + 1.0, &format!("{name}  [{intent}]  {ap} AP  from worn gear"), BODY, 238.0, th().gold);
                ui.p.text_fit(366.0, y as f32 + 10.0, &desc, SMALL, 238.0, th().dim);
                let card = vec![(name.clone(), th().gold), (format!("{intent} skill from worn gear, {ap} AP"), th().info), (desc, th().text), ("Open while the item is worn: it takes no slot, and taking the item off removes it.".into(), th().dim)];
                ui.push(format!("skills.worn.{id}"), "row", name, Rect::new(366, y, 238, 20), true, false, None, card);
            }
            for (k, &(kind, ref id)) in owned.iter().skip(self.scroll).take(ROWS).enumerate() {
                let y = 118 + k as i32 * 22;
                let slotted = DeckTerms::list(&shown, kind).contains(id);
                let full = !slotted && DeckTerms::list(&shown, kind).len() >= t.cap(kind);
                ui.button(&format!("skills.slot.{kind}.{id}"), Rect::new(310, y, 50, 16), if slotted { "Slotted" } else if full { "Full" } else { "Slot" }, PAct::DeckSlot(kind, id.clone()), !slotted && !full);
                let (name, line, desc) = if kind == "active" {
                    let (name, intent, ap, rank, desc) = skill_of(id);
                    (name.clone(), format!("{name}  [{intent}]  Rank {}  {ap} AP", roman(rank)), desc)
                } else {
                    let (name, desc) = passive_of(id);
                    (name.clone(), format!("{name}  [{}]", title_case(kind)), desc)
                };
                ui.p.text_fit(366.0, y as f32 + 1.0, &line, BODY, 238.0, th().text);
                ui.p.text_fit(366.0, y as f32 + 10.0, &desc, SMALL, 238.0, th().dim);
                ui.push(format!("skills.owned.{kind}.{id}"), "row", name.clone(), Rect::new(366, y, 238, 20), true, slotted, Some(PAct::DeckSlot(kind, id.clone())), tip(&name, &desc));
                ui.draggable(format!("deck.own.{kind}.{id}"));
            }
        }
        // While something is dragged the pane says where it can go.
        if let Some(from) = ui.dragging().map(String::from) {
            let (mx, my) = ui.mouse;
            let target = ui.widgets.iter().rev().find(|x| x.drop.is_some() && x.rect.contains(mx, my)).and_then(|x| x.drop.clone());
            self.detail = match target {
                Some(to) if t.after_drop(&self.deck, &from, &to).is_ok() => {
                    if let Some((_, j)) = DeckTerms::follow_key(&to) {
                        format!("Puts it at place {} of the follow-ups.", j + 1)
                    } else if let Some((intent, j)) = DeckTerms::order_key(&to) {
                        format!("Moves it to place {} in the {} order.", j + 1, intent_name(intent))
                    } else if to == "deck.list" {
                        "Drop to take it out.".into()
                    } else {
                        let (kind, j) = DeckTerms::slot_key(&to).unwrap_or(("active", 0));
                        let list = DeckTerms::list(&shown, kind);
                        let from_list = from.strip_prefix("deck.own.").and_then(|r| r.split_once('.')).is_some_and(|(k, id)| !DeckTerms::list(&shown, k).iter().any(|x| x == id));
                        match list.get(j) {
                            Some(old) if from_list => format!("Replaces {}.", name_of(kind, old)),
                            Some(old) => format!("Swaps with {}.", name_of(kind, old)),
                            None => "Drop to slot it here.".into(),
                        }
                    }
                }
                _ if from.starts_with("follow.") => "Drop on a place in the same follow-up list.".into(),
                _ => "Drop on a slot of its own kind, on the owned list to take it out, or on a place in the same intent's order.".into(),
            };
        }
        ui.button("skills.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetSkills, true);
        ui.button_tip("skills.default", Rect::new(144, 312, 70, 18), "Use default", PAct::UseDefault, !self.deck.is_empty(), tip("Use default", "The automatic deck: the first class path skills, no passives, the automatic order."));
        ui.button("skills.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmSkills, self.has_draft(cx));
    }

    /// GEARUI1: three columns like the Godot gear dojo: the hero's stats with this gear (and the change a hovered or
    /// dragged item would make), the equipped slots with the battle pouch under them, the party stash. Clicks and drag
    /// and drop both ways (stash to a slot or the pouch equips, a slot or pouch item to the stash takes it off).
    fn draw_gear(&mut self, ui: &mut Ui<PAct>, cx: &Cx, m: &UnitState) {
        self.load_gear(cx);
        let d = &cx.data;
        let (work, stash) = self.gear.clone().unwrap();
        let built = progression::build_with(d, &work, &cx.gs().world_id, false);
        let head = BODY + 1.0;
        // Middle: the equipped slots, then the pouch.
        let (mx, mw) = (243, 185);
        ui.p.text(mx as f32 + 2.0, 102.0, "Equipped", head, th().info);
        for (i, slot) in LOADOUT_SLOTS.iter().enumerate() {
            let y = 116 + i as i32 * 15;
            // PR5-26 (audit B2): a slot left empty (`NO_ITEM`) shows empty, as no item
            let explicit = work.equipment.get(*slot).cloned().filter(|i| i != NO_ITEM);
            let shown = explicit.clone().or_else(|| {
                let s = emberwake_rules::gear::Slot::parse(slot)?;
                built.as_ref().and_then(|b| b.unit.inventory.get(s).map(|e| e.item.id.clone()))
            });
            let name = shown.as_ref().map(|i| d.item_name(i)).unwrap_or_else(|| "empty".into());
            let label = format!("{}: {name}", title_case(slot));
            // U3.2-S3 (FB-34): the worn item's wield penalty, as the built unit fights with it
            let badge = shown.as_ref().and_then(|i| d.cat.items.get(i)).and_then(|def| wield_badge(built.as_ref().map(|b| &b.unit), def));
            let note = badge.as_ref().map(|(_, n)| format!(" {n}")).unwrap_or_default();
            let card = match (&explicit, &shown) {
                (Some(item), _) => item_tip(d, item, &format!("Click or drag it to the stash to take it off (the class default comes back); drag it onto another slot it fits to move it.{note}")),
                (None, Some(item)) => item_tip(d, item, &format!("Class default gear: drag an item from the stash onto this slot to replace it.{note}")),
                (None, None) => tip(&label, "Empty: drag an item from the stash onto this slot."),
            };
            ui.row(&format!("gear.slot.{slot}"), Rect::new(mx, y, mw, 14), &label, if explicit.is_some() { th().text } else { th().dim }, explicit.as_ref().map(|_| PAct::Unequip(slot.to_string())), false, card);
            if let Some((b, _)) = &badge {
                ui.p.text_right((mx + mw - 3) as f32, y as f32 + 2.0, b, SMALL, th().foe);
            }
            if explicit.is_some() {
                ui.draggable(format!("equip.{slot}"));
            }
            ui.drop_target(format!("slot.{slot}"));
        }
        ui.p.text(mx as f32 + 2.0, 210.0, "Battle pouch", head, th().info);
        if self.gear_stacks {
            self.draw_stacks(ui, d, &work, &stash, mx, mw);
        } else {
            for i in 0..POUCH_MAX {
                let y = 224 + i as i32 * 15;
                let item = work.pouch.get(i);
                let label = match item {
                    Some(item) => format!("{}. {}", i + 1, d.item_name(item)),
                    None => format!("{}. Empty", i + 1),
                };
                let card = match item {
                    Some(item) => item_tip(d, item, "Click or drag it to the stash to take it out; drag it onto another pouch slot to reorder."),
                    None => tip(&label, "Drag a consumable from the stash here; the hero can use it in battle."),
                };
                ui.row(&format!("gear.pouch.{i}"), Rect::new(mx, y, mw, 14), &label, if item.is_some() { th().text } else { th().dim }, item.map(|_| PAct::PouchRemove(i)), false, card);
                if item.is_some() {
                    ui.draggable(format!("pouch.{i}"));
                }
                ui.drop_target(format!("pouch.{i}"));
            }
        }
        // Right: the party stash (the whole column takes items dropped back).
        let (sx, sw) = (436, 179);
        ui.p.text(sx as f32 + 2.0, 102.0, "Party stash", head, th().info);
        let area = Rect::new(sx, 114, sw, 156);
        ui.p.panel(area, th().game("party_area", rgba(20, 28, 46, 160)), th().edge, 2.0);
        ui.push("gear.stash", "panel", "Party stash", area, true, false, None, tip("Party stash", "Drop a worn or pouch item here to take it off."));
        ui.drop_target("stash");
        let mut ids: Vec<&String> = stash.keys().filter(|id| d.cat.items.contains_key(*id)).collect();
        ids.sort_by_key(|id| d.item_name(id));
        const ROWS: usize = 10;
        self.gear_scroll = self.gear_scroll.min(ids.len().saturating_sub(ROWS));
        if ids.is_empty() {
            ui.p.text(sx as f32 + 4.0, 118.0, "Empty", head, th().dim);
        }
        for (row, id) in ids.iter().skip(self.gear_scroll).take(ROWS).enumerate() {
            let def = &d.cat.items[*id];
            let (act, help) = if def.is_equipment() {
                (Some(PAct::Equip((*id).clone())), "Click to wear it, or drag it onto a slot.")
            } else if def.item_type == "CONSUMABLE" {
                (Some(PAct::PouchAdd((*id).clone())), "Click to put one in the battle pouch, or drag it onto a pouch slot.")
            } else {
                (None, "Not wearable.")
            };
            let movable = act.is_some();
            // U3.2-S3 (FB-34): every item shows (training is no gate); one the hero would wear under a wield penalty
            // carries the badge, and its card says why
            let badge = wield_badge(built.as_ref().map(|b| &b.unit), def);
            let help = match &badge {
                Some((_, n)) => format!("{help} {n}"),
                None => help.to_string(),
            };
            let y = 116 + row as i32 * 15;
            ui.row(&format!("gear.stash.{id}"), Rect::new(sx + 2, y, sw - 4, 14), &format!("{} x{}", def.name, stash[*id]), th().text, act, false, item_tip(d, id, &help));
            if let Some((b, _)) = &badge {
                ui.p.text_right((sx + sw - 5) as f32, y as f32 + 2.0, b, SMALL, th().foe);
            }
            if movable {
                ui.draggable(format!("stash.{id}"));
            }
        }
        if ids.len() > ROWS {
            let shown = format!("{}-{} of {} (wheel)", self.gear_scroll + 1, (self.gear_scroll + ROWS).min(ids.len()), ids.len());
            ui.p.text_right((sx + sw - 4) as f32, 262.0, &shown, SMALL, th().dim);
        }
        // Left: the stats with this gear, and the change of the dragged item over a target, else of the hovered item.
        let ruleset = party_ruleset(cx);
        let preview = self.gear_preview(ui, d, &work, &stash);
        let now = sheet(d, &cx.gs().world_id, &ruleset, &work);
        let then = preview.as_ref().map(|p| sheet(d, &cx.gs().world_id, &ruleset, p));
        let (lx, lw) = (25, 210);
        ui.p.text(lx as f32 + 3.0, 102.0, &format!("{}  Lv {}", d.class_name(&cx.gs().world_id, &work.class_id(d)), work.level(d)), head, th().info);
        // DERIVE1 4.4: catalog rows past the built-in ones close the 12 px step up (the last row starts at 260 at most)
        let step = 12f32.min(102.0 / (now.len().saturating_sub(STATS.len()).max(2) - 1) as f32);
        for (i, r) in now.iter().enumerate() {
            let core = i < STATS.len();
            let (x, y, w) = if core { (lx + 3 + (i as i32 % 2) * 104, 117 + (i as i32 / 2) * 12, 100) } else { (lx + 3, 158 + ((i - STATS.len()) as f32 * step) as i32, lw - 6) };
            ui.p.text(x as f32, y as f32, &r.label, BODY, th().dim);
            let right = (x + w - 26) as f32;
            ui.p.text_right(right, y as f32, &r.shown, BODY, th().text);
            let delta = then.as_ref().and_then(|t| t.get(i)).map(|t| t.value - r.value).unwrap_or(0);
            if delta != 0 {
                ui.p.text(right + 3.0, y as f32, &format!("{delta:+}"), BODY, if delta > 0 { th().good } else { th().foe });
            }
            ui.push(format!("gear.stat.{}", r.label), "row", format!("{} {}", r.label, r.shown), Rect::new(x - 2, y - 1, w, if core { 12 } else { step.min(12.0) as i32 }), true, false, None, tip(&r.label, &r.desc));
        }
        let dirty = &work != m || stash != cx.gs().party.stash;
        ui.button("gear.reset", Rect::new(68, 312, 70, 18), "Reset", PAct::ResetGear, dirty);
        ui.button("gear.confirm", Rect::new(498, 312, 70, 18), "Confirm", PAct::ConfirmGear, dirty);
    }
}

impl Character {
    /// INTENT2-U ("Stacks & Auto-Fill", `pouch_stacks`): the battle pouch as up to POUCH_MAX stack slots. Each slot: the
    /// item's name (a click or a drag to the stash empties it; a drag onto another slot moves it), its pips (visual only,
    /// no counter: `pouch_pips`), and lower / raise buttons for its packed count (`1..min(5, max_stack)`). The numbers
    /// are on the hover card.
    fn draw_stacks(&self, ui: &mut Ui<PAct>, d: &Data, work: &UnitState, stash: &Stash, mx: i32, mw: i32) {
        const PIP: i32 = 7;
        const BTN: i32 = 11;
        let slots = work.pouch_slots();
        for i in 0..POUCH_MAX {
            let y = 224 + i as i32 * 15;
            let item = slots.get(i);
            let pips_w = 5 * PIP;
            let name_w = mw - 2 * (BTN + 2) - pips_w - 2;
            let label = match item {
                Some(item) => format!("{}. {}", i + 1, d.item_name(item)),
                None => format!("{}. Empty", i + 1),
            };
            let card = match item {
                Some(item) => {
                    let (cur, packed, left) = (work.pouch_count(item), work.packed(item), stash.get(item).copied().unwrap_or(0));
                    let short = if cur < packed { " Hollow pips: copies still to come from the stash." } else { "" };
                    item_tip(d, item, &format!("In the pouch: {cur}; packs {packed} (stash {left}). Auto-fill tops it up to {packed} from the stash when a fight starts while this hero is active, and after each fight this hero comes home from.{short} Click or drag it to the stash to empty the slot; drag it onto another slot to move it; - and + set how many it packs."))
                }
                None => tip(&label, "Drag a consumable from the stash here (or click it in the stash): the slot packs as many as its stack holds."),
            };
            ui.row(&format!("gear.pouch.{i}"), Rect::new(mx, y, name_w, 14), &label, if item.is_some() { th().text } else { th().dim }, item.map(|_| PAct::PouchRemove(i)), false, card.clone());
            if item.is_some() {
                ui.draggable(format!("pouch.{i}"));
            }
            ui.drop_target(format!("pouch.{i}"));
            let Some(item) = item else { continue };
            let (cur, packed, limit) = (work.pouch_count(item), work.packed(item), stack_limit(d, item));
            let pips = pouch_pips(cur, packed, limit);
            let px = mx + name_w + 2 + (5 - pips.len() as i32) * PIP;
            for (k, pip) in pips.iter().enumerate() {
                let (cx, cy) = ((px + k as i32 * PIP) as f32 + PIP as f32 / 2.0, y as f32 + 7.0);
                match pip {
                    Pip::Filled => ui.p.ellipse(cx, cy, 2.6, 2.6, th().gold),
                    Pip::Hollow => ui.p.ellipse_ring(cx, cy, 2.5, 2.5, th().gold),
                    Pip::Dim => ui.p.ellipse_ring(cx, cy, 2.5, 2.5, { let c = th().dim; [c[0], c[1], c[2], 110] }),
                }
            }
            // the pips' state for the agent link and the tests (filled / packed / limit; nothing drawn)
            ui.push(format!("gear.pouch.{i}.pips"), "pips", format!("{cur}/{packed}/{limit}"), Rect::new(mx + name_w + 2, y, pips_w, 14), true, false, None, card);
            let bx = mx + mw - 2 * BTN - 2;
            ui.button_full(&format!("gear.pouch.{i}.less"), Rect::new(bx, y + 1, BTN, 12), "-", PAct::PouchPack(item.clone(), packed - 1), packed > 1, false, SMALL, tip("Pack fewer", "Packs one copy less (a copy above the new count goes back to the stash now)."));
            ui.button_full(&format!("gear.pouch.{i}.more"), Rect::new(bx + BTN + 2, y + 1, BTN, 12), "+", PAct::PouchPack(item.clone(), packed + 1), packed < limit, false, SMALL, tip("Pack more", "Packs one copy more: auto-fill brings it from the stash when a fight starts or ends."));
        }
    }

    /// The draft a hovered or dragged item would make (UIR-3B stat change): a drag over a target it can drop on, else
    /// a hovered stash item worn in its first slot, else a hovered worn item taken off; None otherwise.
    fn gear_preview(&self, ui: &Ui<PAct>, d: &Data, work: &UnitState, stash: &Stash) -> Option<UnitState> {
        let (mut w, mut st) = (work.clone(), stash.clone());
        let mouse = ui.mouse;
        if let Some(from) = ui.dragging() {
            let to = ui.widgets.iter().rev().find(|x| x.drop.is_some() && x.rect.contains(mouse.0, mouse.1))?.drop.clone()?;
            return drop_move(d, &self.gear_world, &mut w, &mut st, from, &to, self.gear_stacks).ok().map(|_| w);
        }
        let hovered = ui.widgets.iter().rev().find(|x| x.action.is_some() && ui.hover(x.rect))?;
        let ok = if let Some(item) = hovered.id.strip_prefix("gear.stash.") {
            d.cat.items.get(item).is_some_and(|i| i.is_equipment()) && equip(d, &self.gear_world, &mut w, &mut st, item, None).is_ok()
        } else if let Some(slot) = hovered.id.strip_prefix("gear.slot.") {
            take_off(d, &self.gear_world, &mut w, &mut st, slot).is_ok()
        } else {
            false
        };
        ok.then_some(w)
    }
}

/// A deck as the agent link reads it (every key, empty ones too).
/// INPUT1 FL3: places a line in the follow-up editor, and the width of an order entry's follow-up toggle.
const FOLLOW_PER: usize = 5;
const FOLLOW_TOGGLE: i32 = 11;

pub(crate) fn deck_json(d: &Deck) -> Value {
    let lists = |m: &IndexMap<String, Vec<String>>| Value::Object(m.iter().map(|(k, v)| (k.clone(), json!(v))).collect());
    // PR5-26 (audit B4): the actives marked marrow too
    json!({"active": d.active, "passive": d.passive, "keystone": d.keystone, "order": lists(&d.order), "follow_ups": lists(&d.follow_ups), "marrow": d.marrow})
}

/// The Gear tab's stat list: the core stats with gear, then the derived rows (`derived`), as `m` fights in `world`.
fn sheet(d: &Data, world: &str, ruleset: &str, m: &UnitState) -> Vec<DRow> {
    let Some(b) = progression::build_with(d, m, world, false) else { return Vec::new() };
    let mut out: Vec<DRow> = STATS.iter().map(|st| DRow::plain(st, b.unit.stats.stat(st, None) as i64, stat_desc(st))).collect();
    out.extend(derived_of(&d.cat, ruleset, &b.unit));
    out
}

/// An item's hover card: its name, then its description (tokens filled), its numbers and what a click or drag does.
/// U3.2-S3 (FB-34, mock): the wield penalty `unit` wears `item` under (the rules' reading, `Equipment::wield_penalty`:
/// the stat-deficit ramp and proficiency 0, stacking) as a badge (the power and armour factor, "x0.75") and a card
/// sentence; None for an item it wears free, or no unit.
pub(crate) fn wield_badge(unit: Option<&Unit>, item: &emberwake_data::ItemDef) -> Option<(String, String)> {
    let u = unit.filter(|_| item.is_equipment())?;
    let pen = emberwake_rules::gear::Equipment::new(item).wield_penalty(&u.stats);
    if !pen.penalized {
        return None;
    }
    let mut why = Vec::new();
    if pen.untrained {
        why.push(format!("untrained in {} (x{}, +{} burden)", item.weapon_type.replace('_', " "), pen.untrained_mult, pen.burden_add));
    }
    if pen.deficit > 0 {
        why.push(format!("{} stat point{} short (-{}%)", pen.deficit, if pen.deficit == 1 { "" } else { "s" }, (pen.ratio * 100.0).round()));
    }
    Some((format!("x{:.2}", pen.factor()), format!("Wield penalty: weapon power and armour x{:.2}; {}.", pen.factor(), why.join("; "))))
}

fn item_tip(d: &Data, id: &str, help: &str) -> aegis_ui::ui::Tip {
    let Some(def) = d.cat.items.get(id) else { return tip(id, help) };
    let desc = d.item_description(id);
    let numbers = item_numbers(def);
    let body: Vec<&str> = [desc.as_str(), numbers.as_str(), help].into_iter().filter(|s| !s.is_empty()).collect();
    tip(&def.name, &body.join(" "))
}

/// An item's numbers in one sentence: power and range, mitigation, burden, bonuses, requirements, what it heals.
fn item_numbers(def: &emberwake_data::ItemDef) -> String {
    let mut parts: Vec<String> = Vec::new();
    if def.item_type == "WEAPON" {
        parts.push(format!("Power {}, range {}{}", def.base_power, def.base_range, if def.two_handed { ", two-handed" } else { "" }));
    }
    if def.mitigation != 0 {
        parts.push(format!("mitigation {}", def.mitigation));
    }
    if def.burden != 0 {
        parts.push(format!("burden {}", def.burden));
    }
    parts.extend(def.stat_bonuses.iter().map(|(s, v)| format!("{v:+} {s}")));
    if !def.required_stats.is_empty() {
        parts.push(format!("needs {}", def.required_stats.iter().map(|(s, v)| format!("{v} {s}")).collect::<Vec<_>>().join(", ")));
    }
    if def.heal > 0 {
        parts.push(format!("heals {}", def.heal));
    }
    if def.damage > 0 {
        parts.push(format!("{} damage", def.damage));
    }
    if parts.is_empty() {
        return String::new();
    }
    let line = parts.join("; ");
    let mut c = line.chars();
    c.next().map(|f| f.to_uppercase().collect::<String>() + c.as_str() + ".").unwrap_or_default()
}

/// TERMS1 TU5: every intent's name through `cat`'s terms (the game's `terms` row, else the default), by intent id.
pub(crate) fn intent_names(cat: &emberwake_data::Catalog) -> IndexMap<String, String> {
    emberwake_battle::round_intent::RoundIntent::ALL.into_iter().map(|i| (i.id().to_string(), i.label(cat))).collect()
}

/// An intent id's display name ("assault" -> "Assault"; a renamed intent its `names` entry, from `intent_names`).
fn intent_title(names: &IndexMap<String, String>, id: &str) -> String {
    match names.get(id) {
        Some(n) => n.clone(),
        None => emberwake_battle::round_intent::RoundIntent::parse(id).map(|x| x.name().to_string()).unwrap_or_else(|| title_case(id)),
    }
}

fn title_case(slot: &str) -> String {
    slot.split('_').map(|w| {
        let mut c = w.chars();
        match c.next() {
            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            None => String::new(),
        }
    }).collect::<Vec<_>>().join(" ")
}

fn roman(n: i32) -> &'static str {
    ["-", "I", "II", "III", "IV", "V"].get(n.max(0) as usize).copied().unwrap_or("V+")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> (Data, UnitState, Stash) {
        let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
        let d = Data::open(&dir).expect("data").0;
        let gs = emberwake_world::new_game::create_for_world(&d, "ashlands", 1).expect("new game");
        let m = gs.party.members[0].clone();
        let mut stash = Stash::new();
        for (id, def) in &d.cat.items {
            if def.is_equipment() || def.item_type == "CONSUMABLE" {
                stash.insert(id.clone(), 2);
            }
        }
        (d, m, stash)
    }

    fn first(d: &Data, slot: &str) -> String {
        let mut ids: Vec<&String> = d.cat.items.iter().filter(|(_, i)| fits(i, slot)).map(|(id, _)| id).collect();
        ids.sort();
        ids[0].clone()
    }

    /// The first item (by id) the member's class can wear in `slot`.
    fn wearable(d: &Data, m: &UnitState, slot: &str) -> String {
        let mut ids: Vec<&String> = d.cat.items.keys().filter(|id| m.can_wear(d, id, slot).is_ok()).collect();
        ids.sort();
        ids[0].clone()
    }

    fn consumables(d: &Data) -> Vec<String> {
        let mut ids: Vec<String> = d.cat.items.iter().filter(|(_, i)| i.item_type == "CONSUMABLE").map(|(id, _)| id.clone()).collect();
        ids.sort();
        ids
    }

    #[test]
    fn a_stash_item_dropped_on_a_slot_it_fits_is_worn_and_the_old_one_returns() {
        let (d, mut m, mut st) = setup();
        let (a, sword) = (first(&d, "accessory_1"), first(&d, "main_hand"));
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1", false).unwrap();
        assert_eq!(m.equipment.get("accessory_1"), Some(&a));
        assert_eq!(st[&a], 1);
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1", false).unwrap();
        assert_eq!(st[&a], 1, "the replaced copy came back");
        assert!(drop_move(&d, "", &mut m, &mut st, &format!("stash.{sword}"), "slot.body", false).is_err(), "a weapon is not body armour");
    }

    #[test]
    fn a_worn_item_dropped_on_the_stash_comes_off() {
        let (d, mut m, mut st) = setup();
        let a = first(&d, "accessory_1");
        let before = st[&a];
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1", false).unwrap();
        drop_move(&d, "", &mut m, &mut st, "equip.accessory_1", "stash", false).unwrap();
        assert!(!m.equipment.contains_key("accessory_1"));
        assert_eq!(st[&a], before);
        assert!(drop_move(&d, "", &mut m, &mut st, "equip.accessory_1", "stash", false).is_err(), "a class default can't come off");
    }

    #[test]
    fn a_worn_item_moves_to_another_slot_it_fits() {
        let (d, mut m, mut st) = setup();
        let a = first(&d, "accessory_1");
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{a}"), "slot.accessory_1", false).unwrap();
        drop_move(&d, "", &mut m, &mut st, "equip.accessory_1", "slot.accessory_2", false).unwrap();
        assert_eq!((m.equipment.get("accessory_1"), m.equipment.get("accessory_2")), (None, Some(&a)));
        assert!(drop_move(&d, "", &mut m, &mut st, "equip.accessory_2", "slot.body", false).is_err());
    }

    #[test]
    fn the_pouch_takes_consumables_replaces_reorders_and_stops_at_its_size() {
        let (d, mut m, mut st) = setup();
        m.pouch.clear();
        let c = consumables(&d);
        let wear = first(&d, "main_hand");
        assert!(drop_move(&d, "", &mut m, &mut st, &format!("stash.{wear}"), "pouch.0", false).is_err(), "gear is not a consumable");
        st.insert(c[0].clone(), 10);
        for _ in 0..POUCH_MAX {
            pouch_put(&d, &mut m, &mut st, &c[0], None, false).unwrap();
        }
        assert!(pouch_put(&d, &mut m, &mut st, &c[0], None, false).is_err(), "a full pouch");
        let other = c.last().unwrap().clone();
        let n = st[&c[0]];
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{other}"), "pouch.1", false).unwrap();
        assert_eq!(m.pouch[1], other);
        assert_eq!(st[&c[0]], n + 1, "the replaced one came back");
        drop_move(&d, "", &mut m, &mut st, "pouch.1", "pouch.0", false).unwrap();
        assert_eq!(m.pouch[0], other);
        drop_move(&d, "", &mut m, &mut st, "pouch.0", "stash", false).unwrap();
        assert_eq!(m.pouch.len(), POUCH_MAX - 1);
    }

    /// U3.2-S3 (FB-34): a class without training in a weapon type wears it (clicks, drops and Confirm), under the
    /// wield penalty, which the slot's badge shows.
    #[test]
    fn a_class_without_training_wears_the_weapon_under_the_penalty() {
        let (d, mut m, mut st) = setup();
        let class = d.cat.classes.get(&m.class_id(&d)).unwrap().clone();
        let prof = emberwake_rules::stats::Proficiency::from_class(&class);
        let mut untrained: Vec<&String> = d.cat.items.iter().filter(|(_, i)| fits(i, "main_hand") && !prof.can_use(&i.weapon_type)).map(|(id, _)| id).collect();
        untrained.sort();
        let item = untrained.into_iter().find(|id| !m.trains(&d, id)).expect("a weapon this member has no training in").to_string();
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{item}"), "slot.main_hand", false).unwrap();
        assert_eq!(m.equipment.get("main_hand"), Some(&item));
        assert!(m.validate(&d).is_empty(), "Confirm takes it: {:?}", m.validate(&d));
        let built = progression::build_with(&d, &m, "", false).unwrap();
        let (badge, note) = wield_badge(Some(&built.unit), &d.cat.items[&item]).expect("a penalty badge");
        assert!(badge.starts_with('x') && note.contains("untrained"), "{badge} {note}");
    }

    /// PR5-26 (audit B2, B6): a member whose grant `forbid_hands` refuses two-handed items (a scratch passive here) can't
    /// wear one from the stash or move one into a slot; a slot left empty (`NO_ITEM`) shows empty, takes an item without
    /// giving anything back and can't be taken off again; taking off the main hand leaves it empty exactly when the
    /// class default the build would bring back is two-handed (`UnitState::forbids_default`).
    #[test]
    fn a_member_forbidden_two_handers_keeps_them_off_and_its_empty_slot() {
        let (mut d, mut m, mut st) = setup();
        let no_2h = emberwake_data::PassiveDef { id: "t_no_2h".into(), name: "No 2H".into(), effects: vec![json!({"type": "forbid_hands", "hands": 2})], slot: String::new() };
        std::sync::Arc::make_mut(&mut d.cat).passives.insert(no_2h.id.clone(), no_2h);
        let mut great: Vec<&String> = d.cat.items.iter().filter(|(id, i)| i.two_handed && m.can_wear(&d, id, "main_hand").is_ok()).map(|(id, _)| id).collect();
        great.sort();
        let great = great.first().map(|s| s.to_string()).expect("a two-handed weapon the member can wear");
        m.passives.push("t_no_2h".into());
        // the first main hand item it can wear now: one-handed
        let one = wearable(&d, &m, "main_hand");
        assert!(m.can_wear(&d, &great, "main_hand").is_err(), "forbidden now");
        assert!(drop_move(&d, "", &mut m, &mut st, &format!("stash.{great}"), "slot.main_hand", false).is_err(), "no two-hander from the stash");
        m.equipment.insert("main_hand".into(), NO_ITEM.into());
        assert!(take_off(&d, "", &mut m, &mut st, "main_hand").is_err(), "an empty slot can't be taken off");
        let before = st.clone();
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{one}"), "slot.main_hand", false).unwrap();
        assert_eq!(m.equipment.get("main_hand"), Some(&one));
        assert!(!st.contains_key(NO_ITEM) && st[&one] == before[&one] - 1, "the empty slot gives nothing back");
        take_off(&d, "", &mut m, &mut st, "main_hand").unwrap();
        let left = m.forbids_default(&d, "", "main_hand").then(|| NO_ITEM.to_string());
        assert_eq!(m.equipment.get("main_hand").cloned(), left, "empty exactly when the default is a two-hander");
        assert_eq!(st[&one], before[&one], "the worn item went back");
        // review F1: a worn item moved into an emptied slot gives nothing back, and the slot it leaves is empty exactly
        // when its default is a two-hander
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{one}"), "slot.main_hand", false).unwrap();
        m.equipment.insert("off_hand".into(), NO_ITEM.into());
        let before = st.clone();
        drop_move(&d, "", &mut m, &mut st, "equip.main_hand", "slot.off_hand", false).unwrap();
        assert_eq!(m.equipment.get("off_hand"), Some(&one), "moved");
        assert_eq!(st, before, "nothing came back to the stash (no '{NO_ITEM}')");
        assert_eq!(m.equipment.get("main_hand").cloned(), left, "the main hand left behind: empty exactly when the default is a two-hander");
    }

    /// The two-handed rule (decision "Two-handed weapon and the off hand (user 2026-10-20)").
    #[test]
    fn a_two_handed_weapon_and_a_shield_exclude_each_other() {
        let (d, mut m, mut st) = setup();
        let pick = |m: &UnitState, slot: &str, two: bool, kind: &str| {
            let mut ids: Vec<&String> = d.cat.items.iter().filter(|(id, i)| i.two_handed == two && i.item_type == kind && m.can_wear(&d, id, slot).is_ok()).map(|(id, _)| id).collect();
            ids.sort();
            ids.first().map(|s| s.to_string())
        };
        let (Some(great), Some(one), Some(shield)) = (pick(&m, "main_hand", true, "WEAPON"), pick(&m, "main_hand", false, "WEAPON"), pick(&m, "off_hand", false, "SHIELD")) else {
            panic!("a 2H weapon, a 1H weapon and a shield this class can wear")
        };
        // the member's own main hand (or its class default) may be two-handed: a one-handed weapon first frees the off hand
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{one}"), "slot.main_hand", false).unwrap();
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{shield}"), "slot.off_hand", false).unwrap();
        let shields = st[&shield];
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{great}"), "slot.main_hand", false).unwrap();
        assert!(!m.equipment.contains_key("off_hand"), "the shield is stowed");
        assert_eq!(st[&shield], shields + 1);
        let err = drop_move(&d, "", &mut m, &mut st, &format!("stash.{shield}"), "slot.off_hand", false).unwrap_err();
        assert!(err.contains("both hands"), "{err}");
        let built = progression::build_with(&d, &m, "", false).unwrap();
        assert!(built.unit.inventory.get(emberwake_rules::gear::Slot::OffHand).is_none_or(|o| off_hand_ok(Some(&d.cat.items[&great]), &o.item)), "no default shield under the 2H weapon");
        m.equipment.insert("off_hand".into(), shield);
        assert!(m.validate(&d).iter().any(|e| e.contains("two-handed")), "validation refuses the pair");
    }

    /// The focus half of the rule: an orb or tome is not held under a two-handed staff (Equipment.is_focus).
    #[test]
    fn a_two_handed_weapon_and_a_focus_exclude_each_other() {
        let (d, _, mut st) = setup();
        // The plain class row, by id: a named mage such as a founder may start with a two-handed weapon of its own.
        let base = d.cat.units.get("mage").map(|u| u.id.clone()).expect("the mage row");
        let mut m = UnitState::create("test_mage", &base);
        let wear = |m: &UnitState, slot: &str, f: &dyn Fn(&emberwake_data::ItemDef) -> bool| {
            let mut ids: Vec<&String> = d.cat.items.iter().filter(|(id, i)| f(i) && m.can_wear(&d, id, slot).is_ok()).map(|(id, _)| id).collect();
            ids.sort();
            ids.first().map(|s| s.to_string())
        };
        let staff = wear(&m, "main_hand", &|i| i.two_handed && i.item_type == "WEAPON").expect("a 2H weapon a mage can wear");
        let one = wear(&m, "main_hand", &|i| !i.two_handed && i.item_type == "WEAPON").expect("a 1H weapon a mage can wear");
        let focus = wear(&m, "off_hand", &|i| matches!(i.weapon_type.as_str(), "orb" | "tome")).expect("a focus");
        // the class default main hand may be two-handed: a one-handed weapon first frees the off hand for the focus
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{one}"), "slot.main_hand", false).unwrap();
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{focus}"), "slot.off_hand", false).unwrap();
        drop_move(&d, "", &mut m, &mut st, &format!("stash.{staff}"), "slot.main_hand", false).unwrap();
        assert!(!m.equipment.contains_key("off_hand"), "the focus is stowed");
        assert!(drop_move(&d, "", &mut m, &mut st, &format!("stash.{focus}"), "slot.off_hand", false).unwrap_err().contains("both hands"));
        let built = progression::build_with(&d, &m, "", false).unwrap();
        assert!(built.unit.inventory.get(emberwake_rules::gear::Slot::OffHand).is_none(), "no default orb under the staff");
        m.equipment.insert("off_hand".into(), focus);
        assert!(m.validate(&d).iter().any(|e| e.contains("two-handed")));
    }

    #[test]
    fn wearing_an_item_changes_the_stat_sheet_the_preview_shows() {
        let (d, m, mut st) = setup();
        let rs = d.ruleset_id("ashlands");
        let before = sheet(&d, "", &rs, &m);
        let mut changed = false;
        for slot in ["main_hand", "body", "accessory_1"] {
            let mut w = m.clone();
            let item = wearable(&d, &m, slot);
            equip(&d, "", &mut w, &mut st, &item, Some(slot)).unwrap();
            changed |= sheet(&d, "", &rs, &w) != before;
        }
        assert!(changed);
        assert_eq!(before.len(), STATS.len() + derived(&d, "", &rs, &m).len());
    }

    /// A scratch derived-stat row (DERIVE1 4.1 keys; the rest take the schema defaults).
    fn row(v: Value) -> emberwake_data::DerivedStatDef {
        emberwake_data::DerivedStatDef::from_obj(v.as_object().unwrap()).expect("the formula parses")
    }

    /// `d` with exactly `rows` as its derived stats (scratch rows in the catalog, the shipped data untouched).
    fn with_rows(d: &Data, rows: Vec<emberwake_data::DerivedStatDef>) -> Data {
        let mut d = d.clone();
        d.edit_cat(|cat| cat.derived_stats = rows.into_iter().map(|r| (r.id.clone(), r)).collect());
        d
    }

    fn labels(rows: &[DRow]) -> Vec<String> {
        rows.iter().map(|r| r.label.clone()).collect()
    }

    /// DERIVE1 4.4 / 7 (U): with no derived row the party table lists the built-in numbers only; a row joins it after
    /// them when its `show_on` names the party screen and its `rulesets` the ruleset, sorted by `order` (ties in
    /// catalog order), with its value, its suffix and the value the rules give.
    #[test]
    fn catalog_derived_rows_join_the_party_table_by_screen_ruleset_and_order() {
        use emberwake_rules::ruleset::{CLASSIC, TIMELINE};
        let (d, m, _) = setup();
        let built = progression::build_with(&d, &m, "", false).unwrap().unit;
        let today: Vec<String> = builtin_of(&built).iter().map(|r| r.0.to_string()).collect();
        let none = with_rows(&d, Vec::new());
        for rs in [CLASSIC, TIMELINE] {
            assert_eq!(labels(&derived(&none, "", rs, &m)), today, "{rs}: no row, the table of today");
        }
        let d = with_rows(&d, vec![
            row(json!({"id": "dread", "name": "Dread", "formula": "100 - KNOW", "rulesets": ["timeline"], "show_on": ["party"]})),
            row(json!({"id": "zeal", "name": "Zeal", "formula": "DEX + level", "show_on": ["party", "battle"]})),
            row(json!({"id": "nerve", "name": "Nerve", "label": "Nrv", "formula": "KNOW * 2 + CONS", "order": 1, "suffix": "%", "show_on": ["party"]})),
            row(json!({"id": "hush", "name": "Hush", "formula": "INT", "show_on": ["dojo"]})),
        ]);
        let with = |extra: &[&str]| today.iter().cloned().chain(extra.iter().map(|s| s.to_string())).collect::<Vec<String>>();
        assert_eq!(labels(&derived(&d, "", CLASSIC, &m)), with(&["Zeal", "Nrv"]), "classic: the timeline row is absent, the dojo row is not on the party screen");
        assert_eq!(labels(&derived(&d, "", TIMELINE, &m)), with(&["Dread", "Zeal", "Nrv"]), "timeline: order 0 in catalog order, then order 1");
        for rs in [CLASSIC, TIMELINE] {
            let rows = derived(&d, "", rs, &m);
            for r in &rows[today.len()..] {
                let id = d.cat.derived_stats.values().find(|x| x.header() == r.label).unwrap();
                let v = derived::value(&d.cat, rs, &built, &id.id).unwrap();
                assert_eq!((r.value, r.shown.clone()), (v as i64, format!("{v}{}", id.suffix)), "{rs} {}", r.label);
            }
            assert_eq!(rows[..today.len()], derived(&none, "", rs, &m)[..], "{rs}: the built-in rows are unchanged");
        }
        assert_eq!(labels(&data_rows(&d.cat, CLASSIC, &built, "dojo")), ["Hush"]);
        assert_eq!(labels(&data_rows(&d.cat, CLASSIC, &built, "battle")), ["Zeal"]);
        let rs = d.ruleset_id("ashlands");
        assert_eq!(sheet(&d, "", &rs, &m).len(), STATS.len() + derived(&d, "", &rs, &m).len(), "the Gear tab lists the same rows");
    }

    /// DERIVE1 4.4: a row's detail text is its description, its formula and the breakdown: the formula with its inputs
    /// filled in, each modifier by the passive or item it came from, the value and its caps.
    #[test]
    fn a_derived_rows_detail_names_its_formula_inputs_modifiers_and_caps() {
        use emberwake_rules::ruleset::CLASSIC;
        let (d, m, _) = setup();
        let d = with_rows(&d, vec![
            row(json!({"id": "nerve", "name": "Nerve", "description": "Holds the line.", "formula": "base + KNOW * per_know", "constants": {"base": 3, "per_know": 2}, "max": 1000})),
            row(json!({"id": "tiny", "name": "Tiny", "formula": "KNOW * 10", "max": 5})),
        ]);
        let detail = |d: &Data, m: &UnitState, id: &str| -> (i64, String) {
            let header = d.cat.derived_stats[id].header().to_string();
            let r = derived(d, "", CLASSIC, m).into_iter().find(|r| r.label == header).unwrap();
            (r.value, r.desc)
        };
        let built = progression::build_with(&d, &m, "", false).unwrap().unit;
        let know = derived::breakdown(&d.cat, CLASSIC, &built, "nerve").unwrap().inputs.iter().find(|i| i.name == "KNOW").unwrap().value;
        let (v, text) = detail(&d, &m, "nerve");
        assert_eq!(v, 3 + know as i64 * 2);
        assert_eq!(text, format!("Holds the line. Formula: base + KNOW * per_know. 3 + KNOW {know} x 2 = {v} (0-1000)."));
        let (tiny, text) = detail(&d, &m, "tiny");
        assert_eq!(tiny, 5);
        assert!(text.ends_with(&format!("KNOW {know} x 10 = 5 (capped at the max; 0-5).")), "{text}");
        // a passive `stat` effect on the derived id (a scratch passive applied to the built unit): the value rises by it
        // and the breakdown names the passive
        let steady = emberwake_data::PassiveDef { id: "test_steady".into(), name: "Steady Test".into(), effects: vec![json!({"type": "stat", "stat": "nerve", "value": 4})], slot: String::new() };
        let mut d = d.clone();
        d.edit_cat(|cat| {
            cat.passives.insert(steady.id.clone(), steady.clone());
        });
        let mut u = built.clone();
        u.apply_passive(&d.cat, &steady);
        let r = data_rows(&d.cat, CLASSIC, &u, "party").into_iter().find(|r| r.label == "Nerve").unwrap();
        assert_eq!(r.value, v + 4);
        assert!(r.desc.ends_with(&format!("3 + KNOW {know} x 2 = {v}; Steady Test +4; = {} (0-1000).", v + 4)), "{}", r.desc);
    }

    /// DERIVE1 4.4: with several catalog rows the Stats, Level up and Gear tabs close their derived rows up: every
    /// derived row's hover rect stays inside its column of the popup, above the detail pane, and no two overlap.
    #[test]
    fn many_derived_rows_stay_inside_their_tab_without_overlapping() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut data = (*cx.app.data).clone();
        data.edit_cat(|cat| cat.derived_stats = (0..4).map(|i| row(json!({"id": format!("row_{i}"), "name": format!("Row {i}"), "formula": "KNOW + 1", "show_on": ["party"]}))).map(|r| (r.id.clone(), r)).collect());
        cx.app.data = std::sync::Arc::new(data);
        let member = cx.gs().party.members[0].id.clone();
        // (tab, widget prefix of its derived rows, its column: x from, x to); the detail pane starts at y 275
        for (tab, prefix, (x0, x1)) in [("stats", "stats.derived.", (230, 616)), ("level_up", "level.derived.", (340, 616)), ("gear", "gear.stat.", (25, 235))] {
            let mut c = Character::new(&cx, &member, tab);
            let mut g = crate::hud::hud_tests::gfx();
            c.draw(&mut cx, &mut g, (-500, -500), true);
            let rows: Vec<(String, Rect)> = c.w.iter().filter(|w| w.id.starts_with(prefix) && !STATS.iter().any(|s| w.id == format!("{prefix}{s}"))).map(|w| (w.id.clone(), w.rect)).collect();
            for i in 0..4 {
                assert!(rows.iter().any(|r| r.0 == format!("{prefix}Row {i}")), "{tab}: Row {i} is listed");
            }
            for (i, (id, r)) in rows.iter().enumerate() {
                assert!(r.x >= x0 && r.x + r.w <= x1 && r.y >= 96 && r.y + r.h <= 275, "{tab}: {id} {r:?} inside its column, above the detail pane");
                for (other, o) in &rows[i + 1..] {
                    let x = r.intersect(*o);
                    assert!(x.w == 0 || x.h == 0, "{tab}: {id} {r:?} overlaps {other} {o:?}");
                }
            }
        }
    }

    /// ASKNO1: with a draft open, a tab switch and Close ask "Discard the changes on this tab?" with No as the default
    /// (Enter keeps the draft, `y` drops it); "Spend these points?" keeps Yes as the default.
    #[test]
    fn discarding_a_draft_defaults_to_no() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let member = cx.gs().party.members[0].id.clone();
        let mut c = Character::new(&cx, &member, "level_up");
        c.draft.insert(STATS[0].to_string(), 1);
        let ask = |c: &mut Character, cx: &mut Cx, a: PAct| -> Box<dyn Screen<GameCx>> {
            c.act(cx, a);
            let Some(Nav::Push(q)) = cx.nav.pop() else { panic!("a question") };
            assert!(cx.nav.is_empty());
            q
        };
        for (a, tag) in [(PAct::Tab("stats"), "character.discard"), (PAct::Close, "character.close")] {
            let mut q = ask(&mut c, &mut cx, a);
            assert_eq!((q.state(&cx)["question"].clone(), q.state(&cx)["default"].clone()), (json!("Discard the changes on this tab?"), json!("no")));
            q.key(&mut cx, Key::Enter);
            assert!(cx.answers.is_empty(), "{tag}: Enter is No");
            cx.nav.clear();
            c.take_answers(&mut cx);
            assert!(c.has_draft(&cx) && c.tab == "level_up" && cx.nav.is_empty(), "{tag}: the draft stays");
        }
        let mut q = ask(&mut c, &mut cx, PAct::Tab("stats"));
        q.key(&mut cx, Key::Char('y'));
        assert_eq!(cx.answers, ["character.discard"], "y is Yes");
        cx.nav.clear();
        c.take_answers(&mut cx);
        assert!(c.tab == "stats" && c.draft.is_empty(), "y drops the draft and switches");
        c.tab = "level_up";
        c.draft.insert(STATS[0].to_string(), 1);
        let q = ask(&mut c, &mut cx, PAct::ConfirmLevel);
        assert_eq!(q.state(&cx)["default"], json!("yes"), "spending points is not a discard");
    }

    // ---- RULESET1 9 D6: the deck view ------------------------------------------------------------------------------

    fn strs(v: &[&str]) -> Vec<String> {
        v.iter().map(|x| x.to_string()).collect()
    }

    /// Scratch terms for the pure deck rules (test-only ids): three actives over two intents, two passives, a keystone,
    /// one basic per intent; slots 2 / 1 / 1; the auto deck slots two actives, an empty active row plays as `lunge`.
    fn terms() -> DeckTerms {
        DeckTerms {
            slots: emberwake_rules::ruleset::DeckSlots { active: 2, passive: 1, keystone: 1 },
            intents: strs(&["assault", "guard"]),
            basics: strs(&["strike", "hold"]),
            actives: strs(&["cleave", "lunge", "wall"]),
            passives: strs(&["p1", "p2"]),
            keystones: strs(&["k1"]),
            intent_of: [("cleave", "assault"), ("lunge", "assault"), ("wall", "guard"), ("strike", "assault"), ("hold", "guard")].iter().map(|(a, b)| (a.to_string(), b.to_string())).collect(),
            rows_of: IndexMap::new(),
            auto: Deck { active: strs(&["cleave", "wall"]), ..Default::default() },
            auto_actives: strs(&["lunge"]),
            intent_names: IndexMap::new(),
        }
    }

    /// PR5-26 (audit B4): the actives marked marrow survive a deck edit: a passive edit on an automatic deck keeps the
    /// marks (its actives stay automatic), an actives edit pins the deck with them, an active that leaves the deck drops
    /// its mark, and the agent link's deck lists them.
    #[test]
    fn deck_edits_keep_the_marrow_marks() {
        let t = terms();
        let auto = Deck { keystone: strs(&["k1"]), marrow: strs(&["lunge"]), ..Default::default() };
        let d = t.slot(&auto, "passive", "p2").unwrap();
        assert_eq!((d.active.clone(), d.marrow.clone()), (Vec::new(), strs(&["lunge"])), "a passive edit keeps the actives automatic and the mark");
        let d = t.slot(&auto, "active", "cleave").unwrap();
        assert_eq!((d.active.clone(), d.marrow.clone()), (strs(&["lunge", "cleave"]), strs(&["lunge"])), "pinned with the mark");
        let d = t.unslot(&d, "active", 0).unwrap();
        assert_eq!((d.active.clone(), d.marrow.clone()), (strs(&["cleave"]), Vec::new()), "the unslotted active's mark goes");
        let d = t.after_drop(&Deck { active: strs(&["lunge", "cleave"]), marrow: strs(&["lunge"]), ..Default::default() }, "deck.own.active.wall", "deck.active.0").unwrap();
        assert_eq!((d.active.clone(), d.marrow.clone()), (strs(&["wall", "cleave"]), Vec::new()), "a replaced active's mark goes");
        assert_eq!(deck_json(&auto)["marrow"], json!(["lunge"]));
    }

    /// A4.2-K1b (INTENT2 IR15): with derived rows (`rows_of`) a dual skill fits the player's order of either row and the
    /// auto order of its primary row only; a follow-up joins an opener that shares one of its rows.
    #[test]
    fn deck_orders_read_the_derived_rows() {
        let mut t = terms();
        t.intents = strs(&["assault", "guard", "field"]);
        t.intent_of.insert("cleave".into(), "field".into());
        t.rows_of.insert("cleave".into(), strs(&["field", "assault"]));
        let mut deck = Deck { active: strs(&["cleave", "lunge"]), ..Default::default() };
        assert_eq!(t.order(&deck, "field"), strs(&["cleave"]), "auto: the primary row");
        assert!(!t.order(&deck, "assault").contains(&"cleave".to_string()), "auto: not the second row");
        deck.order.insert("assault".into(), strs(&["cleave", "lunge", "strike"]));
        assert_eq!(t.order(&deck, "assault"), strs(&["cleave", "lunge", "strike"]), "the player's order: any row");
        assert!(t.follow_candidates(&deck, "cleave").contains(&"lunge".to_string()), "lunge shares Assault with cleave");
        assert!(t.follow_add(&deck, "cleave", "lunge", None).is_ok());
        assert!(t.follow_add(&deck, "lunge", "cleave", None).is_ok(), "either way round");
    }

    /// D5, D6: an empty deck plays as the auto deck; a passive or keystone edit keeps the actives automatic (user
    /// 2026-10-29: the empty active row plays as the class path's actives), an actives or order edit pins them; a bucket
    /// holds its slot count and only what the hero owns of its kind.
    #[test]
    fn only_an_actives_or_order_edit_pins_the_automatic_actives_and_a_bucket_stops_at_its_slots() {
        let t = terms();
        let empty = Deck::default();
        assert_eq!(t.shown(&empty), t.auto);
        assert!(t.slot(&empty, "active", "lunge").unwrap_err().contains("full"), "the auto deck fills both active slots");
        let d = t.slot(&empty, "passive", "p1").unwrap();
        assert_eq!((d.active.clone(), d.passive.clone()), (Vec::<String>::new(), strs(&["p1"])), "a passive edit leaves the actives automatic");
        assert_eq!(t.shown(&d).active, t.auto_actives, "the shown actives follow the class path");
        let k = t.slot(&d, "keystone", "k1").unwrap();
        assert!(k.active.is_empty() && k.order.is_empty(), "a keystone edit pins nothing either");
        let pinned = t.slot(&d, "active", "cleave").unwrap();
        assert_eq!(pinned.active, [t.auto_actives.clone(), strs(&["cleave"])].concat(), "an actives edit pins the shown actives");
        let ordered = t.after_drop(&d, "order.assault.1", "order.assault.0").unwrap();
        assert_eq!(ordered.active, t.auto_actives, "the order editor pins the actives its entries need");
        assert!(t.slot(&d, "passive", "p2").unwrap_err().contains("full"));
        assert!(t.slot(&d, "passive", "k1").is_err(), "a keystone is not a passive");
        assert!(t.slot(&d, "active", "nope").is_err(), "not owned");
        assert_eq!(t.slot(&d, "passive", "p1").unwrap_err(), "Already slotted.");
        let only_passive = Deck { passive: strs(&["p1"]), ..Default::default() };
        assert_eq!(t.shown(&only_passive).active, t.auto_actives);
    }

    /// D6 "filled by click or drag like today's slots": a bucket takes its own kind only; a dragged owned entry replaces
    /// a filled slot, a slotted one swaps or moves; a slot dropped on the list comes out; the last active stays.
    #[test]
    fn a_deck_bucket_takes_its_own_kind_and_slots_swap_replace_or_come_out() {
        let t = terms();
        let empty = Deck::default();
        assert!(t.after_drop(&empty, "deck.own.passive.p1", "deck.active.0").unwrap_err().contains("active skills only"));
        assert!(t.after_drop(&empty, "deck.active.0", "deck.keystone.0").unwrap_err().contains("keystones only"));
        assert_eq!(t.after_drop(&empty, "deck.own.active.lunge", "deck.active.1").unwrap().active, strs(&["cleave", "lunge"]), "replaces");
        assert_eq!(t.after_drop(&empty, "deck.active.0", "deck.active.1").unwrap().active, strs(&["wall", "cleave"]), "swaps");
        assert_eq!(t.after_drop(&empty, "deck.own.active.wall", "deck.active.0").unwrap().active, strs(&["wall", "cleave"]), "a slotted one moves from its slot");
        let one = t.after_drop(&empty, "deck.active.0", "deck.list").unwrap();
        assert_eq!(one.active, strs(&["wall"]));
        assert!(t.after_drop(&one, "deck.active.0", "deck.list").unwrap_err().contains("at least one"));
        assert!(t.unslot(&one, "active", 0).is_err());
        let p = t.after_drop(&empty, "deck.own.passive.p2", "deck.passive.0").unwrap();
        assert_eq!(p.passive, strs(&["p2"]));
        assert!(t.after_drop(&p, "deck.passive.0", "deck.list").unwrap().passive.is_empty());
        assert!(t.after_drop(&p, "deck.passive.1", "deck.list").is_err(), "an empty slot");
    }

    /// D6 order editor and RULESET1_INTENT FU4: the auto order is the deck's actives of the intent in slot order, then
    /// the basics of the intent; a drag reorders within one intent and writes that intent's list only; a new active
    /// joins a written list at its end, a removed one leaves it; Auto order clears the written lists only.
    #[test]
    fn the_order_editor_reorders_within_an_intent_and_follows_the_slots() {
        let t = terms();
        let empty = Deck::default();
        assert_eq!((t.order(&t.auto, "assault"), t.order(&t.auto, "guard")), (strs(&["cleave", "strike"]), strs(&["wall", "hold"])));
        let d = t.after_drop(&empty, "order.assault.1", "order.assault.0").unwrap();
        assert_eq!(d.order.get("assault"), Some(&strs(&["strike", "cleave"])));
        assert!(!d.order.contains_key("guard"), "the other intents stay automatic");
        assert_eq!(t.order(&d, "assault"), strs(&["strike", "cleave"]));
        assert!(t.after_drop(&d, "order.assault.0", "order.guard.0").unwrap_err().contains("own order"));
        let swapped = t.after_drop(&d, "deck.own.active.lunge", "deck.active.0").unwrap();
        assert_eq!(swapped.order.get("assault"), Some(&strs(&["strike", "lunge"])), "cleave left the written order, lunge joined its end");
        let d2 = t.after_drop(&empty, "order.guard.1", "order.guard.0").unwrap();
        let gone = t.after_drop(&d2, "deck.active.1", "deck.list").unwrap();
        assert_eq!(gone.order.get("guard"), Some(&strs(&["hold"])), "wall left; the basic stays");
        let (mut cx, member) = deck_cx(true);
        let mut c = Character::new(&cx, &member, "skills");
        c.load_skills(&cx);
        c.terms = Some(t.clone());
        c.deck = d.clone();
        c.act(&mut cx, PAct::OrderAuto);
        assert!(c.deck.order.is_empty() && c.deck.active == d.active, "Auto order clears the written orders only");
    }

    /// INPUT1 FL1, FL3: a follow-up edit pins the deck as it plays; a list takes skills of the opener's intent in the
    /// deck (the opener may stand in its own list), reorders within itself, loses an entry or empties ("Use the order");
    /// an actives edit that removes a family takes it out of every list and drops its own list.
    #[test]
    fn follow_up_lists_take_the_openers_intent_and_follow_the_slots() {
        let t = terms();
        let empty = Deck::default();
        let d = t.follow_add(&empty, "cleave", "strike", None).unwrap();
        assert_eq!(d.active, t.auto.active, "a follow-up edit pins the automatic actives");
        assert_eq!(d.follow_ups.get("cleave"), Some(&strs(&["strike"])));
        let d2 = t.follow_add(&d, "cleave", "cleave", None).unwrap();
        assert_eq!(t.follow_list(&d2, "cleave"), strs(&["strike", "cleave"]), "the opener in its own list");
        assert_eq!(t.follow_candidates(&d, "cleave"), strs(&["cleave"]), "the intent's actives and basics not yet listed");
        assert!(t.follow_add(&d, "cleave", "wall", None).unwrap_err().contains("is Guard, not Assault"));
        // TERMS1 TU5: the refusal names the intents through the catalog's terms
        let mut renamed = t.clone();
        renamed.intent_names = intent_names(&emberwake_data::Catalog::from_value(&json!({"terms": [{"id": "intent_assault", "name": "Onslaught", "origin": "test"}]})));
        assert!(renamed.follow_add(&d, "cleave", "wall", None).unwrap_err().contains("is Guard, not Onslaught"));
        assert!(t.follow_add(&d, "cleave", "lunge", None).unwrap_err().contains("not in the deck"));
        assert!(t.follow_add(&d, "lunge", "strike", None).unwrap_err().contains("not an opener"));
        // drag: reorder, add at a place, never across lists
        assert_eq!(t.follow_list(&t.after_drop(&d2, "follow.cleave.1", "follow.cleave.0").unwrap(), "cleave"), strs(&["cleave", "strike"]));
        assert_eq!(t.follow_list(&t.after_drop(&d, "follow.add.cleave.cleave", "follow.cleave.0").unwrap(), "cleave"), strs(&["cleave", "strike"]));
        assert_eq!(t.follow_list(&t.after_drop(&d, "follow.add.cleave.cleave", "follow.cleave.1").unwrap(), "cleave"), strs(&["strike", "cleave"]), "the end place");
        let w = t.follow_add(&d2, "wall", "hold", None).unwrap();
        assert!(t.after_drop(&w, "follow.cleave.0", "follow.wall.0").unwrap_err().contains("own follow-ups"));
        assert!(t.after_drop(&w, "follow.add.cleave.strike", "order.assault.0").is_err());
        // remove, clear
        assert_eq!(t.follow_list(&t.follow_remove(&d2, "cleave", 0).unwrap(), "cleave"), strs(&["cleave"]));
        assert!(!t.follow_remove(&d, "cleave", 0).unwrap().follow_ups.contains_key("cleave"), "the last one out: no list");
        let cleared = t.follow_clear(&w, "cleave").unwrap();
        assert!(!cleared.follow_ups.contains_key("cleave") && cleared.follow_ups.contains_key("wall"), "only that opener's list");
        assert!(t.follow_clear(&cleared, "cleave").is_err(), "nothing to clear");
        // an active that leaves the deck leaves every list and drops its own
        let two = t.after_drop(&empty, "deck.own.active.lunge", "deck.active.1").unwrap();
        assert_eq!(two.active, strs(&["cleave", "lunge"]));
        let lists = t.follow_add(&t.follow_add(&t.follow_add(&two, "lunge", "cleave", None).unwrap(), "lunge", "strike", None).unwrap(), "cleave", "lunge", None).unwrap();
        let gone = t.unslot(&lists, "active", 0).unwrap();
        assert!(!gone.follow_ups.contains_key("cleave"), "cleave's own list is dropped");
        assert_eq!(gone.follow_ups.get("lunge"), Some(&strs(&["strike"])), "cleave left lunge's list");
        let back = t.after_drop(&lists, "deck.own.active.wall", "deck.active.1").unwrap();
        assert!(!back.follow_ups.contains_key("lunge") && back.follow_ups.get("cleave").is_none(), "replacing lunge drops its list and takes it out of cleave's: {back:?}");
    }

    /// FL3 on the Skills tab (shipped data, ids picked by rule): an order entry's toggle opens its list under it; a click
    /// adds a skill of the intent, the entry shows "+", a right click takes an entry out, "Use the order" empties the
    /// list; the editor fits the popup; Confirm saves a deck the member's validation accepts.
    #[test]
    fn the_follow_up_editor_opens_under_its_entry_and_saves_a_valid_list() {
        let (mut cx, member) = deck_cx(true);
        let mut g = crate::hud::hud_tests::gfx();
        let mut c = Character::new(&cx, &member, "skills");
        c.act(&mut cx, PAct::DeckView("order"));
        c.draw(&mut cx, &mut g, (-500, -500), true);
        let t = c.terms.clone().unwrap();
        let shown = t.shown(&c.deck);
        let intent = t.intents.iter().find(|i| t.order(&shown, i).len() > 1).or_else(|| t.intents.iter().find(|i| !t.order(&shown, i).is_empty())).cloned().expect("an intent with an order");
        let order = t.order(&shown, &intent);
        let op = order[0].clone();
        assert!(c.w.iter().any(|w| w.id == format!("skills.follow.{op}")), "every order entry has its toggle");
        assert!(!c.w.iter().any(|w| w.id == format!("skills.follow.clear.{op}")), "closed until toggled");
        c.act(&mut cx, PAct::FollowOpen(op.clone()));
        c.draw(&mut cx, &mut g, (-500, -500), true);
        let ids: Vec<String> = c.w.iter().map(|w| w.id.clone()).collect();
        assert!(ids.contains(&format!("skills.follow.clear.{op}")) && ids.contains(&format!("skills.follow.{op}.end")), "{ids:?}");
        for f in &order {
            assert!(ids.contains(&format!("skills.follow.add.{op}.{f}")), "{f} can be added");
        }
        let entry = rect_of(&c, &format!("skills.order.{intent}.0"));
        assert!(rect_of(&c, &format!("skills.follow.clear.{op}")).y > entry.y, "the list opens under its entry");
        assert_laid_out(&c, "follow-up editor");
        let last = order.last().unwrap().clone();
        c.act(&mut cx, PAct::FollowAdd(op.clone(), last.clone()));
        assert_eq!(c.deck.follow_ups.get(&op), Some(&vec![last.clone()]));
        let recs = crate::hud::hud_tests::frame(&mut c, &mut cx, &mut g, 1.0);
        assert!(recs.iter().any(|r| r.text == "+"), "a set list marks its entry");
        assert!(c.w.iter().any(|w| w.id == format!("skills.follow.{op}.0")));
        if order.len() > 1 {
            c.act(&mut cx, PAct::FollowAdd(op.clone(), order[0].clone()));
            assert!(c.can_drop(&cx, &format!("follow.{op}.1"), &format!("follow.{op}.0")));
            assert_eq!(t.follow_list(&c.deck, &op), [last.clone(), order[0].clone()], "added at the end");
            c.accept_drop(&mut cx, &format!("follow.{op}.1"), &format!("follow.{op}.0"));
            assert_eq!(t.follow_list(&c.deck, &op), [order[0].clone(), last.clone()], "a drag reorders");
        }
        // a right click on the first entry takes it out
        c.draw(&mut cx, &mut g, (-500, -500), true);
        let n = t.follow_list(&c.deck, &op).len();
        let r = rect_of(&c, &format!("skills.follow.{op}.0"));
        c.mouse = (r.x + 2, r.y + 2);
        c.back(&mut cx);
        assert_eq!(t.follow_list(&c.deck, &op).len(), n - 1);
        assert!(cx.nav.is_empty(), "the popup stays open");
        c.act(&mut cx, PAct::FollowAdd(op.clone(), op.clone()));
        let draft = c.deck.clone();
        c.act(&mut cx, PAct::ConfirmSkills);
        assert!(cx.notice.is_empty(), "{:?}", cx.notice);
        let m = cx.gs().party.member(&member).unwrap().clone();
        assert_eq!(m.deck, draft);
        assert!(m.validate(&cx.data).is_empty(), "{:?}", m.validate(&cx.data));
        c.act(&mut cx, PAct::FollowClear(op.clone()));
        assert!(!c.deck.follow_ups.contains_key(&op), "Use the order empties the list");
    }

    /// A game on the shipped data in a world that plays under the deck rule (`deck` true) or without it, and its first
    /// member.
    fn deck_cx(deck: bool) -> (Cx, String) {
        // The test bed (decision "Ashlands is a test bed"), by name: a pick by rule follows whichever world sorts first,
        // and sunk_ward's empty class paths leave a member no actives. It plays classic, so the deck case gives this
        // test's own copy of its manifest the timeline row.
        const WORLD: &str = "ashlands";
        let mut cx = crate::hud::hud_tests::game_cx();
        if deck {
            let manifest = &mut std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data").catalogs.worlds.get_mut(WORLD).expect("the world").manifest;
            manifest.insert("ruleset".into(), json!(emberwake_rules::ruleset::TIMELINE));
        }
        assert_eq!(emberwake_rules::DeckRule::of(&cx.data.ruleset(WORLD)).is_some(), deck, "{WORLD} under the wanted rule");
        cx.gs_mut().world_id = WORLD.into();
        let member = cx.gs().party.members[0].id.clone();
        (cx, member)
    }

    fn rect_of(c: &Character, id: &str) -> Rect {
        c.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("widget {id}")).rect
    }

    /// Every `skills.` widget but the owned-list panel sits inside the popup (above the detail pane, the bottom buttons
    /// aside), none overlapping.
    fn assert_laid_out(c: &Character, what: &str) {
        let ws: Vec<(String, Rect)> = c.w.iter().filter(|w| w.id.starts_with("skills.") && w.id != "skills.list").map(|w| (w.id.clone(), w.rect)).collect();
        for (i, (id, r)) in ws.iter().enumerate() {
            let below = id == "skills.reset" || id == "skills.default" || id == "skills.confirm";
            assert!(r.x >= 25 && r.x + r.w <= 615 && r.y >= 96 && (below || r.y + r.h <= 275), "{what}: {id} {r:?} inside the popup, above the detail pane");
            for (other, o) in &ws[i + 1..] {
                let x = r.intersect(*o);
                assert!(x.w == 0 || x.h == 0, "{what}: {id} {r:?} overlaps {other} {o:?}");
            }
        }
    }

    /// "The classic ruleset shows today's screen unchanged": five class and three extra slots, no deck bucket.
    #[test]
    fn classic_keeps_the_skills_tab_of_today() {
        let (mut cx, member) = deck_cx(false);
        let mut c = Character::new(&cx, &member, "skills");
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(c.terms.is_none());
        let ids: Vec<&str> = c.w.iter().map(|w| w.id.as_str()).collect();
        for i in 0..emberwake_rules::stats::MAX_CLASS_SLOTS {
            assert!(ids.contains(&format!("skills.class.{i}").as_str()));
        }
        assert!(!ids.iter().any(|id| id.starts_with("skills.active.") || id.starts_with("skills.basic.") || id.starts_with("skills.view.")));
        assert!(c.state(&cx).get("deck").is_none());
    }

    /// D6: under the deck rule the tab shows a bucket per slot kind with the ruleset's slot counts, a read-only chip per
    /// basic and, on the right, the owned list or one order list per intent; everything fits the popup.
    #[test]
    fn under_the_deck_rule_the_skills_tab_shows_buckets_basics_and_the_order_editor() {
        let (mut cx, member) = deck_cx(true);
        let rule = emberwake_rules::DeckRule::of(&cx.data.ruleset(&cx.gs().world_id)).unwrap();
        let mut c = Character::new(&cx, &member, "skills");
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        let t = c.terms.clone().expect("deck terms under the deck rule");
        assert_eq!(t.slots, rule.slots);
        let ids: Vec<String> = c.w.iter().map(|w| w.id.clone()).collect();
        for (kind, _, _) in DECK_KINDS {
            assert_eq!(ids.iter().filter(|id| id.starts_with(&format!("skills.{kind}."))).count(), t.cap(kind), "{kind}: one widget per slot");
        }
        assert!(!t.basics.is_empty());
        for b in &t.basics {
            assert!(ids.contains(&format!("skills.basic.{b}")), "basic {b}");
            assert!(!t.actives.contains(b), "a basic is never an active");
        }
        let shown = t.shown(&c.deck);
        assert_eq!(c.state(&cx)["deck_shown"]["active"], json!(shown.active));
        assert!(shown.active.len() <= t.cap("active") && !shown.active.is_empty());
        assert_laid_out(&c, "owned view");
        c.act(&mut cx, PAct::DeckView("order"));
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        for intent in &t.intents {
            assert!(c.w.iter().any(|w| w.id == format!("skills.order.{intent}")), "{intent}");
            let n = c.w.iter().filter(|w| w.id.starts_with(&format!("skills.order.{intent}."))).count();
            assert_eq!(n, t.order(&shown, intent).len(), "{intent}: one entry per skill of its order");
        }
        assert_laid_out(&c, "order view");
    }

    /// U3.2-S4 (OFFHAND1): a worn item's skill shows read-only under the basics (open while worn, no slot), never among
    /// the actives, and goes when the item comes off. The skill effect is test-only, on this test's own data.
    #[test]
    fn under_the_deck_rule_a_worn_items_skill_shows_read_only_and_takes_no_slot() {
        let (mut cx, member) = deck_cx(true);
        let world = cx.gs().world_id.clone();
        let m = cx.gs().party.member(&member).unwrap().clone();
        let owned = m.owned_families_anywhere(&cx.data);
        let skill = cx.data.cat.skills.iter().find(|s| s.tier == 1 && !s.basic && !s.system && !owned.contains(&s.family_id().to_string())).map(|s| s.id.clone()).expect("a skill the member does not own");
        let mut rings: Vec<String> = cx.data.cat.items.iter().filter(|(_, i)| i.item_type == "ACCESSORY").map(|(id, _)| id.clone()).collect();
        rings.sort();
        let ring = rings[0].clone();
        let data = std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data");
        std::sync::Arc::make_mut(&mut data.cat).items.get_mut(&ring).unwrap().effects.push(json!({"type": "skill", "skill": skill}));
        assert!(std::sync::Arc::ptr_eq(&cx.data.cat, cx.data.cat_in(&world)), "{world} builds with the base catalog");
        cx.gs_mut().party.member_mut(&member).unwrap().equipment.insert("accessory_1".into(), ring);
        let mut c = Character::new(&cx, &member, "skills");
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        let t = c.terms.clone().expect("deck terms");
        // the worn rows close the owned list, which scrolls past its 7 rows when the member owns more: scroll to the end
        // (draw clamps it) so the check holds for any owned count
        c.scroll = usize::MAX;
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(c.w.iter().any(|w| w.id == format!("skills.worn.{skill}")), "the worn skill shows");
        assert!(!t.shown(&c.deck).active.iter().any(|f| f == cx.data.cat.family_of(&skill)), "it takes no active slot");
        assert_laid_out(&c, "worn skills");
        cx.gs_mut().party.member_mut(&member).unwrap().equipment.shift_remove("accessory_1");
        let mut c = Character::new(&cx, &member, "skills");
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(!c.w.iter().any(|w| w.id.starts_with("skills.worn.")), "gone with the item");
    }

    /// AUDIT-05 (OFFHAND1) on the classic Skills tab: a worn item's skill closes the owned list read-only (open while
    /// worn, no slot) and goes when the item comes off. The skill effect is test-only, on this test's own data.
    #[test]
    fn the_classic_skills_tab_shows_a_worn_items_skill_read_only() {
        let (mut cx, member) = deck_cx(false);
        let m = cx.gs().party.member(&member).unwrap().clone();
        let owned = m.owned_families_anywhere(&cx.data);
        let skill = cx.data.cat.skills.iter().find(|s| s.tier == 1 && !s.basic && !s.system && !owned.contains(&s.family_id().to_string())).map(|s| s.id.clone()).expect("a skill the member does not own");
        let mut rings: Vec<String> = cx.data.cat.items.iter().filter(|(_, i)| i.item_type == "ACCESSORY").map(|(id, _)| id.clone()).collect();
        rings.sort();
        let ring = rings[0].clone();
        let data = std::sync::Arc::get_mut(&mut cx.app.data).expect("this test's own data");
        std::sync::Arc::make_mut(&mut data.cat).items.get_mut(&ring).unwrap().effects.push(json!({"type": "skill", "skill": skill}));
        cx.gs_mut().party.member_mut(&member).unwrap().equipment.insert("accessory_1".into(), ring);
        let mut c = Character::new(&cx, &member, "skills");
        // the worn rows close the list: scroll to its end (clamped to the last page)
        c.scroll = usize::MAX;
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(c.terms.is_none(), "classic");
        assert!(c.w.iter().any(|w| w.id == format!("skills.worn.{skill}")), "the worn skill shows");
        let worn = c.w.iter().find(|w| w.id == format!("skills.slot.worn.{skill}")).expect("its Worn chip");
        assert!(!worn.enabled, "read-only: no slot");
        let r = rect_of(&c, &format!("skills.worn.{skill}"));
        assert!(r.x >= 310 && r.x + r.w <= 604 && r.y >= 114 && r.y + r.h <= 272, "inside the owned list: {r:?}");
        cx.gs_mut().party.member_mut(&member).unwrap().equipment.shift_remove("accessory_1");
        let mut c = Character::new(&cx, &member, "skills");
        c.scroll = usize::MAX;
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(!c.w.iter().any(|w| w.id.starts_with("skills.worn.")), "gone with the item");
    }

    /// AUDIT-05: a derived-stat breakdown names a worn item's stat effect by its item (its tier, its family mastery),
    /// never the raw `gear_effect` source; the bare kind reads as worn item effects.
    #[test]
    fn a_breakdown_names_the_item_behind_a_worn_stat_effect() {
        let cx = crate::hud::hud_tests::game_cx();
        let cat = &cx.data.cat;
        let mut ids: Vec<&String> = cat.items.keys().collect();
        ids.sort();
        let (id, name) = (ids[0].clone(), cat.items[ids[0]].name.clone());
        let src = |grant: &str| emberwake_rules::unit::gear_effect_source(grant);
        assert_eq!(source_name(cat, &src(&format!("gear:{id}"))), name);
        assert_eq!(source_name(cat, &src(&format!("gear:{id}:tier2"))), format!("{name} II"));
        assert_eq!(source_name(cat, &src(&format!("gear:{id}:mastery"))), format!("{name} mastery"));
        assert_eq!(source_name(cat, GEAR_EFFECT_SOURCE), "Worn item effects");
        for grant in [format!("gear:{id}"), format!("gear:{id}:tier2"), format!("gear:{id}:mastery")] {
            assert!(!source_name(cat, &src(&grant)).contains(GEAR_EFFECT_SOURCE), "{grant}");
        }
    }

    /// D3-D6 end to end: the member owns a slottable passive and keystone (shipped rows picked by `slot`); slotting
    /// one, reordering an intent and Confirm save a deck the member's validation accepts; the battle build applies a
    /// slottable passive only while slotted; a right click on a filled slot takes it out.
    #[test]
    fn a_confirmed_deck_is_valid_and_slots_its_passives_in_battle() {
        let (mut cx, member) = deck_cx(true);
        let pick = |slot: &str| cx.data.cat.passives.values().filter(|p| p.slot == slot).map(|p| p.id.clone()).min().unwrap_or_else(|| panic!("a {slot} passive"));
        let (passive, keystone) = (pick("passive"), pick("keystone"));
        cx.gs_mut().party.member_mut(&member).unwrap().passives.extend([passive.clone(), keystone.clone()]);
        let mut c = Character::new(&cx, &member, "skills");
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        assert!(!c.has_draft(&cx));
        c.act(&mut cx, PAct::DeckSlot("passive", passive.clone()));
        assert_eq!(c.deck.passive, [passive.clone()]);
        let t = c.terms.clone().unwrap();
        let shown = t.shown(&c.deck);
        if let Some(intent) = t.intents.iter().find(|i| t.order(&shown, i).len() > 1).cloned() {
            let before = t.order(&shown, &intent);
            c.act(&mut cx, PAct::OrderUp(intent.clone(), 1));
            let want: Vec<String> = [before[1].clone(), before[0].clone()].into_iter().chain(before[2..].iter().cloned()).collect();
            assert_eq!(t.order(&c.deck, &intent), want);
        }
        assert!(c.has_draft(&cx));
        let draft = c.deck.clone();
        c.act(&mut cx, PAct::ConfirmSkills);
        assert!(cx.notice.is_empty() && cx.nav.is_empty(), "{:?}", cx.notice);
        let m = cx.gs().party.member(&member).unwrap().clone();
        assert_eq!(m.deck, draft);
        assert!(m.validate(&cx.data).is_empty(), "{:?}", m.validate(&cx.data));
        let rule = emberwake_rules::DeckRule::of(&cx.data.ruleset(&cx.gs().world_id)).unwrap();
        let u = progression::build_in(&cx.data, &m, "", Some(&rule)).unwrap().unit;
        assert!(u.passives.contains(&passive) && !u.passives.contains(&keystone), "only the slotted passive applies");
        assert_eq!(u.active_families(), draft.active);
        let back = UnitState::from_dict(m.to_value().as_object().unwrap());
        assert_eq!(back.deck, draft, "the saved form reads back the same deck");
        // a right click on the filled passive slot takes it out; the popup stays open
        c.draw(&mut cx, &mut crate::hud::hud_tests::gfx(), (-500, -500), true);
        let r = rect_of(&c, "skills.passive.0");
        c.mouse = (r.x + 2, r.y + 2);
        c.back(&mut cx);
        assert!(c.deck.passive.is_empty() && cx.nav.is_empty());
        c.act(&mut cx, PAct::UseDefault);
        assert!(c.deck.is_empty() && c.has_draft(&cx));
    }

    /// TEXTFIT1 on the deck view: at every text size no text leaves the canvas, overlaps another or runs out of its
    /// control, in both right-hand views (`text_fit` shortening a long description is allowed, as on today's tab).
    #[test]
    fn the_deck_view_reads_clean_at_every_text_size() {
        let (mut cx, member) = deck_cx(true);
        let mut g = crate::hud::hud_tests::gfx();
        let mut c = Character::new(&cx, &member, "skills");
        for view in ["owned", "order", "follow"] {
            if view == "follow" {
                // INPUT1 FL3: the follow-up editor open under the first entry of the longest order, a list set
                let t = c.terms.clone().unwrap();
                let shown = t.shown(&c.deck);
                let order = t.intents.iter().map(|i| t.order(&shown, i)).max_by_key(Vec::len).unwrap_or_default();
                let op = order.first().cloned().expect("an order entry");
                c.act(&mut cx, PAct::FollowAdd(op.clone(), order.last().cloned().unwrap()));
                c.act(&mut cx, PAct::FollowOpen(op));
            } else {
                c.act(&mut cx, PAct::DeckView(view));
            }
            for (size, scale) in aegis_ui::screen::TEXT_SIZES {
                let recs = crate::hud::hud_tests::frame(&mut c, &mut cx, &mut g, scale as f32);
                let boxes: Vec<(String, f32, f32, f32, f32)> = c.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                for k in ["edge", "overlap", "spill"] {
                    assert_eq!(v["counts"][k], json!(0), "{view} at {size}: {v}");
                }
            }
        }
    }

    /// ROSTER1-U on the Stats tab: a classic world shows no bunk chip and no wounded mark; with a roster block the chip
    /// reads "Bunks used/cap" and a wounded member shows its mark with the HP it takes the field with (W2, the cut max);
    /// the tab reads clean at every text size.
    #[test]
    fn the_stats_tab_marks_the_wounded_and_counts_the_bunks() {
        let mut cx = crate::hud::hud_tests::game_cx();
        let member = cx.gs().party.members[0].id.clone();
        let mut g = crate::hud::hud_tests::gfx();
        let mut c = Character::new(&cx, &member, "stats");
        let texts = |recs: &[aegis_ui::painter::TextRec]| recs.iter().map(|r| r.text.clone()).collect::<Vec<String>>();
        let t = texts(&crate::hud::hud_tests::frame(&mut c, &mut cx, &mut g, 1.0));
        assert!(!t.iter().any(|x| x.starts_with("Bunks") || x == "Wounded"), "classic: {t:?}");
        assert!(!c.w.iter().any(|w| w.id == "stats.wounded"));
        let max = cx.gs().party.roster.party_max;
        cx.gs_mut().party.roster = emberwake_world::state::RosterCaps { party_max: max, manifest: emberwake_data::RosterManifest::from_obj(json!({"bunks": 6}).as_object().unwrap()) };
        cx.gs_mut().party.member_mut(&member).unwrap().add_condition(emberwake_world::roster::WOUNDED);
        let (hp, field, wounded) = crate::hub::field_hp(&cx, cx.gs().party.member(&member).unwrap());
        assert!(wounded && hp <= field);
        let t = texts(&crate::hud::hud_tests::frame(&mut c, &mut cx, &mut g, 1.0));
        assert!(t.contains(&format!("Bunks {}/6", cx.gs().party.members.len())), "{t:?}");
        assert!(t.contains(&"Wounded".to_string()) && t.contains(&format!("HP {hp} / {field}")), "{t:?}");
        assert!(c.w.iter().any(|w| w.id == "stats.wounded"));
        for (size, scale) in aegis_ui::screen::TEXT_SIZES {
            let recs = crate::hud::hud_tests::frame(&mut c, &mut cx, &mut g, scale as f32);
            let boxes: Vec<(String, f32, f32, f32, f32)> = c.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            for k in ["edge", "overlap", "spill"] {
                assert_eq!(v["counts"][k], json!(0), "stats tab at {size}: {v}");
            }
        }
    }
}

/// PORTRAITFX1-R (docs/specs/PORTRAITFX1.md 6.2 row 2, PORTRAITFX1_TASKS 1.6 R1-R3): the Character screen's portrait.
#[cfg(test)]
mod portrait_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};
    use crate::portraits::tests::{roster_cx, row};
    use aegis_ui::fx::Clock;
    use aegis_ui::portrait_fx::EASE_MS;
    use aegis_ui::screen::TEXT_SIZES;

    /// The shown member's portrait rect.
    fn portrait_rect(c: &Character) -> Rect {
        c.w.iter().find(|w| w.id == "character.portrait").expect("the portrait's widget").rect
    }

    /// R1 and R3: on every tab at every text size the portrait is 48 px inside the frame at its top-left (left of the
    /// member picker), overlaps no widget and no text, draws the member's file; the screen reads clean (TEXTFIT1).
    #[test]
    fn r1_r3_the_member_portrait_sits_clear_on_every_tab() {
        let mut cx = roster_cx(game_cx(), "party_r1", Vec::new(), [200, 80, 40]);
        let member = cx.gs().party.members[0].id.clone();
        let fr = Rect::new(16, 16, 608, 328);
        let mut g = gfx();
        for (tab, _) in TABS {
            let mut c = Character::new(&cx, &member, tab);
            for (size, scale) in TEXT_SIZES {
                let recs = frame(&mut c, &mut cx, &mut g, scale as f32);
                let pr = portrait_rect(&c);
                assert_eq!((pr.w, pr.h), (PORTRAIT, PORTRAIT), "{tab} at {size}");
                assert!(pr.x > fr.x && pr.y > fr.y + 24 && pr.x + pr.w < fr.x + fr.w && pr.y + pr.h < fr.y + fr.h, "{tab} at {size}: {pr:?} inside the frame, below its title");
                let picker = c.w.iter().find(|w| w.id == "character.member").expect("the picker").rect;
                assert!(pr.x + pr.w < picker.x, "{tab} at {size}: left of the member picker");
                for w in c.w.iter().filter(|w| w.id != "character.portrait") {
                    let x = pr.intersect(w.rect);
                    assert!(x.w <= 0 || x.h <= 0, "{tab} at {size}: the portrait {pr:?} overlaps {} {:?}", w.id, w.rect);
                }
                let on = recs.iter().find(|r| !r.clipped && r.x < (pr.x + pr.w) as f32 && r.x + r.w > pr.x as f32 && r.y < (pr.y + pr.h) as f32 && r.y + r.h > pr.y as f32);
                assert!(on.is_none(), "{tab} at {size}: text on the portrait: {:?}", on.map(|r| &r.text));
                assert!(c.faces.get(&format!("member:{member}")).and_then(|f| f.composed()).is_some(), "{tab} at {size}: the member's file drew");
                let boxes: Vec<(String, f32, f32, f32, f32)> = c.w.iter().map(|w| (w.id.clone(), w.rect.x as f32, w.rect.y as f32, w.rect.w as f32, w.rect.h as f32)).collect();
                let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
                for k in ["edge", "overlap", "spill"] {
                    assert_eq!(v["counts"][k], json!(0), "{tab} at {size}: {v}");
                }
            }
        }
    }

    /// R2: a wounded member's portrait holds the `wounded` row's desaturate 0.35; E draws the next member's portrait (not
    /// wounded: no desaturate) and the old one's `PortraitFx` is dropped at the next frame; Q goes back. The screen's
    /// `state` lists only the shown member's portrait.
    #[test]
    fn r2_a_wounded_member_desaturates_and_cycling_drops_the_old_portrait() {
        let rows = vec![row(json!({"id": "wounded", "trigger": "condition", "condition": "wounded", "effect": "desaturate", "intensity": 0.35}))];
        let mut cx = game_cx();
        let mut copy = cx.gs().party.members[0].clone();
        copy.id = "test_member_r2".into();
        cx.gs_mut().party.members.push(copy);
        let mut cx = roster_cx(cx, "party_r2", rows, [200, 80, 40]);
        let first = cx.gs().party.members[0].id.clone();
        cx.gs_mut().party.member_mut(&first).unwrap().add_condition(emberwake_world::roster::WOUNDED);
        let mut g = gfx();
        let mut c = Character::new(&cx, &first, "stats");
        let mut draw_at = |c: &mut Character, cx: &mut Cx, t: u64| {
            c.faces.clock = Clock { ms: t as f64, hold: 0.0 };
            frame(c, cx, &mut g, 1.0);
        };
        for t in [1000, 1000 + EASE_MS] {
            draw_at(&mut c, &mut cx, t);
        }
        let now = c.faces.now();
        let key = format!("member:{first}");
        assert!(c.faces.get(&key).and_then(|f| f.level("desaturate", now)).is_some_and(|l| (l.i - 0.35).abs() < 1e-3), "wounded: desaturate 0.35");
        c.key(&mut cx, Key::Char('e'));
        let next = c.member.clone();
        assert_ne!(next, first, "E shows the next member");
        for t in [2000, 2000 + EASE_MS] {
            draw_at(&mut c, &mut cx, t);
        }
        let now = c.faces.now();
        let next_key = format!("member:{next}");
        assert!(c.faces.get(&next_key).is_some_and(|f| f.composed().is_some() && f.level("desaturate", now).is_none()), "the new member's portrait, not wounded");
        assert!(c.faces.get(&key).is_none(), "the old member's portrait is dropped");
        let listed: Vec<Value> = c.state(&cx)["portraits"].as_array().cloned().unwrap_or_default();
        assert!(listed.len() == 1 && listed[0]["key"] == json!(next_key), "{listed:?}");
        c.key(&mut cx, Key::Char('q'));
        draw_at(&mut c, &mut cx, 3000);
        assert_eq!(c.member, first, "Q goes back");
        assert!(c.faces.get(&key).is_some(), "the first member's portrait again");
    }
}

/// INTENT2-U (W2-4 slice 7b; INTENT2 section 6 "Stacks & Auto-Fill"): the Gear tab's battle pouch under the played
/// world's `pouch_stacks` (switched on here in memory, with a stack limit of 5 on a catalog consumable: no shipped
/// numbers): stack slots with pips only (filled, hollow up to the packed count, dimmed above it; no counter), lower and
/// raise buttons, the draft topped up from the stash, a second slot of the same item refused, the packed counts saved on
/// Confirm. With the switch off the classic pouch is unchanged.
#[cfg(test)]
mod pouch_stack_tests {
    use super::*;
    use crate::hud::hud_tests::{frame, game_cx, gfx};

    /// A game whose world plays `pouch_stacks` as `on`, a battle consumable with stack limit 5 and `stash` copies of it
    /// in the stash, the first member's pouch empty. Returns (cx, member, item).
    fn stack_cx(on: bool, stash: i64) -> (Cx, String, String) {
        let mut cx = game_cx();
        let world = cx.gs().world_id.clone();
        let mut data = (*cx.app.data).clone();
        let rid = data.ruleset_id(&world);
        let item = data.cat.items.values().filter(|i| i.offered_in_battle()).map(|i| i.id.clone()).min().expect("a battle consumable");
        data.edit_cat(|cat| {
            let rid = if cat.rulesets.contains_key(&rid) { rid.clone() } else { emberwake_rules::ruleset::CLASSIC.to_string() };
            cat.rulesets.entry(rid.clone()).or_insert_with(|| json!({"id": rid}).as_object().unwrap().clone()).insert("pouch_stacks".into(), json!(if on { "on" } else { "off" }));
            cat.items.get_mut(&item).unwrap().max_stack = 5;
        });
        cx.app.data = std::sync::Arc::new(data);
        assert_eq!(cx.data.ruleset(&world).numbers.pouch_stacks, on);
        let member = cx.gs().party.members[0].id.clone();
        let gs = cx.gs_mut();
        let m = gs.party.member_mut(&member).unwrap();
        m.pouch.clear();
        m.pouch_packed.clear();
        gs.party.stash.shift_remove(&item);
        if stash > 0 {
            gs.party.stash.insert(item.clone(), stash);
        }
        (cx, member, item)
    }

    /// TEXTFIT1: no text leaves the canvas, overlaps another or runs out of its control, at every text size.
    fn assert_clean(screen: &mut dyn Screen<GameCx>, cx: &mut Cx, what: &str) {
        let mut g = gfx();
        for (size, scale) in aegis_ui::screen::TEXT_SIZES {
            let recs = frame(screen, cx, &mut g, scale as f32);
            let boxes: Vec<(String, f32, f32, f32, f32)> = screen.widgets().into_iter().map(|w| {
                let (x, y, ww, h) = crate::hud::hud_tests::rect(&w);
                (w["id"].as_str().unwrap_or("").to_string(), x as f32, y as f32, ww as f32, h as f32)
            }).collect();
            let v = aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes);
            for k in ["edge", "overlap", "spill"] {
                assert_eq!(v["counts"][k], json!(0), "{what} at {size}: {v}");
            }
        }
    }

    fn draft(c: &Character) -> &(UnitState, Stash) {
        c.gear.as_ref().expect("the gear draft")
    }

    fn widget<'a>(c: &'a Character, id: &str) -> Option<&'a Widget<PAct>> {
        c.w.iter().find(|w| w.id == id)
    }

    #[test]
    fn the_pips_are_filled_hollow_to_the_packed_count_then_dimmed_capped_at_5() {
        use Pip::*;
        assert_eq!(pouch_pips(2, 4, 5), [Filled, Filled, Hollow, Hollow, Dim]);
        assert_eq!(pouch_pips(3, 3, 3), [Filled, Filled, Filled], "one pip per copy the stack holds");
        assert_eq!(pouch_pips(0, 1, 9), [Hollow, Dim, Dim, Dim, Dim], "capped at 5 pips");
    }

    /// Assigning a slot packs `min(5, max_stack)` even when the stash holds fewer, and moves no copy (the Party Screen
    /// does not top up: the kit's auto-fill does at combat start and after a fight), so its pips are hollow; the slot
    /// shows pips and no counter text; - and + set the packed count in its range (a lower count gives the copies above it
    /// back to the stash); a second slot of the same item is refused; Confirm saves the packed count with the pouch.
    #[test]
    fn a_stack_slot_packs_its_limit_shows_pips_and_saves_the_packed_count() {
        let (mut cx, member, item) = stack_cx(true, 2);
        let mut c = Character::new(&cx, &member, "gear");
        frame(&mut c, &mut cx, &mut gfx(), 1.0);
        c.act(&mut cx, PAct::PouchAdd(item.clone()));
        let (work, stash) = draft(&c).clone();
        assert_eq!((work.pouch_count(&item), work.packed(&item), stash[&item]), (0, 5, 2), "packs 5 though the stash has 2; no copy moves here");
        let texts: Vec<String> = frame(&mut c, &mut cx, &mut gfx(), 1.0).iter().map(|r| r.text.clone()).collect();
        assert_eq!(widget(&c, "gear.pouch.0.pips").expect("the slot's pips").text, "0/5/5", "filled / packed / limit: all hollow");
        assert!(!texts.iter().any(|t| t.contains("0/5") || t.contains("of 5") || t == "0" || t == "5"), "no counter text: {texts:?}");
        assert!(widget(&c, "gear.pouch.0.more").is_some_and(|w| !w.enabled), "+ is off at the stack limit");
        c.act(&mut cx, PAct::PouchAdd(item.clone()));
        assert_eq!(draft(&c).0.pouch_slots(), std::slice::from_ref(&item), "a second slot of the same item is refused");
        c.act(&mut cx, PAct::PouchPack(item.clone(), 0));
        assert_eq!(draft(&c).0.packed(&item), 5, "below 1 refused");
        c.act(&mut cx, PAct::PouchPack(item.clone(), 3));
        c.act(&mut cx, PAct::ConfirmGear);
        let m = cx.gs().party.member(&member).unwrap();
        assert_eq!((m.pouch_count(&item), m.packed(&item), cx.gs().party.stash[&item]), (0, 3, 2), "saved with the packed count");
        assert!(m.validate(&cx.data).is_empty(), "{:?}", m.validate(&cx.data));
        // copies in the pouch (as auto-fill leaves them): a lower count gives the extra back
        let gs = cx.gs_mut();
        gs.party.member_mut(&member).unwrap().pouch = vec![item.clone(); 3];
        let mut c = Character::new(&cx, &member, "gear");
        frame(&mut c, &mut cx, &mut gfx(), 1.0);
        assert_eq!(widget(&c, "gear.pouch.0.pips").unwrap().text, "3/3/5");
        c.act(&mut cx, PAct::PouchPack(item.clone(), 1));
        let (work, stash) = draft(&c).clone();
        assert_eq!((work.pouch_count(&item), work.packed(&item), stash[&item]), (1, 1, 4), "the copies above 1 go back");
        frame(&mut c, &mut cx, &mut gfx(), 1.0);
        assert!(widget(&c, "gear.pouch.0.less").is_some_and(|w| !w.enabled), "- is off at 1");
        assert_clean(&mut c, &mut cx, "the gear tab with a stack slot");
        c.act(&mut cx, PAct::PouchRemove(0));
        let (work, stash) = draft(&c).clone();
        assert!(work.pouch_slots().is_empty() && work.pouch_packed.is_empty(), "the slot empties");
        assert_eq!(stash[&item], 5, "its copies go back");
    }

    /// Drops under `pouch_stacks`: an item dropped on its own slot is a no-op; one dropped on another item's slot takes
    /// that place (the old one's copies back); a slot dragged to another place keeps it.
    #[test]
    fn stack_slot_drops_keep_their_places() {
        let (cx, member, a) = stack_cx(true, 2);
        let d = &cx.data;
        let others: Vec<String> = d.cat.items.values().filter(|i| i.offered_in_battle() && i.id != a).map(|i| i.id.clone()).collect();
        let (b, c) = (others[0].clone(), others[1].clone());
        let mut m = cx.gs().party.member(&member).unwrap().clone();
        let mut st: Stash = [(a.clone(), 2), (b.clone(), 1), (c.clone(), 1)].into_iter().collect();
        drop_move(d, "", &mut m, &mut st, &format!("stash.{a}"), "pouch.0", true).unwrap();
        drop_move(d, "", &mut m, &mut st, &format!("stash.{b}"), "pouch.1", true).unwrap();
        let before = (m.clone(), st.clone());
        drop_move(d, "", &mut m, &mut st, &format!("stash.{a}"), "pouch.0", true).unwrap();
        assert_eq!((m.clone(), st.clone()), before, "its own slot: nothing changes");
        drop_move(d, "", &mut m, &mut st, "pouch.1", "pouch.0", true).unwrap();
        assert_eq!(m.pouch_slots(), [b.clone(), a.clone()], "the moved slot keeps its place");
        // a copy in the slot, as the kit's auto-fill would have put there at a fight's start
        m.pouch.push(a.clone());
        drop_move(d, "", &mut m, &mut st, &format!("stash.{c}"), "pouch.1", true).unwrap();
        assert_eq!(m.pouch_slots(), [b.clone(), c.clone()], "the dropped item takes that place");
        assert_eq!(st[&a], 3, "the replaced slot's copy came back");
        // a key-less slot (copies with no packed target, as an old save before its first fight) sorts after the keyed
        // ones: a drop on slot 0 puts the new item there and leaves the key-less slot where it is
        let k = others[2].clone();
        let mut m = cx.gs().party.member(&member).unwrap().clone();
        m.pouch = vec![k.clone()];
        m.pouch_packed.clear();
        drop_move(d, "", &mut m, &mut st, &format!("stash.{b}"), "pouch.1", true).unwrap();
        assert_eq!(m.pouch_slots(), [b.clone(), k.clone()], "the keyed slot first, the key-less one after it");
        drop_move(d, "", &mut m, &mut st, &format!("stash.{a}"), "pouch.0", true).unwrap();
        assert_eq!(m.pouch_slots(), [a.clone(), k.clone()], "the new item in slot 0; the key-less slot not moved");
        assert!(!m.pouch_packed.contains_key(&k), "the key-less slot untouched");
    }

    /// The classic pouch (switch off): copies, no pips, no packed counts.
    #[test]
    fn without_pouch_stacks_the_classic_pouch_is_unchanged() {
        let (mut cx, member, item) = stack_cx(false, 2);
        let mut c = Character::new(&cx, &member, "gear");
        frame(&mut c, &mut cx, &mut gfx(), 1.0);
        c.act(&mut cx, PAct::PouchAdd(item.clone()));
        let (work, stash) = draft(&c).clone();
        assert_eq!((work.pouch.len(), work.pouch_packed.len(), stash[&item]), (1, 0, 1), "one copy a click");
        frame(&mut c, &mut cx, &mut gfx(), 1.0);
        assert!(widget(&c, "gear.pouch.0").is_some() && widget(&c, "gear.pouch.0.pips").is_none(), "the classic rows");
    }
}
