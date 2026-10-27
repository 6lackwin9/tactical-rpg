//! ENCOUNTER2-H S2 on the Ashlands classic (the Godot export in parity/out): the last-round verdict moved from the battle
//! screen (BattleBridge.outcome: the session's verdict, else "fled" once no round follows) into
//! `Session::final_outcome`. Every catalog encounter of the export, played to its end with the Balanced preset (as
//! authored, and cut to 1 and 2 rounds so the round limit decides too), gets the same outcome from the session as the
//! screen's old rule gives after every round, holds no gate and no unkillable unit, and its timeout verdict is fled.

use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_data::{Catalog, UnitDef};
use indexmap::IndexMap;
use std::path::PathBuf;
use std::sync::Arc;

/// The battle screen's rule before ENCOUNTER2-H (emberwake_gui battle.rs `outcome`), written out here.
fn screen_rule(s: &Session) -> String {
    if !s.outcome.is_empty() {
        return s.outcome.clone();
    }
    let standing = s.w.allies.iter().any(|a| s.w.units[*a].is_active());
    let alive = s.w.allies.iter().any(|a| s.w.units[*a].is_alive());
    if s.combat_over {
        return if standing { "won" } else if alive { "fled" } else { "lost" }.into();
    }
    if s.phase == Phase::Replay && !s.can_advance_round() {
        return "fled".into();
    }
    String::new()
}

#[test]
fn every_classic_encounter_ends_as_the_screen_ruled() {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../parity/out/catalog.json");
    let Ok(text) = std::fs::read_to_string(&p) else {
        eprintln!("no {}: run games/emberwake/parity/export.sh", p.display());
        return;
    };
    let cat = Arc::new(Catalog::from_value(&serde_json::from_str(&text).unwrap()));
    let mut ids: Vec<String> = cat.encounters.keys().cloned().collect();
    ids.sort();
    assert!(!ids.is_empty());
    let (mut fights, mut timeouts) = (0, 0);
    for (id, cap) in ids.iter().flat_map(|id| [(id, 0), (id, 1), (id, 2)]) {
        let mut def = EncounterDef::from_obj(&cat.encounters[id]);
        if cap > 0 {
            def.max_rounds = cap;
        }
        fights += 1;
        let mut s = Session::start(cat.clone(), def, &IndexMap::<String, UnitDef>::new());
        assert!(s.w.hold.is_none() && s.w.unkillable.is_empty() && s.stay.is_none(), "{id}: no stay-behind field in the classic");
        assert_eq!(s.timeout_outcome, "fled", "{id}");
        loop {
            assert_eq!(s.final_outcome(), screen_rule(&s), "{id} round {}", s.current_round);
            s.plan_apply_preset_all("balanced");
            if !s.confirm_plan() {
                break;
            }
            while s.phase != Phase::Replay {
                s.step_next();
            }
            assert_eq!(s.final_outcome(), screen_rule(&s), "{id} round {} over", s.current_round);
            if s.combat_over || !s.can_advance_round() {
                break;
            }
            s.advance_round();
        }
        if s.outcome.is_empty() && s.final_outcome() == "fled" {
            timeouts += 1;
        }
    }
    assert!(timeouts > 0, "the round limit decided some fights");
    eprintln!("{fights} fights of {} classic encounters end as before ({timeouts} by the round limit)", ids.len());
}
