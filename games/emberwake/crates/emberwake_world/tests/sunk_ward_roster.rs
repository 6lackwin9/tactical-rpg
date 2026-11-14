//! PK2-FOLLOWUP (PR5-36 part 4, FB-232) on the shipped data, read only: the `sunk_ward` world's roster rules. Each
//! case settles one fielded member through `roster::settle_party` (the settle every runner's `after_fight` uses) in
//! act 1 and act 2, and the outcome follows the rule row the world plays in that act (`roster::rule`): a `wounded`
//! value leaves the member in the party and wounded, `dead` departs it. The acceptance on top: in act 1 a member downed
//! in a won, left (fled) or lost battle and a member standing on a loss all end wounded with no departure; in act 2
//! downed on a loss or a flight departs dead while downed on a win stays wounded.

use emberwake_battle::encounter::PartyFate;
use emberwake_world::roster::{self, FightEnd, HeroEnd, DEAD, WOUNDED};
use emberwake_world::state::PartyState;
use emberwake_world::{new_game, Data, GameState};
use std::path::Path;

const WORLD: &str = "sunk_ward";

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

/// A `sunk_ward` game in act `act` whose party is the world's first playable party character (picked by rule: the
/// world names no start party).
fn game(d: &Data, act: i64) -> GameState {
    let m = d
        .catalogs
        .own_entries(WORLD, "characters")
        .iter()
        .filter(|c| aegis_core::json::s(c, "role") == "party")
        .find_map(|c| new_game::hero_from_character(d, WORLD, &aegis_core::json::s(c, "id")))
        .expect("a playable party character in sunk_ward");
    let mut gs = GameState::create(d, WORLD, PartyState { active_ids: vec![m.id.clone()], members: vec![m], ..PartyState::default() });
    gs.campaign.as_mut().expect("sunk_ward is a campaign world").act = act;
    gs
}

/// The end of the first member: downed (0 HP) or standing (full HP), with `fate`.
fn end_of(d: &Data, gs: &GameState, fate: PartyFate) -> HeroEnd {
    let m = &gs.party.members[0];
    let max = roster::field_unit(d, gs, m, WORLD).map_or(1, |f| i64::from(f.unit.max_hp));
    let alive = fate != PartyFate::Downed;
    HeroEnd {
        member: m.id.clone(),
        hp: if alive { max } else { 0 },
        max_hp: max,
        alive,
        pouch: m.pouch.clone(),
        unloaded: Vec::new(),
        fate: Some(fate),
        marks: Vec::new(),
        hostile: String::new(),
        attacked: Vec::new(),
        worn: Vec::new(),
    }
}

/// The four cases: (label, fate, outcome, the rule row's value for it).
fn cases(row: &emberwake_data::RosterRule) -> [(&'static str, PartyFate, &'static str, String); 4] {
    [
        ("downed, won", PartyFate::Downed, "won", row.downed_won.clone()),
        ("downed, left", PartyFate::Downed, "fled", row.downed_left.clone()),
        ("downed, lost", PartyFate::Downed, "lost", row.downed_lost.clone()),
        ("standing, lost", PartyFate::Standing, "lost", row.standing_lost.clone()),
    ]
}

/// Settles each case in act `act` on a fresh game; returns (label, the row's value, what happened: "wounded" or the
/// departed life).
fn settle_all(d: &Data, act: i64) -> Vec<(&'static str, String, String)> {
    let row = roster::rule(d, &game(d, act)).unwrap_or_else(|| panic!("{WORLD} has a roster rule row in act {act}"));
    cases(&row)
        .into_iter()
        .map(|(label, fate, outcome, want)| {
            let mut gs = game(d, act);
            let end = end_of(d, &gs, fate);
            let id = end.member.clone();
            let settled = roster::settle_party(d, &mut gs, &FightEnd::new(outcome, WORLD), &[end]);
            let got = match settled.departed.iter().find(|(m, _)| *m == id) {
                Some((_, life)) => {
                    assert!(gs.party.member(&id).is_none(), "act {act} {label}: a departed member leaves the party");
                    life.clone()
                }
                None => {
                    let m = gs.party.member(&id).expect("still in the party");
                    if m.has_condition(WOUNDED) {
                        assert!(settled.wounded.contains(&id), "act {act} {label}: named wounded");
                        WOUNDED.to_string()
                    } else {
                        "unhurt".to_string()
                    }
                }
            };
            assert_eq!(got, want, "act {act} {label}: the settle follows the row");
            (label, want, got)
        })
        .collect()
}

#[test]
fn act_1_wounds_every_downed_or_beaten_member_and_departs_none() {
    let d = data();
    for (label, _, got) in settle_all(&d, 1) {
        assert_eq!(got, WOUNDED, "act 1 {label}: wounded, no departure");
    }
}

#[test]
fn act_2_is_lethal_for_the_downed_on_a_loss_or_a_flight_and_wounds_the_downed_on_a_win() {
    let d = data();
    let got: Vec<(&str, String)> = settle_all(&d, 2).into_iter().map(|(label, _, got)| (label, got)).collect();
    let of = |label: &str| got.iter().find(|(l, _)| *l == label).map(|(_, g)| g.clone()).unwrap_or_default();
    assert_eq!((of("downed, lost"), of("downed, left")), (DEAD.to_string(), DEAD.to_string()), "act 2: downed and beaten or fled departs dead");
    assert_eq!(of("downed, won"), WOUNDED, "act 2: downed on a win stays wounded");
}
