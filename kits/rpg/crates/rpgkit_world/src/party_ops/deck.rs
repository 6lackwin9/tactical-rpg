//! The deck rule's editing terms (RULESET1 9 D1-D6, rule `deck`; engine-only, no Godot original): what a hero may put in
//! its deck and every pure edit of a deck draft (slot, unslot, drop, orders, follow-up lists), shared by the party screen
//! and the Dojo. PR5-31 moved it from the game's party screen with no rule change.

use super::title_case;
use crate::progression;
use crate::{Data, UnitState};
use indexmap::IndexMap;
use rpgkit_battle::round_intent::RoundIntent;
use rpgkit_data::{Catalog, Deck, UnitDef};
use rpgkit_rules::ruleset::{DeckSlots, Ruleset, FIELD_INTENT};
use rpgkit_rules::{DeckRule, Unit};

/// RULESET1 9 D6: the deck's slot kinds in bucket order: (kind, bucket header, what its slots take).
pub const DECK_KINDS: [(&str, &str, &str); 3] = [("active", "Actives", "active skills"), ("passive", "Passives", "passives"), ("keystone", "Keystone", "keystones")];

/// RULESET1 9 D1-D6 (rule `deck`; engine-only, no Godot original): what a hero may put in its deck: the owned actives
/// (never a basic) and slottable passives and keystones, the ruleset's basics (open to all, no slot) and intents, each
/// family's intent, the slot counts, and what an empty deck (D5 auto) or an empty active row stands for. Editing is pure
/// on these terms (every edit starts from the deck as it plays, so a first edit pins the automatic deck), shared by the
/// party screen and the Dojo.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct DeckTerms {
    pub slots: DeckSlots,
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

/// TERMS1 TU5: every intent's name through `cat`'s terms (the game's `terms` row, else the default), by intent id.
pub fn intent_names(cat: &Catalog) -> IndexMap<String, String> {
    RoundIntent::ALL.into_iter().map(|i| (i.id().to_string(), i.label(cat))).collect()
}

/// An intent id's display name ("assault" -> "Assault"; a renamed intent its `names` entry, from `intent_names`).
pub fn intent_title(names: &IndexMap<String, String>, id: &str) -> String {
    match names.get(id) {
        Some(n) => n.clone(),
        None => RoundIntent::parse(id).map(|x| x.name().to_string()).unwrap_or_else(|| title_case(id)),
    }
}

/// A4.2-K1b: each family's primary intent and rows under `rs` (`intent::allowed`).
fn rows_of<'a>(cat: &Catalog, rs: &Ruleset, fams: impl Iterator<Item = &'a String>) -> (IndexMap<String, String>, IndexMap<String, Vec<String>>) {
    let mut primary = IndexMap::new();
    let mut rows = IndexMap::new();
    for f in fams {
        if let Some(s) = cat.tier_one(f) {
            let r: Vec<String> = rpgkit_rules::intent::allowed(rs, &rpgkit_rules::intent::IntentInput::of_def(s), &s.round_intent).into_iter().map(String::from).collect();
            primary.insert(f.clone(), r.first().cloned().unwrap_or_else(|| rpgkit_battle::round_intent::skill_intent(s).to_string()));
            rows.insert(f.clone(), r);
        }
    }
    (primary, rows)
}

/// A4.2-K1b: the intents a deck order lists: the ruleset's, and Field under `field_intent`.
fn order_intents(rs: &Ruleset) -> Vec<String> {
    let mut out = rs.numbers.intents.clone();
    if rs.has(FIELD_INTENT) && !out.iter().any(|i| i == "field") {
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
    pub fn for_member(d: &Data, m: &UnitState, world: &str, rs: &Ruleset) -> Option<Self> {
        // CLASS-BASICS: the member's class basics are open beside the ruleset's (its class in that world)
        let rule = DeckRule::of(rs)?.with_class(d.cat_in(world), &m.class_id(d));
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
            // PR5-26 (audit B4): the base unit's channeled marks among the automatic actives
            channeled: auto_unit.deck.channeled.iter().filter(|f| auto_active.contains(f)).cloned().collect(),
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
    pub fn for_unit(cat: &Catalog, rs: &Ruleset, def: &UnitDef, u: &Unit) -> Option<Self> {
        let rule = DeckRule::of(rs)?.with_class(cat, &def.class_id);
        let basics: Vec<String> = rule.basics.iter().filter(|b| cat.tier_one(b).is_some()).cloned().collect();
        let mut actives: Vec<String> = Vec::new();
        for s in cat.skills.iter().filter(|s| !s.system && !s.basic && !rpgkit_battle::skill::is_item(s)) {
            let f = s.family_id().to_string();
            if !actives.contains(&f) && !rule.basics.contains(&f) && !cat.is_basic_family(&f) && !cat.is_system(&f) && cat.tier_one(&f).is_some() && u.owns_skill(cat, &s.id) {
                actives.push(f);
            }
        }
        let of_slot = |slot: &str| -> Vec<String> { cat.passives.values().filter(|p| p.slot == slot).map(|p| p.id.clone()).collect() };
        let (passives, keystones) = (of_slot("passive"), of_slot("keystone"));
        let mut empty = def.clone();
        empty.deck = Deck::default();
        let auto_unit = rpgkit_rules::build_unit_in(cat, &empty, 0, Some(&rule));
        let classic = rpgkit_rules::build_unit(cat, def, 0);
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

    /// The entries of `deck` of slot kind `kind`.
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
        // PR5-26 (audit B4): the channeled actives stay (they mark the automatic actives)
        Deck { active: Vec::new(), passive: shown.passive, keystone: shown.keystone, order: IndexMap::new(), follow_ups: IndexMap::new(), channeled: shown.channeled }
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
    /// emptied list is the auto order again; INPUT1 FL3). PR5-26 (audit B4): and its channeled mark.
    fn unordered(d: &mut Deck, fam: &str) {
        d.channeled.retain(|f| f != fam);
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

    /// "Auto order": every intent back to the automatic order (the deck as it plays, pinned).
    pub fn order_auto(&self, draft: &Deck) -> Deck {
        let mut d = self.shown(draft);
        d.order.clear();
        d
    }

    /// `follow.<op>.<k>`: place `k` of opener `op`'s follow-up list (`k` = the list's length: its end).
    pub fn follow_key(key: &str) -> Option<(&str, usize)> {
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
    pub fn slot_key(key: &str) -> Option<(&'static str, usize)> {
        let (k, i) = key.strip_prefix("deck.")?.split_once('.')?;
        Some((DECK_KINDS.iter().find(|x| x.0 == k)?.0, i.parse().ok()?))
    }

    /// `order.<intent>.<i>`: a place in an intent's order.
    pub fn order_key(key: &str) -> Option<(&str, usize)> {
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
