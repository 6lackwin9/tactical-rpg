//! Words for an enemy intent record (IntentText, scripts/ui/combat/intent_text.gd; INTENT1-U): the hostile card's
//! "Next:" line, the hover card's Intent and plan lines and the forecast line. Pure text, display only. RULESET1-U
//! slice 4 (UI5, V2): the round intent's head ("Assault: Wolf"), the card line that leads with it and the reveal head.

use emberwake_battle::grid::NO_CELL;
use emberwake_battle::intent::{Intent, Line, ARRIVING, IDLE};
use emberwake_battle::round_intent::{ManeuverKind, RoundTarget};
use emberwake_battle::world::World;

/// "B1" or "B1-2" for a (start, end) block pair.
pub fn block_text(blocks: (i32, i32)) -> String {
    if blocks.0 == blocks.1 {
        format!("B{}", blocks.0)
    } else {
        format!("B{}-{}", blocks.0, blocks.1)
    }
}

/// What one action does, without its blocks: "Shield Bash on Bryn", "Move toward Sella", "Area Attack at (4, 3)".
pub fn core(w: &World, l: &Line) -> String {
    match l.kind {
        k if k == ARRIVING => "Arriving".into(),
        k if k == IDLE => "Holding".into(),
        k => match l.target {
            Some(t) => format!("{} {} {}", l.skill, if k == "move" || k == "flee" { "toward" } else { "on" }, w.units[t].name),
            None if l.cell != NO_CELL => format!("{} {} ({}, {})", l.skill, if k == "area" { "at" } else { "to" }, l.cell.x, l.cell.y),
            None => l.skill.clone(),
        },
    }
}

/// "Shield Bash on Bryn (B1)"; arriving and holding have no blocks.
pub fn first_line(w: &World, l: &Line) -> String {
    if l.kind == ARRIVING || l.kind == IDLE {
        return core(w, l);
    }
    format!("{} ({})", core(w, l), block_text(l.blocks))
}

/// The hostile card's line: "Next: Shield Bash on Bryn (B1)", or "Holding" / "Arriving" alone.
pub fn card_line(w: &World, l: &Line) -> String {
    if l.kind == ARRIVING || l.kind == IDLE {
        first_line(w, l)
    } else {
        format!("Next: {}", first_line(w, l))
    }
}

/// "B3 Shield Bash on Bryn": one line of a revealed plan.
pub fn plan_line(w: &World, l: &Line) -> String {
    format!("{} {}", block_text(l.blocks), core(w, l))
}

/// "Read by Bryn (Warden's Eye)": who reads the plan and through which passive (`passive_name` names a passive id).
pub fn read_by(w: &World, i: &Intent, passive_name: impl Fn(&str) -> String) -> String {
    let names: Vec<String> = i.readers.iter().map(|r| w.units[*r].name.clone()).collect();
    let source = i.readers.first().map(|r| w.units[*r].foresight_source.clone()).unwrap_or_default();
    let passive = if source.is_empty() { String::new() } else { format!(" ({})", passive_name(&source)) };
    format!("Read by {}{passive}", names.join(", "))
}

/// The forecast line when `hero` is the target of the record's first action: "Targets you in B1" (None otherwise).
pub fn forecast(i: &Intent, hero: usize) -> Option<String> {
    (i.line.target == Some(hero)).then(|| format!("Targets you in {}", block_text(i.line.blocks)))
}

/// RULESET1_INTENT UI5: a record's round intent, Maneuver kind and round target: "Assault: Wolf", "Maneuver (Engage):
/// Wolf", "Guard: self", "Guard: tile 3, 4"; None without a round intent (classic, or none picked yet).
pub fn round_head(w: &World, i: &Intent) -> Option<String> {
    let intent = i.round_intent?;
    let kind = match i.maneuver {
        Some(ManeuverKind::Engage) => " (Engage)",
        Some(ManeuverKind::Disengage) => " (Disengage)",
        Some(ManeuverKind::Utility) => " (Utility)",
        None => "",
    };
    let target = match i.round_target {
        Some(RoundTarget::Myself) => ": self".to_string(),
        Some(RoundTarget::Unit(t)) => format!(": {}", w.units[t].name),
        Some(RoundTarget::Cell(p)) => format!(": tile {}, {}", p.x, p.y),
        None => String::new(),
    };
    Some(format!("{}{kind}{target}", intent.name()))
}

/// UI5: the hostile card's line under the round intent: the intent first, then the first action ("Assault: Wolf;
/// Shield Bash B1"); without a round intent the classic `card_line`.
pub fn round_card_line(w: &World, i: &Intent) -> String {
    match round_head(w, i) {
        None => card_line(w, &i.line),
        Some(head) if i.line.kind == ARRIVING || i.line.kind == IDLE => format!("{head}; {}", core(w, &i.line)),
        Some(head) => format!("{head}; {} {}", i.line.skill, block_text(i.line.blocks)),
    }
}

/// V1, V2: how the record's plan shows: who reads it (foresight), the enrage, or the battle's plan reveal; None when it
/// is not revealed.
pub fn reveal_head(w: &World, i: &Intent, passive_name: impl Fn(&str) -> String) -> Option<String> {
    if !i.revealed {
        return None;
    }
    Some(if !i.readers.is_empty() {
        read_by(w, i, passive_name)
    } else if i.enraged {
        "Enraged: its whole plan shows".into()
    } else {
        "Plan revealed".into()
    })
}
