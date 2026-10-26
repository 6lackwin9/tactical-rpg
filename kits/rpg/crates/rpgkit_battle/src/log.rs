//! The action log (ActionLogEntry, scripts/core/events/action_log_entry.gd; CombatLogBuilder,
//! scripts/combat/session/combat_log_builder.gd): one structured entry per resolved action, riposte, chase or dash and
//! per encounter event (phase, morale break, wave, start buff). The battle metrics read it; screens show it.

use crate::command::{reach_band, CmdKind, Reaction};
use crate::grid::{V2, NO_CELL};
use crate::resolver::ScheduledAction;
use crate::skill;
use crate::world::World;
use rpgkit_rules::status;
use serde_json::{json, Value};

/// One log cell (ActionLogEntry). Units are named as the log shows them; an event's actor may be no unit at all
/// ("Reinforcements", the encounter's name).
#[derive(Debug, Clone, PartialEq)]
pub struct Entry {
    pub round: i32,
    pub block: i32,
    /// ATTACK, DEFEND, BUFF, MOVE, TARGET or EVENT.
    pub category: &'static str,
    pub actor: String,
    pub actor_is_ally: bool,
    pub target: String,
    pub target_is_ally: bool,
    pub skill: String,
    pub subtype: String,
    pub value: i32,
    /// ATTACK: the damage actually dealt.
    pub damage: i32,
    pub outcome: String,
    pub detail: String,
    pub status: String,
    /// Where a move ended (NO_CELL when it is not a move).
    pub coord_to: V2,
}

impl Entry {
    /// ActionLogEntry.new(round, block): an ATTACK cell with a HIT outcome until the builder fills it.
    pub fn new(round: i32, block: i32) -> Self {
        Self {
            round,
            block,
            category: "ATTACK",
            actor: String::new(),
            actor_is_ally: true,
            target: String::new(),
            target_is_ally: false,
            skill: String::new(),
            subtype: String::new(),
            value: 0,
            damage: 0,
            outcome: "HIT".into(),
            detail: String::new(),
            status: String::new(),
            coord_to: NO_CELL,
        }
    }

    /// "R1-B01".
    pub fn ref_tag(&self) -> String {
        format!("R{}-B{:02}", self.round, self.block)
    }

    /// CombatEncounter.make_event_entry: an EVENT cell for narration and triggers.
    #[allow(clippy::too_many_arguments)]
    pub fn event(round: i32, block: i32, actor: &str, actor_is_ally: bool, skill: &str, subtype: &str, outcome: &str, detail: &str, status: &str) -> Self {
        Self {
            category: "EVENT",
            actor: actor.into(),
            actor_is_ally,
            skill: skill.into(),
            subtype: subtype.into(),
            outcome: outcome.into(),
            detail: detail.into(),
            status: status.into(),
            ..Self::new(round, block)
        }
    }

    /// The parity traces' row: [round, block, category, actor, actor is ally, target, target is ally, skill, subtype,
    /// value, damage, outcome, detail, status].
    pub fn trace(&self) -> Value {
        json!([self.round, self.block, self.category, self.actor, self.actor_is_ally, self.target, self.target_is_ally, self.skill, self.subtype, self.value, self.damage, self.outcome, self.detail, self.status])
    }
}

fn trim<'a>(s: &'a str, prefix: &str) -> &'a str {
    s.strip_prefix(prefix).unwrap_or(s)
}

fn plural(n: i32) -> &'static str {
    if n == 1 {
        ""
    } else {
        "s"
    }
}

/// Where `c` stands now (NO_CELL off the grid).
fn cell_of(w: &World, c: usize) -> V2 {
    w.grid.pos(c).unwrap_or(NO_CELL)
}

/// CombatLogBuilder.from_action: the cell of a resolved action.
pub fn from_action(w: &World, a: &ScheduledAction, round: i32, block: i32) -> Entry {
    let mut e = Entry::new(round, block);
    let me = &w.units[a.unit];
    e.actor = me.name.clone();
    e.actor_is_ally = w.allies.contains(&a.unit);
    let cmd = &a.command;
    let set_target = |e: &mut Entry, t: usize| {
        e.target = w.units[t].name.clone();
        e.target_is_ally = w.allies.contains(&t);
    };
    match &cmd.kind {
        CmdKind::Attack { target, skill_mult, min_range, max_range, last_damage, last_crit, .. } => {
            let band = reach_band(w, a.unit, *min_range, *max_range);
            e.skill = cmd.name.clone();
            e.subtype = format!("Reach {}-{}", band.0, band.1);
            e.value = *last_damage;
            e.damage = *last_damage;
            if *last_damage == 0 && cmd.log.starts_with("[Attack Out of Range]") {
                e.outcome = "OUT OF RANGE".into();
            } else if *last_damage == 0 && cmd.log.starts_with("[Attack Miss]") {
                e.outcome = "MISS".into();
            }
            match target {
                Some(t) => {
                    set_target(&mut e, *t);
                    let tu = &w.units[*t];
                    if !tu.is_alive() {
                        e.outcome = "DEFEATED".into();
                    } else if *last_crit && *last_damage > 0 {
                        e.outcome = "CRIT".into();
                    } else if tu.guard > 0.0 {
                        e.outcome = "MITIGATED".into();
                    }
                    e.status = format!("Target HP: {}/{}", tu.hp, tu.max_hp);
                }
                None => e.target = "Target".into(),
            }
            e.detail = format!("Dealt {} | Skill x{:.2} | Range {}", last_damage, skill_mult, me.attack_range);
            // RULESET1 A2, P6 (timeline only): the armour pierced and the poise the hit took
            if cmd.impact.armor_piercing && w.ruleset.has(rpgkit_rules::ruleset::PIERCE) && *last_damage > 0 {
                e.detail += " | Armour pierced";
            }
            if let Some(st) = cmd.staggers.first() {
                e.detail += &format!(" | Stagger {} (poise {} -> {})", st.taken, st.before, st.after);
            }
        }
        CmdKind::Walk { path, approach, moved_tiles, cut_short, blocked_by, .. } => {
            e.category = "MOVE";
            e.skill = if approach.is_some() { "Approach" } else { "Move" }.into();
            e.subtype = format!("{moved_tiles} tile{}", plural(*moved_tiles));
            e.value = *moved_tiles;
            e.coord_to = cell_of(w, a.unit);
            if me.is_withdrawn {
                e.outcome = "WITHDRAWN".into();
                e.status = "Safely exited battle grid".into();
            } else if *cut_short {
                e.outcome = "CUT SHORT".into();
                e.status = format!("Stopped at {} (blocked by {blocked_by})", e.coord_to);
            } else {
                e.outcome = "MOVED".into();
                e.status = format!("Position: {}", e.coord_to);
            }
            e.detail = format!("Walked {moved_tiles} of {} planned tiles", path.len());
        }
        CmdKind::Disengage { landed, moved_tiles, .. } => {
            e.category = "MOVE";
            e.skill = cmd.name.clone();
            e.subtype = "Disengage".into();
            e.value = *moved_tiles;
            e.coord_to = cell_of(w, a.unit);
            e.outcome = if *landed { "JUMPED" } else { "FAILED" }.into();
            e.status = format!("Position: {}", e.coord_to);
            e.detail = trim(&cmd.log, "[Disengage] ").into();
        }
        CmdKind::Defend { mitigation, .. } => {
            e.category = "DEFEND";
            e.skill = cmd.name.clone();
            e.subtype = format!("Guard -{}%", (mitigation * 100.0).round() as i32);
            e.value = (mitigation * 100.0) as i32;
            e.outcome = format!("{}% MITIGATION", e.value);
            e.detail = "Damage mitigation posture active".into();
            e.status = format!("Duration: {} Blocks", cmd.block_cost);
        }
        CmdKind::Target { target } => {
            e.category = "TARGET";
            e.skill = "Switch Target".into();
            e.subtype = "Target Lock".into();
            match target {
                Some(t) => set_target(&mut e, *t),
                None => e.target = "None".into(),
            }
            e.outcome = "LOCKED".into();
            e.detail = "Assigned tactical target focus".into();
            e.status = "Target Locked".into();
        }
        CmdKind::Focus { boost, rounds } => {
            e.category = "BUFF";
            e.skill = cmd.name.clone();
            e.subtype = "Attack Multiplier".into();
            e.value = ((boost - 1.0) * 100.0) as i32;
            e.outcome = format!("+{}% ATK", e.value);
            e.detail = format!("+{}% Attack Power for {rounds} Round(s)", e.value);
            e.status = "Empowered".into();
        }
        // HOOKS1 GH3: a placement aims like an area attack and hits nobody (its hazard has its own cell)
        CmdKind::Area { cells, .. } if cmd.log.starts_with("[Place]") => {
            e.skill = cmd.name.clone();
            e.subtype = PLACEMENT.into();
            e.outcome = "PLACED".into();
            e.target = format!("{} tiles", cells.len());
            e.detail = trim(&cmd.log, "[Place] ").into();
        }
        CmdKind::Area { cells, last_damage, last_hits, .. } => {
            e.skill = cmd.name.clone();
            e.subtype = "Area Attack".into();
            e.value = *last_damage;
            e.damage = *last_damage;
            e.outcome = if *last_hits > 0 { format!("{last_hits} HIT{}", if *last_hits == 1 { "" } else { "S" }) } else { "MISS".into() };
            e.target = format!("{} tiles", cells.len());
            e.detail = trim(&cmd.log, "[Attack] ").into();
        }
        CmdKind::Item { target, used, last_heal, last_damage, last_boost, .. } => {
            e.category = if *last_damage > 0 { "ATTACK" } else { "BUFF" };
            e.skill = cmd.name.clone();
            e.subtype = "Item".into();
            e.value = if *last_damage > 0 { *last_damage } else { *last_heal };
            e.damage = *last_damage;
            set_target(&mut e, *target);
            let tu = &w.units[*target];
            e.status = format!("HP {}/{}", tu.hp, tu.max_hp);
            e.outcome = if !used {
                "NOT USED".into()
            } else if *last_damage > 0 {
                if tu.is_alive() {
                    format!("{last_damage} DMG")
                } else {
                    "DEFEATED".into()
                }
            } else if *last_boost > 0.0 {
                format!("+{}% ATK", ((last_boost - 1.0) * 100.0).round() as i32)
            } else {
                format!("+{last_heal} HP")
            };
            e.detail = trim(&cmd.log, "[Item] ").into();
        }
        CmdKind::Support { target, skill: sk, last_heal, last_status, last_cleansed } => {
            e.category = "BUFF";
            e.skill = cmd.name.clone();
            e.subtype = "Support".into();
            e.value = *last_heal;
            set_target(&mut e, *target);
            let tu = &w.units[*target];
            e.status = format!("HP {}/{}", tu.hp, tu.max_hp);
            e.outcome = if cmd.failed {
                "FAILED".into()
            } else if skill::heals(&w.cat.skills[*sk]) {
                format!("+{last_heal} HP")
            } else if !last_status.is_empty() {
                status::display_name(last_status).to_uppercase()
            } else if !last_cleansed.is_empty() {
                "CLEANSED".into()
            } else {
                "EXECUTED".into()
            };
            e.detail = if !last_status.is_empty() { format!("{} next round", status::display_name(last_status)) } else { trim(&cmd.log, "[Support] ").into() };
        }
        CmdKind::StatusBlock { status: st } => {
            e.category = "EVENT";
            e.skill = status::display_name(st);
            e.subtype = "Block lost".into();
            e.outcome = "LOST".into();
            e.detail = format!("{} can't act this block", me.name);
        }
        CmdKind::Rest { recover_ap } => {
            e.category = "BUFF";
            e.skill = "Rest".into();
            e.subtype = "Recover AP".into();
            e.value = *recover_ap;
            e.outcome = format!("+{recover_ap} AP");
            e.detail = "Catches their breath and settles into Tactical".into();
            e.status = "Recovering".into();
        }
    }
    e
}

/// RULESET1 P9 (engine-only): the EVENT cell of a break: the broken unit, the action it lost, the cause.
pub fn break_entry(w: &World, t: usize, b: &crate::poise::Break, round: i32, block: i32) -> Entry {
    let line = crate::poise::break_line(w, t, b);
    Entry::event(round, block, &w.units[t].name, w.allies.contains(&t), &b.skill, "Break", "BREAK", &line, b.cause.id())
}

/// HOOKS1 FM4: the subtype of a collision's ATTACK cell (the metrics credit it to the driver, not as an attack received).
pub const COLLISION: &str = "Collision";

/// HOOKS1 FM9 (engine-only): the EVENT cell of a forced move: the driver (else the moved unit), the moved unit, the
/// tiles and where it ended; a unit that held firm gets a HELD cell.
pub fn forced_entry(w: &World, m: &crate::forced::Moved, skill: &str, round: i32, block: i32) -> Entry {
    let (subtype, outcome) = if m.held {
        ("Hold", "HELD")
    } else if m.pull {
        ("Pull", "PULLED")
    } else {
        ("Push", "PUSHED")
    };
    let actor = m.by.unwrap_or(m.unit);
    let mut e = Entry::event(round, block, &w.units[actor].name, w.allies.contains(&actor), skill, subtype, outcome, &m.line, &format!("Position: {}", m.to));
    e.target = w.units[m.unit].name.clone();
    e.target_is_ally = w.allies.contains(&m.unit);
    e.value = m.tiles;
    e.coord_to = m.to;
    e
}

/// HOOKS1 FM4 (engine-only): the ATTACK cell of one collision hit: the driver hits `h.unit` for the HP it lost (an
/// arena push has no driver: the actor is "" and the metrics leave it unexplained).
pub fn collision_entry(w: &World, m: &crate::forced::Moved, h: &crate::forced::CollisionHit, skill: &str, round: i32, block: i32) -> Entry {
    let tu = &w.units[h.unit];
    Entry {
        actor: m.by.map_or_else(String::new, |b| w.units[b].name.clone()),
        actor_is_ally: m.by.is_some_and(|b| w.allies.contains(&b)),
        target: tu.name.clone(),
        target_is_ally: w.allies.contains(&h.unit),
        skill: skill.into(),
        subtype: COLLISION.into(),
        value: h.damage,
        damage: h.damage,
        outcome: if tu.is_alive() { "COLLISION" } else { "DEFEATED" }.into(),
        detail: trim(&h.line, "[Collision] ").into(),
        status: format!("Target HP: {}/{}", tu.hp, tu.max_hp),
        ..Entry::new(round, block)
    }
}

/// HOOKS1 GH3: the subtype of a placement action's ATTACK cell (no damage, no target unit).
pub const PLACEMENT: &str = "Placement";
/// HOOKS1 GH5: the subtype of the ground hazards' cells; the ATTACK one (HP lost on entering) is credited to the placer
/// as a collision's is.
pub const HAZARD: &str = "Hazard";

/// HOOKS1 GH2-GH7 (engine-only): the cell of a ground event: a placement (EVENT, PLACED: the placer and the tiles); a
/// hazard that cost HP as the unit entered (ATTACK: the placer hits the unit for the HP it lost); any other hit (EVENT:
/// TRAPPED, DEFEATED or HAZARD; a round-end hit's HP is round-end damage, its value only); a fade (EVENT, FADED: the
/// hazard as the actor).
pub fn ground_entry(w: &World, ev: &crate::ground::Event, round: i32, block: i32) -> Entry {
    use crate::ground::Event;
    match ev {
        Event::Placed(p) => {
            let rounds = format!("{} round{}", p.rounds, plural(p.rounds));
            let mut e = Entry::event(round, block, &w.units[p.by].name, w.allies.contains(&p.by), &p.hazard, HAZARD, "PLACED", trim(&p.line, "[Hazard] "), &rounds);
            e.target = format!("{} tiles", p.cells.len());
            e.value = p.cells.len() as i32;
            e
        }
        Event::Hit(h) => {
            let tu = &w.units[h.unit];
            let outcome = if !tu.is_alive() {
                "DEFEATED"
            } else if h.trap {
                "TRAPPED"
            } else {
                "HAZARD"
            };
            let mut e = Entry {
                actor: w.units[h.by].name.clone(),
                actor_is_ally: w.allies.contains(&h.by),
                target: tu.name.clone(),
                target_is_ally: w.allies.contains(&h.unit),
                skill: h.hazard.clone(),
                subtype: HAZARD.into(),
                value: h.damage,
                outcome: outcome.into(),
                detail: trim(&h.line, "[Hazard] ").into(),
                status: format!("Target HP: {}/{}", tu.hp, tu.max_hp),
                ..Entry::new(round, block)
            };
            if h.damage > 0 && !h.round_end {
                e.damage = h.damage;
            } else {
                e.category = "EVENT";
            }
            e
        }
        Event::Faded(f) => Entry::event(round, block, &f.hazard, w.allies.contains(&f.by), &f.hazard, HAZARD, "FADED", trim(&f.line, "[Hazard] "), &format!("{} tiles", f.cells)),
    }
}

/// CombatLogBuilder.riposte_entry: the defender countered the attack that just resolved.
pub fn riposte_entry(w: &World, r: &Reaction, round: i32, block: i32) -> Entry {
    let att = &w.units[r.attacker];
    let outcome = if r.defeated {
        "DEFEATED"
    } else if r.crit {
        "CRIT"
    } else {
        "COUNTER"
    };
    Entry {
        actor: w.units[r.defender].name.clone(),
        actor_is_ally: w.allies.contains(&r.defender),
        target: att.name.clone(),
        target_is_ally: w.allies.contains(&r.attacker),
        skill: "Riposte".into(),
        subtype: "Counter".into(),
        value: r.damage,
        damage: r.damage,
        outcome: outcome.into(),
        status: format!("Target HP: {}/{}", att.hp, att.max_hp),
        detail: format!("Countered for {}", r.damage),
        ..Entry::new(round, block)
    }
}

/// CombatLogBuilder.chase_entry / dash_entry: the attacker closed the gap as its attack started (`message` is the
/// command's start note: "[Chase] ..." or "[Dash] ..."). None for any other note.
pub fn move_note_entry(w: &World, a: &ScheduledAction, round: i32, block: i32, message: &str) -> Option<Entry> {
    let (skill, subtype, outcome, prefix) = if message.starts_with("[Chase]") {
        // RULESET1 T11: a unit that pays no AP chases for free
        ("Chase", if w.pays_ap(a.unit) { "Closes the gap (1 AP)" } else { "Closes the gap (no AP)" }, "CHASED", "[Chase] ")
    } else if message.starts_with("[Dash]") {
        ("Dash", "Gap-close (no extra AP)", "DASHED", "[Dash] ")
    } else {
        return None;
    };
    let mut e = Entry::new(round, block);
    e.actor = w.units[a.unit].name.clone();
    e.actor_is_ally = w.allies.contains(&a.unit);
    e.category = "MOVE";
    e.skill = skill.into();
    e.subtype = subtype.into();
    e.outcome = outcome.into();
    e.coord_to = cell_of(w, a.unit);
    if let CmdKind::Attack { target: Some(t), .. } = &a.command.kind {
        e.target = w.units[*t].name.clone();
        e.target_is_ally = w.allies.contains(t);
    }
    e.status = format!("Position: {}", e.coord_to);
    e.detail = trim(message, prefix).into();
    Some(e)
}
