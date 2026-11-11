//! PR5-33 (LAYER-UP-DATA) over the SHIPPED catalog: the data's look and clip values resolve in the screens' libraries
//! (a typo would fall back silently), and the looks the GUI's id tables gave before PR5-33 are pinned (weapon families
//! by their blow, shot and look at range; the skills the tables named by their look and clip), with the rank rows
//! inheriting their skill's look (the intended change: the tables matched rank 1 ids only).

use crate::battle::tokens::action_clips;
use crate::fx::*;
use emberwake_world::Data;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn shipped() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// The clip ids of every body's `clips.json` in the game, kit and engine asset layers (`units/**`).
fn rig_clips() -> BTreeSet<String> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../..");
    let mut dirs: Vec<PathBuf> = ["games/emberwake/assets/units", "kits/rpg/assets/units", "engine/assets/units"].iter().map(|d| root.join(d)).collect();
    let mut out = BTreeSet::new();
    while let Some(d) = dirs.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                dirs.push(p);
            } else if p.file_name().is_some_and(|n| n == "clips.json") {
                let v: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&p).unwrap()).unwrap();
                out.extend(v["clips"].as_array().into_iter().flatten().filter_map(|c| c["id"].as_str().map(String::from)));
            }
        }
    }
    out
}

fn known(table: &[(&str, Style)], key: &str) -> bool {
    table.iter().any(|(k, _)| *k == key)
}

/// The weapon families of the shipped data with whether their engine rules make them always ranged.
fn weapon_families(d: &Data) -> Vec<(String, bool)> {
    let rows = d.catalogs.content.get("item_families").expect("item families");
    rows.values()
        .filter(|r| r.get("item_type").and_then(|v| v.as_str()) == Some("WEAPON"))
        .map(|r| {
            let id = r["id"].as_str().unwrap().to_string();
            let ranged = r.get("engine_rules").and_then(|v| v.as_array()).is_some_and(|a| a.iter().any(|x| x == "always_ranged"));
            (id, ranged)
        })
        .collect()
}

#[test]
fn every_shipped_look_and_clip_value_names_a_known_key() {
    let d = shipped();
    let clips = rig_clips();
    assert!(clips.contains("attack") && clips.contains("jab"), "the rigs were read ({} clips)", clips.len());
    for s in &d.cat.skills {
        assert!(s.fx.is_empty() || known(&STRIKES, &s.fx) || known(&SUPPORTS, &s.fx), "skill {}: fx '{}' is no strike or support look", s.id, s.fx);
        assert!(s.clip.is_empty() || clips.contains(&s.clip), "skill {}: clip '{}' is no rig clip", s.id, s.clip);
    }
    let families = weapon_families(&d);
    assert!(!families.is_empty());
    for (id, ranged) in &families {
        let f = d.cat.family_looks.get(id).unwrap_or_else(|| panic!("family {id} has its look row"));
        let table: &[(&str, Style)] = if *ranged { &SHOTS } else { &BLOWS };
        assert!(f.fx.is_empty() || known(table, &f.fx), "family {id}: fx '{}' is no {} look", f.fx, if *ranged { "shot" } else { "blow" });
        assert!(f.fx_far.is_empty() || known(&STRIKES, &f.fx_far), "family {id}: fx_far '{}' is no strike look", f.fx_far);
        for c in f.attack_clips.iter().chain(&f.area_clips) {
            assert!(clips.contains(c), "family {id}: clip '{c}' is no rig clip");
        }
    }
}

/// The looks the id tables gave before PR5-33, now from the shipped data.
#[test]
fn the_shipped_data_gives_the_old_tables_looks() {
    let d = shipped();
    let cat = &d.cat;
    // family id -> (strike adjacent, strike 3 cells away); ranged by the family's engine rule
    let families: [(&str, Style, Style); 17] = [
        ("sword", SLASH, REACH_BOLT),
        ("axe", CLEAVE, REACH_BOLT),
        ("mace", BLUDGEON, REACH_BOLT),
        ("hammer", BLUDGEON, REACH_BOLT),
        ("fist", FISTS, REACH_BOLT),
        ("dagger", STAB, REACH_BOLT),
        ("bow", ARROW, ARROW),
        ("staff", BLUDGEON, ARCANE_BOLT),
        ("wand", SLASH, ARCANE_BOLT),
        ("orb", SLASH, ARCANE_BOLT),
        ("tome", SLASH, ARCANE_BOLT),
        ("crossbow", BOLT, BOLT),
        ("polearm", SLASH, REACH_BOLT),
        ("scourge", SLASH, REACH_BOLT),
        ("censer", SLASH, REACH_BOLT),
        ("bone_saw", SLASH, REACH_BOLT),
        ("sprayer", ARROW, ARROW),
    ];
    let ranged: Vec<(String, bool)> = weapon_families(&d);
    assert_eq!(ranged.len(), families.len(), "every weapon family is pinned");
    for (id, close, far) in families {
        let r = ranged.iter().find(|(f, _)| f == id).unwrap_or_else(|| panic!("{id} ships")).1;
        let fl = family_look(cat, id);
        assert_eq!(style(Act::new("attack", "", fl, r, 1)), close, "{id} adjacent");
        assert_eq!(style(Act::new("attack", "", fl, r, 3)), far, "{id} at 3 cells");
    }
    let bare = family_look(cat, "");
    assert_eq!(style(Act::new("attack", "", bare, false, 1)), FISTS, "no weapon: the fists");
    assert_eq!(action_clips("", "attack", bare), ["jab", "attack"], "no weapon jabs");
    assert_eq!(action_clips("", "area", bare), ["sweep_kick", "kick", "jab", "attack"], "and sweeps an area");
    assert_eq!(action_clips("", "attack", family_look(cat, "sword")), ["attack"]);
    // skill id -> (command kind, look, clips by a sword)
    let skills: [(&str, &str, Style, &[&str]); 21] = [
        ("arcane_bolt", "attack", ARCANE_BOLT, &["attack"]),
        ("shield_bash", "attack", SHIELD_BASH, &["shield_bash", "attack"]),
        ("hex", "attack", HEX, &["attack"]),
        ("arc_lance", "area", ARC_LANCE, &["attack"]),
        ("firebomb", "area", FIREBOMB, &["attack"]),
        ("inferno", "area", FIREBOMB, &["attack"]),
        ("glacial_burst", "area", GLACIAL_BURST, &["attack"]),
        ("smite", "attack", SMITE, &["attack"]),
        ("judgement", "attack", SMITE, &["attack"]),
        ("volley", "area", ARROW_VOLLEY, &["attack"]),
        ("shockwave", "area", SHOCKWAVE, &["attack"]),
        ("sweep_kick", "area", SWEEP, &["sweep_kick", "attack"]),
        ("cyclone_kick", "area", SWEEP, &["sweep_kick", "attack"]),
        ("haymaker", "attack", HAYMAKER, &["haymaker", "attack"]),
        ("crushing_blow", "attack", HAYMAKER, &["attack"]),
        ("purify", "support", PURIFY, &["cast"]),
        ("aegis", "support", AEGIS, &["cast"]),
        ("rally", "support", RALLY, &["cast"]),
        ("taunt_shout", "support", RALLY, &["cast"]),
        ("offhand_jab", "attack", SLASH, &["jab", "attack"]),
        ("caustic_vomit", "attack", SLASH, &["spit", "attack"]),
    ];
    let sword = family_look(cat, "sword");
    let skill = |id: &str| cat.skills.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("skill {id} ships"));
    for (id, kind, look, clips) in skills {
        let s = skill(id);
        assert_eq!(style(Act::new(kind, &s.fx, sword, false, 1)), look, "{id}");
        assert_eq!(action_clips(&s.clip, kind, sword), clips, "{id} clips");
    }
    assert_eq!(style(Act::new("support", &skill("mend").fx, None, false, 1)), MEND, "any other support: the mend motes");
    // PR5-33's intended change: a rank row inherits its skill's look and clip (the tables matched rank 1 ids only)
    let second = skill("shield_bash_2");
    assert_eq!((second.family.as_str(), second.name.as_str()), ("shield_bash", "Shield Bash II"));
    assert_eq!(style(Act::new("attack", &second.fx, sword, false, 1)), SHIELD_BASH);
    assert_eq!(action_clips(&second.clip, "attack", sword), ["shield_bash", "attack"]);
}
