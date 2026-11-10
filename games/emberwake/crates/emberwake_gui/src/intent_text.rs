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
    Some(format!("{}{kind}{target}", intent.label(&w.cat)))
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

/// HOOKS1 HC1 (HOOKS1-UI slice 3): a skill's HP cost in words ("" = it costs none).
pub fn hp_cost_text(cost: i32) -> String {
    if cost > 0 {
        format!("Costs {cost} HP at its start (never below 1 HP)")
    } else {
        String::new()
    }
}

/// HOOKS1 HC1 (HOOKS1-UI slice 3): a planner refusal in words: the HP-cost refusal (`command::hp_cost_problem`,
/// "HP 9 <= cost 10") reads "Not enough HP: 9, it costs 10"; any other reason as the planner gave it.
pub fn refusal_text(reason: &str) -> String {
    let parsed = reason.strip_prefix("HP ").and_then(|r| r.split_once(" <= cost ")).and_then(|(hp, cost)| Some((hp.parse::<i32>().ok()?, cost.parse::<i32>().ok()?)));
    match parsed {
        Some((hp, cost)) if emberwake_battle::command::hp_cost_problem(hp, cost) == reason => format!("Not enough HP: {hp}, it costs {cost}"),
        _ => reason.to_string(),
    }
}

/// PK1-S2c (PK1 4.3 item 5, Vent's two reasons): a skill-menu refusal in words: the HP cost's as `refusal_text`
/// ("Not enough HP: 9, it costs 10"), a meter gate's (`meter::gate_problem`, "Strain 30 < 50") "Needs Strain 50 or
/// more to start: 30"; any other reason as the planner gave it. PK1-S2c-2: two reasons that both apply (the kit joins
/// them with `REASONS_JOIN`) each in words, joined the same way.
pub fn menu_refusal_text(w: &World, reason: &str) -> String {
    let join = emberwake_battle::round_intent_plan::REASONS_JOIN;
    if reason.contains(join) {
        return reason.split(join).map(|r| one_refusal_text(w, r)).collect::<Vec<_>>().join(join);
    }
    one_refusal_text(w, reason)
}

/// One reason of `menu_refusal_text` in words.
fn one_refusal_text(w: &World, reason: &str) -> String {
    let hp = refusal_text(reason);
    if hp != reason {
        return hp;
    }
    let gate = reason.rsplit_once(" < ").and_then(|(left, min)| {
        let (name, have) = left.rsplit_once(' ')?;
        Some((name, have.parse::<i32>().ok()?, min.parse::<i32>().ok()?))
    });
    if let Some((name, have, min)) = gate {
        if let Some(def) = w.cat.unit_meters.values().find(|m| m.name == name) {
            let g = emberwake_data::MeterGate { meter: def.id.clone(), min };
            if emberwake_battle::meter::gate_problem(w, Some(&g), |_| Some(have)) == reason {
                return format!("Needs {name} {min} or more to start: {have}");
            }
        }
    }
    reason.to_string()
}

/// INPUT1 MR4: a skill's range in words for its menu row tip and hover card: "Range 2: dash to a flank, then hit" or
/// "Range 3: dash behind, then hit" for a skill with a position (its range N), else its band ("Range 2-4", "Range 1").
pub fn range_line(w: &World, c: usize, s: &emberwake_data::SkillDef) -> String {
    use emberwake_battle::round_intent;
    match (round_intent::position_of(w, s), round_intent::dash_range(w, c, s)) {
        (Some(p), Some(n)) => format!("Range {n}: dash {}, then hit", if p == emberwake_data::Position::Behind { "behind" } else { "to a flank" }),
        _ => {
            let (lo, hi) = emberwake_battle::skill::reach_by(s, &w.units[c], &w.ruleset);
            if lo == hi || lo <= 0 { format!("Range {hi}") } else { format!("Range {lo}-{hi}") }
        }
    }
}

/// INTENT2-U (section 6 "Menu"): a refused item cell's cost for its hover card (a live one shows the kit's trial cost),
/// "<AP> AP, <N> blocks": `command::item_ap`, and the item's `use_blocks` with the Assault row's longer wind-up for an
/// area item (`command::item_blocks`).
pub fn item_cost_line(w: &World, c: usize, item: &emberwake_data::ItemDef, intent: Option<emberwake_battle::round_intent::RoundIntent>) -> String {
    use emberwake_battle::command;
    let assault = intent == Some(emberwake_battle::round_intent::RoundIntent::Assault) && command::item_area_on(w, item);
    let blocks = command::item_blocks(w, item, assault, w.total_blocks);
    format!("{} AP, {blocks} block{}", command::item_ap(w, c), if blocks == 1 { "" } else { "s" })
}

/// INTENT2-U (section 6, IR17): what an area item does in its `intent` row under the rule `field_intent`, as a skill's
/// `variant_line`: the Field row (thrown at a tile fixed at planning, full power, its derived reach bonus, the units its
/// `affects` takes in the marks) or the Assault row (locked on the enemy, `splash_mult` off it, the longer wind-up).
/// None for any other row or item, and without the rule.
pub fn item_variant_line(w: &World, item: &emberwake_data::ItemDef, intent: emberwake_battle::round_intent::RoundIntent) -> Option<String> {
    use emberwake_battle::round_intent::{self, RoundIntent};
    use emberwake_data::Affects;
    if !w.ruleset.has(emberwake_rules::ruleset::FIELD_INTENT) || !emberwake_battle::command::item_area_on(w, item) || !round_intent::item_allows(w, item, intent) {
        return None;
    }
    let who = |default: Affects| match item.affects.unwrap_or(default) {
        Affects::All => "everyone",
        Affects::Enemies => "foes",
        Affects::Allies => "allies",
    };
    match intent {
        RoundIntent::Field => {
            let reach = if emberwake_battle::command::item_field_reach(item) > item.battle_reach() { ", reach +1" } else { "" };
            Some(format!("Field: thrown at a tile, fixed when planned, full power{reach}; hits {} in the marks", who(Affects::All)))
        }
        RoundIntent::Assault => {
            let n = w.ruleset.numbers.assault_shape_blocks;
            Some(format!("Assault: locked on the enemy, x{} off it, +{n} wind-up block{}; {} only", w.ruleset.numbers.splash_mult, if n == 1 { "" } else { "s" }, who(Affects::Enemies)))
        }
        _ => None,
    }
}

/// A4.2-K1b (INTENT2 IR17, IR21; the skill variant previews): what skill `s` does in its `intent` row under `rules`:
/// the Field row (aimed at a tile fixed at planning, its derived Field bonus, every unit in the marks) or the shaped
/// Assault row (locked on the enemy, `splash_mult` off it, the longer wind-up, foes only). None for any other row or
/// skill, and without the rule `field_intent` (classic and round_intent-only show nothing new).
pub fn variant_line(rules: &emberwake_rules::ruleset::Ruleset, s: &emberwake_data::SkillDef, intent: emberwake_battle::round_intent::RoundIntent) -> Option<String> {
    use emberwake_battle::round_intent::RoundIntent;
    use emberwake_rules::intent::FieldBonus;
    if !rules.has(emberwake_rules::ruleset::FIELD_INTENT) || !emberwake_battle::skill::is_area(s) {
        return None;
    }
    let rows = emberwake_rules::intent::allowed(rules, &emberwake_rules::intent::IntentInput::of_def(s), &s.round_intent);
    match intent {
        RoundIntent::Field if rows.contains(&"field") => {
            let bonus = match (emberwake_battle::skill::field_bonus(s), emberwake_battle::skill::area(s)) {
                (Some(FieldBonus::Area), emberwake_battle::skill::Area::Arc) => format!(", {} across", emberwake_battle::skill::FIELD_ARC_WIDTH),
                (Some(FieldBonus::Area), _) => format!(", {} long", emberwake_battle::skill::FIELD_LINE_LENGTH),
                (Some(FieldBonus::Reach), _) => ", reach +1".to_string(),
                (Some(FieldBonus::Lingering), _) => ", hazard +1 round".to_string(),
                (Some(FieldBonus::Power), _) => format!(", power x{}", emberwake_battle::skill::FIELD_POWER_MULT),
                _ => String::new(),
            };
            Some(format!("Field: aimed at a tile, fixed when planned{bonus}; hits everyone in the marks"))
        }
        RoundIntent::Assault if rows.contains(&"assault") => {
            let n = rules.numbers.assault_shape_blocks;
            Some(format!("Assault: locked on the enemy, x{} off it, +{n} wind-up block{}; foes only", rules.numbers.splash_mult, if n == 1 { "" } else { "s" }))
        }
        _ => None,
    }
}
