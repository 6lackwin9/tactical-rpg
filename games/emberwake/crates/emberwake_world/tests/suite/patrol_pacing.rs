//! PR7-42 (PACE1, PT-18) on the shipped data (read only): a zone binding's patrol pacing (`explore::VISIT_CAP`,
//! `REST_BEATEN`, `REPEAT_XP_PCT`). A party pacing two danger tiles of a paced Act 1 location meets no patrol before
//! the binding's `steps_min`, at most `visit_cap` patrols in one visit (a 120-step walk) and a new count on the next
//! visit; a beaten patrol rests until the next day; a repeat win pays the binding's `repeat_xp_pct` of the full XP and
//! nothing past the list. Every number comes from the bindings; no location, tile or encounter is named.

use aegis_core::json::{self, Dict};
use emberwake_world::explore::{self, Pos};
use emberwake_world::{battle, new_game, state, Data, GameState};
use std::path::Path;

const WORLD: &str = "sunk_ward";
const WALK: usize = 120;

fn data() -> Data {
    Data::open(&Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder")).unwrap().0
}

/// A paced location of the world (by rule: the first whose bindings set a visit cap) and two side-by-side open tiles
/// under one paced binding, clear of every marker (roamers on any step) by more than its radius, plus that binding.
fn paced_spot(d: &Data) -> (String, Pos, Pos, Dict) {
    let gs = new_game::create_for_world(d, WORLD, 1).expect("a new game");
    for loc in d.catalogs.own_entries(WORLD, "locations") {
        let id = json::s(loc, "id");
        let loc = explore::resolve_location(d, &gs, &id).clone();
        let t = explore::template_of(d, &loc).clone();
        if t.is_empty() {
            continue;
        }
        let near: Vec<(Pos, i64)> = explore::markers_of(&loc).iter().flat_map(|m| (0..200).map(move |k| (m.clone(), k))).map(|(m, k)| (explore::marker_pos(&t, &m, k), json::i(&m, "radius", 1).max(1) + 2)).collect();
        let clear = |p: Pos| near.iter().all(|(q, r)| (p.0 - q.0).abs() + (p.1 - q.1).abs() > *r);
        let (w, h) = explore::size_of(&t);
        for y in 0..h {
            for x in 0..w - 1 {
                let (a, b) = ((x, y), (x + 1, y));
                let (ba, bb) = (explore::binding_at(&loc, &t, a), explore::binding_at(&loc, &t, b));
                if explore::is_open(&t, a.0, a.1) && explore::is_open(&t, b.0, b.1) && json::i(&ba, explore::VISIT_CAP, 0) > 0 && ba == bb && clear(a) && clear(b) {
                    return (id, a, b, ba);
                }
            }
        }
    }
    panic!("{WORLD} has a paced danger zone with two open tiles side by side");
}

/// A new game standing on `a` in location `id` (entered at its spawn, then placed).
fn standing(d: &Data, id: &str, a: Pos) -> GameState {
    let mut gs = new_game::create_for_world(d, WORLD, 1).expect("a new game");
    assert!(json::b(&explore::enter(d, &mut gs, id, explore::SPAWN), "ok", false), "{id}: entering works");
    gs.position = a;
    gs
}

/// Paces between `a` and `b` for `n` steps; each counter fight ends `outcome`. Returns the step numbers of the
/// counter fights and their encounters.
fn pace(d: &Data, gs: &mut GameState, a: Pos, b: Pos, n: usize, outcome: &str) -> Vec<(usize, String)> {
    let mut fights = Vec::new();
    for i in 1..=n {
        let to = if gs.position == a { b } else { a };
        let dir = (to.0 - gs.position.0, to.1 - gs.position.1);
        let r = explore::step(d, gs, dir);
        assert!(json::b(&r, "ok", false), "step {i}: {:?}", json::sget(&r, "errors"));
        for e in json::aget(&r, "events") {
            let e = json::dict(e);
            if json::s(e, "kind") != "fight" {
                continue;
            }
            assert_eq!(json::s(e, "source"), "counter", "step {i}: only counter fights on these tiles");
            fights.push((i, json::s(e, "encounter")));
            assert!(json::b(&explore::after_fight(d, gs, e, outcome, None), "ok", false), "step {i}: the fight settles");
        }
    }
    fights
}

#[test]
fn a_120_step_walk_meets_at_most_the_visit_cap_and_none_before_steps_min() {
    let d = data();
    let (id, a, b, bind) = paced_spot(&d);
    let (cap, lo, hi) = (json::i(&bind, explore::VISIT_CAP, 0), json::i(&bind, "steps_min", 0), json::i(&bind, "steps_max", 0));
    let mut gs = standing(&d, &id, a);
    let fights = pace(&d, &mut gs, a, b, WALK, "fled");
    println!("{id}: cap {cap}, steps {lo}..{hi}: patrols at {fights:?} over {WALK} steps");
    assert!(fights.len() as i64 <= cap, "{id}: at most {cap} patrols a visit: {fights:?}");
    if WALK as i64 >= cap * hi {
        assert_eq!(fights.len() as i64, cap, "{id}: {WALK} steps meet the cap of {cap} (steps_max {hi}): {fights:?}");
    }
    let mut last = 0;
    for (at, _) in &fights {
        assert!((*at - last) as i64 >= lo, "{id}: patrols come at least {lo} steps apart: {fights:?}");
        last = *at;
    }
    assert!(state::explore_errors(&gs.explore).is_empty(), "the patrol record saves: {:?}", gs.explore);
    // a new visit counts again
    let before = gs.position;
    assert!(json::b(&explore::enter(&d, &mut gs, &id, explore::SPAWN), "ok", false));
    gs.position = before;
    let again = pace(&d, &mut gs, a, b, hi as usize + 1, "fled");
    assert_eq!(again.len(), 1, "{id}: the next visit meets a patrol within steps_max {hi}: {again:?}");
}

#[test]
fn a_beaten_patrol_rests_until_the_next_day() {
    let d = data();
    let (id, a, b, bind) = paced_spot(&d);
    assert!(json::b(&bind, explore::REST_BEATEN, false), "{id}: the paced binding rests beaten patrols");
    let mut gs = standing(&d, &id, a);
    let hi = json::i(&bind, "steps_max", 0) as usize;
    let first = pace(&d, &mut gs, a, b, hi, "won");
    let (_, qid) = first.first().cloned().expect("a patrol within steps_max");
    assert!(explore::patrol_resting(&gs, &qid), "{qid} rests after its win");
    // the rest of the day (new visits reset the cap): the beaten encounter never comes back
    for _ in 0..3 {
        assert!(json::b(&explore::enter(&d, &mut gs, &id, explore::SPAWN), "ok", false));
        gs.position = a;
        let more = pace(&d, &mut gs, a, b, 2 * hi, "won");
        assert!(more.iter().all(|(_, q)| *q != qid), "{qid} does not return the same day: {more:?}");
    }
    gs.time += 1;
    assert!(!explore::patrol_resting(&gs, &qid), "{qid} may come again the next day");
}

#[test]
fn patrol_xp_decays_by_repeat_to_zero_after_the_list() {
    let d = data();
    let (id, a, _, bind) = paced_spot(&d);
    let pcts: Vec<i64> = json::aget(&bind, explore::REPEAT_XP_PCT).iter().map(json::to_int).collect();
    assert!(!pcts.is_empty(), "{id}: the paced binding sets repeat_xp_pct");
    let mut gs = standing(&d, &id, a);
    let qid = json::s(d.resolve(WORLD, "encounters", &json::gstr(&json::arr(&json::aget(&bind, "encounters")[0])[0])), "_qid");
    let fight: Dict = json::obj([("kind", "fight".into()), ("source", "counter".into()), ("encounter", qid.clone().into()), ("marker", "".into()), ("elite", false.into())]);
    let member = gs.party.members[gs.party.active_indexes()[0]].id.clone();
    let mut paid = Vec::new();
    for win in 0..pcts.len() + 2 {
        let mut full = gs.clone();
        let want = json::i(json::dget(json::dget(&battle::settle(&d, &mut full, &qid, "won", None), "xp"), &member), "xp", -1);
        let got = json::i(json::dget(json::dget(&explore::after_fight(&d, &mut gs, &fight, "won", None), "xp"), &member), "xp", -1);
        let pct = if win == 0 { 100 } else { pcts.get(win - 1).copied().unwrap_or(0) };
        assert!(want > 0, "win {win}: the patrol pays XP in full ({want})");
        assert_eq!(got, want * pct / 100, "win {win}: {pct}% of {want}");
        assert_eq!(explore::patrol_wins(&gs, &qid), win as i64 + 1);
        paid.push(got);
    }
    println!("{id} {qid}: XP by win {paid:?} (repeat pcts {pcts:?})");
    assert_eq!(*paid.last().unwrap(), 0, "a repeat past the list pays nothing");
}
