//! Bands and crossings (engine-only, no Godot original): the shared helper of the campaign meters (CAMPAIGN1 M2, M5;
//! `rpgkit_world::campaign::meter`) and the unit meters (HOOKS1 UM4, UM5; `rpgkit_battle::meter`). A value's band is the
//! last band whose `from` is at or below it; a rule point is crossed `up` when a move goes from below it to at or above
//! it, `down` when it goes from above it to at or below it; crossed points come in crossing order. Pure.

use serde_json::Value;

/// M2, UM4: the index of the band holding `value` among bands starting at `froms` (ascending, data order): the last
/// whose `from` is at or below it; None when the value lies below every band.
pub fn band_index(froms: impl IntoIterator<Item = i64>, value: i64) -> Option<usize> {
    let mut out = None;
    for (i, from) in froms.into_iter().enumerate() {
        if from <= value {
            out = Some(i);
        }
    }
    out
}

/// U1.2-S5: the value a band given in percent of the max (`from_pct`, 0-100) starts at: the smallest whole value at or
/// above `pct` % of `max` (`ceil(max x pct / 100)`), so "below half" (a band `from_pct` 50 above it) holds every value
/// under half the max.
pub fn pct_point(pct: i64, max: i64) -> i64 {
    let n = max * pct;
    if n <= 0 {
        return n / 100;
    }
    (n + 99) / 100
}

/// M5, UM5: a rule's point: a band id (that band's `from`) or a whole number; None when it names no band.
pub fn point<'a>(raw: &Value, bands: impl IntoIterator<Item = (&'a str, i64)>) -> Option<i64> {
    match raw {
        Value::String(id) => bands.into_iter().find(|(b, _)| *b == id).map(|(_, from)| from),
        Value::Number(n) => n.as_i64().or_else(|| n.as_f64().filter(|f| f.fract() == 0.0).map(|f| f as i64)),
        _ => None,
    }
}

/// M5, UM5: whether a move from `old` to `new` crosses `at` in its direction (`up`: old below, new at or above;
/// `down`: old above, new at or below).
pub fn crosses(at: i64, up: bool, old: i64, new: i64) -> bool {
    if up {
        old < at && new >= at
    } else {
        old > at && new <= at
    }
}

/// M5 (K5), UM5: sorts crossed rules into crossing order: rising points for a rise, falling for a fall; data order on
/// a tie (a stable sort).
pub fn crossing_order<T>(items: &mut [T], old: i64, new: i64, at: impl Fn(&T) -> i64) {
    if new >= old {
        items.sort_by_key(|x| at(x));
    } else {
        items.sort_by_key(|x| std::cmp::Reverse(at(x)));
    }
}
