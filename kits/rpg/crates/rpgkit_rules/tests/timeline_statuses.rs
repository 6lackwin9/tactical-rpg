//! The timeline's status rows and the intent tag (RULESET1-S, docs/specs/RULESET1_TASKS.md 1.7): `steadfast` and
//! `stealth` are buffs as the table row gives them, `intent_bound` is true for riposte and stealth only (RULESET1_INTENT
//! IN6), the grant lists name them under their grant kind, and a status's intent tag lives exactly as long as the
//! status: removed with it, dropped at round roll-over when the status ends, kept while it runs. Classic code paths
//! never set a tag. Scratch catalog only.

use rpgkit_data::Catalog;
use rpgkit_rules::status::{self, Stacking};
use rpgkit_rules::{build_unit, Unit};
use serde_json::json;

fn unit() -> Unit {
    let cat = Catalog::from_value(&json!({"units": [{"id": "dummy", "name": "Dummy", "stat_mode": "fixed", "hp": 40}]}));
    build_unit(&cat, &cat.units["dummy"], 10)
}

#[test]
fn steadfast_and_stealth_are_the_rows_the_spec_gives() {
    let steadfast = status::row_of("steadfast").expect("row 16");
    assert_eq!((steadfast.buff, steadfast.resistable, steadfast.default_rounds, steadfast.stacking, steadfast.cap, steadfast.grant), (true, false, 1, Stacking::RefreshMaxDuration, 0, "support"));
    let stealth = status::row_of("stealth").expect("row 17");
    assert_eq!((stealth.buff, stealth.resistable, stealth.default_rounds, stealth.stacking, stealth.cap, stealth.grant, stealth.display), (true, false, 2, Stacking::RefreshMaxDuration, 0, "defend", "Stealthed"));
    assert_eq!(status::ROWS.iter().position(|r| r.id == "steadfast"), Some(15), "appended after the fifteen classic rows");
    assert_eq!(status::ROWS.iter().position(|r| r.id == "stealth"), Some(16));
    let bound: Vec<&str> = status::ROWS.iter().filter(|r| r.intent_bound).map(|r| r.id).collect();
    assert_eq!(bound, ["riposte", "stealth"], "IN6: only these rows are intent-bound");
    assert!(status::intent_bound("riposte") && !status::intent_bound("haste") && !status::intent_bound("nothing"));
    assert!(status::defend_buffs().contains(&"stealth") && !status::defend_buffs().contains(&"steadfast"));
    assert!(status::buffs().contains(&"steadfast") && !status::debuffs().contains(&"stealth"));
    assert!(!status::has_magnitude("steadfast") && !status::has_magnitude("stealth"));
}

#[test]
fn an_intent_tag_lives_as_long_as_its_status() {
    let mut u = unit();
    // a classic grant sets no tag
    status::grant_now(&mut u, "riposte", 2, 0, "");
    assert!(u.st.intents.is_empty(), "classic never tags");
    // a running tagged status keeps its tag across a roll-over; an ended one loses it
    status::grant_now(&mut u, "stealth", 2, 0, "");
    u.st.intents.insert("stealth".into(), "guard".into());
    status::grant_now(&mut u, "steadfast", 1, 0, "");
    u.st.intents.insert("steadfast".into(), "support".into());
    status::roll_over(&mut u, 0);
    assert!(u.st.active.contains("stealth") && !u.st.active.contains("steadfast"), "stealth runs 2 rounds, steadfast 1");
    assert_eq!(u.st.intents.keys().cloned().collect::<Vec<_>>(), ["stealth"]);
    // removed with the status
    u.remove_status("stealth");
    assert!(u.st.intents.is_empty());
    // a queued (pending) instance keeps the tag when only the active one is consumed
    status::grant_now(&mut u, "haste", 1, 0, "");
    status::queue(&mut u, "haste", 1, 0, "");
    u.st.intents.insert("haste".into(), "support".into());
    status::remove_active(&mut u, "haste");
    assert_eq!(u.st.intents.get("haste").map(String::as_str), Some("support"), "the queued instance still carries it");
    status::roll_over(&mut u, 0);
    assert!(u.st.active.contains("haste") && u.st.intents.contains_key("haste"), "the queued instance is active now, tagged");
}
