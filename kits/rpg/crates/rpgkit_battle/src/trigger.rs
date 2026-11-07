//! Events and triggers (HOOKS1 8.1-8.4, rule `triggers`; engine-only, no Godot original). Units hold triggers from
//! their passives and worn items (`Unit::held_triggers`) and, while a skill's command runs (start to impact), from the
//! skill's own `triggers`. A battle raises events as an action resolves (hits taken and dealt, kills, a friend downed,
//! breaks, an attack that found nobody, heals and cleanses, an HP share crossing down), as a Defend starts, after the
//! poise refill and after the round-end ticks; each is handled after the action and the resolver's combo pass, in raise
//! order, holders in registration order then list order (8.1). A trigger fires when every `when` condition holds and its
//! `limit` (round, battle, none) allows; its effects run in list order and raise no new event (depth 1). `ally_hit` is
//! the one event handled inside the hit, before its damage (4.3 step 1): its `guard` and `redirect` effects shape the
//! hit (8.4: one redirect a hit; a running skill's triggers first, then passives and worn items), its other effects run
//! after the action with the rest. A `guard` effect sets a held guard (8.3) that counts as guarding (P6's guard factor,
//! `guarded_hit`, `immune: guarding`) until its last block, a break, `end_guard` or the round's end. A skill's `riders`
//! (RD1, HOOKS1-K) run at 4.3 step 8 per unit the skill landed on, with 8.2's conditions and effects (their `to`
//! defaults to that unit). Every entry point reads the rule first, so classic battles never fire, log, draw or record
//! anything new.

use crate::command::CmdKind;
use crate::forced::{self, Collide, Moved};
use crate::resolver::Resolver;
use crate::world::{UnitId, World};
use aegis_core::value::{self, Obj};
use indexmap::IndexMap;
use rpgkit_data::TriggerRow;
use rpgkit_rules::grants::Target;
use rpgkit_rules::ruleset;
use rpgkit_rules::stats::roundi;
use rpgkit_rules::status;
use rpgkit_rules::unit::HeldGuard;

/// 8.2: a `grant` of `rounds` -1 lasts the battle.
pub const BATTLE_ROUNDS: i32 = 999;
/// 8.2: the largest `ap` a trigger gives.
pub const MAX_AP: i32 = 4;

/// Whether the battle plays by events and triggers.
pub fn on(w: &World) -> bool {
    w.ruleset.has(ruleset::TRIGGERS)
}

/// One damaging hit an action dealt (its events read them): the unit, the HP it lost, whether the hit was ranged.
#[derive(Debug, Clone, PartialEq)]
pub struct Struck {
    pub target: UnitId,
    pub damage: i32,
    pub ranged: bool,
}

/// One trigger a unit holds now: its display source (the skill, passive or item name), its `limit` key and its row.
#[derive(Debug, Clone, PartialEq)]
pub struct Holder {
    pub unit: UnitId,
    pub source: String,
    pub key: String,
    pub row: TriggerRow,
}

/// One event raised for one holder (8.1): `other` as the table says, `ally` the friend of an ally event, the event's
/// hit (ranged or melee) and skill family, the other unit's HP share before the event (0-100), and for `hp_below` the
/// holder's share before and after.
#[derive(Debug, Clone, PartialEq)]
pub struct Event {
    pub on: &'static str,
    pub holder: UnitId,
    pub other: Option<UnitId>,
    pub ally: Option<UnitId>,
    pub ranged: Option<bool>,
    pub family: String,
    pub other_hp: Option<f64>,
    pub hp_from: f64,
    pub hp_to: f64,
}

impl Event {
    pub(crate) fn new(on: &'static str, holder: UnitId, other: Option<UnitId>) -> Self {
        Self { on, holder, other, ally: None, ranged: None, family: String::new(), other_hp: None, hp_from: 0.0, hp_to: 0.0 }
    }
}

/// What one trigger (or rider) did: the holder, the event (`rider` for a rider) and its other unit, the source, one log
/// line per effect with the HP that effect took (`dealt`) and its HP journal rows held back for the line's own log cell
/// (`rows`), and the forced moves it drove (each collision hit holds its own rows).
#[derive(Debug, Clone, PartialEq)]
pub struct Fired {
    pub holder: UnitId,
    pub on: String,
    pub other: Option<UnitId>,
    pub source: String,
    pub lines: Vec<String>,
    pub dealt: Vec<i32>,
    pub rows: Vec<Vec<(UnitId, i32)>>,
    pub moves: Vec<Moved>,
}

impl Fired {
    pub(crate) fn new(holder: UnitId, on: &str, other: Option<UnitId>, source: &str) -> Self {
        Self { holder, on: on.to_string(), other, source: source.to_string(), lines: Vec::new(), dealt: Vec::new(), rows: Vec::new(), moves: Vec::new() }
    }

    /// One effect's log line, the HP it took and its held-back journal rows.
    pub(crate) fn push(&mut self, line: String, dealt: i32, rows: Vec<(UnitId, i32)>) {
        self.lines.push(line);
        self.dealt.push(dealt);
        self.rows.push(rows);
    }
}

/// 8.4: one `ally_hit` trigger that may fire inside a hit, and how many more times it may fire (-1 = no limit).
#[derive(Debug, Clone, PartialEq)]
pub struct Guardian {
    pub holder: Holder,
    pub left: i32,
}

/// 8.4: an `ally_hit` trigger fired inside a hit: the guardian, the event, its log lines and the effects left for after
/// the action.
#[derive(Debug, Clone, PartialEq)]
pub struct AllyFired {
    pub guardian: usize,
    pub event: Event,
    pub lines: Vec<String>,
    pub rest: Vec<Obj>,
}

/// 8.4: the `ally_hit` triggers an attack's hits consult (set by the resolver before the attack lands; rule triggers).
#[derive(Debug, Clone, PartialEq)]
pub struct AllyWatch {
    pub block: i32,
    pub guardians: Vec<Guardian>,
    pub fired: Vec<AllyFired>,
}

/// 8.4: a redirect a hit takes: the holder, its share and the fired trigger (`AllyWatch::fired` index) its line joins.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Redirect {
    pub holder: UnitId,
    pub pct: i32,
    pub fired: usize,
}

impl AllyWatch {
    /// Adds the redirect's line (what the holder took) to the trigger that claimed it.
    pub fn note(&mut self, r: &Redirect, what: String) {
        if let Some(f) = self.fired.get_mut(r.fired) {
            f.lines.push(what);
        }
    }
}

/// The battle's trigger state (on the resolver): `limit` counts, the HP shares last seen (`hp_below`), the grants that
/// end with a guard, the round a `reveal` shows, and the fired triggers the session has not logged yet.
#[derive(Debug, Clone, Default)]
pub struct State {
    counts: IndexMap<(UnitId, String), (i32, i32)>,
    hp_seen: Vec<i32>,
    until_guard: Vec<(UnitId, String)>,
    reveal_round: i32,
    reveal_base: Option<bool>,
    pub fired: Vec<Fired>,
}

impl State {
    /// The fired triggers since the last call (the session logs them).
    pub fn take(&mut self) -> Vec<Fired> {
        std::mem::take(&mut self.fired)
    }

    fn allows(&self, unit: UnitId, h: &Holder) -> i32 {
        let (round, battle) = self.counts.get(&(unit, h.key.clone())).copied().unwrap_or((0, 0));
        match h.row.limit.as_str() {
            "none" => -1,
            "battle" => (1 - battle).max(0),
            _ => (1 - round).max(0),
        }
    }

    fn count(&mut self, unit: UnitId, key: &str) {
        let e = self.counts.entry((unit, key.to_string())).or_insert((0, 0));
        e.0 += 1;
        e.1 += 1;
    }

    fn watch_hp(&mut self, w: &World) {
        while self.hp_seen.len() < w.units.len() {
            self.hp_seen.push(w.units[self.hp_seen.len()].hp);
        }
    }
}

/// The state an action's events compare with (taken before it lands).
#[derive(Debug, Clone)]
pub struct Before {
    hp: Vec<i32>,
    alive: Vec<bool>,
    guarding: Vec<bool>,
}

/// 8.3: `c` guards at `block`: a running Defend (P6, G2) or a held guard.
pub fn guarding(w: &World, sched: &[crate::resolver::ScheduledAction], c: UnitId, block: i32) -> bool {
    crate::poise::guarded(sched, c, block) || held_at(w, c, block).is_some()
}

/// 8.3: `c`'s held guard mitigation at `block` (None without one).
pub fn held_at(w: &World, c: UnitId, block: i32) -> Option<f64> {
    w.units[c].held_guard.filter(|g| block <= g.last_block).map(|g| g.mitigation)
}

/// 8.3: `prepare_block` raises each unit's guard to its held guard when that is larger (rule triggers).
pub fn raise_held(w: &mut World, c: UnitId, block: i32) {
    if let Some(m) = held_at(w, c, block) {
        let u = &mut w.units[c];
        u.guard = u.guard.max(m);
    }
}

/// The holders of `c` now: the triggers of its commands that run (start to impact) at `block`, then its passives' and
/// worn items' (8.1), then its meters' (U1.2-S5). None when it is not on the field.
fn holders_of(res: &Resolver, w: &World, c: UnitId, block: i32, skills: bool, held: bool) -> Vec<Holder> {
    let mut out = Vec::new();
    if !w.has(c) {
        return out;
    }
    if skills {
        for a in res.scheduled.iter().filter(|a| a.unit == c && !a.cancelled && a.start <= block && block <= a.impact) {
            for (i, row) in a.command.triggers.iter().enumerate() {
                out.push(Holder { unit: c, source: a.command.name.clone(), key: format!("skill:{}#{i}", a.command.name), row: row.clone() });
            }
        }
    }
    if held {
        for t in w.units[c].held_triggers() {
            out.push(Holder { unit: c, source: source_name(w, &t.source), key: t.key.clone(), row: t.row.clone() });
        }
        // U1.2-S5: then the trigger rows of the meters it holds
        out.extend(crate::meter::held_triggers(w, c));
    }
    out
}

/// The catalog name of a held trigger's source (`passive:<id>`, `gear:<item id>`), else the id.
fn source_name(w: &World, source: &str) -> String {
    if let Some(id) = source.strip_prefix("passive:") {
        return w.cat.passives.get(id).map_or_else(|| id.to_string(), |p| p.name.clone());
    }
    if let Some(id) = source.strip_prefix("gear:") {
        return w.cat.items.get(id).map_or_else(|| id.to_string(), |p| p.name.clone());
    }
    source.to_string()
}

fn share(w: &World, c: UnitId) -> f64 {
    let u = &w.units[c];
    if u.max_hp <= 0 {
        return 0.0;
    }
    u.hp as f64 * 100.0 / u.max_hp as f64
}

fn share_of(w: &World, c: UnitId, hp: i32) -> f64 {
    let max = w.units[c].max_hp;
    if max <= 0 {
        return 0.0;
    }
    hp as f64 * 100.0 / max as f64
}

/// 8.2: every `when` condition of `row` holds for `ev` (`meter` `{id, at_least}`: the holder's value of that meter is at
/// least `at_least`; never without the rule unit_meters or the meter, HOOKS1-M).
fn matches(w: &World, ev: &Event, when: &Obj) -> bool {
    if ev.on == "hp_below" {
        let pct = value::float(when, "pct", 0.0);
        if !(ev.hp_from > pct && ev.hp_to <= pct) {
            return false;
        }
    }
    if let Some(f) = ev.ally {
        let range = value::int(when, "range", 1).max(1);
        if !w.has(ev.holder) || w.pos(ev.holder).dist(w.pos(f)) > range {
            return false;
        }
    }
    if value::boolean(when, "ranged", false) && ev.ranged != Some(true) {
        return false;
    }
    if value::boolean(when, "melee", false) && ev.ranged != Some(false) {
        return false;
    }
    let skills: Vec<String> = value::array(when, "skills").iter().filter_map(|v| v.as_str().map(str::to_string)).collect();
    if !skills.is_empty() && !skills.contains(&ev.family) {
        return false;
    }
    if when.contains_key("other_hp_below") {
        let pct = value::float(when, "other_hp_below", 0.0);
        if ev.other_hp.is_none_or(|s| s > pct) {
            return false;
        }
    }
    if value::boolean(when, "other_broken", false) && !ev.other.is_some_and(|o| w.poise.get(&o).is_some_and(|s| s.broken)) {
        return false;
    }
    let st = value::string(when, "other_status", "");
    if !st.is_empty() && !ev.other.is_some_and(|o| w.units[o].has_status(&st)) {
        return false;
    }
    let intent = value::string(when, "intent", "");
    if !intent.is_empty() && crate::round_intent::intent_of(w, ev.holder).map(|i| i.id()) != Some(intent.as_str()) {
        return false;
    }
    match when.get("meter") {
        None => true,
        Some(m) => m.as_object().is_some_and(|m| crate::meter::value(w, ev.holder, &value::string(m, "id", "")).is_some_and(|v| v >= value::int(m, "at_least", 0))),
    }
}

fn has_redirect(row: &TriggerRow) -> bool {
    row.effects.iter().any(|e| value::string(e, "type", "") == "redirect")
}

/// The unit an effect acts on: `to` self (the default) or other.
fn target_of(e: &Obj, ev: &Event) -> Option<UnitId> {
    if value::string(e, "to", "self") == "other" {
        ev.other
    } else {
        Some(ev.holder)
    }
}

/// 8.3: sets `c`'s held guard (the larger mitigation and the later last block when it holds one) and raises its guard.
fn hold_guard(w: &mut World, c: UnitId, mitigation: f64, blocks: i32, block: i32) -> HeldGuard {
    let last = if blocks <= 0 { w.total_blocks } else { (block + blocks - 1).min(w.total_blocks) };
    let mut g = HeldGuard { mitigation: mitigation.clamp(0.1, 0.9), last_block: last };
    if let Some(old) = w.units[c].held_guard.filter(|o| block <= o.last_block) {
        g = HeldGuard { mitigation: g.mitigation.max(old.mitigation), last_block: g.last_block.max(old.last_block) };
    }
    w.units[c].held_guard = Some(g);
    let u = &mut w.units[c];
    u.guard = u.guard.max(g.mitigation);
    g
}

fn guard_line(w: &World, c: UnitId, g: &HeldGuard, block: i32) -> String {
    let n = g.last_block - block + 1;
    format!("{} holds guard ({}% less damage, {n} block{})", w.units[c].name, roundi(g.mitigation * 100.0), if n == 1 { "" } else { "s" })
}

/// 8.4: the `ally_hit` triggers that may fire in the action `idx`'s hits: a running skill's triggers first, then
/// passives and worn items, each in registration order; empty unless the rule is on and the action attacks.
fn guardians(res: &Resolver, w: &World, idx: usize) -> Vec<Guardian> {
    let a = &res.scheduled[idx];
    if !matches!(a.command.kind, CmdKind::Attack { .. } | CmdKind::Area { .. }) {
        return Vec::new();
    }
    let block = a.impact;
    let mut out = Vec::new();
    for skills in [true, false] {
        for c in 0..w.units.len() {
            for h in holders_of(res, w, c, block, skills, !skills) {
                if h.row.on != "ally_hit" {
                    continue;
                }
                let left = res.triggers.allows(c, &h);
                if left != 0 {
                    out.push(Guardian { holder: h, left });
                }
            }
        }
    }
    out
}

/// 8.4, 4.3 step 1: friend `f` of the guardians is about to take `src`'s attack hit: each `ally_hit` trigger whose
/// conditions hold fires (H within `range` of F, H active, H not the attacker, F alive; `skip`: units the hit's area
/// also strikes); its `guard` effects apply now, the first with a `redirect` claims the hit (a later one with a redirect
/// does not fire), its other effects wait for the action's end. Returns the redirect.
pub fn ally_hit(w: &mut World, watch: &mut AllyWatch, src: UnitId, f: UnitId, ranged: bool, family: &str, skip: &[UnitId]) -> Option<Redirect> {
    let mut claim: Option<Redirect> = None;
    if !w.units[f].is_alive() {
        return None;
    }
    let other_hp = Some(share(w, src));
    for gi in 0..watch.guardians.len() {
        let g = &watch.guardians[gi];
        let h = g.holder.unit;
        if g.left == 0 || h == src || h == f || skip.contains(&h) || !w.has(h) || !w.friends_of(h).contains(&f) {
            continue;
        }
        let redirects = has_redirect(&g.holder.row);
        if redirects && claim.is_some() {
            continue;
        }
        let ev = Event { ally: Some(f), ranged: Some(ranged), family: family.to_string(), other_hp, ..Event::new("ally_hit", h, Some(src)) };
        if !matches(w, &ev, &g.holder.row.when) {
            continue;
        }
        let row = g.holder.row.clone();
        if watch.guardians[gi].left > 0 {
            watch.guardians[gi].left -= 1;
        }
        let mut lines = Vec::new();
        let mut rest = Vec::new();
        for e in &row.effects {
            match value::string(e, "type", "").as_str() {
                "guard" => {
                    let Some(to) = target_of(e, &ev) else { continue };
                    if !w.units[to].is_active() {
                        continue;
                    }
                    let held = hold_guard(w, to, value::float(e, "mitigation", 0.5), value::int(e, "blocks", 0).clamp(0, 6), watch.block);
                    lines.push(guard_line(w, to, &held, watch.block));
                }
                "redirect" => {
                    if claim.is_none() {
                        claim = Some(Redirect { holder: h, pct: value::int(e, "pct", 100).clamp(1, 100), fired: watch.fired.len() });
                    }
                }
                _ => rest.push(e.clone()),
            }
        }
        watch.fired.push(AllyFired { guardian: gi, event: ev, lines, rest });
    }
    claim
}

/// 8.4: the HP share of a hit's `pct` part: rounded, at least 1 when the part is above 0.
pub fn part(dmg: i32, pct: i32) -> i32 {
    if pct <= 0 || dmg <= 0 {
        return 0;
    }
    roundi(dmg as f64 * pct as f64 / 100.0).max(1)
}

/// Before the action `idx` lands (rule triggers): its `ally_hit` watch, and the state its events compare with.
pub fn before_action(res: &mut Resolver, w: &World, idx: usize) -> Option<Before> {
    if !on(w) {
        return None;
    }
    res.triggers.watch_hp(w);
    let block = res.scheduled[idx].impact;
    let g = guardians(res, w, idx);
    res.scheduled[idx].command.ally = if g.is_empty() { None } else { Some(AllyWatch { block, guardians: g, fired: Vec::new() }) };
    let n = w.units.len();
    Some(Before {
        hp: (0..n).map(|c| w.units[c].hp).collect(),
        alive: (0..n).map(|c| w.units[c].is_alive()).collect(),
        guarding: (0..n).map(|c| guarding(w, &res.scheduled, c, block)).collect(),
    })
}

/// After the action `idx` and the combo pass (8.1): the `ally_hit` triggers it fired are counted and logged and their
/// other effects run, then its events in raise order: per damaging hit `hit_taken`, `guarded_hit`, `hit_dealt`, `kill`;
/// `ally_downed`; `broken`, `break_dealt`; `evaded`; `heal_given`, `cleanse_given`; then each `hp_below` crossing.
pub fn after_action(res: &mut Resolver, w: &mut World, idx: usize, before: &Before) {
    let block = res.scheduled[idx].impact;
    let src = res.scheduled[idx].unit;
    let watch = res.scheduled[idx].command.ally.take();
    let events = action_events(res, w, idx, before);
    if let Some(watch) = watch {
        for af in &watch.fired {
            let holder = &watch.guardians[af.guardian].holder;
            res.triggers.count(holder.unit, &holder.key);
            let mut fired = Fired::new(holder.unit, "ally_hit", af.event.other, &holder.source);
            // the redirect's HP was lost inside the hit (the attack's own log cell explains it)
            for l in &af.lines {
                let l = line(w, holder, l);
                res.log.push(l.clone());
                fired.push(l, 0, Vec::new());
            }
            let h = holder.clone();
            run_effects(res, w, &af.event, &h, &af.rest, block, true, &mut fired);
            if !fired.lines.is_empty() || !fired.moves.is_empty() {
                res.triggers.fired.push(fired);
            }
        }
    }
    dispatch(res, w, events, block);
    end_guard_grants(res, w, block, false);
    raise_held(w, src, block);
}

/// The holder key prefix of a rider (riders have no `limit`; the log tags their lines `[Rider]`).
const RIDER_KEY: &str = "rider:";

/// 4.3 step 8, RD1 (rule triggers): the riders of the action `idx` run per unit it landed on: each unit its hits
/// damaged (hit order, once each; an intercept's holder in place of the friend), or a support skill's target. The
/// event (`rider`) has the skill's user as holder and that unit as `other` (an effect's `to` defaults to it), the hit's
/// reach and family, and that unit's HP share before the action; 8.2's conditions apply. One line per effect.
pub fn riders(res: &mut Resolver, w: &mut World, idx: usize, before: &Before) {
    let a = &res.scheduled[idx];
    let (src, cmd) = (a.unit, &a.command);
    if cmd.riders.is_empty() || cmd.failed || !w.units[src].is_active() {
        return;
    }
    let landed: Vec<(UnitId, Option<bool>)> = match &cmd.kind {
        CmdKind::Attack { .. } | CmdKind::Area { .. } => {
            let mut out: Vec<(UnitId, Option<bool>)> = Vec::new();
            // RD1 "damaged by a hit": a status-only attack lands no rider, nor a hit the unit took no HP from (an
            // intercept's holder already at 1 HP)
            for &(t, _) in &cmd.hits {
                let Some(s) = cmd.struck.iter().find(|s| s.target == t && s.damage > 0) else { continue };
                if !out.iter().any(|(u, _)| *u == t) {
                    out.push((t, Some(s.ranged)));
                }
            }
            out
        }
        CmdKind::Support { target, .. } => vec![(*target, None)],
        _ => Vec::new(),
    };
    let (name, family, riders, block) = (cmd.name.clone(), cmd.family.clone(), cmd.riders.clone(), a.impact);
    for (t, ranged) in landed {
        for (i, r) in riders.iter().enumerate() {
            let ev = Event { ranged, family: family.clone(), other_hp: Some(share_of(w, t, before.hp[t])), ..Event::new("rider", src, Some(t)) };
            if !matches(w, &ev, &r.when) {
                continue;
            }
            let row = TriggerRow { on: "rider".into(), when: r.when.clone(), effects: r.effects.clone(), limit: "none".into() };
            let h = Holder { unit: src, source: name.clone(), key: format!("{RIDER_KEY}{name}#{i}"), row };
            // 8.2: a rider's effect acts on the unit it landed on unless it says `to: self`
            let effects: Vec<Obj> = r
                .effects
                .iter()
                .map(|e| {
                    let mut e = e.clone();
                    if !e.contains_key("to") {
                        e.insert("to".into(), "other".into());
                    }
                    e
                })
                .collect();
            let mut fired = Fired::new(src, "rider", Some(t), &name);
            run_effects(res, w, &ev, &h, &effects, block, true, &mut fired);
            if !fired.lines.is_empty() || !fired.moves.is_empty() {
                res.triggers.fired.push(fired);
            }
        }
    }
}

/// The events of the action `idx` in raise order (8.1), the `hp_below` crossings last (the seen HP updates).
fn action_events(res: &mut Resolver, w: &World, idx: usize, before: &Before) -> Vec<Event> {
    let src = res.scheduled[idx].unit;
    let mut events = Vec::new();
    let cmd = &res.scheduled[idx].command;
    let family = cmd.family.clone();
    let hp_before = |c: UnitId| Some(share_of(w, c, before.hp[c]));
    for s in &cmd.struck {
        if s.damage <= 0 || !w.foes_of(src).contains(&s.target) {
            continue;
        }
        let t = s.target;
        let hit = |on: &'static str, holder: UnitId, other: UnitId| Event { ranged: Some(s.ranged), family: family.clone(), other_hp: hp_before(other), ..Event::new(on, holder, Some(other)) };
        events.push(hit("hit_taken", t, src));
        if before.guarding[t] {
            events.push(hit("guarded_hit", t, src));
        }
        events.push(hit("hit_dealt", src, t));
        if before.alive[t] && !w.units[t].is_alive() && !events.iter().any(|e: &Event| e.on == "kill" && e.other == Some(t)) {
            events.push(hit("kill", src, t));
        }
    }
    for d in 0..w.units.len() {
        if before.alive[d] && !w.units[d].is_alive() {
            for &f in w.friends_of(d) {
                if f != d {
                    events.push(Event { ally: Some(d), ..Event::new("ally_downed", f, Some(d)) });
                }
            }
        }
    }
    let mut broke: Vec<UnitId> = cmd.staggers.iter().filter(|st| st.broke.is_some()).map(|st| st.target).collect();
    for m in &cmd.pushes {
        broke.extend(m.hits.iter().filter(|h| h.stagger.as_ref().is_some_and(|st| st.broke.is_some())).map(|h| h.unit));
    }
    for b in broke {
        events.push(Event { family: family.clone(), ..Event::new("broken", b, Some(src)) });
        if b != src {
            events.push(Event { family: family.clone(), other_hp: hp_before(b), ..Event::new("break_dealt", src, Some(b)) });
        }
    }
    if let Some(t) = cmd.evaded {
        events.push(Event { family: family.clone(), ..Event::new("evaded", t, Some(src)) });
    }
    match &cmd.kind {
        CmdKind::Support { target, last_heal, last_cleansed, .. } => {
            if *last_heal > 0 {
                events.push(Event { family: family.clone(), other_hp: hp_before(*target), ..Event::new("heal_given", src, Some(*target)) });
            }
            if !last_cleansed.is_empty() {
                events.push(Event { family: family.clone(), other_hp: hp_before(*target), ..Event::new("cleanse_given", src, Some(*target)) });
            }
        }
        CmdKind::Item { target, last_heal, .. } if *last_heal > 0 => {
            events.push(Event { family: family.clone(), other_hp: hp_before(*target), ..Event::new("heal_given", src, Some(*target)) });
        }
        _ => {}
    }
    events.extend(hp_crossings(res, w));
    events
}

/// `hp_below` events: each unit whose HP share went down since it was last seen (any cause); the seen HP updates.
fn hp_crossings(res: &mut Resolver, w: &World) -> Vec<Event> {
    res.triggers.watch_hp(w);
    let mut out = Vec::new();
    for c in 0..w.units.len() {
        let (was, now) = (res.triggers.hp_seen[c], w.units[c].hp);
        if now < was {
            out.push(Event { hp_from: share_of(w, c, was), hp_to: share_of(w, c, now), ..Event::new("hp_below", c, None) });
        }
        res.triggers.hp_seen[c] = now;
    }
    out
}

/// Handles `events` in order: each holder's triggers whose event, limit and conditions match fire (8.1).
fn dispatch(res: &mut Resolver, w: &mut World, events: Vec<Event>, block: i32) {
    // the drops not passed yet (a seen HP above the HP now; a rise is simply seen)
    let pending: Vec<i32> = (0..w.units.len()).map(|c| res.triggers.hp_seen.get(c).map_or(0, |s| (s - w.units[c].hp).max(0))).collect();
    for ev in events {
        if !w.units[ev.holder].is_active() {
            continue;
        }
        for h in holders_of(res, w, ev.holder, block, true, true) {
            if h.row.on != ev.on || res.triggers.allows(h.unit, &h) == 0 || !matches(w, &ev, &h.row.when) {
                continue;
            }
            res.triggers.count(h.unit, &h.key);
            let mut fired = Fired::new(h.unit, ev.on, ev.other, &h.source);
            let effects = h.row.effects.clone();
            run_effects(res, w, &ev, &h, &effects, block, true, &mut fired);
            if !fired.lines.is_empty() || !fired.moves.is_empty() {
                res.triggers.fired.push(fired);
            }
        }
    }
    // depth 1: the HP the effects changed raises no `hp_below` later; a drop not yet passed (one at a block's start)
    // keeps its usual timing
    for c in 0..w.units.len().min(res.triggers.hp_seen.len()).min(pending.len()) {
        res.triggers.hp_seen[c] = w.units[c].hp + pending[c];
    }
}

/// `[Trigger] <holder>'s <source>: <what>` (a rider: `[Rider] ...`; a meter's own line, HOOKS1-M: `[Meter] ...`).
pub(crate) fn line(w: &World, h: &Holder, what: &str) -> String {
    let tag = if h.key.starts_with(RIDER_KEY) {
        "Rider"
    } else if h.key.starts_with(crate::meter::METER_KEY) {
        "Meter"
    } else {
        "Trigger"
    };
    format!("[{tag}] {}'s {}: {what}", w.units[h.unit].name, h.source)
}

/// 8.2: runs `effects` of the holder `h` for `ev` in list order (a `redirect` outside `ally_hit` does nothing). One log
/// line per effect that did something. A `meter` effect (and a meter rule's `meter_set`, UM5) moves `to`'s meter
/// (`meter::from_effect`, its band and rule lines follow); with `rules` false (a meter rule's own effects) it crosses no
/// further rule (depth 1).
#[allow(clippy::too_many_arguments)]
pub(crate) fn run_effects(res: &mut Resolver, w: &mut World, ev: &Event, h: &Holder, effects: &[Obj], block: i32, rules: bool, fired: &mut Fired) {
    for e in effects {
        let Some(to) = target_of(e, ev) else { continue };
        if !w.units[to].is_active() {
            continue;
        }
        let name = w.units[to].name.clone();
        let (mut dealt, mut rows) = (0, Vec::new());
        let what = match value::string(e, "type", "").as_str() {
            "ap" => {
                // party side only; never re-plans the round
                if !w.allies.contains(&to) {
                    continue;
                }
                let v = value::int(e, "value", 1).clamp(1, MAX_AP);
                let u = &mut w.units[to];
                let before = u.ap;
                u.ap = u.ap.max(u.max_ap.min(u.ap + v));
                if u.ap == before {
                    continue;
                }
                format!("{name} gains {} AP ({}/{})", u.ap - before, u.ap, u.max_ap)
            }
            "grant" => {
                let st = value::string(e, "status", "");
                if st.is_empty() {
                    continue;
                }
                let rounds = value::int(e, "rounds", 1);
                let rounds = if rounds < 0 { BATTLE_ROUNDS } else { rounds.max(1) };
                let src = w.units[h.unit].name.clone();
                status::grant_now(&mut w.units[to], &st, rounds, value::int(e, "magnitude", 0), &src);
                let until = value::string(e, "until", "") == "guard_end";
                if until {
                    res.triggers.until_guard.push((to, st.clone()));
                }
                format!("{name} gains {}{}", status::display_name(&st), if until { " while guarding" } else { "" })
            }
            "inflict" => {
                let st = value::string(e, "status", "");
                if st.is_empty() {
                    continue;
                }
                let src = w.units[h.unit].name.clone();
                let ok = status::try_inflict(&mut w.units[to], &st, value::int(e, "potency", 100), Some(&mut res.rng), value::int(e, "rounds", 1), value::int(e, "magnitude", 0), &src);
                format!("{} {}", if ok { "inflicts" } else { "resisted" }, status::display_name(&st)) + &format!(" on {name}")
            }
            "guard" => {
                let held = hold_guard(w, to, value::float(e, "mitigation", 0.5), value::int(e, "blocks", 0).clamp(0, 6), block);
                guard_line(w, to, &held, block)
            }
            "move" => {
                if !forced::on(w) || to == h.unit || !w.has(h.unit) {
                    continue;
                }
                let pull = value::string(e, "kind", "push") == "pull";
                let origin = w.pos(h.unit);
                let dir = forced::direction(w, h.unit, origin, w.pos(to), pull);
                let collide = Collide { attacker: None, mult: 0.0, ranged: false, offhand: false, flat: 0, stagger: 0, sure_break: false, blocks: 1 };
                let m = forced::shove_unit(res, w, to, Some(h.unit), dir, value::int(e, "tiles", 1).clamp(1, 3), pull.then_some(origin), false, &collide, block);
                // HOOKS1-GF: a trap sprung on the way cancels the moved unit's later Walks (the session logs the ground)
                crate::ground::settle(res, w);
                // each collision hit keeps its HP rows for its own log cell
                fired.moves.push(m);
                continue;
            }
            "crit" => {
                let v = value::float(e, "value", 0.0);
                w.units[to].trigger_crit += v;
                format!("{name} gains {v} crit chance for the battle")
            }
            "resist" => {
                let st = value::string(e, "status", "");
                let v = value::int(e, "value", 0);
                // S1.2-S3: folded by U4's combine rule for `resist` (summed, capped at 100), as a passive's resist is
                let fold = Target::Resist { status: st.clone() }.combine();
                let u = &mut w.units[to];
                let cur = u.status_resist.get(&st).copied().unwrap_or(0);
                u.status_resist.insert(st.clone(), fold.fold_i(cur, v));
                format!("{name} resists {} {v} for the battle", status::display_name(&st))
            }
            "stagger" => {
                let collide = Collide { attacker: None, mult: 0.0, ranged: false, offhand: false, flat: 0, stagger: value::int(e, "value", 0).max(0), sure_break: false, blocks: 1 };
                let Some(st) = forced::collision_stagger(res, w, to, &collide, block, false) else { continue };
                format!("{name} staggered {} (poise {} -> {}){}", st.taken, st.before, st.after, if st.broke.is_some() { ", broken" } else { "" })
            }
            "damage" => {
                let mark = w.hp_journal.as_ref().map_or(0, |j| j.len());
                let before = w.units[to].hp;
                w.damage(to, value::int(e, "value", 0).max(0), false);
                rows = w.hp_journal.as_mut().map(|j| j.split_off(mark.min(j.len()))).unwrap_or_default();
                let lost = before - w.units[to].hp;
                dealt = lost;
                format!("{lost} damage to {name}{}", if w.units[to].is_alive() { "" } else { ", defeated" })
            }
            "reveal" => {
                if !w.allies.contains(&h.unit) {
                    continue;
                }
                res.triggers.reveal_round = res.current_round + 1;
                "the foes' next round is revealed".to_string()
            }
            "end_guard" => {
                let mut n = 0;
                for a in res.scheduled.iter_mut().filter(|a| a.unit == to && !a.cancelled && a.start <= block && block <= a.end) {
                    if let CmdKind::Defend { guard_broken, .. } = &mut a.command.kind {
                        if !*guard_broken {
                            *guard_broken = true;
                            n += 1;
                        }
                    }
                }
                let had = w.units[to].held_guard.take().is_some_and(|g| block <= g.last_block);
                w.units[to].guard = 0.0;
                if n == 0 && !had {
                    continue;
                }
                format!("{name}'s guard ends")
            }
            "stop_moves" => {
                if forced::stop_walks(res, to) == 0 {
                    continue;
                }
                format!("{name}'s moves this round are cancelled")
            }
            // HOOKS1 UM2, UM5: the meter's own lines (rule unit_meters; nothing without it)
            "meter" | rpgkit_data::METER_SET => {
                crate::meter::from_effect(res, w, h, to, e, rules, block, fired);
                continue;
            }
            _ => continue,
        };
        let l = line(w, h, &what);
        res.log.push(l.clone());
        fired.push(l, dealt, rows);
    }
}

/// 8.2 `until` guard_end: removes the grants of the units that no longer guard (`all`: the round is over).
fn end_guard_grants(res: &mut Resolver, w: &mut World, block: i32, all: bool) {
    let list = std::mem::take(&mut res.triggers.until_guard);
    for (c, st) in list {
        if all || !guarding(w, &res.scheduled, c, block) {
            status::remove(&mut w.units[c], &st);
        } else {
            res.triggers.until_guard.push((c, st));
        }
    }
}

/// End of `prepare_block` (rule triggers): held guards raise the guards, then the `hp_below` crossings of the HP costs
/// `paid` as actions started (HC1: each payer's cost alone, from its HP before paying; every other drop since the last
/// pass keeps its usual timing, the next action's or the round end's pass), then the `guard_start` events of the Defends
/// that started now, then the grants of units that stopped guarding end.
pub fn on_block_start(res: &mut Resolver, w: &mut World, block: i32, paid: &[crate::resolver::HpPaid]) {
    if !on(w) {
        return;
    }
    res.triggers.watch_hp(w);
    for c in 0..w.units.len() {
        raise_held(w, c, block);
    }
    let mut events = Vec::new();
    for p in paid {
        events.push(Event { hp_from: share_of(w, p.unit, p.before), hp_to: share_of(w, p.unit, p.before - p.paid), ..Event::new("hp_below", p.unit, None) });
        // the cost is passed now: a later pass sees only the unit's other drops
        if let Some(seen) = res.triggers.hp_seen.get_mut(p.unit) {
            *seen -= p.paid;
        }
    }
    let started: Vec<UnitId> = res.scheduled.iter().filter(|a| !a.cancelled && a.start == block && a.command.is_defend() && w.units[a.unit].is_active()).map(|a| a.unit).collect();
    events.extend(started.into_iter().map(|c| Event::new("guard_start", c, None)));
    dispatch(res, w, events, block);
    end_guard_grants(res, w, block, false);
}

/// Round start, after the poise refill (rule triggers): the round's `limit` counts reset, a `reveal` shows this round's
/// plans (the battle's `plan_reveal` comes back the round after), then `round_start` fires for every unit on the field.
pub fn round_start(res: &mut Resolver, w: &mut World) {
    if !on(w) {
        return;
    }
    for v in res.triggers.counts.values_mut() {
        v.0 = 0;
    }
    if let Some(base) = res.triggers.reveal_base.take() {
        w.plan_reveal = base;
    }
    if res.triggers.reveal_round == res.current_round && res.triggers.reveal_round > 0 {
        res.triggers.reveal_base = Some(w.plan_reveal);
        w.plan_reveal = true;
        res.triggers.reveal_round = 0;
    }
    res.triggers.watch_hp(w);
    let events = (0..w.units.len()).map(|c| Event::new("round_start", c, None)).collect();
    dispatch(res, w, events, 1);
}

/// After the round-end ticks (rule triggers): the `hp_below` crossings they caused, then `round_end` for every unit on
/// the field; the held guards and the grants that end with a guard end with the round.
pub fn round_end(res: &mut Resolver, w: &mut World) {
    if !on(w) {
        return;
    }
    let block = w.total_blocks;
    let mut events = hp_crossings(res, w);
    events.extend((0..w.units.len()).map(|c| Event::new("round_end", c, None)));
    dispatch(res, w, events, block);
    end_guard_grants(res, w, block, true);
    for u in w.units.iter_mut() {
        u.held_guard = None;
    }
}

/// The HP seen before a block's actions start, for the units not seen yet (rule triggers).
pub fn before_block(res: &mut Resolver, w: &World) {
    if on(w) {
        res.triggers.watch_hp(w);
    }
}

/// The HP seen before the round-end ticks (so their crossings fire after them).
pub fn before_round_end(res: &mut Resolver, w: &World) {
    if on(w) {
        res.triggers.watch_hp(w);
    }
}
