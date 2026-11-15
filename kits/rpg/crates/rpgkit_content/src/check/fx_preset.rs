//! Effect presets (PR5-53 VFX-DATA1, FB-231 (d); engine-only, no Godot original): the checks of one row of the kind
//! `fx_presets` (the look key, the role, the delivery and its numbers, each parameter object's keys, numbers and
//! colours, the fields a role takes) and across rows (unique ids). The loader keeps the rows that pass, in file order
//! (`rpgkit_data::looks::FxPreset`). The whole-catalog report warns on a look key no row draws once the table has rows
//! (`fx_warnings`).

use super::View;
use crate::source::fields;
use aegis_core::json::{self, gstr, Dict};
use rpgkit_data::looks::{
    parse_rgba, FxDelivery, FxRole, FxShotLook, FxSlot, BEAM_KEYS, FLASH_KEYS, FX_AIM_FLASH_KEY, FX_AIM_RING_KEY, FX_ARC_KEY, FX_BEAM_KEY, FX_DELIVERY_KEY, FX_DROP_KEY, FX_DY_KEY, FX_FLASH_KEY, FX_ON_ACTOR_KEY, FX_ORBIT_KEY,
    FX_PERIOD_KEY, FX_RING_KEY, FX_ROLE_KEY, FX_SHOT_KEY, FX_SLOT_KEY, FX_SPARKS_KEY, FX_SPEED_KEY, FX_TINT_KEY, FX_TRAIL_KEY, FX_VOLLEY_KEY, FX_WAVE_KEY, ORBIT_KEYS, RING_KEYS, SHOT_KEYS, SPARKS_KEYS, TINT_KEYS, TRAIL_KEYS,
};

/// The catalog kind (file data/catalog/fx_presets.json; a missing file is an empty table).
pub const KIND: &str = rpgkit_data::FX_PRESETS_KIND;
/// The look key's length range.
pub const ID_MIN: usize = 2;
pub const ID_MAX: usize = 24;
/// The ranges: px of radii and ellipses, a trail's radius, an angle (deg), a shot's speed and arc, a projectile's size,
/// the ms of rings, flashes and beams (1 up), of trails and sparks (0 = the game's), of an orbit's turn, a spark burst's
/// count, speed, gravity and size, a beam's and a trail's width, an orb's tail, a volley, a drop, a loop's lowering and
/// a status look's period.
pub const RADIUS_PX: (f64, f64) = (0.0, 64.0);
pub const TRAIL_RADIUS: (f64, f64) = (1.0, 64.0);
pub const ANGLE: (f64, f64) = (-360.0, 360.0);
pub const SPEED: (f64, f64) = (1.0, 4000.0);
pub const ARC: (f64, f64) = (0.0, 64.0);
pub const SHOT_SIZE: (f64, f64) = (0.5, 16.0);
pub const MS: (f64, f64) = (1.0, 5000.0);
pub const MS_OR_GAME: (f64, f64) = (0.0, 5000.0);
pub const ORBIT_MS: (f64, f64) = (1.0, 10000.0);
pub const SPARKS_COUNT: (f64, f64) = (1.0, 24.0);
pub const SPARKS_SPEED: (f64, f64) = (0.0, 1000.0);
pub const SPREAD: (f64, f64) = (0.0, 360.0);
pub const GRAVITY: (f64, f64) = (-500.0, 500.0);
pub const SPARKS_SIZE: (f64, f64) = (1.0, 4.0);
pub const ORBIT_COUNT: (f64, f64) = (1.0, 8.0);
pub const ORBIT_SIZE: (f64, f64) = (1.0, 2.0);
pub const TRAIL_WIDTH: (f64, f64) = (1.0, 4.0);
pub const BEAM_WIDTH: (f64, f64) = (1.0, 3.0);
pub const TAIL: (f64, f64) = (0.0, 3.0);
pub const ALPHA: (f64, f64) = (0.0, 255.0);
pub const VOLLEY_MAX: i64 = 12;
pub const DROP_MAX: f64 = 200.0;
pub const DY: (f64, f64) = (-32.0, 32.0);
pub const PERIOD: (i64, i64) = (50, 10000);

/// The fields only an action look takes and those only a status look takes (each refused on the other role while set;
/// `trail`, `ring`, `flash` and `sparks` serve both).
pub const ACTION_FIELDS: [&str; 11] = [FX_DELIVERY_KEY, FX_SPEED_KEY, FX_ARC_KEY, FX_SHOT_KEY, FX_BEAM_KEY, FX_WAVE_KEY, FX_VOLLEY_KEY, FX_DROP_KEY, FX_AIM_RING_KEY, FX_AIM_FLASH_KEY, FX_ON_ACTOR_KEY];
pub const STATUS_FIELDS: [&str; 5] = [FX_SLOT_KEY, FX_DY_KEY, FX_PERIOD_KEY, FX_ORBIT_KEY, FX_TINT_KEY];
/// The loop shapes of a head, body or feet status look (exactly one).
pub const LOOP_SHAPES: [&str; 5] = [FX_TRAIL_KEY, FX_RING_KEY, FX_FLASH_KEY, FX_SPARKS_KEY, FX_ORBIT_KEY];

fn chars(s: &str) -> usize {
    s.chars().count()
}

/// The schema field `key` of the kind.
fn field(key: &str) -> Option<&'static Dict> {
    fields("content", KIND).iter().map(json::dict).find(|f| json::s(f, "key") == key)
}

/// The schema options of a field of the kind.
fn options(key: &str) -> Vec<String> {
    field(key).map(|f| json::aget(f, "options").iter().map(gstr).collect()).unwrap_or_default()
}

/// Field `key` of row `e` differs from its schema default (it is set).
fn set(e: &Dict, key: &str) -> bool {
    let Some(f) = field(key) else { return false };
    e.get(key).is_some_and(|v| !json::deep_equal(v, json::get(f, "default")))
}

/// The problems of each row (same order as the rows), on its own and against the others (unique ids).
pub fn check_rows(rows: &[Dict]) -> Vec<Vec<String>> {
    let mut out: Vec<Vec<String>> = rows.iter().map(row_errors).collect();
    for (i, found) in out.iter_mut().enumerate() {
        let id = json::s(&rows[i], "id");
        if !id.is_empty() && rows[..i].iter().any(|r| json::s(r, "id") == id) {
            found.push(format!("ID '{id}' is already used by another effect preset."));
        }
    }
    out
}

/// A number key of parameter object `p` (labelled `label`): within `(lo, hi)` and whole when `whole`; `required` or
/// left out.
fn number(p: &Dict, key: &str, label: &str, (lo, hi): (f64, f64), whole: bool, required: bool, out: &mut Vec<String>) {
    match p.get(key) {
        None if required => out.push(format!("{label}: '{key}' is required.")),
        None => {}
        Some(v) => {
            let ok = v.as_f64().is_some_and(|x| (!whole || x.fract() == 0.0) && (lo..=hi).contains(&x));
            if !ok {
                let what = if whole { "a whole number" } else { "a number" };
                out.push(format!("{label}: '{key}' must be {what} from {lo} to {hi} (is {v})."));
            }
        }
    }
}

/// A colour key of parameter object `p`: `#rrggbb`, or `#rrggbbaa` when `alpha`; `required` or left out.
fn colour(p: &Dict, key: &str, label: &str, alpha: bool, required: bool, out: &mut Vec<String>) {
    match p.get(key) {
        None if required => out.push(format!("{label}: '{key}' is required.")),
        None => {}
        Some(v) => {
            let s = v.as_str().unwrap_or("");
            let ok = parse_rgba(s).is_some() && (alpha || s.len() == 7);
            if !ok {
                let form = if alpha { "#rrggbb or #rrggbbaa" } else { "#rrggbb" };
                out.push(format!("{label}: '{key}' must be {form} (is {v})."));
            }
        }
    }
}

/// A true / false key of parameter object `p` (left out: false).
fn flag(p: &Dict, key: &str, label: &str, out: &mut Vec<String>) {
    if p.get(key).is_some_and(|v| !v.is_boolean()) {
        out.push(format!("{label}: '{key}' must be true or false."));
    }
}

/// Parameter object `key` of row `e` (None: left out or empty); not an object, or a key outside `keys`, is a problem.
fn part<'a>(e: &'a Dict, key: &str, label: &str, keys: &[&str], out: &mut Vec<String>) -> Option<&'a Dict> {
    let v = e.get(key)?;
    let Some(p) = v.as_object() else {
        if !(v.is_null() || v.as_array().is_some_and(|a| a.is_empty())) {
            out.push(format!("{label} must be an object {{{}}}.", keys.join(", ")));
        }
        return None;
    };
    for k in p.keys().filter(|k| !keys.contains(&k.as_str())) {
        out.push(format!("{label}: unknown key '{k}' (keys: {}).", keys.join(", ")));
    }
    (!p.is_empty()).then_some(p)
}

fn ring(e: &Dict, key: &str, label: &str, out: &mut Vec<String>) -> bool {
    let Some(p) = part(e, key, label, &RING_KEYS, out) else { return false };
    number(p, "r0", label, RADIUS_PX, false, true, out);
    number(p, "r1", label, RADIUS_PX, false, true, out);
    colour(p, "color", label, true, true, out);
    number(p, "ms", label, MS, true, true, out);
    true
}

fn flash(e: &Dict, key: &str, label: &str, out: &mut Vec<String>) -> bool {
    let Some(p) = part(e, key, label, &FLASH_KEYS, out) else { return false };
    number(p, "rx", label, RADIUS_PX, false, true, out);
    number(p, "ry", label, RADIUS_PX, false, true, out);
    colour(p, "color", label, true, true, out);
    number(p, "ms", label, MS, true, true, out);
    true
}

/// The trail object; a status trail needs its own length, an action's swing takes the game's (no `ms`).
fn trail(e: &Dict, status: bool, out: &mut Vec<String>) -> bool {
    let label = "Trail";
    let Some(p) = part(e, FX_TRAIL_KEY, label, &TRAIL_KEYS, out) else { return false };
    number(p, "radius", label, TRAIL_RADIUS, false, true, out);
    number(p, "from", label, ANGLE, false, true, out);
    number(p, "to", label, ANGLE, false, true, out);
    number(p, "width", label, TRAIL_WIDTH, true, true, out);
    colour(p, "color", label, true, true, out);
    if status {
        number(p, "ms", label, MS, true, true, out);
    } else if p.get("ms").is_some_and(|v| v.as_f64() != Some(0.0)) {
        out.push(format!("{label}: 'ms' is for a status trail (an action's swing takes the game's length)."));
    }
    true
}

/// The sparks object; status sparks need their own life.
fn sparks(e: &Dict, status: bool, out: &mut Vec<String>) -> bool {
    let label = "Sparks";
    let Some(p) = part(e, FX_SPARKS_KEY, label, &SPARKS_KEYS, out) else { return false };
    number(p, "count", label, SPARKS_COUNT, true, true, out);
    colour(p, "color", label, true, true, out);
    colour(p, "color2", label, true, false, out);
    number(p, "dir", label, ANGLE, false, status, out);
    number(p, "spread", label, SPREAD, false, true, out);
    number(p, "speed", label, SPARKS_SPEED, false, true, out);
    number(p, "gravity", label, GRAVITY, false, false, out);
    number(p, "size", label, SPARKS_SIZE, true, false, out);
    number(p, "ms", label, if status { MS } else { MS_OR_GAME }, true, status, out);
    flag(p, "foot", label, out);
    true
}

fn orbit(e: &Dict, out: &mut Vec<String>) -> bool {
    let label = "Orbit";
    let Some(p) = part(e, FX_ORBIT_KEY, label, &ORBIT_KEYS, out) else { return false };
    number(p, "rx", label, RADIUS_PX, false, true, out);
    number(p, "ry", label, RADIUS_PX, false, true, out);
    number(p, "ms", label, ORBIT_MS, true, true, out);
    number(p, "count", label, ORBIT_COUNT, true, true, out);
    number(p, "size", label, ORBIT_SIZE, true, false, out);
    colour(p, "color", label, true, true, out);
    colour(p, "color2", label, true, false, out);
    number(p, "phase", label, ANGLE, false, false, out);
    flag(p, "star", label, out);
    true
}

fn shot(e: &Dict, out: &mut Vec<String>) -> bool {
    let label = "Projectile";
    let Some(p) = part(e, FX_SHOT_KEY, label, &SHOT_KEYS, out) else { return false };
    let look = p.get("look").map(gstr).unwrap_or_default();
    if FxShotLook::parse(&look).is_none() {
        out.push(format!("{label}: 'look' must be one of {} (is '{look}').", FxShotLook::IDS.join(", ")));
    }
    number(p, "size", label, SHOT_SIZE, false, true, out);
    colour(p, "color", label, true, true, out);
    colour(p, "color2", label, true, false, out);
    number(p, "tail", label, TAIL, true, false, out);
    true
}

fn beam(e: &Dict, out: &mut Vec<String>) -> bool {
    let label = "Beam";
    let Some(p) = part(e, FX_BEAM_KEY, label, &BEAM_KEYS, out) else { return false };
    number(p, "width", label, BEAM_WIDTH, true, true, out);
    colour(p, "core", label, true, true, out);
    colour(p, "glow", label, true, true, out);
    number(p, "ms", label, MS, true, true, out);
    true
}

fn tint(e: &Dict, out: &mut Vec<String>) -> bool {
    let label = "Tint";
    let Some(p) = part(e, FX_TINT_KEY, label, &TINT_KEYS, out) else { return false };
    colour(p, "color", label, false, true, out);
    number(p, "lo", label, ALPHA, true, true, out);
    number(p, "hi", label, ALPHA, true, true, out);
    if json::f(p, "lo", 0.0) > json::f(p, "hi", 0.0) {
        out.push(format!("{label}: 'lo' must not be above 'hi'."));
    }
    true
}

/// The own problems of one (normalized) row.
pub fn row_errors(e: &Dict) -> Vec<String> {
    let mut out = Vec::new();
    let known: Vec<String> = fields("content", KIND).iter().map(|f| json::s(json::dict(f), "key")).collect();
    for k in e.keys().filter(|k| !known.contains(k)) {
        out.push(format!("Unknown field '{k}'."));
    }
    let id = json::s(e, "id");
    if !json::id_ok(&id) || !(ID_MIN..=ID_MAX).contains(&chars(&id)) {
        out.push(format!("ID '{id}' must be {ID_MIN}-{ID_MAX} characters of a-z, 0-9 and _."));
    }
    for key in [FX_ROLE_KEY, FX_DELIVERY_KEY, FX_SLOT_KEY, "origin"] {
        let v = json::s(e, key);
        let allowed = options(key);
        if !allowed.contains(&v) {
            out.push(format!("{}: '{v}' is not one of {}.", field(key).map(|f| json::s(f, "label")).unwrap_or_default(), allowed.iter().map(|o| if o.is_empty() { "\"\"" } else { o.as_str() }).collect::<Vec<_>>().join(", ")));
        }
    }
    let role = FxRole::parse(&json::s(e, FX_ROLE_KEY));
    if role == FxRole::Status {
        status_errors(e, &mut out);
    } else {
        action_errors(e, &mut out);
    }
    out
}

/// An action look (blow, shot, strike, support): a delivery and its numbers, the parameter objects it takes.
fn action_errors(e: &Dict, out: &mut Vec<String>) {
    for key in STATUS_FIELDS.iter().filter(|k| set(e, k)) {
        out.push(format!("'{key}' is for a status look (role status)."));
    }
    let delivery = FxDelivery::parse(&json::s(e, FX_DELIVERY_KEY));
    if delivery == FxDelivery::None {
        out.push("An action look needs a delivery (melee, shot, beam or cast).".into());
    }
    let shooting = delivery == FxDelivery::Shot;
    let speed = json::f(e, FX_SPEED_KEY, 0.0);
    if shooting && !(SPEED.0..=SPEED.1).contains(&speed) {
        out.push(format!("Shot speed must be {}-{} px/s with delivery shot (is {speed}).", SPEED.0, SPEED.1));
    }
    let arc = json::f(e, FX_ARC_KEY, 0.0);
    if !(ARC.0..=ARC.1).contains(&arc) {
        out.push(format!("Shot arc must be {}-{} px (is {arc}).", ARC.0, ARC.1));
    }
    let volley = json::i(e, FX_VOLLEY_KEY, 0);
    if !(0..=VOLLEY_MAX).contains(&volley) {
        out.push(format!("Volley must be 0-{VOLLEY_MAX} (is {volley})."));
    }
    let drop = json::f(e, FX_DROP_KEY, 0.0);
    if !(0.0..=DROP_MAX).contains(&drop) {
        out.push(format!("Drop must be 0-{DROP_MAX} px (is {drop})."));
    }
    if !shooting {
        for key in [FX_SPEED_KEY, FX_ARC_KEY, FX_VOLLEY_KEY, FX_DROP_KEY].iter().filter(|k| set(e, k)) {
            out.push(format!("'{key}' is for delivery shot."));
        }
    }
    let has_shot = shot(e, out);
    if shooting && !has_shot {
        out.push("Delivery shot needs a projectile (shot).".into());
    }
    if has_shot && !shooting {
        out.push("A projectile (shot) is for delivery shot.".into());
    }
    let has_beam = beam(e, out);
    if (delivery == FxDelivery::Beam) != has_beam {
        out.push("A beam goes with delivery beam (both or neither).".into());
    }
    trail(e, false, out);
    ring(e, FX_RING_KEY, "Ring", out);
    flash(e, FX_FLASH_KEY, "Flash", out);
    sparks(e, false, out);
    ring(e, FX_WAVE_KEY, "Wave", out);
    let aimed = ring(e, FX_AIM_RING_KEY, "Aim ring", out);
    if flash(e, FX_AIM_FLASH_KEY, "Aim flash", out) && !aimed {
        out.push("Aim flash needs an aim ring.".into());
    }
}

/// A status look: its slot, period and lowering, one loop shape (head, body, feet), a tint (tint) or nothing (alpha).
fn status_errors(e: &Dict, out: &mut Vec<String>) {
    for key in ACTION_FIELDS.iter().filter(|k| set(e, k)) {
        out.push(format!("'{key}' is for an action look (roles blow, shot, strike, support)."));
    }
    let slot = FxSlot::parse(&json::s(e, FX_SLOT_KEY));
    let looped = matches!(slot, FxSlot::Head | FxSlot::Body | FxSlot::Feet);
    let shapes = [trail(e, true, out), ring(e, FX_RING_KEY, "Ring", out), flash(e, FX_FLASH_KEY, "Flash", out), sparks(e, true, out), orbit(e, out)];
    let n = shapes.iter().filter(|s| **s).count();
    let tinted = tint(e, out);
    let period = json::i(e, FX_PERIOD_KEY, 0);
    let dy = json::f(e, FX_DY_KEY, 0.0);
    if !(DY.0..=DY.1).contains(&dy) {
        out.push(format!("Lowered must be {}-{} px (is {dy}).", DY.0, DY.1));
    }
    match slot {
        FxSlot::None => out.push("A status look needs a slot (head, body, feet, tint or alpha).".into()),
        _ if looped && n != 1 => out.push(format!("A {} look needs exactly one loop shape ({}; has {n}).", json::s(e, FX_SLOT_KEY), LOOP_SHAPES.join(", "))),
        FxSlot::Tint if !tinted || n > 0 => out.push("A tint look needs a tint and no loop shape.".into()),
        FxSlot::Alpha if n > 0 || tinted || period != 0 => out.push("An alpha look (the token faded) takes no shape, tint or period.".into()),
        _ => {}
    }
    if tinted && slot != FxSlot::Tint {
        out.push("A tint is for slot tint.".into());
    }
    if (looped || slot == FxSlot::Tint) && !(PERIOD.0..=PERIOD.1).contains(&period) {
        out.push(format!("Period must be {}-{} ms (is {period}).", PERIOD.0, PERIOD.1));
    }
    if !looped && dy != 0.0 {
        out.push("Lowered is for a head, body or feet loop.".into());
    }
}

/// The Forge's check of one entry (normalized) as entry `current_id` ("" = a new entry): its own problems and those
/// against the loaded rows (its old row taken out); the entry is checked last.
pub fn entry_errors(v: &View, e: &Dict, current_id: &str) -> Vec<String> {
    let mut rows: Vec<Dict> = v.entries(KIND).iter().filter(|r| current_id.is_empty() || json::s(r, "id") != current_id).cloned().collect();
    rows.push(e.clone());
    check_rows(&rows).pop().unwrap_or_default()
}

/// The whole-catalog warnings of look keys no preset draws, once the table has rows (an empty table: every key draws
/// the game's own look): a skill's `fx` (a strike or support row), an item family's `fx` (a blow or shot row) and
/// `fx_far` (a strike row), and each status row's `fx` (a status row), as (entry label, message).
pub fn fx_warnings(v: &View) -> Vec<(String, String)> {
    let presets = v.entries(KIND);
    if presets.is_empty() {
        return Vec::new();
    }
    let role_of = |key: &str| presets.iter().find(|p| json::s(p, "id") == key).map(|p| FxRole::parse(&json::s(p, FX_ROLE_KEY)));
    let mut out = Vec::new();
    let mut need = |label: String, field: &str, key: &str, roles: &[FxRole]| {
        if key.is_empty() {
            return;
        }
        let names = roles.iter().map(|r| r.id()).collect::<Vec<_>>().join(" or ");
        match role_of(key) {
            None => out.push((label, format!("{field} '{key}' names no effect preset (a {names} row draws it)."))),
            Some(r) if !roles.contains(&r) => out.push((label, format!("{field} '{key}' names a {} effect preset (a {names} row draws it).", r.id()))),
            _ => {}
        }
    };
    for s in v.entries("skills") {
        need(format!("skills:{}", json::s(s, "id")), "fx", &json::s(s, rpgkit_data::looks::FX_KEY), &[FxRole::Strike, FxRole::Support]);
    }
    for f in v.entries("item_families") {
        let label = format!("item_families:{}", json::s(f, "id"));
        need(label.clone(), "fx", &json::s(f, rpgkit_data::looks::FX_KEY), &[FxRole::Blow, FxRole::Shot]);
        need(label, "fx_far", &json::s(f, rpgkit_data::looks::FX_FAR_KEY), &[FxRole::Strike]);
    }
    for r in rpgkit_rules::status::ROWS.iter() {
        need(format!("statuses:{}", r.id), "fx", r.fx, &[FxRole::Status]);
    }
    out
}
