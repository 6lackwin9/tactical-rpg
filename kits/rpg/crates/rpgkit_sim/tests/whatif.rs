//! BALANCE-WHATIF1 (ENGINE-26): the feel mode's what-if keys change a world encounter's foes for a report run, with no
//! data change: `foe_level=N` fights every foe N levels up (`WorldRoster::foe_levels`, never an ally), `foes=N` adds N
//! more of the encounter's last foe, `foe_hp=P` scales each foe's HP to P % (through the permanent offset, so a stat
//! refresh keeps it); the keys are feel keys with their ranges. Scratch world built here.

use rpgkit_sim::args::{SimArgs, FOES_MAX, FOE_HP_MAX, FOE_LEVEL_MAX};
use rpgkit_sim::runner::Runner;
use rpgkit_battle::session::Roster;
use rpgkit_world::battle::WorldRoster;
use rpgkit_world::Data;
use serde_json::json;

const BASE: i32 = 3;

/// World `vale`: encounter `pack` (two wolves against the hero `ash`).
fn data() -> Data {
    let unit = |id: &str, side: &str, tier: &str| json!({"id": id, "name": id, "side": side, "class_id": "drill", "level": BASE, "tier": tier});
    let catalog = json!({
        "classes": [{"id": "drill", "name": "Drill", "base_stats": {"STR": 12, "DEX": 10, "INT": 6, "CONS": 12, "PERC": 10, "KNOW": 10}}],
        "skills": [
            {"id": "strike", "name": "Strike", "kind": "ATTACK", "level": 1, "blocks": 1, "basic": true, "target_mode": "ENEMY"},
            {"id": "move", "name": "Move", "kind": "MOVE", "level": 1, "blocks": 1, "system": true, "basic": true}
        ],
        "units": [unit("wolf", "enemy", "standard"), unit("ash", "party", "hero")]
    });
    let worlds = json!({
        "vale": {
            "manifest": {"id": "vale"},
            "own": {
                "characters": [{"id": "wolf", "name": "Wolf", "base_unit": "wolf"}, {"id": "ash", "name": "Ash", "base_unit": "ash"}],
                "encounters": [{"id": "pack", "name": "Pack", "party": ["ash"], "enemies": ["wolf", "wolf"], "max_rounds": 1}]
            }
        }
    });
    Data::from_values(&catalog, &worlds, &json!({}))
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse(d, &line.split_whitespace().map(String::from).collect::<Vec<_>>())
}

#[test]
fn the_keys_are_feel_keys_within_their_ranges() {
    let d = data();
    let a = parse(&d, "feel enc=world:vale/pack foe_hp=150 foes=+1 foe_level=2").expect("the keys parse");
    assert_eq!((a.foe_hp, a.foes, a.foe_level), (150, 1, 2));
    let a = parse(&d, "feel enc=world:vale/pack").unwrap();
    assert_eq!((a.foe_hp, a.foes, a.foe_level), (100, 0, 0), "as authored by default");
    for bad in ["foe_hp=0".to_string(), format!("foe_hp={}", FOE_HP_MAX + 1), format!("foes={}", FOES_MAX + 1), format!("foe_level={}", FOE_LEVEL_MAX + 1), "foes=x".into()] {
        assert!(parse(&d, &format!("feel enc=world:vale/pack {bad}")).is_err(), "{bad}");
    }
    assert!(parse(&d, "duel foe_hp=150").is_err(), "a feel key only");
}

#[test]
fn foe_levels_raise_the_foes_and_never_an_ally() {
    let d = data();
    let cat = d.cat.clone();
    let at = |levels: i64, allies: Vec<String>| WorldRoster { foe_levels: levels, allies, ..WorldRoster::new(&d, "vale") };
    assert_eq!(at(0, Vec::new()).entry(&cat, "wolf").map(|u| u.level), Some(BASE));
    assert_eq!(at(2, Vec::new()).entry(&cat, "wolf").map(|u| u.level), Some(BASE + 2));
    assert_eq!(at(2, vec!["ash".into()]).entry(&cat, "ash").map(|u| u.level), Some(BASE), "an ally keeps its level");
}

/// One fight of `pack` with `line`'s keys: the enemies' (count, max HP of each).
fn foes_of(d: &Data, line: &str) -> (usize, Vec<i32>) {
    let a = parse(d, &format!("feel enc=world:vale/pack seeds=1 {line}")).expect("the line parses");
    let mut r = Runner::new(d, &a);
    let (mut def, mut roster) = rpgkit_world::battle::authored_fight(d, "vale", "pack", 0).expect("the scratch fight");
    roster.foe_levels = i64::from(a.foe_level);
    let last = def.enemies.last().cloned().unwrap();
    def.enemies.extend(std::iter::repeat_n(last, a.foes as usize));
    let s = rpgkit_battle::session::Session::start(d.cat.clone(), def, &roster);
    let base: Vec<i32> = s.w.enemies.iter().map(|&e| s.w.units[e].max_hp).collect();
    // the runner's own fight (the same keys) must play without a problem
    r.fight_world("vale", "pack", 1);
    (base.len(), base)
}

#[test]
fn foes_add_copies_of_the_last_foe_and_foe_hp_scales_through_the_offset() {
    let d = data();
    let (n, hp) = foes_of(&d, "");
    let (n2, _) = foes_of(&d, "foes=2");
    assert_eq!((n, n2), (2, 4));
    // the HP scale, as the runner applies it, and kept by a stat refresh
    let a = parse(&d, "feel enc=world:vale/pack seeds=1 foe_hp=200").unwrap();
    let (def, roster) = rpgkit_world::battle::authored_fight(&d, "vale", "pack", 0).unwrap();
    let mut s = rpgkit_battle::session::Session::start(d.cat.clone(), def, &roster);
    rpgkit_sim::runner::scale_foe_hp(&mut s, a.foe_hp);
    let e = s.w.enemies[0];
    assert_eq!(s.w.units[e].max_hp, hp[0] * 2);
    assert_eq!(s.w.units[e].hp, hp[0] * 2, "a full bar");
    let cat = d.cat.clone();
    s.w.units[e].sync_from_stats_and_gear(&cat);
    assert_eq!(s.w.units[e].max_hp, hp[0] * 2, "a stat refresh keeps the scale");
}
