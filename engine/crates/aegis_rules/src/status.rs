//! StatusRegistry + StatusEffects (scripts/combat/status/): the one status table, stacking, inflicting, round
//! roll-over. Statuses inflicted this round become active next round; DEFEND buffs (riposte) are active at once.

use crate::rng::GodotRng;
use crate::stats::roundi;
use crate::unit::Unit;
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stacking {
    RefreshMaxDuration,
    ExtendDuration,
    StackPotency,
    None,
}

#[derive(Debug, Clone, Copy)]
pub struct StatusRow {
    pub id: &'static str,
    pub buff: bool,
    pub resistable: bool,
    pub default_rounds: i32,
    pub stacking: Stacking,
    pub cap: i32,
    pub display: &'static str,
    /// "support" / "defend" for buffs a skill grants; "" otherwise.
    pub grant: &'static str,
    pub magnitude_min: i32,
    pub magnitude_max: i32,
}

const fn row(id: &'static str, buff: bool, resistable: bool, default_rounds: i32, stacking: Stacking, cap: i32, display: &'static str, grant: &'static str, mmin: i32, mmax: i32) -> StatusRow {
    StatusRow { id, buff, resistable, default_rounds, stacking, cap, display, grant, magnitude_min: mmin, magnitude_max: mmax }
}

use Stacking::*;
pub const ROWS: [StatusRow; 15] = [
    row("stagger", false, true, 1, RefreshMaxDuration, 0, "Staggered", "", 0, 0),
    row("stun", false, true, 1, RefreshMaxDuration, 0, "Stunned", "", 0, 0),
    row("root", false, true, 1, RefreshMaxDuration, 0, "Rooted", "", 0, 0),
    row("bleed", false, true, 1, RefreshMaxDuration, 0, "Bleeding", "", 0, 0),
    row("weaken", false, true, 2, RefreshMaxDuration, 25, "Weakened", "", 1, 25),
    row("haste", true, false, 1, RefreshMaxDuration, 0, "Hasted", "support", 0, 0),
    row("taunt", true, false, 1, RefreshMaxDuration, 0, "Taunting", "support", 0, 0),
    row("riposte", true, false, 2, RefreshMaxDuration, 0, "Riposte", "defend", 0, 0),
    row("slow", false, true, 1, RefreshMaxDuration, 0, "Slowed", "", 0, 0),
    row("expose", false, true, 2, RefreshMaxDuration, 20, "Exposed", "", 1, 20),
    row("fortify", true, false, 2, RefreshMaxDuration, 20, "Fortified", "support", 1, 20),
    row("regen", true, false, 2, RefreshMaxDuration, 10, "Regenerating", "support", 1, 10),
    row("poison", false, true, 3, StackPotency, 30, "Poisoned", "", 1, 15),
    row("burn", false, true, 2, RefreshMaxDuration, 10, "Burning", "", 1, 10),
    row("empower", true, false, 2, RefreshMaxDuration, 25, "Empowered", "", 1, 25),
];

pub const BLEED_PCT: f64 = 0.06;
pub const DEBT_STAGGER: i32 = 3;
pub const DEBT_STUN: i32 = 5;

pub fn row_of(status: &str) -> Option<&'static StatusRow> {
    ROWS.iter().find(|r| r.id == status)
}

pub fn cap_of(status: &str) -> i32 {
    row_of(status).map(|r| r.cap).unwrap_or(0)
}

pub fn stacking_of(status: &str) -> Stacking {
    row_of(status).map(|r| r.stacking).unwrap_or(RefreshMaxDuration)
}

pub fn display_name(status: &str) -> String {
    row_of(status).map(|r| r.display.to_string()).unwrap_or_else(|| status.to_string())
}

/// Debuff ids in table order (cleanse lists follow it).
pub fn debuffs() -> Vec<&'static str> {
    ROWS.iter().filter(|r| !r.buff).map(|r| r.id).collect()
}

pub fn defend_buffs() -> Vec<&'static str> {
    ROWS.iter().filter(|r| r.buff && r.grant == "defend").map(|r| r.id).collect()
}

/// StatusRegistry.magnitude_unit: "pct", "flat" (armor points; poison HP per round) or "pct_max_hp".
pub fn magnitude_unit(status: &str) -> &'static str {
    match status {
        "expose" | "fortify" | "poison" => "flat",
        "regen" | "burn" => "pct_max_hp",
        _ => "pct",
    }
}

pub fn has_magnitude(status: &str) -> bool {
    row_of(status).is_some_and(|r| r.magnitude_max > 0)
}

pub fn magnitude_or_min(status: &str, magnitude: i32) -> i32 {
    if magnitude <= 0 && has_magnitude(status) {
        row_of(status).unwrap().magnitude_min
    } else {
        magnitude
    }
}

/// StatusRegistry.stack: (rounds, potency) of an existing application (rounds 0 = absent) plus a new one.
pub fn stack(existing_rounds: i32, existing_potency: i32, new_rounds: i32, new_potency: i32, status: &str) -> (i32, i32) {
    let rule = stacking_of(status);
    let cap = cap_of(status);
    let add = new_rounds.max(1);
    let (mut rounds, mut potency) = (add, new_potency);
    if existing_rounds > 0 {
        match rule {
            ExtendDuration => {
                rounds = existing_rounds + add;
                potency = existing_potency.max(new_potency);
            }
            StackPotency => {
                rounds = existing_rounds.max(add);
                potency = existing_potency + new_potency;
            }
            None => {
                rounds = existing_rounds;
                potency = existing_potency;
            }
            RefreshMaxDuration => {
                rounds = existing_rounds.max(add);
                potency = existing_potency.max(new_potency);
            }
        }
    }
    if cap > 0 {
        if rule == ExtendDuration {
            rounds = rounds.min(cap);
        } else {
            potency = potency.min(cap);
        }
    }
    (rounds, potency)
}

pub fn bleed_damage(c: &Unit) -> i32 {
    roundi(c.max_hp as f64 * BLEED_PCT).max(1)
}

pub fn poison_damage(c: &Unit) -> i32 {
    c.active_magnitude("poison")
}

pub fn burn_damage(c: &Unit) -> i32 {
    roundi(c.max_hp as f64 * c.active_magnitude("burn") as f64 / 100.0).max(1)
}

pub fn regen_amount(c: &Unit) -> i32 {
    roundi(c.max_hp as f64 * c.active_magnitude("regen") as f64 / 100.0).max(1)
}

pub fn blocked_blocks(status: &str) -> &'static [i32] {
    match status {
        "stagger" => &[1, 4],
        "stun" => &[1, 2],
        _ => &[],
    }
}

pub fn apply_chance(target: &Unit, status: &str, potency: i32) -> i32 {
    let resist = target.debuff_resistance() + target.resist_for(status) as f64;
    roundi(potency as f64 - resist).clamp(0, 100)
}

/// Rolls a skill-inflicted status (randi_range(1, 100) <= chance); on success it is queued for next round.
pub fn try_inflict(target: &mut Unit, status: &str, potency: i32, rng: Option<&mut GodotRng>, rounds: i32, magnitude: i32, source: &str) -> bool {
    if !target.is_alive() {
        return false;
    }
    let chance = apply_chance(target, status, potency);
    let roll = match rng {
        Some(r) => r.randi_range(1, 100),
        Option::None => 100,
    };
    if roll > chance {
        return false;
    }
    queue(target, status, rounds, magnitude, source);
    true
}

/// Queues `status` for the target's next round (stacking rule and cap; the newer source wins).
pub fn queue(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    let magnitude = magnitude_or_min(status, magnitude);
    let st = &mut target.st;
    st.pending.insert(status.to_string());
    let existing = st.pending_rounds.get(status).copied().unwrap_or(0);
    let (r, p) = stack(existing, st.pending_magnitudes.get(status).copied().unwrap_or(0), rounds, magnitude, status);
    st.pending_rounds.insert(status.to_string(), r);
    if has_magnitude(status) {
        st.pending_magnitudes.insert(status.to_string(), p);
    }
    if !source.is_empty() {
        st.pending_sources.insert(status.to_string(), source.to_string());
    }
}

/// Makes a buff active right now, lasting `rounds` rounds counting the current one.
pub fn grant_now(target: &mut Unit, status: &str, rounds: i32, magnitude: i32, source: &str) {
    let magnitude = magnitude_or_min(status, magnitude);
    let st = &mut target.st;
    let (r, p) = stack(st.rounds_left.get(status).copied().unwrap_or(0), st.magnitudes.get(status).copied().unwrap_or(0), rounds, magnitude, status);
    st.active.insert(status.to_string());
    st.rounds_left.insert(status.to_string(), r);
    if has_magnitude(status) {
        st.magnitudes.insert(status.to_string(), p);
    }
    if !source.is_empty() {
        st.sources.insert(status.to_string(), source.to_string());
    }
}

/// A skill's extra_grants rows ({status, rounds, magnitude}); `defend` makes defend buffs active at once.
pub fn grant_extras(target: &mut Unit, extras: &[Value], defend: bool, source: &str) -> Vec<String> {
    let mut granted = Vec::new();
    for row in extras {
        let Some(o) = row.as_object() else { continue };
        let status = aegis_data::value::string(o, "status", "");
        if status.is_empty() {
            continue;
        }
        let rounds = aegis_data::value::int(o, "rounds", 1);
        let magnitude = aegis_data::value::int(o, "magnitude", 0);
        if defend && defend_buffs().contains(&status.as_str()) {
            grant_now(target, &status, rounds, magnitude, source);
        } else {
            queue(target, &status, rounds, magnitude, source);
        }
        granted.push(status);
    }
    granted
}

/// Removes `status` active and pending; true when anything was there.
pub fn remove(target: &mut Unit, status: &str) -> bool {
    let found = target.st.has_any(status);
    target.remove_status(status);
    found
}

/// Removes only the ACTIVE instance (a queued one stays; exploit consume).
pub fn remove_active(target: &mut Unit, status: &str) -> bool {
    let st = &mut target.st;
    let found = st.active.contains(status);
    st.active.shift_remove(status);
    st.rounds_left.shift_remove(status);
    st.magnitudes.shift_remove(status);
    st.sources.shift_remove(status);
    found
}

pub fn debt_status(debt: i32) -> &'static str {
    if debt >= DEBT_STUN {
        "stun"
    } else if debt >= DEBT_STAGGER {
        "stagger"
    } else {
        ""
    }
}

/// Round start: last round's statuses expire (multi-round ones count down), queued ones and AP-debt penalties take
/// effect, then AP recovers. Returns the statuses now active.
pub fn roll_over(c: &mut Unit, ap_regen: i32) -> Vec<String> {
    let st = &mut c.st;
    let carried: Vec<(String, i32)> = st.rounds_left.iter().filter_map(|(k, v)| (v - 1 > 0).then(|| (k.clone(), v - 1))).collect();
    let old_magnitudes = st.magnitudes.clone();
    let old_sources = st.sources.clone();
    st.active.clear();
    st.rounds_left.clear();
    st.magnitudes.clear();
    st.sources.clear();
    for (s, left) in carried {
        st.active.insert(s.clone());
        st.rounds_left.insert(s.clone(), left);
        if let Some(m) = old_magnitudes.get(&s) {
            st.magnitudes.insert(s.clone(), *m);
        }
        if let Some(src) = old_sources.get(&s) {
            st.sources.insert(s.clone(), src.clone());
        }
    }
    let pending: Vec<String> = st.pending.iter().cloned().collect();
    for s in pending {
        let queued = st.pending_rounds.get(&s).copied().unwrap_or(1);
        if has_magnitude(&s) {
            let (_, p) = stack(st.rounds_left.get(&s).copied().unwrap_or(0), st.magnitudes.get(&s).copied().unwrap_or(0), queued, st.pending_magnitudes.get(&s).copied().unwrap_or(0), &s);
            st.magnitudes.insert(s.clone(), p);
        }
        st.active.insert(s.clone());
        let left = st.rounds_left.get(&s).copied().unwrap_or(0).max(queued);
        st.rounds_left.insert(s.clone(), left);
        if let Some(src) = st.pending_sources.get(&s).cloned() {
            st.sources.insert(s.clone(), src);
        }
    }
    st.pending.clear();
    st.pending_rounds.clear();
    st.pending_magnitudes.clear();
    st.pending_sources.clear();
    c.riposted.clear();
    let penalty = debt_status((-c.ap).max(0));
    if !penalty.is_empty() {
        c.st.active.insert(penalty.to_string());
    }
    c.ap = if ap_regen < 0 { c.max_ap } else { c.max_ap.min(c.ap + ap_regen + c.ap_regen_bonus) };
    c.st.active.iter().cloned().collect()
}
