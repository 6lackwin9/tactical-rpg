//! PR5-33 (LAYER-UP-DATA): the game's sim description is data (`games/emberwake/sim.json`, `emberwake_sim::game`), read
//! by the kit (`rpgkit_sim::config`). It parses, and every id it names is in the shipped data: the default and story
//! world are game worlds, the classes catalog classes, the loadout skills and their L7 swaps catalog skills, the feel
//! encounters catalog encounters, the roster units classes (or the generated `@elite` / `@boss`).

use emberwake_world::Data;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

#[test]
fn the_sim_description_names_what_the_data_holds() {
    let g = emberwake_sim::game();
    let d = data();
    assert!(d.catalogs.worlds.contains_key(&g.default_world), "default world {}", g.default_world);
    let story = g.story.as_ref().expect("the game has a story run");
    assert!(d.catalogs.worlds.contains_key(&story.world), "story world {}", story.world);
    assert!(story.branch_flag.starts_with(&format!("{}.", story.world)), "the branch flag is the story world's");
    assert!(!g.feel_encounters.is_empty());
    for e in &g.feel_encounters {
        assert!(d.cat.encounters.contains_key(e), "feel encounter {e}");
    }
    let c = g.classes;
    let class = |id: &str| d.cat.classes.contains_key(id);
    let skill = |id: &str| d.cat.skill(id).is_some();
    for id in c.ids.iter().chain(c.default_comp).chain([&c.trio_lead, &c.elite, &c.boss, &c.support_baseline]) {
        assert!(class(id), "class {id}");
    }
    for r in c.roles {
        assert!(class(r.class), "role row class {}", r.class);
        for (role, skills) in r.loadouts {
            assert!(skills.iter().all(|s| skill(s)), "{} {role}: {skills:?}", r.class);
        }
        for (role, from, to) in r.l7_swaps {
            assert!(skill(from) && skill(to), "{} {role}: {from} -> {to}", r.class);
        }
    }
    let unit = |u: &(&str, &str)| class(u.0) || u.0.starts_with('@');
    for g in c.generated {
        assert!(g.enemies.iter().chain(g.wave).all(unit), "generated {}", g.id);
    }
    for s in c.scenarios {
        assert!(s.allies.iter().all(|a| class(a)) && s.enemies.iter().all(unit), "scenario {}", s.id);
    }
    assert!(c.s2_enemies.iter().all(unit));
    assert_eq!(emberwake_sim::game().default_world, g.default_world, "read once, the same each call");
}
