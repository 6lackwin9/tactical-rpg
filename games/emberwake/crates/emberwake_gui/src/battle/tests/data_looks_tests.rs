//! PR5-33 (LAYER-UP-DATA) over the SHIPPED catalog: the data's look and clip values resolve (PR5-53: a look key to an
//! effect preset row of its role; a typo would draw the neutral look), and the looks the GUI's id tables gave before
//! PR5-33 are pinned (weapon families by their blow, shot and look at range; the skills the tables named by their look
//! and clip; PR5-53 pass 2: each look the frozen library's, `fx::library`), with the rank rows inheriting their skill's
//! look (the intended change: the tables matched rank 1 ids only).

use crate::battle::tokens::action_clips;
use crate::fx::*;
use emberwake_data::looks::FxRole;
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

/// PR5-53: whether the data's effect presets hold a row of role `role` for look key `key`.
fn known(d: &Data, role: FxRole, key: &str) -> bool {
    d.cat.fx_presets.get(key).is_some_and(|p| p.role == role)
}

/// PR5-53 pass 2: the frozen library's look of key `key` (the look the old tables drew, `fx::library`).
fn lk(key: &str) -> Style {
    library::look(key)
}

/// An `Act` over the shipped catalog's effect presets.
fn act<'a>(d: &'a emberwake_data::Catalog, kind: &'a str, fx: &'a str, family: Option<&'a emberwake_data::looks::FamilyLook>, ranged: bool, dist: i32) -> Act<'a> {
    Act { presets: Some(&d.fx_presets), ..Act::new(kind, fx, family, ranged, dist) }
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
        assert!(s.fx.is_empty() || known(&d, FxRole::Strike, &s.fx) || known(&d, FxRole::Support, &s.fx), "skill {}: fx '{}' is no strike or support look", s.id, s.fx);
        assert!(s.clip.is_empty() || clips.contains(&s.clip), "skill {}: clip '{}' is no rig clip", s.id, s.clip);
    }
    let families = weapon_families(&d);
    assert!(!families.is_empty());
    for (id, ranged) in &families {
        let f = d.cat.family_looks.get(id).unwrap_or_else(|| panic!("family {id} has its look row"));
        let role = if *ranged { FxRole::Shot } else { FxRole::Blow };
        assert!(f.fx.is_empty() || known(&d, role, &f.fx), "family {id}: fx '{}' is no {} look", f.fx, if *ranged { "shot" } else { "blow" });
        assert!(f.fx_far.is_empty() || known(&d, FxRole::Strike, &f.fx_far), "family {id}: fx_far '{}' is no strike look", f.fx_far);
        for c in f.attack_clips.iter().chain(&f.area_clips) {
            assert!(clips.contains(c), "family {id}: clip '{c}' is no rig clip");
        }
    }
}

/// GEAR-REWORK1 (PR5-51b, `docs/sessions/U3_GEAR_REWORK.md` 2-4): the WEAPON-type families the rework added (a look
/// row each, checked above) and the ones it retired.
const NEW_FAMILIES: [&str; 5] = ["greatsword", "knife", "gauntlet", "ammo", "cloak"];
const RETIRED_FAMILIES: [&str; 3] = ["axe", "mace", "wand"];

/// The looks the id tables gave before PR5-33, now from the shipped data.
#[test]
fn the_shipped_data_gives_the_old_tables_looks() {
    let d = shipped();
    let cat = &d.cat;
    // family id -> (strike adjacent, strike 3 cells away); ranged by the family's engine rule. GEAR-REWORK1 (PR5-51b):
    // the retired families (axe, mace, wand) left the table; the new ones (`NEW_FAMILIES`) were never in the id tables
    let families: [(&str, Style, Style); 14] = [
        ("sword", lk("slash"), lk("reach_bolt")),
        ("hammer", lk("bludgeon"), lk("reach_bolt")),
        ("fist", lk("fists"), lk("reach_bolt")),
        ("dagger", lk("stab"), lk("reach_bolt")),
        ("bow", lk("arrow"), lk("arrow")),
        ("staff", lk("bludgeon"), lk("arcane_orb")),
        ("orb", lk("slash"), lk("arcane_orb")),
        ("tome", lk("slash"), lk("arcane_orb")),
        ("crossbow", lk("bolt"), lk("bolt")),
        ("polearm", lk("slash"), lk("reach_bolt")),
        ("scourge", lk("slash"), lk("reach_bolt")),
        ("censer", lk("slash"), lk("reach_bolt")),
        ("bone_saw", lk("slash"), lk("reach_bolt")),
        ("sprayer", lk("arrow"), lk("arrow")),
    ];
    let ranged: Vec<(String, bool)> = weapon_families(&d);
    for (id, _) in &ranged {
        assert!(families.iter().any(|(f, _, _)| f == id) || NEW_FAMILIES.contains(&id.as_str()), "weapon family {id} is pinned or new in PR5-51b");
    }
    for id in RETIRED_FAMILIES {
        assert!(!ranged.iter().any(|(f, _)| f == id), "{id} is retired");
    }
    for (id, close, far) in families {
        let r = ranged.iter().find(|(f, _)| f == id).unwrap_or_else(|| panic!("{id} ships")).1;
        let fl = family_look(cat, id);
        assert_eq!(style(act(cat, "attack", "", fl, r, 1)), close, "{id} adjacent");
        assert_eq!(style(act(cat, "attack", "", fl, r, 3)), far, "{id} at 3 cells");
    }
    let bare = family_look(cat, "");
    assert_eq!(style(act(cat, "attack", "", bare, false, 1)), lk("fists"), "no weapon: the fists");
    assert_eq!(action_clips("", "attack", bare), ["jab", "attack"], "no weapon jabs");
    assert_eq!(action_clips("", "area", bare), ["sweep_kick", "kick", "jab", "attack"], "and sweeps an area");
    assert_eq!(action_clips("", "attack", family_look(cat, "sword")), ["attack"]);
    // skill id -> (command kind, look, clips by a sword)
    let skills: [(&str, &str, Style, &[&str]); 21] = [
        ("arcane_bolt", "attack", lk("arcane_orb"), &["attack"]),
        ("shield_bash", "attack", lk("bash_ring"), &["shield_bash", "attack"]),
        ("hex", "attack", lk("curse_orb"), &["attack"]),
        ("arc_lance", "area", lk("beam"), &["attack"]),
        ("firebomb", "area", lk("fire_lob"), &["attack"]),
        ("inferno", "area", lk("fire_lob"), &["attack"]),
        ("glacial_burst", "area", lk("frost_lob"), &["attack"]),
        ("smite", "attack", lk("falling_orb"), &["attack"]),
        ("judgement", "attack", lk("falling_orb"), &["attack"]),
        ("volley", "area", lk("arrow_volley"), &["attack"]),
        ("shockwave", "area", lk("ground_wave"), &["attack"]),
        ("sweep_kick", "area", lk("sweep"), &["sweep_kick", "attack"]),
        ("cyclone_kick", "area", lk("sweep"), &["sweep_kick", "attack"]),
        ("haymaker", "attack", lk("heavy_blow"), &["haymaker", "attack"]),
        ("crushing_blow", "attack", lk("heavy_blow"), &["attack"]),
        ("purify", "support", lk("cleanse_motes"), &["cast"]),
        ("aegis", "support", lk("ward_ring"), &["cast"]),
        ("rally", "support", lk("rally_ring"), &["cast"]),
        ("taunt_shout", "support", lk("rally_ring"), &["cast"]),
        ("offhand_jab", "attack", lk("slash"), &["jab", "attack"]),
        ("caustic_vomit", "attack", lk("slash"), &["spit", "attack"]),
    ];
    let sword = family_look(cat, "sword");
    let skill = |id: &str| cat.skills.iter().find(|s| s.id == id).unwrap_or_else(|| panic!("skill {id} ships"));
    for (id, kind, look, clips) in skills {
        let s = skill(id);
        assert_eq!(style(act(cat, kind, &s.fx, sword, false, 1)), look, "{id}");
        assert_eq!(action_clips(&s.clip, kind, sword), clips, "{id} clips");
    }
    assert_eq!(style(act(cat, "support", &skill("mend").fx, None, false, 1)), lk("heal_motes"), "any other support: the mend motes");
    // PR5-33's intended change: a rank row inherits its skill's look and clip (CLASSPASS1-DATA flattened all shipped
    // ranks, so verify inheritance through source expansion on a scratch copy of shield_bash with a synthetic rank 2).
    let tmp = std::env::temp_dir().join("emberwake_data_looks_rank");
    let _ = std::fs::remove_dir_all(&tmp);
    fn copy_dir(from: &Path, to: &Path) {
        std::fs::create_dir_all(to).unwrap();
        for e in std::fs::read_dir(from).unwrap() {
            let p = e.unwrap().path();
            let dst = to.join(p.file_name().unwrap());
            if p.is_dir() { copy_dir(&p, &dst); } else { std::fs::copy(&p, &dst).unwrap(); }
        }
    }
    copy_dir(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder"), &tmp);
    let mut sb = emberwake_content::store::view(&tmp).get("skills", "shield_bash").clone();
    sb.insert("ranks".into(), serde_json::json!([{"power_mult": 0.9}]));
    emberwake_content::store::upsert(&tmp, None, "skills", &sb, None).expect("upsert synthetic rank");
    let ranked_cat = emberwake_data::Catalog::from_value(&emberwake_content::source::load(&tmp).catalog);
    let _ = std::fs::remove_dir_all(&tmp);
    let second = ranked_cat.skill("shield_bash_2").expect("synthetic shield_bash_2 expanded");
    assert_eq!((second.family.as_str(), second.name.as_str()), ("shield_bash", "Shield Bash II"));
    assert_eq!(style(act(&ranked_cat, "attack", &second.fx, sword, false, 1)), lk("bash_ring"));
    assert_eq!(action_clips(&second.clip, "attack", sword), ["shield_bash", "attack"]);
}
