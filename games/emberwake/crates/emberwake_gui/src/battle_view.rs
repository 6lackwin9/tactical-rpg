//! BATTLE-UI2 (PR7-40, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` sections 2-6; `docs/specs/UISPLIT1.md`
//! US1): the battle screen's view-model. `model` reads what the screen shows from the session (the top bar's round,
//! objective and AP line, the party and guest plates, the foe plates and groups, the docked foe card, the status tabs),
//! with no rects or colours; the battle drawing (`battle/plates.rs`, `battle/card.rs`) draws it to the contract, and
//! `view` names the same as a `screen_layout` view for a layout file (`ui/layouts/battle.json`, Art/UI) and the agent
//! link. Its fixture is `ui/views/battle.view.json` (`tests::the_fixture_has_the_view_shape`).
//! Every damage and pip number comes from the kit's one forecast convention over the round as it will run
//! (`intent::planned_schedule`, `intent::hits_on`, `intent::foe_card`), never a list of its own.

use aegis_ui::screen_layout::{Field, RowView, View};
use emberwake_battle::intent::{self, FoeCard};
use emberwake_battle::resolver::ScheduledAction;
use emberwake_battle::round_intent::{self as ri, RoundIntent};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::{Phase, Session};
use emberwake_battle::world::{UnitId, World};

/// The view's screen name (its layout and fixture files).
pub const SCREEN: &str = "battle";
/// Row lists: the party plates, the guest plates, the foe plates, the foe card's rows.
pub const PARTY: &str = "party";
pub const GUESTS: &str = "guests";
pub const FOES: &str = "foes";
pub const CARD_ROWS: &str = "card_rows";
/// Actions: the bar's controls; `select` (arg the unit) and `pin` (arg the foe) from a plate.
pub const ACTIONS: [(&str, &str); 7] = [("party_tactic", "Party tactic"), ("hero_tactic", "Hero tactic"), ("auto", "Auto"), ("clear", "Clear"), ("confirm", "Confirm"), ("select", "Select"), ("pin", "Pin")];

/// Contract 4: up to this many foes each have a full plate; from the next, all but the target go compact.
pub const FULL_PLATES: usize = 6;
/// Contract 4: from this many foes, identical ones group into one row.
pub const GROUP_FROM: usize = 11;
/// Contract 3: a plate's status column holds this many slots (3 tabs and "+N" past it).
pub const STATUS_SLOTS: usize = 4;
/// Contract 4: a compact foe plate shows up to this many status tabs.
pub const COMPACT_TABS: usize = 2;
/// Contract 5: a card row shows this many status chips, then "+N".
pub const CARD_CHIPS: usize = 3;

/// A unit's AP for the plate's bar (contract 3): the start AP, the part the plan spends, the cap.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ApBar {
    pub start: i32,
    pub planned: i32,
    pub cap: i32,
}

impl ApBar {
    /// The balance after the plan ("4/12").
    pub fn left(&self) -> i32 {
        (self.start - self.planned).max(0)
    }
}

/// One status tab (contract 6): its icon (the kit's `statuses/<id>.png`), buff or debuff rim, the rounds left (`None` a
/// permanent one; a full bar), dashed when a planned move applies it; its list line.
#[derive(Debug, Clone, PartialEq)]
pub struct Tab {
    pub status: String,
    pub icon: String,
    pub buff: bool,
    pub rounds: Option<i32>,
    pub planned: bool,
    pub name: String,
}

impl Tab {
    /// The full list's row: "Bleed 3 HP: 2 rounds", "new" for a planned one (contract 6).
    pub fn line(&self) -> String {
        let left = match (self.planned, self.rounds) {
            (true, _) => "new".to_string(),
            (_, None) => "lasts".to_string(),
            (_, Some(1)) => "1 round".to_string(),
            (_, Some(n)) => format!("{n} rounds"),
        };
        format!("{}: {left}", self.name)
    }
}

/// A foe's next blow (contract 4, the "when it lands" row): its kind, its target and the block it lands.
#[derive(Debug, Clone, PartialEq)]
pub struct Lands {
    pub kind: String,
    pub target: Option<UnitId>,
    pub impact: i32,
}

/// One plate (contract 3, 4): a party member, a guest or a foe.
#[derive(Debug, Clone, PartialEq)]
pub struct Plate {
    pub unit: UnitId,
    /// The plate's name (a foe's short name, `UnitDef::plate_name`; a hero's name).
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    /// The HP the round's planned blows take from it (a red loss at the bar's end).
    pub incoming: i32,
    /// Contract 6 (PR7-40 part 2): the HP the round's planned heals restore on it (a pale moss gain on the bar;
    /// `intent::heal_on`).
    pub heal: i32,
    pub ap: Option<ApBar>,
    /// Poise pips (lit, of) under the rule poise.
    pub poise: Option<(i32, i32)>,
    pub tabs: Vec<Tab>,
    pub intent: Option<RoundIntent>,
    pub boss: bool,
    pub guest: bool,
    pub protect: bool,
    pub active: bool,
    pub selected: bool,
    pub lands: Option<Lands>,
}

/// One row of the foe panel (contract 4).
#[derive(Debug, Clone, PartialEq)]
pub enum FoeEntry {
    Full(Plate),
    Compact(Plate),
    /// Identical foes in one row ("Ash-Crawler x4"; a click expands it). `key` is their catalog unit; an open group
    /// is a slim header (a click collapses it) followed by its foes' compact plates.
    Group { key: String, name: String, plates: Vec<Plate>, open: bool },
}

impl FoeEntry {
    pub fn units(&self) -> Vec<UnitId> {
        match self {
            FoeEntry::Full(p) | FoeEntry::Compact(p) => vec![p.unit],
            FoeEntry::Group { plates, open: false, .. } => plates.iter().map(|p| p.unit).collect(),
            // an open group's foes are the compact plates after it
            FoeEntry::Group { .. } => Vec::new(),
        }
    }
}

/// The top bar (contract 2).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct TopBar {
    /// "Round 1 of 8".
    pub round: String,
    /// The goal and the protect line ("" none).
    pub goal: String,
    pub protect: String,
    /// The selected hero's AP: now, picked, the ghost's, the cap (contract 2's AP line).
    pub ap_now: i32,
    pub ap_picked: i32,
    pub ap_ghost: i32,
    pub ap_cap: i32,
    /// The blocks a foe's blow lands on the selected hero (the ruler's red marks).
    pub impacts: Vec<i32>,
}

impl TopBar {
    /// "AP 10/12 -6".
    pub fn ap_text(&self) -> String {
        if self.ap_picked > 0 { format!("AP {}/{} -{}", self.ap_now, self.ap_cap, self.ap_picked) } else { format!("AP {}/{}", self.ap_now, self.ap_cap) }
    }

    /// "(-4)" for the ghost's cost ("" none).
    pub fn ghost_text(&self) -> String {
        if self.ap_ghost > 0 { format!("(-{})", self.ap_ghost) } else { String::new() }
    }
}

/// One row of the docked foe card (contract 5): in the hero view one move of that hero, in the party view one block.
#[derive(Debug, Clone, PartialEq)]
pub struct CardRow {
    /// The hero view: the move's skill, first block and landing block (its lit blocks); the party view: the block.
    pub skill: String,
    pub name: String,
    pub start: i32,
    pub impact: i32,
    /// The allies hitting in it (the party view's faces; the hero alone in the hero view).
    pub units: Vec<UnitId>,
    pub damage: i32,
    /// The pips each hit breaks, in order; `true` beside a hit that empties the bar (the split shield).
    pub pips: Vec<(i32, bool, bool)>,
    pub statuses: Vec<String>,
    pub after_ko: bool,
    /// The party view: the foe falls in this block ("KO").
    pub ko: bool,
}

/// The docked foe card (contract 5).
#[derive(Debug, Clone, PartialEq)]
pub struct CardView {
    pub foe: UnitId,
    /// The full name and the character's title ("" none).
    pub name: String,
    pub title: String,
    pub boss: bool,
    /// The hero view's hero (None: the party view).
    pub hero: Option<UnitId>,
    pub read: FoeCard,
    pub rows: Vec<CardRow>,
}

impl CardView {
    /// "KO in block 5", with " (star 3)" when the crits down it by then half the time; "" without a KO block.
    pub fn ko_text(&self) -> String {
        match (self.read.ko_block, self.read.crit_star) {
            (Some(b), Some(n)) => format!("KO in block {b} (star {n})"),
            (Some(b), None) => format!("KO in block {b}"),
            _ => String::new(),
        }
    }

    /// "BREAK in block 4" ("" none).
    pub fn break_text(&self) -> String {
        self.read.break_block.map_or_else(String::new, |b| format!("BREAK in block {b}"))
    }

    /// The selected hero's share and the rest of the party's of the planned damage (the header's bright and dark).
    pub fn shares(&self) -> (i32, i32) {
        let own = self.hero.map_or(0, |h| self.read.total(Some(h)));
        (own, self.read.total(None) - own)
    }
}

/// The whole view-model.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BattleModel {
    pub top: TopBar,
    pub party: Vec<Plate>,
    pub guests: Vec<Plate>,
    pub foes: Vec<FoeEntry>,
    /// "Hostiles N" and the reinforcement tag ("+2 round 3"; "" none).
    pub hostiles: String,
    pub reinforce: String,
    pub card: Option<CardView>,
}

/// The round as it will run: while planning the plan booked after the enemies' actions (`planned_schedule`), else
/// the resolver's.
pub fn round_schedule(s: &Session) -> Vec<ScheduledAction> {
    if s.phase == Phase::Planning && !s.plan_confirmed {
        intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan)
    } else {
        s.resolver.scheduled.clone()
    }
}

/// Contract 6 order: the soonest to run out first, ties to the one applied first; queued ones after; `planned` ones
/// (statuses a planned move adds) last.
pub fn tabs_of(w: &World, u: UnitId, planned: &[String]) -> Vec<Tab> {
    use emberwake_rules::status;
    let st = &w.units[u].st;
    let tab = |id: &str, rounds: Option<i32>, planned: bool, magnitude: i32| Tab { status: id.to_string(), icon: emberwake_content::art::status_icon(id), buff: status::is_buff(id), rounds, planned, name: status::label_with_magnitude(id, magnitude) };
    let mut now: Vec<(usize, Tab)> = st
        .active
        .iter()
        .enumerate()
        .map(|(k, id)| (k, tab(id, st.rounds_left.get(id).copied().filter(|r| *r > 0), false, st.magnitudes.get(id).copied().unwrap_or(0))))
        .collect();
    now.sort_by_key(|(k, t)| (t.rounds.unwrap_or(i32::MAX), *k));
    let mut out: Vec<Tab> = now.into_iter().map(|x| x.1).collect();
    out.extend(st.pending.iter().filter(|id| !st.active.contains(*id)).map(|id| tab(id, st.pending_rounds.get(id).copied().filter(|r| *r > 0), false, st.pending_magnitudes.get(id).copied().unwrap_or(0))));
    for id in planned {
        if !out.iter().any(|t| &t.status == id) {
            out.push(tab(id, None, true, 0));
        }
    }
    out
}

/// A foe's first live blow this round (the "when it lands" row): the kind of its first Attack or Area, its target and
/// impact block.
fn lands_of(w: &World, sched: &[ScheduledAction], foe: UnitId) -> Option<Lands> {
    use emberwake_battle::command::CmdKind;
    intent::actions_of(foe, sched).into_iter().find_map(|a| {
        let target = match &a.command.kind {
            CmdKind::Attack { target, .. } => target.or(w.units[foe].active_target),
            CmdKind::Area { target, .. } => *target,
            _ => return None,
        };
        let kind = w.cat.skill(&a.command.skill).map_or_else(|| "ATTACK".to_string(), |s| s.kind.clone());
        Some(Lands { kind, target, impact: a.impact })
    })
}

/// The poise pips (lit, of) of `u` under the rule poise.
fn poise_of(w: &World, u: UnitId) -> Option<(i32, i32)> {
    use emberwake_battle::poise;
    let st = poise::state(w, u).filter(|st| st.max > 0)?;
    let n = &w.ruleset.numbers;
    if st.broken {
        return Some((0, n.poise_pips.max(1)));
    }
    Some(if poise::pips_mode(n) { (poise::pips_now(n, st).min(n.poise_pips.max(1)), n.poise_pips.max(1)) } else { (poise::bar_pips(n, st), n.poise_pips.max(1)) })
}

/// What the model reads beside the session: the selected hero, the pinned or hovered foe, the round-start AP of each
/// member, each unit's plate name and the foe's full name and title (from the game's data), the bosses.
pub struct Inputs<'a> {
    pub sel: Option<UnitId>,
    pub card_foe: Option<UnitId>,
    pub round_ap: &'a [(UnitId, i32)],
    pub plate_name: &'a dyn Fn(UnitId) -> String,
    pub full_name: &'a dyn Fn(UnitId) -> (String, String),
    pub intent_of: &'a dyn Fn(UnitId) -> Option<RoundIntent>,
    /// Contract 4: the groups the player opened (their catalog units).
    pub open_groups: &'a [String],
}

/// BATTLE-UI2: the view-model of session `s` (contract 2-6).
pub fn model(s: &Session, inp: &Inputs) -> BattleModel {
    let w = &s.w;
    let sched = round_schedule(s);
    let planning = s.phase == Phase::Planning && !s.plan_confirmed;
    let timeline = ri::on(w);
    let mut m = BattleModel::default();
    let name_of = |r: &str| s.units_of_ref(r).first().map(|&c| w.units[c].name.clone()).unwrap_or_else(|| r.to_string());
    m.top.round = format!("Round {} of {}", s.current_round.max(1), s.def.max_rounds);
    m.top.goal = emberwake_battle::encounter::Objectives::goal_of(&s.def.raw, name_of);
    m.top.protect = emberwake_battle::encounter::Objectives::protect_of(&s.def.raw, name_of);
    // the hits the enemies' blows land on each ally this round
    let incoming = |u: UnitId| -> Vec<intent::HitOn> { w.enemies.iter().flat_map(|&e| intent::hits_on(w, &sched, e, u)).collect() };
    if let Some(h) = inp.sel.filter(|h| w.allies.contains(h)) {
        let f = rip::ap_forecast(w, &s.plan, h);
        let start = inp.round_ap.iter().find(|r| r.0 == h).map_or(f.now, |r| r.1);
        (m.top.ap_now, m.top.ap_cap) = (start, f.max);
        if planning {
            (m.top.ap_picked, m.top.ap_ghost) = (f.picked, f.filled);
        }
        let mut blocks: Vec<i32> = incoming(h).iter().map(|x| x.block).collect();
        blocks.sort();
        blocks.dedup();
        m.top.impacts = blocks;
    }
    // the foe card (contract 5): while a hero is planning, its target (the card stays there); else the pinned or hovered
    // foe, else the selected hero's target
    let target_of = |h: UnitId| w.round_states.get(&h).and_then(|r| r.target).and_then(|t| match t {
        ri::RoundTarget::Unit(u) => Some(u),
        _ => None,
    });
    let planning_target = inp.sel.filter(|_| planning).and_then(target_of).filter(|f| w.enemies.contains(f));
    let card_foe = planning_target.or(inp.card_foe).or_else(|| inp.sel.and_then(target_of)).filter(|f| w.enemies.contains(f));
    let cards: Vec<(UnitId, FoeCard)> = w.enemies.iter().filter(|e| w.units[**e].is_active()).map(|&e| (e, intent::foe_card(w, &sched, e))).collect();
    let plate = |u: UnitId, guest: bool| {
        let unit = &w.units[u];
        let ally = w.allies.contains(&u);
        let ap = (ally && !guest && timeline).then(|| {
            let start = inp.round_ap.iter().find(|r| r.0 == u).map_or(unit.ap, |r| r.1);
            let planned = if planning { rip::ap_forecast(w, &s.plan, u).spent } else { 0 };
            ApBar { start, planned: planned.min(start), cap: unit.max_ap }
        });
        let planned_statuses: Vec<String> = cards.iter().find(|c| c.0 == u).map(|c| c.1.hits.iter().flat_map(|h| h.statuses.clone()).collect()).unwrap_or_default();
        let incoming_hp = if ally { incoming(u).iter().map(|x| x.damage).sum() } else { cards.iter().find(|c| c.0 == u).map_or(0, |c| c.1.total(None)) };
        Plate {
            unit: u,
            name: (inp.plate_name)(u),
            hp: unit.hp,
            max_hp: unit.max_hp,
            incoming: incoming_hp.min(unit.hp.max(0)),
            heal: if planning { intent::heal_on(w, &sched, u) } else { 0 },
            ap,
            poise: poise_of(w, u),
            tabs: tabs_of(w, u, &planned_statuses),
            intent: (inp.intent_of)(u).filter(|_| unit.is_active()),
            boss: !ally && unit.tier == "boss",
            guest,
            protect: s.protected.contains(&u),
            active: unit.is_active(),
            selected: inp.sel == Some(u),
            lands: if ally { None } else { lands_of(w, &sched, u) },
        }
    };
    for &a in &w.allies {
        let guest = s.guests.contains(&a);
        let p = plate(a, guest);
        if guest { m.guests.push(p) } else { m.party.push(p) }
    }
    // the foes on the field (reinforcements not placed yet are left out)
    let shown: Vec<UnitId> = w.enemies.iter().copied().filter(|&e| w.grid.pos(e).is_some() || !w.units[e].is_alive() || w.units[e].is_withdrawn).collect();
    let active = shown.iter().filter(|e| w.units[**e].is_active()).count();
    m.hostiles = format!("Hostiles {active}");
    m.reinforce = s.phases.next_reinforcement().map_or_else(String::new, |(n, r)| format!("+{n} round {r}"));
    let keep_full = card_foe.or_else(|| inp.sel.and_then(target_of));
    if shown.len() >= GROUP_FROM {
        let mut groups: Vec<(String, Vec<Plate>)> = Vec::new();
        for &e in &shown {
            let p = plate(e, false);
            if Some(e) == keep_full {
                groups.push((String::new(), vec![p]));
                continue;
            }
            let key = w.units[e].catalog_id.clone();
            match groups.iter_mut().find(|g| g.0 == key) {
                Some(g) => g.1.push(p),
                None => groups.push((key, vec![p])),
            }
        }
        for (key, plates) in groups {
            if key.is_empty() || plates.len() == 1 {
                let p = plates.into_iter().next().expect("one plate");
                m.foes.push(if Some(p.unit) == keep_full { FoeEntry::Full(p) } else { FoeEntry::Compact(p) });
            } else {
                let name = format!("{} x{}", plates[0].name, plates.len());
                let open = inp.open_groups.contains(&key);
                let members: Vec<Plate> = if open { plates.clone() } else { Vec::new() };
                m.foes.push(FoeEntry::Group { key, name, plates, open });
                m.foes.extend(members.into_iter().map(FoeEntry::Compact));
            }
        }
    } else {
        let compact = shown.len() > FULL_PLATES;
        for &e in &shown {
            let p = plate(e, false);
            m.foes.push(if compact && Some(e) != keep_full { FoeEntry::Compact(p) } else { FoeEntry::Full(p) });
        }
    }
    m.card = card_foe.and_then(|f| {
        let read = cards.iter().find(|c| c.0 == f)?.1.clone();
        let hero = inp.sel.filter(|h| w.allies.contains(h) && w.units[*h].is_active());
        let rows = match hero {
            Some(h) => read.hits.iter().filter(|x| x.unit == h).map(|x| CardRow { skill: x.skill.clone(), name: x.name.clone(), start: x.start, impact: x.impact, units: vec![h], damage: x.damage, pips: vec![(x.pips, x.breaks, x.on_broken)], statuses: x.statuses.clone(), after_ko: x.after_ko, ko: false }).collect(),
            None => read
                .by_block()
                .into_iter()
                .map(|(b, hs)| {
                    let mut units: Vec<UnitId> = hs.iter().map(|x| x.unit).collect();
                    units.dedup();
                    CardRow { skill: String::new(), name: format!("Block {b}"), start: b, impact: b, units, damage: hs.iter().map(|x| x.damage).sum(), pips: hs.iter().map(|x| (x.pips, x.breaks, x.on_broken)).collect(), statuses: hs.iter().flat_map(|x| x.statuses.clone()).collect(), after_ko: hs.iter().all(|x| x.after_ko), ko: read.ko_block == Some(b) }
                })
                .collect(),
        };
        let (name, title) = (inp.full_name)(f);
        Some(CardView { foe: f, name, title, boss: w.units[f].tier == "boss", hero, read, rows })
    });
    m
}

/// "180/242".
fn hp_text(p: &Plate) -> String {
    format!("{}/{}", p.hp, p.max_hp)
}

fn plate_row(p: &Plate) -> RowView {
    let tabs: Vec<String> = p.tabs.iter().map(Tab::line).collect();
    RowView { id: p.unit.to_string(), label: p.name.clone(), detail: hp_text(p), detail2: p.ap.map_or_else(String::new, |a| format!("{}/{}", a.left(), a.cap)), sub: tabs.join(", "), picture: None, enabled: p.active, ..Default::default() }
}

/// The model as a `screen_layout` view (UISPLIT1 US1): texts, row lists and the bar's actions.
pub fn view(m: &BattleModel) -> View {
    let foes: Vec<RowView> = m
        .foes
        .iter()
        .map(|f| match f {
            FoeEntry::Full(p) | FoeEntry::Compact(p) => plate_row(p),
            FoeEntry::Group { name, plates, .. } => RowView { id: plates.iter().map(|p| p.unit.to_string()).collect::<Vec<_>>().join(","), label: name.clone(), detail: plates.iter().map(hp_text).collect::<Vec<_>>().join(" "), ..RowView::default() },
        })
        .collect();
    let card = m.card.as_ref();
    let card_rows: Vec<RowView> = card
        .map(|c| {
            c.rows
                .iter()
                .enumerate()
                .map(|(k, r)| {
                    let pips: i32 = r.pips.iter().map(|x| x.0).sum();
                    let mut sub: Vec<String> = r.statuses.iter().take(CARD_CHIPS).cloned().collect();
                    if r.statuses.len() > CARD_CHIPS {
                        sub.push(format!("+{}", r.statuses.len() - CARD_CHIPS));
                    }
                    if r.after_ko {
                        sub.push("after KO".into());
                    }
                    RowView { id: k.to_string(), label: r.name.clone(), detail: format!("-{}", r.damage), detail2: format!("{pips} pips"), sub: sub.join(", "), picture: None, enabled: true, ..Default::default() }
                })
                .collect()
        })
        .unwrap_or_default();
    let mut v = View::new(SCREEN)
        .field("round", Field::Text(m.top.round.clone()))
        .field("goal", Field::Text(m.top.goal.clone()))
        .field("protect", Field::Text(m.top.protect.clone()))
        .field("ap", Field::Text(m.top.ap_text()))
        .field("ap_ghost", Field::Text(m.top.ghost_text()))
        .field(PARTY, Field::Rows(m.party.iter().map(plate_row).collect()))
        .field(GUESTS, Field::Rows(m.guests.iter().map(plate_row).collect()))
        .field("hostiles", Field::Text(m.hostiles.clone()))
        .field("reinforce", Field::Text(m.reinforce.clone()))
        .field(FOES, Field::Rows(foes))
        .field("card_name", Field::Text(card.map_or_else(String::new, |c| c.name.clone())))
        .field("card_title", Field::Text(card.map_or_else(String::new, |c| c.title.clone())))
        .field("card_hp", Field::Meter { value: card.map_or(0.0, |c| c.read.hp as f64), max: card.map_or(0.0, |c| c.read.max_hp as f64) })
        .field("card_ko", Field::Text(card.map_or_else(String::new, CardView::ko_text)))
        .field("card_break", Field::Text(card.map_or_else(String::new, CardView::break_text)))
        .field(CARD_ROWS, Field::Rows(card_rows));
    for (id, label) in ACTIONS {
        v = v.action(id, label, true);
    }
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fixture_has_the_view_shape() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets");
        let layers = aegis_core::assets::Layers::of(vec![root]);
        let fixture = aegis_ui::screen_layout::load_fixture(&layers, SCREEN).expect("the fixture file").expect("it parses");
        let shape = |v: &View| (v.screen.clone(), v.fields.iter().map(|(n, f)| (n.clone(), f.kind())).collect::<Vec<_>>(), v.actions.iter().map(|a| a.id.clone()).collect::<Vec<_>>());
        assert_eq!(shape(&fixture), shape(&view(&BattleModel::default())));
    }

    #[test]
    fn the_ap_line_and_ko_texts_follow_the_contract() {
        let top = TopBar { ap_now: 10, ap_picked: 6, ap_ghost: 2, ap_cap: 12, ..TopBar::default() };
        assert_eq!((top.ap_text(), top.ghost_text()), ("AP 10/12 -6".to_string(), "(-2)".to_string()));
        assert_eq!(ApBar { start: 10, planned: 6, cap: 12 }.left(), 4);
        let mut c = CardView { foe: 1, name: String::new(), title: String::new(), boss: false, hero: None, read: FoeCard { ko_block: Some(5), break_block: Some(4), ..FoeCard::default() }, rows: Vec::new() };
        assert_eq!((c.ko_text(), c.break_text()), ("KO in block 5".to_string(), "BREAK in block 4".to_string()));
        c.read.crit_star = Some(3);
        assert_eq!(c.ko_text(), "KO in block 5 (star 3)");
    }
}
