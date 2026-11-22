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
use emberwake_battle::session::{Phase, PhaseMark, Session};
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
    /// STATUSVIS1 section 3 (PR7-57): the plate priority by primitive (`status_view::plate_priority`: 1 high, 3 low).
    pub priority: u8,
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

/// PASSIVE-ICON1 (ENGINE-24, STATUSVIS1 SV10): a slotted passive of a hero's loadout, shown under the portrait: the
/// passive, its name and its archetype's icon file.
#[derive(Debug, Clone, PartialEq)]
pub struct PlatePassive {
    pub id: String,
    pub name: String,
    pub icon: String,
    /// SV13 (PR7-57): what it does, its socket's hover ("" unknown).
    pub about: String,
}

/// SV10: the loadout's passive slots shown under a plate's portrait.
pub const PLATE_PASSIVES: usize = 2;

/// SV10: hero `u`'s slotted passives (its deck's, under the rule deck; none for a guest or a foe), at most
/// `PLATE_PASSIVES`, each with its archetype's icon (`passive_icon::of`).
pub fn plate_passives(w: &World, u: UnitId) -> Vec<PlatePassive> {
    let unit = &w.units[u];
    if unit.deck_rule.is_none() {
        return Vec::new();
    }
    unit.deck
        .passive
        .iter()
        .filter_map(|id| w.cat.passives.get(id))
        .take(PLATE_PASSIVES)
        .map(|p| PlatePassive { id: p.id.clone(), name: p.name.clone(), icon: emberwake_data::passive_icon::path(&emberwake_data::passive_icon::of(p)), about: String::new() })
        .collect()
}

/// One plate (contract 3, 4): a party member, a guest or a foe.
#[derive(Debug, Clone, PartialEq)]
pub struct Plate {
    pub unit: UnitId,
    /// The plate's name (a foe's short name, `UnitDef::plate_name`; a hero's name).
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    /// The HP the round's planned blows take from it (a red loss at the bar's end); STATUSVIS1 SV6 (PR7-57): the hits
    /// and the round-end ticks together, one planned loss (`status_view::planned_loss`).
    pub incoming: i32,
    /// SV6, SV7: the planned loss's hover lines (each hit and each tick, then the total) and whether a modifier moved
    /// its hits (-1 lowered, 1 raised, 0 neither: the number's tint and arrow).
    pub loss: Vec<NumLine>,
    pub loss_sign: i32,
    /// Contract 6 (PR7-40 part 2): the HP the round's planned heals restore on it (a pale moss gain on the bar;
    /// `intent::heal_on`); SV6: with the round-end healing ticks (regen).
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
    /// SV10: the loadout's slotted passives (a party hero's; empty otherwise).
    pub passives: Vec<PlatePassive>,
    /// SV13 (PR7-57): the hero has a loadout's passive sockets (a party hero under the rule deck): its two sockets show,
    /// an empty one dim.
    pub sockets: bool,
}

/// STATUSVIS1 SV6, SV7 (PR7-57): one line of a number's frameless hover: its text and its tint (-1 lowered, 1 raised,
/// 0 plain; 2 a heading).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NumLine {
    pub text: String,
    pub sign: i32,
}

impl NumLine {
    pub fn new(text: impl Into<String>, sign: i32) -> Self {
        NumLine { text: text.into(), sign }
    }
}

/// SV7: a hit's hover lines: the unmodified number, then each modifier with its percent ("Weakened -25%"), then the
/// number shown; one line "No modifiers" when none moved it.
pub fn parts_lines(p: &emberwake_battle::status_view::HitParts) -> Vec<NumLine> {
    if p.mods.is_empty() && p.base == p.damage {
        return vec![NumLine::new(format!("{} HP, no modifiers", p.damage), 0)];
    }
    let mut out = vec![NumLine::new(format!("Base {} HP", p.base), 0)];
    for m in &p.mods {
        let name = emberwake_rules::status::display_name(&m.source);
        out.push(NumLine::new(format!("{name} {}{}%", if m.pct > 0 { "+" } else { "" }, m.pct), m.pct.signum()));
    }
    out.push(NumLine::new(format!("Final {} HP", p.damage), p.sign()));
    out
}

/// SV7: the sign of a sum of hits (the sum's base against its number).
pub fn sum_sign(parts: &[&emberwake_battle::status_view::HitParts]) -> i32 {
    let (base, dmg): (i32, i32) = parts.iter().fold((0, 0), |(b, d), p| (b + p.base, d + p.damage));
    (dmg - base).signum()
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
    /// OBJUI1 (OBJ1 rule 9, MORALE1 3): the goal's hover: the full objective line and the morale line ("" none).
    pub objective: String,
    pub morale: String,
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
    /// STATUSVIS1 SV7 (PR7-57): whether a modifier moved the number (-1 lowered, 1 raised, 0 neither) and its frameless
    /// hover (the base and each modifier's percent).
    pub sign: i32,
    pub parts: Vec<NumLine>,
    /// SV9: the hitters a riposte answers, each with the likely return on it (the "Riposte" chip; empty: none).
    pub riposte: Vec<(UnitId, i32)>,
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
    /// OBJUI1 (MORALE1 3): the foe's morale state and whether it leads (None, false without morale).
    pub morale: Option<emberwake_battle::obj_view::MoraleMark>,
    pub leader: bool,
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

/// BOSSUI1 (PR7-54, contract 9.2): a boss intent row's cell, or its total: the HP the hits take (0: "-"), the poise
/// pips they break, the statuses they bring, whether one empties the poise bar (BREAK), and how many hits it holds.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImpactCell {
    pub hp: i32,
    pub pips: i32,
    pub statuses: Vec<String>,
    pub breaks: bool,
    pub hits: usize,
}

impl ImpactCell {
    /// Adds hit `h` taking `hp` HP (a status once).
    fn add(&mut self, h: &intent::CardHit, hp: i32) {
        self.hp += hp;
        self.pips += h.pips;
        self.breaks |= h.breaks;
        self.hits += 1;
        for st in &h.statuses {
            if !self.statuses.contains(st) {
                self.statuses.push(st.clone());
            }
        }
    }

    /// The HP cell's text: "-1250", "-12K"; "-" where the row does not hit (contract 9.2 numbers).
    pub fn hp_text(&self) -> String {
        if self.hits == 0 || self.hp <= 0 { "-".to_string() } else { format!("-{}", impact_num(self.hp)) }
    }
}

/// Contract 9.2 numbers: whole up to 9999, K from 10K ("12K", "120K"), M from 1M (both rounded down).
pub fn impact_num(n: i32) -> String {
    let a = n.unsigned_abs();
    let sign = if n < 0 { "-" } else { "" };
    if a >= 1_000_000 {
        format!("{sign}{}M", a / 1_000_000)
    } else if a >= 10_000 {
        format!("{sign}{}K", a / 1000)
    } else {
        format!("{sign}{a}")
    }
}

/// BOSSUI1 (contract 9.2): one boss intent row: the selected hero, "Others", one member, or "Party"; a cell per block
/// and the total of what lands.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ImpactRow {
    pub unit: Option<UnitId>,
    pub name: String,
    pub cells: Vec<ImpactCell>,
    pub total: ImpactCell,
}

/// BOSSUI1 BU11, BU16 (contract 9.2): the boss intent rows while planning: the selected hero's row, "Others" (the rest
/// of the party summed, never the hero again), the hover's member rows and "Party" line; the block a phase break
/// lands (`ends`, with the next phase's number) and the block the poise breaks.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct BossRows {
    pub blocks: i32,
    pub hero: Option<ImpactRow>,
    pub others: ImpactRow,
    pub members: Vec<ImpactRow>,
    pub party: ImpactRow,
    pub ends: Option<i32>,
    pub next_phase: usize,
    pub break_block: Option<i32>,
}

impl BossRows {
    /// Block `b`'s header: "Block N", "Block N: ends" where the phase breaks, "Ph. 2 next round" on the first block it
    /// cancels (contract 9.2).
    pub fn header(&self, b: i32) -> String {
        match self.ends {
            Some(e) if e == b => format!("Block {b}: ends"),
            Some(e) if b == e + 1 => format!("Ph. {} next round", self.next_phase),
            _ => format!("Block {b}"),
        }
    }

    /// Block `b` does not run (after the phase break): greyed, its numbers struck.
    pub fn struck(&self, b: i32) -> bool {
        self.ends.is_some_and(|e| b > e)
    }
}

/// BOSSUI1 (contract 9.1, row 3): the boss's own next blow: its skill (icon) and name, its target, the block it lands
/// and its damage.
#[derive(Debug, Clone, PartialEq)]
pub struct BossBlow {
    pub skill: String,
    pub name: String,
    pub target: Option<UnitId>,
    pub impact: i32,
    pub damage: i32,
}

/// BOSSUI1 BU7, BU10 (contract 9.1, row 3): the boss's poise: pips lit, of (its own `poise_pips`), the pips the plan
/// breaks, the bar's pip threshold, BREAK planned.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct BossPoise {
    pub lit: i32,
    pub of: i32,
    pub planned: i32,
    pub threshold: i32,
    pub breaks: bool,
}

impl BossPoise {
    /// "6/10".
    pub fn text(&self) -> String {
        format!("{}/{}", self.lit, self.of)
    }
}

/// BOSSUI1 BU10: the most pips a boss bar draws.
pub const BOSS_PIPS_MAX: i32 = 10;

/// BOSSUI1 (PR7-54, contract 9.1): the boss bar across the board's top (in place of the boss's plate and its card).
#[derive(Debug, Clone, PartialEq)]
pub struct BossBar {
    pub unit: UnitId,
    /// The title (the unit's full name; BU3b, PR7-59: "<name>, <form>" when the phase sets a `form`) and the
    /// character's title ("" none, or under a form).
    pub name: String,
    pub title: String,
    pub marks: Vec<PhaseMark>,
    pub current: usize,
    /// The current phase's span: HP left of it and its size ("160/450"), and the HP the plan takes of it.
    pub span_left: i32,
    pub span: i32,
    /// PR7-75 (PT-40): the boss's whole HP and max (the span is the phase's share; the hover says both).
    pub whole: (i32, i32),
    /// PR7-83 (PT-75): HP carried above the phase's span from a phase that ended unfinished (a time phase): still owed,
    /// so the read-out shows it ("346/346 +138") and the hover names it.
    pub carried: i32,
    pub planned: i32,
    pub poise: Option<BossPoise>,
    /// The boss passives: (display name, description) for the tags (BU4).
    pub passives: Vec<(String, String)>,
    pub tabs: Vec<Tab>,
    pub blow: Option<BossBlow>,
    /// The intent rows: only while planning, when a hero targets the boss or it is pinned (contract 9.2).
    pub rows: Option<BossRows>,
}

impl BossBar {
    /// "160/450".
    pub fn span_text(&self) -> String {
        if self.carried > 0 {
            format!("{}/{} +{}", self.span_left, self.span, self.carried)
        } else {
            format!("{}/{}", self.span_left, self.span)
        }
    }

    /// PR7-75 (PT-40): the HP read-out's hover: the phase's share and the whole ("This phase: 142/304", "In all:
    /// 142/608"); one line when they are the same.
    pub fn hp_tip(&self) -> Vec<String> {
        let all = format!("In all: {}/{}", self.whole.0, self.whole.1);
        let mut out = if self.span == self.whole.1 { vec![all] } else { vec![format!("This phase: {}/{}", self.span_left, self.span), all] };
        if self.carried > 0 {
            out.push(format!("Carried over: {} HP from an earlier phase that ended by time, still to take.", self.carried));
        }
        out
    }
}

/// BOSSUI1 (contract 9.2): the boss intent rows over the boss's card read `read` (the kit's `foe_card` over the live
/// plan), cut where the plan breaks the phase (`intent::phase_cut` on the span's `floor`): the hits after the
/// breaking one do not land (the same block's are dropped, a later block's are shown struck); totals count what
/// lands. `hero` the selected hero, `allies` the members in plate order, `next_phase` the next phase's number.
pub fn boss_rows(read: &FoeCard, floor: i32, blocks: i32, hero: Option<UnitId>, allies: &[(UnitId, String)], next_phase: usize) -> BossRows {
    let blocks = blocks.max(1);
    let cut = intent::phase_cut(read, floor);
    let row = |unit: Option<UnitId>, name: &str| ImpactRow { unit, name: name.to_string(), cells: vec![ImpactCell::default(); blocks as usize], total: ImpactCell::default() };
    let mut out = BossRows { blocks, hero: hero.map(|h| row(Some(h), &allies.iter().find(|a| a.0 == h).map_or_else(String::new, |a| a.1.clone()))), others: row(None, "Others"), members: allies.iter().map(|a| row(Some(a.0), &a.1)).collect(), party: row(None, "Party"), ends: cut.map(|c| c.block), next_phase, break_block: None };
    for (k, h) in read.hits.iter().enumerate() {
        let (hp, lands) = match cut {
            Some(c) if k == c.hit => (c.landed, true),
            Some(c) if k > c.hit && h.impact == c.block => continue,
            Some(c) if k > c.hit => (h.damage, false),
            _ => (h.damage, true),
        };
        let b = (h.impact.clamp(1, blocks) - 1) as usize;
        if lands && h.breaks && out.break_block.is_none() {
            out.break_block = Some(h.impact);
        }
        let mine = Some(h.unit) == hero;
        let put = |r: &mut ImpactRow| {
            r.cells[b].add(h, hp);
            if lands {
                r.total.add(h, hp);
            }
        };
        match out.hero.as_mut() {
            Some(r) if mine => put(r),
            _ => put(&mut out.others),
        }
        if let Some(r) = out.members.iter_mut().find(|r| r.unit == Some(h.unit)) {
            put(r);
        }
        put(&mut out.party);
    }
    out
}

/// The boss's first live blow this round (`lands_of` with its skill and damage).
fn boss_blow(w: &World, sched: &[ScheduledAction], boss: UnitId) -> Option<BossBlow> {
    let l = lands_of(w, sched, boss)?;
    let a = intent::actions_of(boss, sched).into_iter().find(|a| a.impact == l.impact)?;
    let hit = l.target.and_then(|t| intent::hits_on(w, sched, boss, t).into_iter().find(|h| h.block == l.impact));
    Some(BossBlow { skill: a.command.skill.clone(), name: hit.as_ref().map_or_else(|| w.cat.skill(&a.command.skill).map_or_else(String::new, |s| s.name.clone()), |h| h.skill.clone()), target: l.target, impact: l.impact, damage: hit.map_or(0, |h| h.damage) })
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
    /// BOSSUI1 (PR7-54, contract 9): the boss bar (None: no boss on the field); the boss then has no plate and no card.
    pub boss: Option<BossBar>,
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

/// Contract 6 order, STATUSVIS1 section 3 (PR7-57): by plate priority (`status_view::plate_priority`; a `rule_flag`
/// first inside priority 1, contract 6's steadfast); inside a priority the soonest to run out first, ties to the one
/// applied first, queued ones after the active, `planned` ones (statuses a planned move adds) last.
pub fn tabs_of(w: &World, u: UnitId, planned: &[String]) -> Vec<Tab> {
    use emberwake_rules::status;
    let st = &w.units[u].st;
    let tab = |id: &str, rounds: Option<i32>, planned: bool, magnitude: i32| Tab { status: id.to_string(), icon: emberwake_content::art::status_icon(id), buff: status::is_buff(id), rounds, planned, name: status::label_with_magnitude(id, magnitude), priority: emberwake_battle::status_view::plate_priority(id) };
    // (group: 0 active, 1 queued, 2 planned; the order applied) beside each tab
    let mut all: Vec<(u8, usize, Tab)> = st
        .active
        .iter()
        .enumerate()
        .map(|(k, id)| (0, k, tab(id, st.rounds_left.get(id).copied().filter(|r| *r > 0 && !emberwake_battle::trigger::lasts_battle(*r)), false, st.magnitudes.get(id).copied().unwrap_or(0))))
        .collect();
    all.extend(st.pending.iter().filter(|id| !st.active.contains(*id)).enumerate().map(|(k, id)| (1, k, tab(id, st.pending_rounds.get(id).copied().filter(|r| *r > 0 && !emberwake_battle::trigger::lasts_battle(*r)), false, st.pending_magnitudes.get(id).copied().unwrap_or(0)))));
    for (k, id) in planned.iter().enumerate() {
        if !all.iter().any(|t| &t.2.status == id) {
            all.push((2, k, tab(id, None, true, 0)));
        }
    }
    let flag = |id: &str| status::row_of(id).is_some_and(|r| r.has_primitive(status::Primitive::RuleFlag));
    all.sort_by_key(|(g, k, t)| (t.priority, !flag(&t.status), *g, if *g == 0 { t.rounds.unwrap_or(i32::MAX) } else { 0 }, *k));
    all.into_iter().map(|x| x.2).collect()
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
    let full = poise::full_pips(n, st);
    if st.broken {
        return Some((0, full));
    }
    Some(if poise::pips_mode(n) { (poise::pips_now(n, st).min(full), full) } else { (poise::bar_pips(n, st), full) })
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
    /// BOSSUI1 (contract 9.2): the inspected (pinned) unit, and a passive's description by id (the boss tags' hover).
    pub pinned: Option<UnitId>,
    pub passive_about: &'a dyn Fn(&str) -> String,
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
    m.top.objective = emberwake_battle::encounter::Objectives::text_of(&s.def.raw, name_of);
    m.top.morale = emberwake_battle::obj_view::header_line(s);
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
        // STATUSVIS1 SV6, SV7 (PR7-57): one planned loss, the hits and the round-end ticks, with its hover lines
        let (incoming_hp, loss, loss_sign, gain) = planned_loss_view(w, &sched, u);
        Plate {
            unit: u,
            name: (inp.plate_name)(u),
            hp: unit.hp,
            max_hp: unit.max_hp,
            incoming: incoming_hp,
            loss,
            loss_sign,
            heal: if planning { (intent::heal_on(w, &sched, u) + gain).min((unit.max_hp - unit.hp).max(0)) } else { 0 },
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
            passives: if ally && !guest { plate_passives(w, u).into_iter().map(|p| PlatePassive { about: (inp.passive_about)(&p.id), ..p }).collect() } else { Vec::new() },
            sockets: ally && !guest && unit.deck_rule.is_some(),
        }
    };
    for &a in &w.allies {
        let guest = s.guests.contains(&a);
        let p = plate(a, guest);
        if guest { m.guests.push(p) } else { m.party.push(p) }
    }
    // the foes on the field (reinforcements not placed yet are left out)
    // BOSSUI1 (contract 9.1): a boss on the bar leaves the foe panel
    let phase_bar = s.phase_bar();
    let boss = phase_bar.as_ref().map(|p| p.unit);
    let shown: Vec<UnitId> = w.enemies.iter().copied().filter(|&e| Some(e) != boss && (w.grid.pos(e).is_some() || !w.units[e].is_alive() || w.units[e].is_withdrawn)).collect();
    // PR7-75 (PT-40): the count takes in the boss on the bar while it stands (its plate stays above)
    let active = shown.iter().chain(boss.as_ref()).filter(|e| w.units[**e].is_active()).count();
    m.hostiles = format!("Hostiles {active}");
    m.reinforce = s.phases.next_reinforcement().map_or_else(String::new, |(n, r)| format!("+{n} round {r}"));
    if boss.is_some() && m.reinforce.is_empty() {
        m.reinforce = BOSS_ABOVE.to_string();
    }
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
        // SV7, SV9 (PR7-57): each hit's breakdown and the counter it draws, one for one with the card's hits
        let reads = emberwake_battle::status_view::loss_reads(w, &sched, f);
        let read_of = |k: usize| reads.get(k).filter(|_| reads.len() == read.hits.len());
        let rows = match hero {
            Some(h) => read
                .hits
                .iter()
                .enumerate()
                .filter(|(_, x)| x.unit == h)
                .map(|(k, x)| {
                    let r = read_of(k);
                    // HITCALC1 (PR7-45): a hit that fails at its impact says how after its name
                    let name = x.fails.map_or_else(|| x.name.clone(), |f| format!("{} ({})", x.name, crate::intent_text::fail_words(Some(w), f)));
                    CardRow { skill: x.skill.clone(), name, start: x.start, impact: x.impact, units: vec![h], damage: x.damage, pips: vec![(x.pips, x.breaks, x.on_broken)], statuses: x.statuses.clone(), after_ko: x.after_ko, ko: false, sign: r.map_or(0, |r| r.parts.sign()), parts: r.map(|r| parts_lines(&r.parts)).unwrap_or_default(), riposte: r.and_then(|r| r.counter).map(|n| vec![(h, n)]).unwrap_or_default() }
                })
                .collect(),
            None => {
                let mut k0 = 0;
                read.by_block()
                    .into_iter()
                    .map(|(b, hs)| {
                        let mut units: Vec<UnitId> = hs.iter().map(|x| x.unit).collect();
                        units.dedup();
                        let rs: Vec<&emberwake_battle::status_view::HitRead> = (k0..k0 + hs.len()).filter_map(read_of).collect();
                        k0 += hs.len();
                        let parts: Vec<&emberwake_battle::status_view::HitParts> = rs.iter().map(|r| &r.parts).collect();
                        let mut lines: Vec<NumLine> = Vec::new();
                        for (x, r) in hs.iter().zip(&rs) {
                            let sign = r.parts.sign();
                            lines.push(NumLine::new(format!("{}: -{} (base {})", x.name, r.parts.damage, r.parts.base), sign));
                            lines.extend(r.parts.mods.iter().map(|m| NumLine::new(format!("  {} {}{}%", emberwake_rules::status::display_name(&m.source), if m.pct > 0 { "+" } else { "" }, m.pct), m.pct.signum())));
                        }
                        let mut riposte: Vec<(UnitId, i32)> = Vec::new();
                        for (x, r) in hs.iter().zip(&rs) {
                            if let Some(n) = r.counter.filter(|_| !riposte.iter().any(|q| q.0 == x.unit)) {
                                riposte.push((x.unit, n));
                            }
                        }
                        CardRow { skill: String::new(), name: format!("Block {b}"), start: b, impact: b, units, damage: hs.iter().map(|x| x.damage).sum(), pips: hs.iter().map(|x| (x.pips, x.breaks, x.on_broken)).collect(), statuses: hs.iter().flat_map(|x| x.statuses.clone()).collect(), after_ko: hs.iter().all(|x| x.after_ko), ko: read.ko_block == Some(b), sign: sum_sign(&parts), parts: lines, riposte }
                    })
                    .collect()
            }
        };
        let (name, title) = (inp.full_name)(f);
        Some(CardView { foe: f, name, title, boss: w.units[f].tier == "boss", hero, read, rows, morale: emberwake_battle::obj_view::morale_mark(s, f), leader: emberwake_battle::obj_view::is_leader(s, f) })
    });
    // BOSSUI1 (contract 9): the boss bar; the boss's card folds into it
    if let Some(pb) = phase_bar {
        let b = pb.unit;
        if m.card.as_ref().is_some_and(|c| c.foe == b) {
            m.card = None;
        }
        let unit = &w.units[b];
        let read = cards.iter().find(|c| c.0 == b).map_or_else(|| intent::foe_card(w, &sched, b), |c| c.1.clone());
        let (span_left, span) = pb.span(unit.hp);
        let hero = inp.sel.filter(|h| w.allies.contains(h) && w.units[*h].is_active());
        let allies: Vec<(UnitId, String)> = w.allies.iter().copied().filter(|&a| w.units[a].is_active()).map(|a| (a, (inp.plate_name)(a))).collect();
        let rows = boss_rows(&read, pb.floor, s.def.total_blocks, hero, &allies, pb.current + 2);
        let show_rows = planning && (hero.and_then(target_of) == Some(b) || inp.pinned == Some(b));
        let poise = poise_of(w, b).map(|(lit, of)| {
            let st = emberwake_battle::poise::state(w, b).copied().unwrap_or_default();
            BossPoise { lit, of, planned: rows.party.total.pips.min(lit), threshold: emberwake_battle::poise::pip_threshold(&w.ruleset.numbers, &st), breaks: rows.break_block.is_some() }
        });
        let (full, title) = (inp.full_name)(b);
        let mut passives: Vec<(String, String)> = Vec::new();
        let ids = w.cat.units.get(&unit.catalog_id).map(|d| d.passives.clone()).unwrap_or_default();
        for id in ids.iter().chain(unit.passives.iter()) {
            if let Some(p) = w.cat.passives.get(id).filter(|p| !passives.iter().any(|x| x.0 == p.name)) {
                passives.push((p.name.clone(), (inp.passive_about)(id)));
            }
        }
        let planned_statuses: Vec<String> = rows.party.total.statuses.clone();
        m.boss = Some(BossBar {
            unit: b,
            // BU3b (PR7-59): the unit's name and title, never the phase's name (the phase banner shows it); a phase's
            // `form` takes the title's place ("<name>, <form>")
            name: if pb.form.is_empty() { full } else { format!("{full}, {}", pb.form) },
            title: if pb.form.is_empty() { title } else { String::new() },
            marks: pb.marks.clone(),
            current: pb.current,
            span_left,
            span,
            whole: (unit.hp, unit.max_hp),
            carried: (unit.hp - pb.top).max(0),
            planned: if planning { rows.party.total.hp.min(span_left) } else { 0 },
            poise,
            passives,
            tabs: tabs_of(w, b, if planning { &planned_statuses } else { &[] }),
            blow: boss_blow(w, &sched, b),
            rows: show_rows.then_some(rows),
        });
    }
    m
}

/// STATUSVIS1 SV6, SV7 (PR7-57): unit `u`'s one planned loss over `sched` (`status_view::planned_loss`): the HP (hits
/// and damage ticks within its HP), the hover lines (each hit with its block and skill, each tick, then the total), the
/// hits' tint (`sum_sign`) and the healing ticks' gain.
pub fn planned_loss_view(w: &World, sched: &[ScheduledAction], u: UnitId) -> (i32, Vec<NumLine>, i32, i32) {
    use emberwake_battle::status_view as sv;
    let pl = sv::planned_loss(w, sched, u);
    let reads = sv::loss_reads(w, sched, u);
    let parts: Vec<&sv::HitParts> = reads.iter().map(|r| &r.parts).collect();
    let mut lines: Vec<NumLine> = Vec::new();
    for (k, (_, h)) in pl.hits.iter().enumerate() {
        let sign = parts.get(k).map_or(0, |p| p.sign());
        // HITCALC1 (PR7-45): a hit that fails at its impact says how
        let amount = match h.fails {
            None => format!("-{}", h.damage),
            Some(f @ emberwake_battle::intent::HitFail::Intercepted { pct, .. }) if pct < 100 => format!("-{} ({})", h.damage, crate::intent_text::fail_words(Some(w), f)),
            Some(f) => crate::intent_text::fail_words(Some(w), f),
        };
        lines.push(NumLine::new(format!("B{} {}: {amount}", h.block, h.skill), sign));
    }
    for t in &pl.ticks {
        let name = emberwake_rules::status::display_name(&t.status);
        lines.push(NumLine::new(if t.hp < 0 { format!("Round end, {name}: -{}", -t.hp) } else { format!("Round end, {name}: +{}", t.hp) }, 0));
    }
    if pl.loss > 0 {
        // the loss stops at the HP left: the total says so when the hits and ticks come to more
        let raw = pl.hit_hp() + pl.ticks.iter().filter(|t| t.hp < 0).map(|t| -t.hp).sum::<i32>();
        let capped = if raw > pl.loss { " (capped at HP)" } else { "" };
        lines.push(NumLine::new(format!("Total -{}{capped}", pl.loss), 2));
    }
    (pl.loss, lines, sum_sign(&parts), pl.gain)
}

/// BOSSUI1 (take 31): the foe panel's note while the boss is on the bar.
pub const BOSS_ABOVE: &str = "Boss above";

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
            // PR7-40 part 3 (FB-358): an open group is its header alone; its members are the compact rows after it
            FoeEntry::Group { key, name, open: true, .. } => RowView { id: key.clone(), label: name.clone(), ..RowView::default() },
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
        let mut c = CardView { foe: 1, name: String::new(), title: String::new(), boss: false, hero: None, read: FoeCard { ko_block: Some(5), break_block: Some(4), ..FoeCard::default() }, rows: Vec::new(), morale: None, leader: false };
        assert_eq!((c.ko_text(), c.break_text()), ("KO in block 5".to_string(), "BREAK in block 4".to_string()));
        c.read.crit_star = Some(3);
        assert_eq!(c.ko_text(), "KO in block 5 (star 3)");
    }
}
