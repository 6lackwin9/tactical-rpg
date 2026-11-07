//! Campaign HUD view (CAMPAIGN1 W4, P1, M8; kit hook, engine-only, no Godot original): what the map HUD shows, read
//! from the purse and the campaign state: the currency and the upkeep with the purse's names, icons and tips (today's
//! "Gold" and "Supplies" outside a campaign world), then the visible meters by `hud.order`, each with its value and
//! unit, its band's tone and a tip naming the meter (or its label), the band, the next rule point and the description.
//! The game draws it; it names nothing of a story.

use super::meter::MeterDef;
use super::purse::purse;
use crate::data::Data;
use crate::state::GameState;
use aegis_core::json;

/// What a HUD chip counts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChipKind {
    Currency,
    Upkeep,
    Meter,
}

/// One HUD chip (W4): `id` is "currency", "upkeep" or the meter id; `short` is the text drawn before the value (""
/// for the purse counters, which draw only the icon and the amount); `value` the amount with its unit; `tone` the
/// band's tone (`calm`, `warn`, `danger`, `fatal`; "" for the purse); `title` and `lines` the tip.
#[derive(Debug, Clone, PartialEq)]
pub struct HudChip {
    pub id: String,
    pub kind: ChipKind,
    pub short: String,
    pub icon: String,
    pub value: String,
    pub tone: String,
    pub title: String,
    pub lines: Vec<String>,
}

/// A value with the meter's unit: "12%", "37 days", "1 day", or the bare number.
pub fn with_unit(value: i64, unit: &str) -> String {
    match unit {
        "%" => format!("{value}%"),
        "days" if value == 1 => "1 day".to_string(),
        "days" => format!("{value} days"),
        _ => value.to_string(),
    }
}

/// M5: the next point a rule of meter `def` fires at from `value` in `act`: the lowest rising point above a gauge's
/// value, or the highest falling point below a running countdown's days left; a `once` rule that fired is skipped.
/// None when no rule is ahead (and for a stopped countdown).
pub fn next_point(def: &MeterDef, value: i64, act: i64, running: bool, fired: &[String]) -> Option<i64> {
    let live = def.rules.iter().filter(|r| r.acts.is_empty() || r.acts.contains(&act)).filter(|r| !(r.once && fired.contains(&format!("rule:{}/{}", def.id, r.id))));
    if def.countdown {
        if !running {
            return None;
        }
        live.filter(|r| !r.up && r.at < value).map(|r| r.at).max()
    } else {
        live.filter(|r| r.up && r.at > value).map(|r| r.at).min()
    }
}

/// A purse tip as hover lines (none for an empty tip).
fn tip_lines(tip: &str) -> Vec<String> {
    if tip.is_empty() {
        Vec::new()
    } else {
        vec![tip.to_string()]
    }
}

/// W4: the HUD chips of `gs`, left to right: the currency, the upkeep (`amount/supplies_max`, the game's supply cap), then each visible meter
/// by its `hud.order` (data order on a tie). Outside a campaign world: today's two counters.
pub fn hud(d: &Data, gs: &GameState) -> Vec<HudChip> {
    let p = purse(d, &gs.world_id);
    let mut out = vec![
        HudChip { id: "currency".into(), kind: ChipKind::Currency, short: String::new(), icon: p.currency.icon.clone(), value: gs.party.gold.to_string(), tone: String::new(), title: p.currency.name.clone(), lines: tip_lines(&p.currency.tip) },
        HudChip { id: "upkeep".into(), kind: ChipKind::Upkeep, short: String::new(), icon: p.upkeep.icon.clone(), value: format!("{}/{}", gs.party.supplies, gs.party.purse.supplies_max), tone: String::new(), title: p.upkeep.name.clone(), lines: tip_lines(&p.upkeep.tip) },
    ];
    let Some(c) = gs.campaign.as_ref() else { return out };
    let mut shown: Vec<(i64, usize, HudChip)> = Vec::new();
    for (i, (id, def)) in c.defs.iter().enumerate() {
        let Some(st) = c.meters.get(id).filter(|st| st.visible) else { continue };
        let hud = json::dget(&def.row, "hud");
        let unit = json::s(&def.row, "unit");
        let value = st.value(def, gs.time);
        let band = def.band(value);
        let name = if st.label.is_empty() { def.name.clone() } else { st.label.clone() };
        let mut lines = Vec::new();
        if let Some(b) = band {
            lines.push(format!("Band: {}", b.name));
        }
        if def.countdown && !st.running {
            lines.push("Not running.".to_string());
        }
        if let Some(at) = next_point(def, value, c.act, st.running, &c.fired) {
            let named = def.bands.iter().find(|b| b.from == at).map(|b| format!(" ({})", b.name)).unwrap_or_default();
            lines.push(format!("Next: {}{named}", with_unit(at, &unit)));
        }
        let description = json::s(&def.row, "description");
        if !description.is_empty() {
            lines.push(description);
        }
        let chip = HudChip {
            id: id.clone(),
            kind: ChipKind::Meter,
            short: json::s_or(hud, "short", &name),
            icon: json::s(hud, "icon"),
            value: with_unit(value, &unit),
            tone: band.map(|b| b.tone.clone()).unwrap_or_default(),
            title: name,
            lines,
        };
        shown.push((json::i(hud, "order", 0), i, chip));
    }
    shown.sort_by_key(|(order, i, _)| (*order, *i));
    out.extend(shown.into_iter().map(|(.., chip)| chip));
    out
}
