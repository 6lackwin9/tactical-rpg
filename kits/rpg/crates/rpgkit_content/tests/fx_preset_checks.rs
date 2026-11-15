//! PR5-53 (VFX-DATA1, FB-231 (d)): the catalog kind `fx_presets` on a scratch data folder built here (no shipped data):
//! a missing file is an empty table; an action look (roles blow, shot, strike, support) and a status look pass with
//! their parameter objects; each mistake (unknown fields and keys, colours, ranges, a delivery without its parts, a field
//! of the other role, loop shapes, tints, fades) has its text; the loader keeps the rows that pass and they reach
//! `Catalog::fx_presets` typed; the writer stores a row as given (every default left out) and re-saves it byte for byte;
//! once the table has rows, a look key no row draws (or a row of another role) is a whole-catalog warning.

use aegis_core::json::{self, Dict};
use rpgkit_content::check::fx_preset;
use rpgkit_content::{check, source, store};
use rpgkit_data::looks::{FxDelivery, FxRole, FxShotLook, FxSlot, SPARKS_GRAVITY, SPARKS_SIZE};
use rpgkit_data::{Catalog, FX_PRESETS_KIND};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

fn scratch(tag: &str) -> (aegis_core::scratch::Scratch, PathBuf) {
    let t = aegis_core::scratch::Scratch::vacant(&format!("rpgkit_fx_presets_{tag}"));
    let data = t.join("data");
    fs::create_dir_all(data.join("catalog")).unwrap();
    fs::create_dir_all(data.join("worlds")).unwrap();
    (t, data)
}

fn write_kind(data: &Path, kind: &str, entries: Value) {
    let version = if kind == "skills" { 2 } else { 1 };
    fs::write(data.join("catalog").join(format!("{kind}.json")), json!({"kind": kind, "version": version, "entries": entries}).to_string()).unwrap();
}

fn dict(v: Value) -> Dict {
    json::dict(&v).clone()
}

fn with(mut row: Value, key: &str, v: Value) -> Value {
    row[key] = v;
    row
}

fn without(mut row: Value, key: &str) -> Value {
    row.as_object_mut().unwrap().remove(key);
    row
}

/// A blow: a swing trail and sparks along the attack.
fn blow(id: &str) -> Value {
    json!({"id": id, "role": "blow", "delivery": "melee",
        "trail": {"radius": 13, "from": -60, "to": 50, "width": 2, "color": "#9ca3af"},
        "sparks": {"count": 6, "color": "#9ca3af", "color2": "#d9cbb4", "spread": 90, "speed": 70}})
}

/// A lobbed orb with an aim ring and flash and rising sparks.
fn lob(id: &str) -> Value {
    json!({"id": id, "role": "strike", "delivery": "shot", "speed": 380.0, "arc": 16.0,
        "shot": {"look": "orb", "size": 3, "color": "#fbbf24", "color2": "#d97706", "tail": 3},
        "sparks": {"count": 10, "color": "#d97706", "color2": "#fbbf24", "dir": -90, "spread": 160, "speed": 70, "gravity": -20},
        "aim_ring": {"r0": 4, "r1": 19.2, "color": "#d97706e6", "ms": 260},
        "aim_flash": {"rx": 10, "ry": 7, "color": "#d977068c", "ms": 160}})
}

fn beam(id: &str) -> Value {
    json!({"id": id, "role": "strike", "delivery": "beam", "beam": {"width": 2, "core": "#d9cbb4", "glow": "#c084fc", "ms": 160}})
}

fn rally(id: &str) -> Value {
    json!({"id": id, "role": "support", "delivery": "cast", "ring": {"r0": 4, "r1": 20, "color": "#d9cbb4c8", "ms": 300}, "on_actor": true})
}

fn orbit(id: &str) -> Value {
    json!({"id": id, "role": "status", "slot": "head", "period": 900,
        "orbit": {"rx": 7, "ry": 2.5, "ms": 900, "count": 3, "size": 2, "color": "#fbbf24", "color2": "#d9cbb4", "star": true}})
}

fn embers(id: &str) -> Value {
    json!({"id": id, "role": "status", "slot": "body", "dy": 4.0, "period": 600,
        "sparks": {"count": 7, "color": "#d97706", "color2": "#fbbf24", "dir": -90, "spread": 70, "speed": 25, "gravity": -20, "size": 2, "ms": 500}})
}

fn arc(id: &str) -> Value {
    json!({"id": id, "role": "status", "slot": "body", "period": 1500, "trail": {"radius": 9, "from": -40, "to": 40, "width": 1, "color": "#9ca3af", "ms": 300}})
}

fn tint(id: &str) -> Value {
    json!({"id": id, "role": "status", "slot": "tint", "period": 1600, "tint": {"color": "#6b7280", "lo": 40, "hi": 70}})
}

fn fade(id: &str) -> Value {
    json!({"id": id, "role": "status", "slot": "alpha"})
}

#[test]
fn a_missing_file_is_an_empty_table() {
    let (_scratch, data) = scratch("missing");
    let src = source::load(&data);
    assert!(json::arr(&src.catalog[FX_PRESETS_KIND]).is_empty());
    assert!(src.errors.iter().all(|e| !e.contains(&format!("{FX_PRESETS_KIND}.json"))), "{:?}", src.errors);
    assert!(Catalog::from_value(&src.catalog).fx_presets.is_empty());
}

#[test]
fn action_and_status_looks_pass_with_their_parameters() {
    let (_scratch, data) = scratch("valid");
    let v = store::view(&data);
    for row in [blow("b_one"), lob("l_one"), beam("r_one"), rally("s_one"), orbit("o_one"), embers("e_one"), arc("a_one"), tint("t_one"), fade("f_one")] {
        assert_eq!(check::validate_entry(&v, FX_PRESETS_KIND, &dict(row.clone()), ""), Vec::<String>::new(), "{row}");
    }
}

#[test]
fn each_preset_mistake_has_its_text() {
    let (_scratch, data) = scratch("texts");
    let v = store::view(&data);
    let check = |e: Value| check::validate_entry(&v, FX_PRESETS_KIND, &dict(e), "");
    assert_eq!(check(with(blow("b_one"), "glow", json!(1))), ["Unknown field 'glow'."]);
    assert_eq!(check(with(blow("b_one"), "trail", json!({"radius": 13, "from": -60, "to": 50, "width": 2, "color": "#9ca3af", "len": 3}))), ["Trail: unknown key 'len' (keys: radius, from, to, width, color, ms)."]);
    assert_eq!(check(with(blow("b_one"), "trail", json!({"radius": 13, "from": -60, "to": 50, "width": 2, "color": "steel"}))), ["Trail: 'color' must be #rrggbb or #rrggbbaa (is \"steel\")."]);
    assert_eq!(check(with(blow("b_one"), "trail", json!({"radius": 13, "from": -60, "to": 50, "width": 2, "color": "#9ca3af", "ms": 200}))), ["Trail: 'ms' is for a status trail (an action's swing takes the game's length)."]);
    assert_eq!(check(with(blow("b_one"), "trail", json!({"radius": 13, "from": -60, "to": 50, "width": 2.5, "color": "#9ca3af"}))), ["Trail: 'width' must be a whole number from 1 to 4 (is 2.5)."]);
    assert_eq!(check(with(blow("b_one"), "sparks", json!({"count": 30, "color": "#9ca3af", "spread": 90, "speed": 70}))), ["Sparks: 'count' must be a whole number from 1 to 24 (is 30)."]);
    assert_eq!(check(with(blow("b_one"), "sparks", json!({"count": 3, "color": "#9ca3af", "speed": 70}))), ["Sparks: 'spread' is required."]);
    assert_eq!(check(with(blow("b_one"), "ring", json!([1, 2]))), ["Ring must be an object {r0, r1, color, ms}."]);
    assert_eq!(check(without(blow("b_one"), "delivery")), ["An action look needs a delivery (melee, shot, beam or cast)."]);
    assert_eq!(check(with(blow("b_one"), "role", json!("swing"))), ["Role: 'swing' is not one of blow, shot, strike, support, status."]);
    assert_eq!(check(with(blow("b_one"), "period", json!(500))), ["'period' is for a status look (role status)."]);
    assert_eq!(check(with(blow("b_one"), "speed", json!(300.0))), ["'speed' is for delivery shot."]);
    assert_eq!(check(without(lob("l_one"), "shot")), ["Delivery shot needs a projectile (shot)."]);
    assert_eq!(check(with(lob("l_one"), "speed", json!(0.0))), ["Shot speed must be 1-4000 px/s with delivery shot (is 0)."]);
    assert_eq!(check(with(lob("l_one"), "shot", json!({"look": "comet", "size": 3, "color": "#fbbf24"}))), ["Projectile: 'look' must be one of arrow, bolt, stone, orb (is 'comet')."]);
    assert_eq!(check(without(lob("l_one"), "aim_ring")), ["Aim flash needs an aim ring."]);
    assert_eq!(check(with(blow("b_one"), "shot", json!({"look": "orb", "size": 3, "color": "#fbbf24"}))), ["A projectile (shot) is for delivery shot."]);
    assert_eq!(check(without(beam("r_one"), "beam")), ["A beam goes with delivery beam (both or neither)."]);
    assert_eq!(check(with(orbit("o_one"), "delivery", json!("cast"))), ["'delivery' is for an action look (roles blow, shot, strike, support)."]);
    assert_eq!(check(without(orbit("o_one"), "slot")), ["A status look needs a slot (head, body, feet, tint or alpha)."]);
    assert_eq!(check(with(orbit("o_one"), "ring", json!({"r0": 2, "r1": 7, "color": "#991b1bc8", "ms": 700}))), ["A head look needs exactly one loop shape (trail, ring, flash, sparks, orbit; has 2)."]);
    assert_eq!(check(with(orbit("o_one"), "period", json!(20))), ["Period must be 50-10000 ms (is 20)."]);
    assert_eq!(check(with(arc("a_one"), "trail", json!({"radius": 9, "from": -40, "to": 40, "width": 1, "color": "#9ca3af"}))), ["Trail: 'ms' is required."]);
    assert_eq!(check(with(embers("e_one"), "sparks", json!({"count": 7, "color": "#d97706", "spread": 70, "speed": 25, "ms": 500}))), ["Sparks: 'dir' is required."]);
    assert_eq!(check(without(tint("t_one"), "tint")), ["A tint look needs a tint and no loop shape."]);
    assert_eq!(check(with(tint("t_one"), "tint", json!({"color": "#6b7280", "lo": 80, "hi": 70}))), ["Tint: 'lo' must not be above 'hi'."]);
    assert_eq!(check(with(tint("t_one"), "tint", json!({"color": "#6b728080", "lo": 40, "hi": 70}))), ["Tint: 'color' must be #rrggbb (is \"#6b728080\")."]);
    assert_eq!(check(with(fade("f_one"), "period", json!(900))), ["An alpha look (the token faded) takes no shape, tint or period."]);
    assert_eq!(check(with(fade("f_one"), "dy", json!(2.0))), ["Lowered is for a head, body or feet loop."]);
    let bad_id = check(blow("B"));
    assert!(bad_id.iter().any(|e| e.starts_with("ID 'B'")), "{bad_id:?}");
}

#[test]
fn the_loader_keeps_the_presets_that_pass_and_types_them() {
    let (_scratch, data) = scratch("loader");
    write_kind(&data, FX_PRESETS_KIND, json!([blow("b_one"), lob("l_one"), blow("b_one"), with(tint("t_bad"), "slot", json!("hat")), orbit("o_one"), fade("f_one")]));
    let src = source::load(&data);
    let kept: Vec<String> = json::arr(&src.catalog[FX_PRESETS_KIND]).iter().map(|e| json::s(json::dict(e), "id")).collect();
    assert_eq!(kept, ["b_one", "l_one", "o_one", "f_one"]);
    assert!(src.errors.iter().any(|e| e.ends_with("effect preset 'b_one': ID 'b_one' is already used by another effect preset.")), "{:?}", src.errors);
    assert!(src.errors.iter().any(|e| e.contains("effect preset 't_bad': Slot: 'hat' is not one of")), "{:?}", src.errors);
    let cat = Catalog::from_value(&src.catalog);
    assert_eq!(cat.fx_presets.keys().collect::<Vec<_>>(), ["b_one", "l_one", "o_one", "f_one"], "keyed by look key in file order");
    let b = &cat.fx_presets["b_one"];
    assert_eq!((b.role, b.delivery), (FxRole::Blow, FxDelivery::Melee));
    let t = b.trail.expect("a trail");
    assert_eq!((t.radius, t.from, t.to, t.width, t.color, t.ms), (13.0, -60.0, 50.0, 2, [0x9c, 0xa3, 0xaf, 255], 0));
    let s = b.sparks.expect("sparks");
    assert_eq!((s.count, s.colors, s.dir, s.gravity, s.size, s.ms, s.foot), (6, [[0x9c, 0xa3, 0xaf, 255], [0xd9, 0xcb, 0xb4, 255]], None, SPARKS_GRAVITY, SPARKS_SIZE, 0, false), "left-out keys: their defaults");
    let l = &cat.fx_presets["l_one"];
    assert_eq!((l.delivery, l.speed, l.arc), (FxDelivery::Shot, 380.0, 16.0));
    assert_eq!(l.shot.map(|s| (s.look, s.size, s.tail)), Some((FxShotLook::Orb, 3.0, 3)));
    assert_eq!(l.aim_ring.map(|r| (r.r1, r.color)), Some((19.2, [0xd9, 0x77, 0x06, 0xe6])), "#rrggbbaa: the alpha");
    assert_eq!(l.sparks.and_then(|s| s.dir), Some(-90.0));
    let o = &cat.fx_presets["o_one"];
    assert_eq!((o.role, o.slot, o.period), (FxRole::Status, FxSlot::Head, 900));
    assert_eq!(o.orbit.map(|o| (o.count, o.size, o.star)), Some((3, 2, true)));
    assert_eq!(cat.fx_presets["f_one"].slot, FxSlot::Alpha);
}

#[test]
fn the_writer_stores_a_preset_as_given_and_resaves_it() {
    let (_scratch, data) = scratch("writer");
    let file = data.join("catalog").join(format!("{FX_PRESETS_KIND}.json"));
    let refused = store::upsert(&data, None, FX_PRESETS_KIND, &dict(without(lob("l_one"), "shot")), None).err().expect("a bad row is refused");
    assert!(refused.iter().any(|e| e.ends_with("Delivery shot needs a projectile (shot).")), "{refused:?}");
    for row in [lob("l_one"), orbit("o_one"), fade("f_one")] {
        store::upsert(&data, None, FX_PRESETS_KIND, &dict(row), None).expect("a valid row");
    }
    let text = fs::read_to_string(&file).unwrap();
    let doc: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(doc["entries"][0], lob("l_one"), "every field at its default left out");
    assert_eq!(doc["entries"][2], fade("f_one"));
    assert_eq!(store::resave_text(&data, None, FX_PRESETS_KIND).unwrap(), text, "a re-save without changes is byte-identical");
    store::upsert(&data, None, FX_PRESETS_KIND, &dict(lob("l_one")), None).expect("the same row again");
    assert_eq!(fs::read_to_string(&file).unwrap(), text, "an upsert of the same row changes nothing");
}

#[test]
fn a_look_key_without_a_preset_warns_once_the_table_has_rows() {
    let (_scratch, data) = scratch("warnings");
    write_kind(&data, "item_families", json!([{"id": "t_blade", "name": "Blade", "item_type": "WEAPON", "fx": "b_one", "fx_far": "l_one"}, {"id": "t_rod", "name": "Rod", "item_type": "WEAPON", "fx": "o_one"}]));
    write_kind(&data, "skills", json!([{"id": "t_lob", "name": "Lob", "kind": "ATTACK", "level": 1, "blocks": 2, "fx": "l_nope"}]));
    let empty = store::view(&data);
    assert!(fx_preset::fx_warnings(&empty).is_empty(), "no rows: every key draws the game's own look");
    write_kind(&data, FX_PRESETS_KIND, json!([blow("b_one"), lob("l_one"), orbit("o_one")]));
    let v = store::view(&data);
    let found = fx_preset::fx_warnings(&v);
    let of = |label: &str| found.iter().filter(|(l, _)| l == label).map(|(_, m)| m.clone()).collect::<Vec<_>>();
    assert_eq!(of("skills:t_lob"), ["fx 'l_nope' names no effect preset (a strike or support row draws it)."]);
    assert_eq!(of("item_families:t_blade"), Vec::<String>::new(), "a blow fx and a strike fx_far");
    assert_eq!(of("item_families:t_rod"), ["fx 'o_one' names a status effect preset (a blow or shot row draws it)."]);
    // each status row with a look key: no status preset row in this table
    let keyed = rpgkit_rules::status::ROWS.iter().filter(|r| !r.fx.is_empty()).count();
    assert_eq!(found.iter().filter(|(l, _)| l.starts_with("statuses:")).count(), keyed);
    let report = check::warnings_all(&v);
    assert!(report.get("skills:t_lob").is_some_and(|ms| ms.iter().any(|m| m.contains("l_nope"))), "the whole-catalog report carries it: {report:?}");
}
