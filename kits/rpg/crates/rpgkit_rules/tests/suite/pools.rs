//! The typed pool facade (U1.2 slice 1, `FB-04`) on a scratch catalog (no shipped data): a unit's HP and AP pools are
//! views of its typed fields (0 to `max_hp`, 0 to `max_ap`), follow every write to them, and are never clamped by the
//! view; poise is the battle's (None on a unit); pool ids round-trip by name.

use rpgkit_data::Catalog;
use rpgkit_rules::build_unit;
use rpgkit_rules::pool::{PoolId, PoolView};
use serde_json::json;

fn probe() -> rpgkit_rules::Unit {
    let cat = Catalog::from_value(&json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 8, "CONS": 13, "PERC": 9, "KNOW": 14}}],
        "units": [{"id": "probe", "name": "Probe", "class_id": "drill", "level": 2, "side": "player"}]
    }));
    build_unit(&cat, &cat.units["probe"], 7)
}

#[test]
fn hp_and_ap_pools_view_the_typed_fields() {
    let mut u = probe();
    assert_eq!(u.pool(PoolId::Hp), Some(PoolView { current: u.hp, min: 0, max: u.max_hp }));
    assert_eq!(u.pool(PoolId::Ap), Some(PoolView { current: u.ap, min: 0, max: u.max_ap }));
    assert!(u.pool(PoolId::Hp).unwrap().is_full());
    assert_eq!(u.pool(PoolId::Poise), None, "poise lives in the battle");
    let dealt = u.take_damage(5, true);
    let hp = u.pool(PoolId::Hp).unwrap();
    assert_eq!((hp.current, hp.room()), (u.max_hp - dealt, dealt));
    u.take_damage(u.max_hp, true);
    assert!(u.pool(PoolId::Hp).unwrap().is_empty(), "HP 0: knocked");
    u.ap = u.max_ap + 2;
    let ap = u.pool(PoolId::Ap).unwrap();
    assert_eq!((ap.current, ap.room(), ap.is_full()), (u.max_ap + 2, 0, true), "the view does not clamp");
}

#[test]
fn pool_ids_round_trip_by_name() {
    for id in PoolId::ALL {
        assert_eq!(PoolId::from_name(id.name()), Some(id));
    }
    assert_eq!(PoolId::from_name("composure"), None, "data pools are not typed ids");
}
