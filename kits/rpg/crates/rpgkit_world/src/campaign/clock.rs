//! The day clock's countdowns (CAMPAIGN1 K1-K5, M1; kit hook, engine-only, no Godot original): the effects
//! `clock_start`, `clock_set` and `clock_stop` on a countdown meter's state, and the days a leg, a rest or a retreat
//! passes. A countdown's rules fire in `campaign::tick` as the days pass (K2, K5); a soft rule's restart is
//! `campaign::fire`'s (K4).

use super::Campaign;
use aegis_core::json::{self, Dict};

/// K1, K3: applies a clock effect (`clock_start`, `clock_set`, `clock_stop`) to countdown `id` at `time`; false when
/// `effect` is not one of them. A gauge or an unknown meter is left alone (the content check refuses those).
///
/// A start (and a set that starts a stopped clock) moves the meter's baseline with it: a stopped countdown reads its
/// `max` (M1), so the jump from `max` to the days left is no crossing and fires no rule. A set on a running clock is a
/// move like `meter_set`: the rules it crosses fire. A stop reads `max` again with no crossing either. PR6-6 (B4): a
/// set with `keep_longer` true on a running clock keeps the later of its due time and the set's (`due.max(time +
/// days)`); on a stopped clock it sets as any set.
pub fn apply(c: &mut Campaign, id: &str, effect: &str, fx: &Dict, time: i64) -> bool {
    if !matches!(effect, "clock_start" | "clock_set" | "clock_stop") {
        return false;
    }
    let Some(def) = c.defs.get(id).cloned().filter(|m| m.countdown) else { return true };
    let Some(st) = c.meters.get_mut(id) else { return true };
    let was_running = st.running;
    match effect {
        "clock_start" => {
            st.due = time + json::i(fx, "days", def.max).clamp(0, def.max);
            st.running = true;
            st.visible = true;
        }
        "clock_set" => {
            // PR6-6 (`PK4_PK5_CAMPAIGN_HUB.md` 2, B4): `keep_longer` never shortens a running clock's banked time
            let due = time + json::i(fx, "days", 0).clamp(0, def.max);
            st.due = if json::b(fx, "keep_longer", false) && was_running { st.due.max(due) } else { due };
            st.running = true;
            let label = json::s(fx, "label");
            if !label.is_empty() {
                st.label = label;
            }
        }
        _ => st.running = false,
    }
    let now = st.value(&def, time);
    if effect != "clock_set" || !was_running {
        c.last.insert(id.to_string(), now);
    }
    true
}
