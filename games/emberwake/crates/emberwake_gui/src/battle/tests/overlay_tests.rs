//! RULESET1-U slice 3 (RULESET1_INTENT UI3, RULESET1 K6, GD5, MV1): the board overlays under the round intent. A catalog
//! encounter played by the catalog's timeline row; every expectation is a kit query (`target_problem`, `attack_band`,
//! `in_front`, `facing`, `hidden`, the compiled plan), never a shipped number. Classic draws none (no state key).

use crate::battle::overlays::{overlay_json, overlay_of, reach_text, strike_tile, MoveKind, Overlay, FRONT_RADIUS};
use crate::battle::BattleScreen;
use crate::battle::tips::unit_tip;
use crate::screens::{Cx, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::grid::{V2, GRID_H, GRID_W};
use emberwake_battle::round_intent::{self as ri, RoundIntent, RoundTarget};
use emberwake_battle::session::Session;
use aegis_ui::art::Art;
use emberwake_battle::skill::Kind;
use serde_json::json;
use emberwake_rules::ruleset::TIMELINE;

fn arena(ruleset: Option<&str>) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let b = BattleScreen::arena_with(&cx, None, &encounter, ruleset.map(String::from), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn overlay(b: &BattleScreen) -> Overlay {
    overlay_of(s(b), b.selected, true).expect("the round intent's overlays")
}

/// The hero and its first foe on the field, the hero set to Assault on that foe through the command API.
fn assault(cx: &mut Cx, b: &mut BattleScreen) -> (usize, usize) {
    let h = b.selected.expect("a hero");
    let foe = s(b).w.enemies.iter().copied().find(|&e| s(b).w.units[e].is_active() && s(b).w.has(e)).expect("a foe");
    let r = b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    (h, foe)
}

/// Classic: no overlay, no state key, the hover card's Reach is the unit's attack range as before.
#[test]
fn classic_draws_no_overlay() {
    let (mut cx, mut g, mut b) = arena(None);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(overlay_of(s(&b), b.selected, true).is_none());
    assert!(b.state(&cx).get("overlay").is_none(), "a classic battle's state keeps its shape");
    let w = &s(&b).w;
    for i in 0..w.units.len() {
        assert_eq!(reach_text(w, i), w.units[i].attack_range.to_string());
    }
}

/// UI1 (valid cells lit), UI3 (the target ring, move paths), K6 (the reach band), MV1 (enemy facings): under Assault
/// on a foe the ring sits on the foe; the lit units are exactly those the intent table takes (the target left out);
/// the band is every tile at the plain attack band's distance from the tile the first attack strikes from; the moves
/// follow the compiled plan tile by tile from the hero; every enemy with a facing shows it. The state lists the same.
#[test]
fn the_overlays_follow_the_kit() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    // the slice 3 review: the hero starts 3 or 4 tiles from its foe, so the plan walks 2 tiles or more
    {
        let h = b.selected.unwrap();
        let w = &s(&b).w;
        let foe = w.enemies.iter().copied().find(|&e| w.units[e].is_active() && w.has(e)).unwrap();
        let far = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).find(|c| w.grid.is_valid(*c) && w.occupant(*c).is_none() && (3..=4).contains(&c.dist(w.pos(foe))) && c.y == w.pos(foe).y).expect("a free tile 3-4 tiles from the foe on its row");
        b.host.session.as_mut().unwrap().w.grid.positions[h] = Some(far);
    }
    let (h, foe) = assault(&mut cx, &mut b);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let o = overlay(&b);
    let w = &s(&b).w;
    assert!(o.moves.iter().any(|(t, k)| *k == MoveKind::Walk && t.len() >= 3), "a walk of 2+ tiles: {:?}", o.moves);
    assert_eq!(o.target, Some(w.pos(foe)), "the ring on the round target");
    let valid: Vec<V2> = (0..w.units.len())
        .filter(|&u| u != foe && w.units[u].is_active() && w.has(u) && ri::target_problem(w, h, RoundIntent::Assault, ri::normalize(h, RoundTarget::Unit(u))).is_empty())
        .map(|u| w.pos(u))
        .collect();
    assert_eq!(o.valid, valid);
    assert!(o.valid.iter().all(|c| w.occupant(*c).is_some_and(|u| w.foes_of(h).contains(&u))), "Assault lights foes only");
    let st = s(&b).plan.compile(w, h);
    let at = st.entries.iter().find(|e| emberwake_battle::skill::kind(&w.cat.skills[e.skill]) == Kind::Attack).map(|e| e.pos_after).unwrap_or_else(|| w.pos(h));
    let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[h], &w.ruleset);
    let band: Vec<V2> = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && c.dist(at) >= lo && c.dist(at) <= hi).collect();
    assert_eq!(o.band, band, "the band around {at:?}");
    let mut from = w.pos(h);
    let mut ends = Vec::new();
    for e in st.entries.iter().filter(|e| e.pos_after.x >= 0) {
        if e.pos_after != from {
            ends.push((from, e.pos_after));
            from = e.pos_after;
        }
    }
    assert_eq!(o.moves.iter().map(|(t, _)| (t[0], *t.last().unwrap())).collect::<Vec<_>>(), ends, "one drawn move per change of tile");
    assert!(o.moves.iter().all(|(t, _)| t.windows(2).all(|p| p[0] != p[1])));
    // the drawn tiles walk the plan's steps: every one on the board, each a step from the last (a carried move's
    // dash may cover more than one)
    assert!(o.moves.iter().flat_map(|(t, _)| t.iter()).all(|c| w.grid.is_valid(*c)), "{:?}", o.moves);
    assert!(o.moves.iter().filter(|(_, k)| *k == MoveKind::Walk).all(|(t, _)| t.windows(2).all(|p| p[0].dist(p[1]) == 1)), "{:?}", o.moves);
    assert!(o.front.is_empty(), "no front shaded under Assault");
    let facings: Vec<(V2, V2)> = w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.has(e) && ri::facing(w, e) != emberwake_battle::grid::ZERO).map(|e| (w.pos(e), ri::facing(w, e))).collect();
    assert_eq!(o.facings, facings);
    // (VISCLAR1 AR3: with the target arrows the last frame drew)
    let drawn = Overlay { arrows: b.arrows_shown.clone(), ..o.clone() };
    // (INPUT1 PB1: with the impact tracers drawing now: none while planning)
    let mut want = overlay_json(&drawn);
    want["tracers"] = json!([]);
    assert_eq!(b.state(&cx)["overlay"], want, "the agent link's state lists the overlays");
    // the slice 3 review: an attack strikes from after its carried Engage move, from before its Disengage retreat
    let mut e = st.entries[0].clone();
    (e.pos_before, e.pos_after) = (V2::new(1, 1), V2::new(2, 1));
    for (carry, want) in [(emberwake_battle::command::Carry::Retreat(V2::new(2, 1)), e.pos_before), (emberwake_battle::command::Carry::Engage(V2::new(2, 1)), e.pos_after), (emberwake_battle::command::Carry::None, e.pos_after)] {
        e.carry = carry;
        assert_eq!(strike_tile(&e), want, "{carry:?}");
    }
    // playback: no hero overlays, the facings stay
    let r = b.send(&cx, json!({"cmd": "confirm"}));
    assert_eq!(r["ok"], json!(true), "{r}");
    let o = overlay_of(s(&b), b.selected, false).unwrap();
    assert!(o.target.is_none() && o.valid.is_empty() && o.band.is_empty() && o.moves.is_empty());
}

/// UI3, MV3: under Maneuver the target's front within FRONT_RADIUS is shaded, exactly the tiles `in_front` names
/// (dot(tile - target, facing) > 0); no front without a facing.
#[test]
fn maneuver_shades_the_targets_front() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    let (h, foe) = assault(&mut cx, &mut b);
    // the intent stands even when the planner offers no Maneuver round here
    let r = b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "maneuver", "target": foe}));
    assert_eq!(r["ok"], json!(true), "{r}");
    for f in [V2::new(-1, 0), V2::new(0, 1)] {
        b.host.session.as_mut().unwrap().w.round_states.entry(foe).or_default().facing = f;
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let o = overlay(&b);
        let w = &s(&b).w;
        let tp = w.pos(foe);
        assert!(!o.front.is_empty(), "facing {f:?}");
        for c in (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).filter(|c| w.grid.is_valid(*c) && *c != tp && c.dist(tp) <= FRONT_RADIUS) {
            let d = c - tp;
            assert_eq!(o.front.contains(&c), d.x * f.x + d.y * f.y > 0, "{c:?} facing {f:?}");
            assert_eq!(o.front.contains(&c), ri::in_front(w, foe, c));
        }
    }
    b.host.session.as_mut().unwrap().w.round_states.entry(foe).or_default().facing = emberwake_battle::grid::ZERO;
    assert!(overlay(&b).front.is_empty(), "no facing, no front");
}

/// GD5: a stealthed unit wears a veil, hidden once it moved this round (the hover card says so); K6: the hover card's
/// Reach is the kit's plain attack band under the rule `reach`.
#[test]
fn a_stealthed_unit_wears_a_veil_and_reach_reads_the_band() {
    let (mut cx, mut g, mut b) = arena(Some(TIMELINE));
    let h = b.selected.unwrap();
    let sess = b.host.session.as_mut().unwrap();
    emberwake_rules::status::grant_now(&mut sess.w.units[h], "stealth", 2, 0, "test");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let at = s(&b).w.pos(h);
    assert_eq!(overlay(&b).veils, vec![(at, false)], "veiled, not hidden before it moves");
    b.host.session.as_mut().unwrap().w.round_states.entry(h).or_default().moved = true;
    assert_eq!(overlay(&b).veils, vec![(at, true)], "hidden once it moved (GD5)");
    assert!(unit_tip(s(&b), h, Some(h), &[], &[], &[]).iter().any(|t| t.0.starts_with("Hidden")));
    let w = &s(&b).w;
    for i in 0..w.units.len() {
        let want = if w.ruleset.has(emberwake_rules::ruleset::REACH) {
            let (lo, hi) = emberwake_battle::skill::attack_band(&w.units[i], &w.ruleset);
            if lo == hi { lo.to_string() } else { format!("{lo}-{hi}") }
        } else {
            w.units[i].attack_range.to_string()
        };
        assert_eq!(reach_text(w, i), want, "{}", w.units[i].name);
    }
}
