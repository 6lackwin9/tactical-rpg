//! OBJUI1 (`OBJ1-U`, `MORALE1-U`): what the battle screen shows of the objective and of morale. Morale is words with a
//! flavour line, never its value or a flee ratio (MORALE1 rule 13 and section 3); the objective marks name the slay
//! target and the guests (and which guests must survive); the results read one objective line and, after a win by a
//! break, the break lines.

use crate::encounter::Objectives;
use crate::session::Session;
use crate::world::UnitId;

/// MORALE1 section 3: the card's line on how kills and crits move morale.
pub const INSPECT_LINE: &str = "Kills and critical hits shake their group.";
/// MORALE1 section 3: the leader's card line.
pub const LEADER_LINE: &str = "Leader: its fall shakes every group";
/// OBJ1 section 3: the results line of a fight not won.
pub const FAILED_LINE: &str = "The objective failed.";

/// One unit's morale as shown: the state word, its flavour line and, with more than one group, the group letter.
#[derive(Debug, Clone, PartialEq)]
pub struct MoraleMark {
    pub state: &'static str,
    pub label: &'static str,
    pub line: &'static str,
    pub group: Option<char>,
}

impl MoraleMark {
    /// "Wavering", or "Wavering, group B" when the battle has several groups.
    pub fn title(&self) -> String {
        match self.group {
            Some(g) => format!("{}, group {g}", self.label),
            None => self.label.to_string(),
        }
    }
}

/// The word of a morale state (`Session::morale_state_of`); "" for none.
pub fn state_label(state: &str) -> &'static str {
    match state {
        "steady" => "Steady",
        "shaken" => "Shaken",
        "wavering" => "Wavering",
        "broken" => "Broken",
        "fearless" => "Fearless",
        _ => "",
    }
}

/// The flavour line of a morale state (MORALE1 section 3); "" for none.
pub fn state_line(state: &str) -> &'static str {
    match state {
        "steady" => "holding the line",
        "shaken" => "glancing at the exit; runs sooner when hurt",
        "wavering" => "eyes on the exit; runs once bloodied",
        "broken" => "throws down its arms",
        "fearless" => "will not break",
        _ => "",
    }
}

/// Enemy `c`'s morale mark while it stands; None with morale off, for a unit in no group or one off the field.
pub fn morale_mark(s: &Session, c: UnitId) -> Option<MoraleMark> {
    if !s.w.enemies.contains(&c) || !s.w.units[c].is_active() {
        return None;
    }
    let state = s.morale_state_of(c);
    if state.is_empty() {
        return None;
    }
    let gi = s.morale_group_of(c);
    let group = (s.morale.groups.len() > 1 && gi >= 0).then(|| (b'A' + (gi as u8 % 26)) as char);
    Some(MoraleMark { state, label: state_label(state), line: state_line(state), group })
}

/// Whether `c` is the morale leader (and still stands).
pub fn is_leader(s: &Session, c: UnitId) -> bool {
    s.morale_leader() == Some(c) && s.w.units[c].is_active()
}

/// MORALE1 section 3: the planning header's morale line ("" with morale off).
pub fn header_line(s: &Session) -> String {
    if !s.morale.on || s.morale.groups.is_empty() {
        return String::new();
    }
    if s.morale.brk != 50 {
        return "They give up as their losses mount.".into();
    }
    let leader = s.morale_leader().map(|c| s.w.units[c].name.clone());
    match (s.morale.groups.len() > 1, leader) {
        (false, Some(l)) => format!("They give up when {l} falls or half of them are down; shaken ones run sooner."),
        (false, None) => "They give up when half of them are down; shaken ones run sooner.".into(),
        (true, Some(l)) => format!("Each group gives up when half of it is down; {l}'s fall shakes them all."),
        (true, None) => "Each group gives up when half of it is down.".into(),
    }
}

/// MORALE1 rule 12: the lines of the groups that broke, in group order.
pub fn break_lines(s: &Session) -> Vec<String> {
    s.morale.groups.iter().filter(|g| g.broken && !g.line.is_empty()).map(|g| g.line.clone()).collect()
}

/// OBJ1 section 3: a unit's objective marks.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ObjMarks {
    /// The slay target (the crown).
    pub slay: bool,
    /// A guest (the shield).
    pub guest: bool,
    /// A guest the party must keep alive ("Must survive").
    pub must_survive: bool,
}

pub fn marks(s: &Session, c: UnitId) -> ObjMarks {
    let guest = s.guests.contains(&c);
    ObjMarks { slay: s.slay_target() == Some(c), guest, must_survive: guest && s.protected.contains(&c) }
}

/// OBJ1 section 3: the results' objective line, and after a win by a break the break lines under it. `outcome` is the
/// battle's verdict (won, lost, fled, ...).
pub fn result_lines(s: &Session, outcome: &str) -> Vec<String> {
    let w = &s.w;
    let name_of = |r: &str| s.units_of_ref(r).first().map(|&c| w.units[c].name.clone()).unwrap_or_else(|| r.to_string());
    if outcome != "won" {
        return vec![FAILED_LINE.into()];
    }
    let mut out = vec![format!("Objective met: {}", Objectives::goal_of(&s.def.raw, name_of))];
    out.extend(break_lines(s));
    out
}
