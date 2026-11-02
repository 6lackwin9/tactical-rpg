//! Round input: clicks on units and tiles, the popup and skill menu rows, targets, picks and ghosts, member and skill hotkeys.

use aegis_ui::theme::th;
use super::marks::skills_hp;
use super::playback::PlaySpeed;
use super::{Action, BattleScreen, ALLY, TACTICS, TACTIC_HELP, TACTIC_INTENT_HELP, TACTIC_LABELS};
use aegis_ui::canvas::Rect;
use crate::intent_text;
use crate::screens::Cx;
use aegis_ui::ui::{Tip, H, W};
use emberwake_battle::grid::V2;
use emberwake_battle::round_intent::{self as ri, ManeuverKind, RoundIntent, RoundTarget};
use emberwake_battle::round_intent_plan as rip;
use emberwake_battle::session::Session;
use emberwake_battle::world::World;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub(super) enum RowAct {
    Step(Value),
    PartyTactic(usize),
    HeroTactic(usize),
    /// INPUT1 IF2-IF4 (round intent): a row of the skill menu: the picks it appends on the menu's target.
    Pick(Vec<rip::Pick>),
    /// SL6: the ally menu's last row, "Select <ally>".
    SelectAlly(usize),
}

/// INPUT1 GH2, IN6 timing (decision "INPUT1 answers, part 4" Q18): a member's ghost that waits for Confirm because its
/// intent change would drop intent-bound buffs: its tactic's draft (`preset`) or a player-set target's round change
/// (`round_intent`). The screen draws it from copies (`view`) and Confirm sends the same change (`req`).
#[derive(Debug, Clone)]
pub(super) enum Deferred {
    Preset(&'static str),
    Round(rip::RoundSet),
}

impl Deferred {
    /// The host request Confirm sends for member `m`.
    pub(super) fn req(&self, m: usize) -> Value {
        match self {
            Deferred::Preset(p) => json!({"cmd": "preset", "unit": m, "preset": p}),
            Deferred::Round(set) => {
                debug_assert!(set.kind.is_none() && set.opener.is_none(), "the screen defers only an intent and a target");
                let mut r = json!({"cmd": "round_intent", "unit": m});
                if let Some(i) = set.intent {
                    r["intent"] = json!(i.id());
                }
                if let Some(t) = set.target {
                    r["target"] = target_json(t);
                }
                r
            }
        }
    }

    /// Member `m`'s round as the change would leave it, on copies (the tactic's draft as the host's `preset` makes it;
    /// a round change through the kit's `preview_round`, the source of the host's `round_intent`).
    pub(super) fn view(&self, s: &Session, m: usize) -> Option<(World, emberwake_battle::plan::Plan)> {
        match self {
            Deferred::Preset(p) => {
                let mut w2 = s.w.clone();
                let mut p2 = rip::copy_plan(&s.plan);
                emberwake_battle::ai::plan_party_intents(&mut w2, &mut p2, &[(m, p.to_string())]);
                Some((w2, p2))
            }
            Deferred::Round(set) => rip::preview_round(&s.w, &s.plan, m, set).ok(),
        }
    }
}

pub(super) fn kind_name(k: ManeuverKind) -> &'static str {
    match k {
        ManeuverKind::Engage => "Engage",
        ManeuverKind::Disengage => "Disengage",
        ManeuverKind::Utility => "Utility",
    }
}

/// A pick as the host's `pick` request takes it (INPUT1_TASKS 1.3).
pub(super) fn pick_json(w: &World, p: &rip::Pick) -> Value {
    match p {
        rip::Pick::Skill { skill, kind } => {
            let mut v = json!({"skill": w.cat.skills[*skill].id});
            if let Some(k) = kind {
                v["kind"] = json!(emberwake_battle::intent::maneuver_id(*k));
            }
            v
        }
        rip::Pick::Item(id) => json!({"item": id}),
        rip::Pick::Rest => json!("rest"),
        rip::Pick::MoveHere(c) => json!({"move": [c.x, c.y]}),
        rip::Pick::StepAfter(c) => json!({"step_after": [c.x, c.y]}),
    }
}

/// INPUT1 GH2 b: the intents a round target of this kind tries, in order (an enemy: Assault, else Maneuver; an ally:
/// Support; the unit itself: Guard, else Support; a tile: Guard, else Assault, else Maneuver).
fn intents_for(w: &World, h: usize, at: RoundTarget) -> &'static [RoundIntent] {
    match at {
        RoundTarget::Unit(t) if w.foes_of(h).contains(&t) => &[RoundIntent::Assault, RoundIntent::Maneuver],
        RoundTarget::Unit(_) => &[RoundIntent::Support],
        RoundTarget::Myself => &[RoundIntent::Guard, RoundIntent::Support],
        RoundTarget::Cell(_) => &[RoundIntent::Guard, RoundIntent::Assault, RoundIntent::Maneuver],
    }
}

/// Whether `intent` on `at` plans a round for `h` (GH2 b: "when it can act on that target"), on copies.
fn plans_on(s: &Session, h: usize, intent: RoundIntent, at: RoundTarget) -> bool {
    if !s.w.ruleset.numbers.intents.iter().any(|i| i == intent.id()) || !ri::target_problem(&s.w, h, intent, at).is_empty() {
        return false;
    }
    let mut w2 = s.w.clone();
    let mut p2 = rip::copy_plan(&s.plan);
    ri::set_intent(&mut w2, h, intent).is_ok() && ri::set_target(&mut w2, h, at).is_ok() && rip::plan_unit(&mut w2, &mut p2, h).is_ok()
}

/// A menu row's label (IF3): the skill with its hotkey digit (HK1's first nine), "Use <item> (<n> left)", Rest, the
/// tile rows (ET1).
fn pick_label(s: &Session, h: usize, picks: &[rip::Pick]) -> String {
    let w = &s.w;
    let skill = |sid: usize| {
        let name = w.cat.skills[sid].name.clone();
        match rip::hotkey_skills(w, h).iter().position(|k| *k == sid).filter(|k| *k < 9) {
            Some(k) => format!("{} {name}", k + 1),
            None => name,
        }
    };
    match picks {
        [rip::Pick::Skill { skill: sid, .. }] => skill(*sid),
        [rip::Pick::Item(id)] => {
            let name = emberwake_battle::command::pouch_item(w, h, id).map_or(id.clone(), |i| i.name.clone());
            format!("Use {name} ({} left)", s.plan.items_left(w, h, id, -1))
        }
        [rip::Pick::Rest] => "Rest".into(),
        [rip::Pick::MoveHere(_)] => "Move here".into(),
        [rip::Pick::MoveHere(_), rip::Pick::Skill { skill: sid, .. }] => format!("Move here, then {}", w.cat.skills[*sid].name),
        [rip::Pick::StepAfter(_)] => "Step here after".into(),
        _ => picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", then "),
    }
}

/// A menu row's detail (IF3): "<blocks> blocks, <ap> AP; <Intent>" (a Maneuver row adds its kind), or its refusal.
pub(super) fn pick_detail(cat: &emberwake_data::Catalog, o: &rip::PickOption) -> String {
    if let Some(why) = &o.problem {
        return why.clone();
    }
    let blocks = format!("{} block{}, {} AP", o.blocks, if o.blocks == 1 { "" } else { "s" }, o.ap);
    match (o.intent, o.kind) {
        (Some(i @ RoundIntent::Maneuver), Some(k)) => format!("{blocks}; {} ({})", i.label(cat), kind_name(k)),
        (Some(i), _) => format!("{blocks}; {}", i.label(cat)),
        (None, _) => blocks,
    }
}

/// A round target as the `round_intent` request takes it ("self", a unit index or a cell).
fn target_json(t: RoundTarget) -> Value {
    match t {
        RoundTarget::Myself => json!("self"),
        RoundTarget::Unit(u) => json!(u),
        RoundTarget::Cell(p) => json!([p.x, p.y]),
    }
}

/// A round target's name ("self", the unit's, "tile 3, 4").
pub(super) fn target_name(w: &World, t: RoundTarget) -> String {
    match t {
        RoundTarget::Myself => "self".into(),
        RoundTarget::Unit(u) => w.units[u].name.clone(),
        RoundTarget::Cell(p) => format!("tile {}, {}", p.x, p.y),
    }
}

/// UI2's Rest gain ("Rest: 1 AP, 10 poise"; the poise only under the rule poise).
pub(super) fn rest_text(g: rip::RestGain) -> String {
    let poise = if g.poise > 0 { format!(", {} poise", g.poise) } else { String::new() };
    format!("Rest: {} AP{poise}", g.ap)
}

#[derive(Debug, Clone)]
pub(super) struct Row {
    pub(super) label: String,
    pub(super) detail: String,
    pub(super) valid: bool,
    pub(super) act: RowAct,
    pub(super) tip: Tip,
    /// IF3, IN6: picking the row changes the intent and drops intent-bound buffs (a red corner).
    pub(super) warn: bool,
    /// INPUT1 AP3, AP8: a skill-menu row refused for its AP: how many AP short it is (the kit's `PickOption::ap_short`).
    pub(super) ap_short: Option<i32>,
}

#[derive(Debug, Clone)]
pub(super) struct Popup {
    pub(super) title: String,
    x: i32,
    y: i32,
    pub(super) w: i32,
    pub(super) rows: Vec<Row>,
    /// INPUT1 IF1-IF5: the skill menu's hero and target (None: a tactic or step menu).
    pub(super) menu: Option<(usize, RoundTarget)>,
}

impl Popup {
    /// The menu's rect with rows `row_h` high (TEXTFIT1: the pitch grows with the text size; 11 at Normal).
    pub(super) fn rect(&self, row_h: i32) -> Rect {
        Rect::new(self.x, self.y, self.w, row_h + 4 + self.rows.len() as i32 * row_h)
    }

    /// Row `k`'s rect (under the title).
    pub(super) fn row(&self, row_h: i32, k: usize) -> Rect {
        Rect::new(self.x + 2, self.y + row_h + 2 + k as i32 * row_h, self.w - 4, row_h)
    }
}

impl BattleScreen {
    /// Dropdown of the six tactics under the Party / Hero picker.
    pub(super) fn tactic_menu(&mut self, party: bool) {
        let current = if party { self.party_tactic } else { self.selected.map_or(self.hero_tactic, |h| self.tactic_of(h)) };
        let help = if emberwake_battle::round_intent::on(&self.session().w) { TACTIC_INTENT_HELP } else { TACTIC_HELP };
        let rows = (0..TACTICS.len())
            .map(|i| Row {
                label: TACTIC_LABELS[i].to_string(),
                detail: if i == current { "current".into() } else { String::new() },
                valid: true,
                act: if party { RowAct::PartyTactic(i) } else { RowAct::HeroTactic(i) },
                tip: vec![(TACTIC_LABELS[i].to_string(), th().gold), (help[i].to_string(), th().text)],
                warn: false,
                ap_short: None,
            })
            .collect();
        let anchor = self.w.iter().find(|w| w.id == if party { "strip.party" } else { "strip.hero" }).map(|w| w.rect).unwrap_or_default();
        self.popup = Some(Popup { title: if party { "Draft the whole party".into() } else { "Draft the selected hero".into() }, x: anchor.x, y: anchor.y + anchor.h + 1, w: 120, rows, menu: None });
    }

    pub(super) fn unit_click(&mut self, cx: &Cx, u: usize) {
        let s = self.session();
        let is_ally = s.w.allies.contains(&u);
        let names: Vec<String> = s.w.units.iter().map(|x| x.name.clone()).collect();
        // INPUT1 SL1, SL4-SL7 (round intent): with a hero selected and planning a click targets (SL1, SL6) and a click on
        // the hero itself selects nothing (SL4); with none selected, or outside planning, a click on an ally selects it
        // and one on an enemy inspects it (SL5, SL7)
        if ri::on(&s.w) {
            match self.selected {
                Some(h) if h == u => {
                    self.selected = None;
                    self.popup = None;
                }
                // INPUT2 (decision "Playtest 2026-11-02"): while the hero's round is its ghost (no pick holds a target)
                // a click on another ally selects that ally; Ctrl+click targets it (SL6: the hero's support menu)
                Some(h) if self.planning() && is_ally && !cx.held.ctrl && !rip::picks_of(&s.w, h).iter().any(|p| p.holds_target()) => {
                    self.selected = Some(u);
                    self.popup = None;
                }
                Some(h) if self.planning() => self.target_click(cx, h, RoundTarget::Unit(u)),
                _ if is_ally => {
                    self.selected = Some(u);
                    self.popup = None;
                }
                _ => self.inspect_unit(u),
            }
            return;
        }
        if !self.planning() || self.selected.is_none() {
            if is_ally {
                self.selected = Some(u);
            }
            return;
        }
        let h = self.selected.unwrap();
        if is_ally && u != h {
            let rows = self.options_for(h, "ally", Some(u), None);
            if rows.iter().any(|r| r.valid) {
                self.open_popup(format!("{} -> {}", names[h], names[u]), rows);
            } else {
                self.selected = Some(u);
            }
            return;
        }
        let (title, rows) = if u == h { (format!("{}: own actions", names[h]), self.options_for(h, "self", None, None)) } else { (format!("Attack {}", names[u]), self.options_for(h, "foe", Some(u), None)) };
        self.open_popup(title, rows);
    }

    /// INPUT1 IS1: pins the threat intel panel on unit `u` until another inspect, Esc or the round's end.
    pub(super) fn inspect_unit(&mut self, u: usize) {
        let (round, name) = (self.session().current_round, self.session().w.units[u].name.clone());
        self.inspect = Some((u, round));
        self.message = format!("Inspecting {name} (Esc clears).");
    }

    /// IS1: the inspected unit while its round lasts (round intent only; None = the panel follows the hover).
    pub(super) fn inspected(&self) -> Option<usize> {
        let s = self.host.session.as_ref().filter(|s| ri::on(&s.w))?;
        self.inspect.filter(|&(u, r)| r == s.current_round && u < s.w.units.len()).map(|(u, _)| u)
    }

    /// INPUT1 TV1: whether key T may open the full timeline view: a timeline battle (the rule `round_intent`, or any
    /// ruleset beyond classic); classic keeps its screen as it was.
    pub(super) fn timeline_view_on(&self) -> bool {
        self.host.session.as_ref().is_some_and(|s| ri::on(&s.w) || !s.w.ruleset.rules.is_empty())
    }

    /// TV1: the timeline view is open and shown (a classic battle never shows it).
    pub(super) fn timeline_view_open(&self) -> bool {
        self.tv_open && self.timeline_view_on()
    }

    /// TV1, PB5: the timeline view closes; a playback it held at a block's end goes on (Block mode keeps holding it).
    pub(super) fn close_view(&mut self, cx: &Cx) {
        self.tv_open = false;
        if self.waiting.is_some_and(|w| !w.1) && PlaySpeed::of(&cx.settings.v) != PlaySpeed::Block {
            self.release_block();
        }
    }

    /// SL2: the unit (token or card, either side) under the mouse in the last frame (round intent only).
    pub(super) fn unit_under_mouse(&self) -> Option<usize> {
        if !self.host.session.as_ref().is_some_and(|s| ri::on(&s.w)) {
            return None;
        }
        match aegis_ui::ui::hit(&self.w, self.mouse) {
            Some(Action::UnitClick(u) | Action::Select(u)) => Some(u),
            _ => None,
        }
    }

    pub(super) fn tile_click(&mut self, cx: &Cx, c: V2) {
        if !self.planning() {
            return;
        }
        let Some(h) = self.selected else { return };
        if ri::on(&self.session().w) {
            self.target_click(cx, h, RoundTarget::Cell(c));
            return;
        }
        let rows = self.options_for(h, "tile", None, Some(c));
        self.open_popup(format!("Tile {}, {}", c.x, c.y), rows);
    }

    // ---- INPUT1-U: one input flow under the round intent (INPUT1 2-5: IF, HK, GH, ET) ------------------------------

    /// GH2 a: member `m`'s hero tactic (the party tactic unless the hero tactic menu set one).
    pub(super) fn tactic_of(&self, m: usize) -> usize {
        self.member_tactic.get(&m).copied().unwrap_or(self.party_tactic)
    }

    /// GH2 a: `m`'s ghost is its tactic's draft: an active party member with no pick and no player-set target this
    /// round.
    fn tactic_ghost(&self, m: usize) -> bool {
        let w = &self.session().w;
        ri::on(w) && w.allies.contains(&m) && w.units[m].is_active() && w.has(m) && rip::picks_of(w, m).is_empty() && !self.player_target.contains(&m)
    }

    /// GH2 a: drafts `m`'s ghost with its tactic (`preset`). A draft whose intent change would drop intent-bound buffs
    /// waits for Confirm (IN6, Q18 a: the panel warns "Changing loses ..." meanwhile); a draft equal to the plan is not
    /// sent again. Auto-fill off: nothing (the kit rests every free block).
    fn draft_ghost(&mut self, cx: &Cx, m: usize) {
        if !self.tactic_ghost(m) || !self.session().w.autofill {
            return;
        }
        let draft = Deferred::Preset(TACTICS[self.tactic_of(m)]);
        let s = self.session();
        let Some((w2, p2)) = draft.view(s, m) else { return };
        let drops = ri::intent_of(&w2, m).map(|i| ri::would_drop(&s.w, m, i)).unwrap_or_default();
        let same = ri::intent_of(&w2, m) == ri::intent_of(&s.w, m) && plan_rows(&w2, &p2, m) == plan_rows(&s.w, &s.plan, m) && w2.round_states.get(&m).and_then(|r| r.target) == s.w.round_states.get(&m).and_then(|r| r.target);
        self.deferred.retain(|d| d.0 != m);
        if !drops.is_empty() {
            self.deferred.push((m, draft));
        } else if !same {
            self.send(cx, draft.req(m));
        }
    }

    /// GH2, GH6, GH7: the planning start of a round: the player-set targets and the waiting ghosts reset, the battle
    /// takes the Auto-fill setting (host `autofill`, recorded), every member without a pick gets its tactic's ghost.
    pub(super) fn plan_start(&mut self, cx: &Cx) {
        self.ghosted = self.session().current_round;
        self.player_target.clear();
        self.deferred.clear();
        let on = cx.settings.v["auto_fill"].as_bool().unwrap_or(true);
        if !on || self.session().w.autofill != on {
            self.send(cx, json!({"cmd": "autofill", "on": on}));
        }
        for m in self.session().w.allies.clone() {
            self.draft_ghost(cx, m);
        }
    }

    /// GH7: the setting changed while planning: every member's ghost is rebuilt at once.
    pub(super) fn sync_autofill(&mut self, cx: &Cx, on: bool) {
        self.send(cx, json!({"cmd": "autofill", "on": on}));
        if !on {
            self.deferred.clear();
        }
        for m in self.session().w.allies.clone() {
            self.draft_ghost(cx, m);
        }
    }

    /// PL8, GH2: after `h`'s round changed, the members after it that show their tactic's ghost draft it again (the kit
    /// rebuilt them by their intent).
    pub(super) fn reghost_after(&mut self, cx: &Cx, h: usize) {
        let later: Vec<usize> = self.session().w.allies.iter().copied().skip_while(|&m| m != h).skip(1).collect();
        for m in later {
            self.draft_ghost(cx, m);
        }
    }

    /// IF1, GH2 b: a click names `at` as `h`'s round target (no pick holds one): the hero's intent when it plans a round
    /// on it, else the first intent of that kind of target that does; an intent change that would drop intent-bound
    /// buffs waits for Confirm (IN6). The target becomes player-set.
    fn apply_target(&mut self, cx: &Cx, h: usize, at: RoundTarget) {
        let s = self.session();
        let cur = ri::intent_of(&s.w, h);
        if s.w.round_states.get(&h).and_then(|r| r.target) == Some(at) && self.player_target.contains(&h) {
            return;
        }
        let chosen = cur.filter(|i| plans_on(s, h, *i, at)).or_else(|| intents_for(&s.w, h, at).iter().copied().find(|i| plans_on(s, h, *i, at)));
        let fits_now = cur.is_some_and(|i| ri::target_problem(&s.w, h, i, at).is_empty());
        let drops = chosen.filter(|i| Some(*i) != cur).map(|i| ri::would_drop(&s.w, h, i)).unwrap_or_default();
        self.deferred.retain(|d| d.0 != h);
        let set = match chosen {
            None if fits_now => rip::RoundSet { target: Some(at), ..Default::default() },
            None => return,
            Some(i) if Some(i) == cur => rip::RoundSet { target: Some(at), ..Default::default() },
            Some(i) => rip::RoundSet { intent: Some(i), target: Some(at), ..Default::default() },
        };
        if !self.player_target.contains(&h) {
            self.player_target.push(h);
        }
        let change = Deferred::Round(set);
        if drops.is_empty() {
            self.send(cx, change.req(h));
        } else {
            self.deferred.push((h, change));
        }
    }

    /// IF1: a click on a unit or a tile while `h` plans: the round target first (GH2 b) unless a pick holds it, then the
    /// skill menu at the click.
    pub(super) fn target_click(&mut self, cx: &Cx, h: usize, at: RoundTarget) {
        let held = rip::picks_of(&self.session().w, h).iter().any(|p| p.holds_target());
        if !held {
            self.apply_target(cx, h, at);
            self.reghost_after(cx, h);
        }
        self.open_menu(h, at, None, false);
    }

    /// IF2: the skill menu's title and rows for `h` on `at` (the kit's `pick_options`, in hotkey order), with "Select
    /// <ally>" last on another ally (SL6).
    fn menu_rows(&self, h: usize, at: RoundTarget) -> (String, Vec<Row>) {
        let s = self.session();
        let w = &s.w;
        let opts = rip::pick_options(w, &s.plan, h, at);
        let mut rows: Vec<Row> = opts
            .iter()
            .map(|o| Row { label: pick_label(s, h, &o.picks), detail: pick_detail(&w.cat, o), valid: o.problem.is_none(), act: RowAct::Pick(o.picks.clone()), tip: pick_tip(s, h, o), warn: !o.drops.is_empty(), ap_short: o.ap_short })
            .collect();
        let held = rip::picks_of(w, h).iter().any(|p| p.holds_target());
        let round = w.round_states.get(&h).and_then(|r| r.target);
        if let RoundTarget::Unit(u) = at {
            if u != h && w.allies.contains(&u) {
                rows.push(Row { label: format!("Select {}", w.units[u].name), detail: String::new(), valid: true, act: RowAct::SelectAlly(u), tip: vec![(format!("Select {}", w.units[u].name), th().gold), ("Plan this hero's round instead.".into(), th().text)], warn: false, ap_short: None });
            }
        }
        let skill_rows = rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)));
        let title = if held && !skill_rows && !matches!(at, RoundTarget::Cell(_)) && round.is_some_and(|r| r != at) {
            format!("One target per round: {}", target_name(w, round.unwrap()))
        } else if !rows.iter().any(|r| r.valid && matches!(r.act, RowAct::Pick(_))) {
            format!("No skill reaches {} this round", target_name(w, at))
        } else {
            format!("{} -> {}", w.units[h].name, target_name(w, at))
        };
        (title, rows)
    }

    /// IF1, IF4: opens (or, after a pick, re-lists in place) `h`'s skill menu on `at`. Another ally with no usable row
    /// is selected at once (SL6); after a pick the menu closes when no skill row is left.
    fn open_menu(&mut self, h: usize, at: RoundTarget, place: Option<(i32, i32)>, after_pick: bool) {
        let (title, rows) = self.menu_rows(h, at);
        let ally = match at {
            RoundTarget::Unit(u) if u != h && self.session().w.allies.contains(&u) => Some(u),
            _ => None,
        };
        let usable = rows.iter().any(|r| r.valid && matches!(r.act, RowAct::Pick(_)));
        if let Some(u) = ally.filter(|_| !usable && !after_pick) {
            self.selected = Some(u);
            self.popup = None;
            return;
        }
        if rows.is_empty() || (after_pick && !rows.iter().any(|r| matches!(r.act, RowAct::Pick(_)))) {
            self.popup = None;
            if !after_pick {
                self.message = title;
            }
            return;
        }
        let (x, y) = place.unwrap_or(self.mouse);
        let (pw, ph) = (MENU_W, self.menu_row + 4 + rows.len() as i32 * self.menu_row);
        self.popup = Some(Popup { title, x: x.clamp(2, W - pw - 2), y: y.clamp(2, H - ph - 2), w: pw, rows, menu: Some((h, at)) });
    }

    /// IF4: appends `picks` (a menu row or a hotkey) to `h`'s round on `at` through the host's `pick`; the message names
    /// the pick, a later member's dropped pick (GH5) or, from the battle's events, the buffs an intent change dropped
    /// (IN6). An open menu stays at its place and re-lists.
    pub(super) fn choose_pick(&mut self, cx: &Cx, h: usize, at: RoundTarget, picks: &[rip::Pick], label: &str) {
        let w = &self.session().w;
        let mut req = json!({"cmd": "pick", "unit": h, "target": target_json(at), "pick": pick_json(w, &picks[0])});
        if let Some(then) = picks.get(1) {
            req["then"] = pick_json(w, then);
        }
        let events = self.session().events.len();
        let r = self.send(cx, req);
        if r["ok"] != json!(true) {
            return;
        }
        self.deferred.retain(|d| d.0 != h);
        let name = self.session().w.units[h].name.clone();
        let shown = label.split_once(' ').filter(|(d, _)| d.len() == 1 && d.chars().all(|c| c.is_ascii_digit())).map_or(label, |(_, rest)| rest);
        if let Some(d) = r["dropped"].as_array().and_then(|a| a.first()) {
            self.message = d["reason"].as_str().unwrap_or("").to_string();
        } else if self.session().events.len() == events {
            self.message = format!("{name}: {shown}");
        }
        self.reghost_after(cx, h);
        if let Some(place) = self.popup.as_ref().filter(|p| p.menu.is_some()).map(|p| (p.x, p.y)) {
            self.open_menu(h, at, Some(place), true);
        }
    }

    /// IF7: undo (Z, a right click) takes `h`'s last pick back, Clear (`all`) every pick; with none left the hero shows
    /// its ghost again (GH2 a or b).
    pub(super) fn unpick(&mut self, cx: &Cx, h: usize, all: bool) {
        let name = self.session().w.units[h].name.clone();
        if !self.planning() {
            return;
        }
        if rip::picks_of(&self.session().w, h).is_empty() {
            self.message = format!("{name}: no pick to {}.", if all { "clear" } else { "undo" });
            return;
        }
        let r = self.send(cx, if all { json!({"cmd": "clear", "unit": h}) } else { json!({"cmd": "unpick", "unit": h}) });
        if r["ok"] != json!(true) {
            return;
        }
        self.message = if all { format!("{name}: picks cleared.") } else { format!("{name}: last pick undone.") };
        self.draft_ghost(cx, h);
        self.reghost_after(cx, h);
    }

    /// INPUT2: key `d` (1-4) selects the party's `d`-th member (the party cards' order, `World.allies`); one not on the
    /// field says so. An open menu closes.
    pub(super) fn select_member(&mut self, d: usize) {
        let w = &self.session().w;
        let Some(&u) = w.allies.get(d - 1) else {
            self.message = format!("No party member on key {d}.");
            return;
        };
        if !w.units[u].is_active() {
            self.message = format!("{} is not on the field.", w.units[u].name);
            return;
        }
        self.selected = Some(u);
        self.popup = None;
    }

    /// INPUT2: the selected hero's ghost becomes picks (host `accept_ghost`): all of it (key F) or up to plan entry
    /// `upto` (a click on a ghost block of the strip). A ghost that waits for Confirm (IN6) sends its change first, as
    /// Confirm would.
    pub(super) fn accept_ghost(&mut self, cx: &Cx, upto: Option<usize>) {
        let Some(h) = self.selected.filter(|_| self.planning() && ri::on(&self.session().w)) else { return };
        let w = &self.session().w;
        if !w.allies.contains(&h) || !w.units[h].is_active() {
            return;
        }
        let name = w.units[h].name.clone();
        if let Some(d) = self.deferred.iter().find(|d| d.0 == h).map(|d| d.1.clone()) {
            if self.send(cx, d.req(h))["ok"] != json!(true) {
                return;
            }
            self.deferred.retain(|d| d.0 != h);
        }
        let mut req = json!({"cmd": "accept_ghost", "unit": h});
        if let Some(k) = upto {
            req["upto"] = json!(k);
        }
        let r = self.send(cx, req);
        if r["ok"] != json!(true) {
            self.message = format!("{name}: {}", r["error"].as_str().unwrap_or("refused"));
            return;
        }
        self.popup = None;
        let n = r["added"].as_u64().unwrap_or(0);
        self.message = format!("{name}: {n} auto-filled pick{} accepted.", if n == 1 { "" } else { "s" });
        self.reghost_after(cx, h);
    }

    /// HK2, HK3 (INPUT2: on Shift+digit): digit `d` (1-9). With the skill menu open it picks the listed row with that
    /// digit (or says why not);
    /// with it closed it picks that hotkey skill on the round target (the proposed one counts, TG6; a self-only skill
    /// picked first targets self).
    pub(super) fn hotkey(&mut self, cx: &Cx, d: usize) {
        let Some(h) = self.selected.filter(|_| self.planning() && ri::on(&self.session().w)) else { return };
        let s = self.session();
        let w = &s.w;
        let keys = rip::hotkey_skills(w, h);
        let skill_name = keys.get(d - 1).map(|k| w.cat.skills[*k].name.clone());
        if let Some(p) = self.popup.as_ref().filter(|p| p.menu.is_some()) {
            let prefix = format!("{d} ");
            match p.rows.iter().position(|r| r.label.starts_with(&prefix)) {
                Some(k) if p.rows[k].valid => self.run_choose(cx, k),
                Some(k) => self.message = format!("{}: {}", &p.rows[k].label[prefix.len()..], p.rows[k].detail),
                None => self.message = skill_name.map_or_else(|| format!("No skill on key {d}."), |n| format!("{n} is not offered on this target now.")),
            }
            return;
        }
        let Some(&sid) = keys.get(d - 1) else {
            self.message = format!("No skill on key {d}.");
            return;
        };
        // HK2, HK3: the kit names the target (self for a self-only skill picked first)
        let Some(at) = rip::hotkey_target(w, h, sid) else {
            self.message = "Pick a target first (click a unit or a tile)".into();
            return;
        };
        let name = w.cat.skills[sid].name.clone();
        let opts = rip::pick_options(w, &s.plan, h, at);
        match opts.iter().find(|o| matches!(o.picks[..], [rip::Pick::Skill { skill, .. }] if skill == sid)) {
            Some(o) if o.problem.is_none() => {
                let picks = o.picks.clone();
                self.choose_pick(cx, h, at, &picks, &name);
            }
            Some(o) => self.message = format!("{name}: {}", o.problem.clone().unwrap_or_default()),
            None => self.message = format!("{name} can't act on {} this round.", target_name(w, at)),
        }
    }

    /// `Action::Choose(k)` from a key (the same path as a click).
    fn run_choose(&mut self, cx: &Cx, k: usize) {
        let Some(p) = self.popup.clone() else { return };
        let Some(row) = p.rows.get(k).filter(|r| r.valid).cloned() else { return };
        match (row.act, p.menu) {
            (RowAct::Pick(picks), Some((h, at))) => self.choose_pick(cx, h, at, &picks, &row.label),
            (RowAct::SelectAlly(u), _) => {
                self.selected = Some(u);
                self.popup = None;
            }
            _ => {}
        }
    }


    /// The ruleset chip of an Arena fight beyond classic (RULESET1-U, user 2026-10-29: the Scene Runner card's text);
    /// None in a world battle and in a plain classic one.
    pub(super) fn arena_chip(&self) -> Option<String> {
        let s = self.host.session.as_ref()?;
        if self.world_mode() || s.w.ruleset.rules.is_empty() {
            return None;
        }
        Some(crate::runner::chip_text(&s.w.ruleset.name, &self.arena_rules, &self.arena_diagonal, s.w.plan_reveal && s.w.ruleset.has(emberwake_rules::ruleset::PLAN_BADGES)))
    }

    fn open_popup(&mut self, title: String, rows: Vec<Row>) {
        if rows.is_empty() {
            self.message = "Nothing to do there.".into();
            return;
        }
        let (mx, my) = self.mouse;
        let w = 170;
        let h = self.menu_row + 4 + rows.len() as i32 * self.menu_row;
        self.popup = Some(Popup { title, x: mx.clamp(2, W - w - 2), y: my.clamp(2, H - h - 2), w, rows, menu: None });
    }

    /// Candidate steps for hero `h` (`what`: foe / ally / self / tile), each probed through the API.
    pub(super) fn options_for(&mut self, h: usize, what: &str, target: Option<usize>, cell: Option<V2>) -> Vec<Row> {
        let skills = self.host.handle(&json!({"cmd": "skills", "unit": h}))["skills"].as_array().cloned().unwrap_or_default();
        let mut steps: Vec<(String, Value, Tip)> = Vec::new();
        let target_cell = target.map(|t| {
            let p = self.session().w.pos(t);
            json!([p.x, p.y])
        });
        for sk in &skills {
            let id = sk["skill"].as_str().unwrap_or("").to_string();
            let name = sk["name"].as_str().unwrap_or("").to_string();
            let kind = sk["kind"].as_str().unwrap_or("");
            let area = sk["area"].as_str().unwrap_or("NONE") != "NONE";
            let mut tip: Tip = vec![(name.clone(), th().gold)];
            if let Some(d) = sk["description"].as_str().filter(|d| !d.is_empty()) {
                tip.push((d.to_string(), th().text));
            }
            tip.push((format!("{} AP, {} block{}, reach {}-{}", sk["ap"], sk["blocks"], if sk["blocks"] == json!(1) { "" } else { "s" }, sk["reach"][0], sk["reach"][1]), th().dim));
            // HOOKS1-UI slice 3 (HC1): the host's `hp_cost` (only on a skill that costs HP)
            if let Some(c) = sk["hp_cost"].as_i64().filter(|c| *c > 0) {
                tip.push((intent_text::hp_cost_text(c as i32), th().foe));
            }
            match what {
                "foe" => match kind {
                    "ATTACK" if area => steps.push((name, json!({"mode": "area", "skill": id, "cell": target_cell}), tip)),
                    "ATTACK" | "DISENGAGE" => steps.push((name, json!({"mode": "engage", "skill": id, "target": target}), tip)),
                    _ => {}
                },
                "ally" if kind == "SUPPORT" => steps.push((name, json!({"mode": "ally", "skill": id, "target": target}), tip)),
                "self" if matches!(kind, "DEFEND" | "FOCUS" | "REST" | "DISENGAGE" | "SUPPORT") => steps.push((name, json!({"mode": "self", "skill": id}), tip)),
                "tile" if kind == "ATTACK" && area => steps.push((name, json!({"mode": "area", "skill": id, "cell": cell.map(|c| json!([c.x, c.y]))}), tip)),
                _ => {}
            }
        }
        // ITEMUSE1 rule 2: "Use <item> (<n> left)" for each pouch item that may be aimed at the clicked unit.
        if let Some(t) = if what == "self" { Some(h) } else { target.filter(|_| what != "tile") } {
            let items = self.host.handle(&json!({"cmd": "items", "unit": h, "target": t}))["items"].as_array().cloned().unwrap_or_default();
            for it in items {
                let name = it["name"].as_str().unwrap_or("").to_string();
                let mut tip: Tip = vec![(name.clone(), th().gold)];
                if let Some(d) = it["description"].as_str().filter(|d| !d.is_empty()) {
                    tip.push((d.to_string(), th().text));
                }
                let reach = match it["reach"].as_i64().unwrap_or(0) {
                    0 => "self only".to_string(),
                    r => format!("reach {r}"),
                };
                tip.push((format!("{} AP, {} block{}, {reach}; used when the action resolves", it["ap"], it["blocks"], if it["blocks"] == json!(1) { "" } else { "s" }), th().dim));
                steps.push((format!("Use {name} ({} left)", it["left"]), json!({"mode": "item", "item": it["item"], "target": t}), tip));
            }
        }
        if what == "tile" {
            if let Some(c) = cell {
                steps.insert(0, ("Move here".into(), json!({"mode": "goto", "cell": [c.x, c.y]}), vec![("Move here".into(), th().gold), ("1 AP per tile; moving must come first in a round.".into(), th().text)]));
            }
        }
        let mut rows = Vec::new();
        for (label, step, mut tip) in steps {
            let p = self.host.handle(&json!({"cmd": "probe", "unit": h, "step": step}));
            let valid = p["valid"] == json!(true);
            let detail = if valid {
                let es = p["entries"].as_array().cloned().unwrap_or_default();
                let dmg: i64 = es.iter().map(|e| e["est_damage"].as_i64().unwrap_or(0)).sum();
                let heal: i64 = es.iter().map(|e| e["est_heal"].as_i64().unwrap_or(0)).sum();
                let first = es.first().map(|e| e["start"].as_i64().unwrap_or(0)).unwrap_or(0);
                let last = es.last().map(|e| e["end"].as_i64().unwrap_or(0)).unwrap_or(0);
                tip.push((format!("Blocks {first}-{last}, AP left after: {}", p["ap_left"]), ALLY));
                let mut d = format!("{} AP", p["ap"].as_i64().unwrap_or(0));
                if let Some(c) = skills_hp(&skills, &step) {
                    d.push_str(&format!("  {c} HP"));
                }
                if dmg > 0 {
                    d.push_str(&format!("  ~{dmg} dmg"));
                }
                if heal > 0 {
                    d.push_str(&format!("  +{heal} HP"));
                }
                d
            } else {
                // HOOKS1-UI slice 3: an HP-cost refusal in words
                let r = intent_text::refusal_text(p["reason"].as_str().unwrap_or(""));
                tip.push((r.clone(), th().foe));
                r
            };
            rows.push(Row { label, detail, valid, act: RowAct::Step(step), tip, warn: false, ap_short: None });
        }
        rows.sort_by_key(|r| !r.valid);
        rows
    }
}

/// INPUT1 IF1: the skill menu's width (wider than the step menus: "4 blocks, 4 AP; Maneuver (Engage)" on a row).
const MENU_W: i32 = 214;

/// IF3: a skill-menu row's hover card: the name, the skill's description and its range line (MR4), the refusal, the
/// buffs an intent change drops (IN6).
fn pick_tip(s: &Session, h: usize, o: &rip::PickOption) -> Tip {
    let w = &s.w;
    let mut tip: Tip = vec![(o.picks.iter().map(|p| rip::pick_name(w, p)).collect::<Vec<_>>().join(", then "), th().gold)];
    if let Some(rip::Pick::Skill { skill, .. }) = o.picks.iter().find(|p| matches!(p, rip::Pick::Skill { .. })) {
        let sk = &w.cat.skills[*skill];
        if let Some(d) = sk.raw.get("description").and_then(Value::as_str).filter(|d| !d.is_empty()) {
            tip.push((d.to_string(), th().text));
        }
        tip.push((intent_text::range_line(w, h, sk), th().dim));
        // INPUT2: the row's hotkey (the label's digit) is Shift with that digit
        if let Some(k) = rip::hotkey_skills(w, h).iter().position(|x| x == skill).filter(|k| *k < 9) {
            tip.push((format!("Key: Shift+{}", k + 1), th().dim));
        }
    }
    if let Some(why) = &o.problem {
        tip.push((why.clone(), th().foe));
    }
    if !o.drops.is_empty() {
        tip.push((format!("Changing loses {}.", o.drops.join(", ")), th().foe));
    }
    tip
}

/// INPUT1 MR13: the open position tiles of a Maneuver Engage pick of `h` on the unit `at` (true: the preferred tier),
/// read from where `h`'s picks leave it; empty for any other row and for a protected flank.
pub(super) fn engage_positions(s: &Session, h: usize, at: RoundTarget, picks: &[rip::Pick]) -> Vec<(V2, bool)> {
    let w = &s.w;
    let (RoundTarget::Unit(t), [rip::Pick::Skill { skill, kind: Some(ManeuverKind::Engage) }]) = (at, picks) else { return Vec::new() };
    let sk = &w.cat.skills[*skill];
    let Some(position) = ri::position_of(w, sk).filter(|_| w.has(t) && w.has(h)) else { return Vec::new() };
    let ghost = rip::ghost_entries(w, &s.plan, h);
    let from = s.plan.entries_for(w, h).iter().zip(&ghost).filter(|(e, g)| !**g && e.step_index >= 0).last().map(|(e, _)| e.pos_after).unwrap_or_else(|| w.pos(h));
    let blocked = s.plan.blocked_cells(w, h);
    let p = ri::positions(w, t, h, position, from, ri::diagonal(w, h, Some(sk)), &blocked);
    p.preferred.iter().map(|c| (*c, true)).chain(p.fallback.iter().map(|c| (*c, false))).collect()
}

/// `m`'s compiled plan in short (skill, start, target, tile after) to tell two plans apart.
pub(super) fn plan_rows(w: &World, plan: &emberwake_battle::plan::Plan, m: usize) -> Vec<(usize, i32, Option<usize>, V2)> {
    plan.entries_for(w, m).iter().map(|e| (e.skill, e.start_block, e.target, e.pos_after)).collect()
}
