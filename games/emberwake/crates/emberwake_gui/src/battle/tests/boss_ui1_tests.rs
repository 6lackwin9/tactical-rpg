//! BOSSUI1 (PR7-54, `games/emberwake/docs/specs/SCREEN_CONTRACT_BATTLE.md` 9.1, 9.2 and 7, `FB-364` (1)): the boss bar
//! takes the boss off the foe panel and out of the card; poise reads as one bar of the unit's own pips ("6/10"); the
//! intent rows show only while planning, the selected hero's row and "Others" summing the rest with no duplication,
//! the hover's members and "Party" line; the impact numbers' format (9999, 10K, 1M); a phase break's "Block N: ends"
//! and "Ph. 2 next round" with the blocks after it struck; the skill menu beside a tile keeps clear of the docked card,
//! the foe panel and the target. Battles are the first catalog encounter edited in memory (its first enemy made a boss
//! with an `hp_below` phase; no shipped data changes); every number is the kit's. PR7-59 (`FB-368`): the bar keeps the
//! unit's name (a phase `form` after it), and on the Vane fight (the `boss_bar` scenario's) the Stage board shifts down
//! until the bar covers no unit or open tile.

use crate::battle::{Action, BattleScreen};
use crate::battle_view::{boss_rows, impact_num, BossRows};
use crate::screens::{Cx, Screen};
use aegis_ui::canvas::Rect;
use aegis_ui::ui::Gfx;
use emberwake_battle::intent::{CardHit, FoeCard};
use emberwake_battle::session::{PhaseMark, Session};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::json;

/// The first catalog encounter under the timeline with its first enemy a boss (`pips` its own `poise_pips`; 0 the
/// ruleset's) and an `hp_below` phase at `pct` % on it ("Unbound"; none at 0).
fn boss_arena(pips: i32, pct: i32) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    std::sync::Arc::get_mut(&mut cx.app.data).expect("the test owns the data").edit_cat(|c| {
        let row = c.encounters.get_mut(&encounter).expect("the row");
        let first = row["enemies"][0].as_str().expect("an enemy ref").to_string();
        if pct > 0 {
            row.insert("phases".into(), json!([{"id": "unbound", "name": "Unbound", "trigger": {"kind": "hp_below", "unit": first, "pct": pct}, "effects": [{"type": "line", "text": "It breaks loose."}]}]));
        }
        let u = c.units.get_mut(&first).expect("the boss row");
        u.tier = "boss".into();
        u.poise_pips = pips;
    });
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

fn s(b: &BattleScreen) -> &Session {
    b.host.session.as_ref().unwrap()
}

fn has(b: &BattleScreen, id: &str) -> bool {
    b.w.iter().any(|w| w.id == id)
}

fn text(b: &BattleScreen, id: &str) -> String {
    b.w.iter().find(|w| w.id == id).map(|w| w.text.clone()).unwrap_or_else(|| panic!("no widget {id}"))
}

fn boss_of(b: &BattleScreen) -> usize {
    s(b).phase_bar().expect("a boss on the field").unit
}

/// The selected hero plans an Assault on `foe` (its blocks fill from the kit's plan).
fn target(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen, foe: usize) -> usize {
    let h = b.selected.expect("a hero");
    assert_eq!(b.send(cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": foe}))["ok"], json!(true));
    b.draw(cx, g, (-500, -500), true);
    h
}

/// Contract 9.1: the boss takes the bar across the board's top and leaves the foe panel and the card; the markers are
/// one per phase (the first plus one per `hp_below` phase), the first current; the span is the kit's phase span.
#[test]
fn the_boss_takes_the_bar_off_the_panel_and_the_card() {
    let (mut cx, mut g, mut b) = boss_arena(0, 50);
    let boss = boss_of(&b);
    assert!(has(&b, "boss.bar"), "the bar is drawn");
    assert!(!has(&b, &format!("foe.{boss}")), "the boss has no plate on the foe panel");
    let pb = s(&b).phase_bar().unwrap();
    assert_eq!(pb.marks, vec![PhaseMark::Current, PhaseMark::Coming], "two phases, the first current");
    assert!(has(&b, "boss.phase.0") && has(&b, "boss.phase.1") && !has(&b, "boss.phase.2"));
    let hp = s(&b).w.units[boss].hp;
    let (left, span) = pb.span(hp);
    assert_eq!(text(&b, "boss.hp"), format!("{left}/{span}"));
    assert_eq!(pb.floor, 50 * s(&b).w.units[boss].max_hp / 100, "the span ends at the phase's threshold");
    // the card never shows the boss: targeting it folds the card into the bar's rows
    target(&mut cx, &mut g, &mut b, boss);
    let card = b.w.iter().find(|w| w.id == "intel.unit" || w.id == "card.unit").map(|w| w.text.clone());
    assert_ne!(card.as_deref(), Some(s(&b).w.units[boss].name.as_str()), "no card for the boss");
    assert!(has(&b, "boss.rows"), "the rows in its place");
}

/// BU7, BU10, contract 9.1 row 3: poise is one bar of the unit's own `poise_pips` with the count beside it ("10/10"),
/// each pip `max_poise / pips`; a unit without the field keeps the ruleset's count.
#[test]
fn poise_reads_one_bar_of_the_units_own_pips() {
    use emberwake_battle::poise;
    let (_cx, _g, b) = boss_arena(10, 0);
    let boss = boss_of(&b);
    let w = &s(&b).w;
    let Some(st) = poise::state(w, boss).copied().filter(|st| st.max > 0) else { return };
    let n = &w.ruleset.numbers;
    assert_eq!(poise::full_pips(n, &st), 10, "the unit's own pips");
    assert_eq!(poise::pip_threshold(n, &st), poise::threshold_over(st.max, 10), "each pip is its max over its pips");
    let lit = if poise::pips_mode(n) { st.pips } else { poise::bar_pips(n, &st) };
    assert_eq!(lit, 10, "a full bar lights every pip");
    assert_eq!(text(&b, "boss.poise"), "10/10");
    let (_cx, _g, b) = boss_arena(0, 0);
    let boss = boss_of(&b);
    let w = &s(&b).w;
    let st = poise::state(w, boss).copied().unwrap();
    let n = &w.ruleset.numbers;
    assert_eq!(poise::full_pips(n, &st), n.poise_pips.max(1), "the fallback: the ruleset's count");
    assert_eq!(poise::pip_threshold(n, &st), poise::threshold(n, st.max), "and its threshold");
    assert_eq!(text(&b, "boss.poise"), format!("{0}/{0}", n.poise_pips.max(1)));
}

/// Contract 9.2 (take 31): the intent rows only while planning with the boss targeted; once the plan is confirmed the
/// bar folds back to 84 px.
#[test]
fn the_intent_rows_show_only_while_planning() {
    let (mut cx, mut g, mut b) = boss_arena(0, 50);
    let boss = boss_of(&b);
    let bar = |b: &BattleScreen| b.w.iter().find(|w| w.id == "boss.bar").expect("the bar").rect;
    assert!(!has(&b, "boss.rows"), "no rows before a hero targets the boss");
    assert_eq!(bar(&b).h, crate::battle::boss_bar::BAR_H);
    target(&mut cx, &mut g, &mut b, boss);
    assert!(has(&b, "boss.rows"), "the rows while planning");
    assert_eq!(bar(&b).h, crate::battle::boss_bar::BAR_H + crate::battle::boss_bar::ROWS_H);
    b.act(&mut cx, Action::Confirm);
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert!(!has(&b, "boss.rows"), "no rows once the plan is confirmed");
    assert_eq!(bar(&b).h, crate::battle::boss_bar::BAR_H, "84 px outside planning");
}

/// A card hit of `unit` landing in `block` for `damage` (pips, statuses as given).
fn hit(unit: usize, block: i32, damage: i32, pips: i32, status: &str) -> CardHit {
    CardHit { unit, skill: String::new(), name: String::new(), start: block, impact: block, damage, crit_damage: damage, crit_chance: 0.0, pips, breaks: false, on_broken: false, statuses: if status.is_empty() { Vec::new() } else { vec![status.to_string()] }, after_ko: false, fails: None }
}

fn names() -> Vec<(usize, String)> {
    vec![(0, "A".into()), (1, "B".into()), (2, "C".into())]
}

/// Contract 9.2: the selected hero's row and "Others" (the rest summed, never the hero again) add up to the Party
/// line in every block and in total; the member rows add up to it too.
#[test]
fn others_sum_the_rest_with_no_duplication() {
    let read = FoeCard { hp: 10_000, max_hp: 10_000, hits: vec![hit(0, 1, 34, 2, ""), hit(1, 2, 22, 0, "burn"), hit(2, 2, 10, 0, "slow"), hit(0, 6, 14, 0, ""), hit(2, 3, 40, 3, "daze")], ..FoeCard::default() };
    let rows = boss_rows(&read, 0, 6, Some(0), &names(), 2);
    let hero = rows.hero.as_ref().expect("the hero row");
    for b in 0..6 {
        let (h, o, p) = (&hero.cells[b], &rows.others.cells[b], &rows.party.cells[b]);
        assert_eq!((h.hp + o.hp, h.pips + o.pips, h.hits + o.hits), (p.hp, p.pips, p.hits), "block {}", b + 1);
        let members: i32 = rows.members.iter().map(|m| m.cells[b].hp).sum();
        assert_eq!(members, p.hp, "the members add up to the party in block {}", b + 1);
    }
    assert_eq!(hero.total.hp, read.total(Some(0)), "the hero's total is its own hits");
    assert_eq!(rows.others.total.hp, read.total(None) - read.total(Some(0)), "Others is the rest, the hero not again");
    assert_eq!(rows.party.total.hp, read.total(None));
    assert_eq!(rows.others.total.statuses, vec!["burn".to_string(), "slow".to_string(), "daze".to_string()]);
    assert_eq!(rows.members.len(), 3, "every member listed");
    assert!(rows.ends.is_none(), "no phase break on the last phase");
    // the live plan: the hero's row is the kit's card hits of that hero
    let (mut cx, mut g, mut b) = boss_arena(0, 50);
    let boss = boss_of(&b);
    let h = target(&mut cx, &mut g, &mut b, boss);
    let sched = crate::battle_view::round_schedule(s(&b));
    let card = emberwake_battle::intent::foe_card(&s(&b).w, &sched, boss);
    let live = boss_rows(&card, 0, s(&b).def.total_blocks, Some(h), &[(h, String::new())], 2);
    assert_eq!(live.hero.as_ref().unwrap().total.hp, card.total(Some(h)));
    assert_eq!(live.others.total.hp + live.hero.unwrap().total.hp, card.total(None));
}

/// Contract 9.2 numbers: whole up to 9999, K from 10K, M from 1M.
#[test]
fn impact_numbers_are_whole_then_k_then_m() {
    assert_eq!(impact_num(233), "233");
    assert_eq!(impact_num(1250), "1250");
    assert_eq!(impact_num(9999), "9999");
    assert_eq!(impact_num(10_000), "10K");
    assert_eq!(impact_num(12_500), "12K");
    assert_eq!(impact_num(120_000), "120K");
    assert_eq!(impact_num(999_999), "999K");
    assert_eq!(impact_num(1_000_000), "1M");
    assert_eq!(impact_num(-12_000), "-12K");
    let c = crate::battle_view::ImpactCell { hp: 12_000, hits: 1, ..Default::default() };
    assert_eq!(c.hp_text(), "-12K");
    assert_eq!(crate::battle_view::ImpactCell::default().hp_text(), "-", "a block the row does not hit");
}

/// BU15, BU16 (contract 9.2): a hit that takes the phase's span to 0 ends the round: its block reads "Block N: ends",
/// the next "Ph. 2 next round", the blocks after it are struck; the breaking hit stops at the threshold, the same
/// block's later hits are dropped and the totals count only what lands.
#[test]
fn a_phase_break_labels_the_blocks_and_counts_what_lands() {
    let read = FoeCard { hp: 100, max_hp: 200, hits: vec![hit(0, 1, 30, 0, ""), hit(1, 2, 30, 0, ""), hit(2, 2, 25, 0, "burn"), hit(0, 4, 30, 0, ""), hit(1, 5, 30, 0, "")], ..FoeCard::default() };
    let rows: BossRows = boss_rows(&read, 50, 6, Some(0), &names(), 2);
    let cut = emberwake_battle::intent::phase_cut(&read, 50).expect("the plan breaks the phase");
    assert_eq!((cut.hit, cut.block, cut.landed), (1, 2, 20), "the second hit stops at the threshold");
    assert_eq!(rows.ends, Some(2));
    assert_eq!(rows.header(1), "Block 1");
    assert_eq!(rows.header(2), "Block 2: ends");
    assert_eq!(rows.header(3), "Ph. 2 next round");
    assert_eq!(rows.header(4), "Block 4");
    assert!(!rows.struck(2) && (3..=6).all(|b| rows.struck(b)), "the blocks after the break are struck");
    assert_eq!(rows.party.total.hp, 50, "only what lands: the span to the threshold");
    assert_eq!(rows.party.cells[1].hp, 20, "the breaking block: the capped hit, the same block's later hit dropped");
    assert_eq!(rows.party.cells[3].hp, 30, "a later block keeps its number, struck");
    assert!(rows.party.total.statuses.is_empty(), "a dropped hit brings no status");
    assert!(emberwake_battle::intent::phase_cut(&read, 0).is_none(), "the last phase never breaks");
}

/// `FB-364` (1), contract 7 and 9.2 (BU12): beside a tile the skill menu keeps clear of the kept rects (the docked card
/// or the boss bar, the foe panel, the target), flipping side or opening under or above the tile; never over the tile.
#[test]
fn the_menu_keeps_clear_of_the_card_the_panel_and_the_target() {
    use crate::battle::skill_menu::{menu_place_clear, overlaps};
    let view = Rect::new(126, 28, 388, 292);
    let size = (120, 80);
    let at = |p: (i32, i32)| Rect::new(p.0, p.1, size.0, size.1);
    // the card on the right of a tile in the middle: the menu goes left
    let tile = Rect::new(300, 150, 32, 32);
    let card = Rect::new(334, 100, 150, 120);
    let r = at(menu_place_clear(tile, size, view, &[card], false));
    assert!(!overlaps(r, card) && !overlaps(r, tile), "left of the tile: {r:?}");
    assert!(r.x + r.w <= tile.x, "flipped to the left: {r:?}");
    // both sides blocked: under (or above) the tile
    let left = Rect::new(126, 120, 172, 90);
    let r = at(menu_place_clear(tile, size, view, &[card, left], false));
    assert!(!overlaps(r, card) && !overlaps(r, left) && !overlaps(r, tile), "under or above: {r:?}");
    // the layout's preferred side first
    let r = at(menu_place_clear(tile, size, view, &[Rect::new(0, 0, 1, 1)], true));
    assert!(r.x + r.w <= tile.x, "left first when preferred: {r:?}");
    // nothing to keep clear: today's placement
    assert_eq!(menu_place_clear(tile, size, view, &[], false), crate::battle::skill_menu::menu_place(tile, size, view));
    // on the live board: a foe at each corner, the menu clear of the card, the foe panel and the tile
    use crate::stage::{Geom, View};
    use emberwake_battle::grid::{V2, GRID_H, GRID_W};
    let (mut cx, mut g, mut b) = boss_arena(0, 0);
    let boss = boss_of(&b);
    let foe = s(&b).w.enemies.iter().copied().find(|&e| e != boss && s(&b).w.units[e].is_active() && s(&b).w.has(e));
    let geo = Geom::new(View::of_setting(&cx.settings.v["battle_view"]), &cx.tuning.value);
    for unit in [Some(boss), foe].into_iter().flatten() {
        for corner in [V2::new(GRID_W - 1, 2), V2::new(GRID_W / 2, GRID_H / 2), V2::new(GRID_W - 1, GRID_H - 1), V2::new(0, GRID_H - 1)] {
            if s(&b).w.occupant(corner).is_some_and(|o| o != unit) {
                continue;
            }
            b.host.session.as_mut().unwrap().w.grid.positions[unit] = Some(corner);
            b.after_change(&cx);
            b.act(&mut cx, Action::ClosePopup);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            b.act(&mut cx, Action::UnitClick(unit));
            b.draw(&mut cx, &mut g, (-500, -500), true);
            b.draw(&mut cx, &mut g, (-500, -500), true);
            let Some(p) = b.popup.as_ref() else { continue };
            let menu = p.rect(b.menu_row);
            let tile = geo.cell_rect(corner);
            assert!(!overlaps(menu, tile), "unit {unit} at {corner}: not over the tile: {menu:?}");
            for (name, r) in b.panels.iter().filter(|p| p.0 == "intel" || p.0 == "hostiles") {
                assert!(!overlaps(menu, *r), "unit {unit} at {corner}: the menu {menu:?} covers the {name} panel {r:?}");
            }
        }
    }
}

/// BU3b (PR7-59, `FB-368` (1)): the bar's title is the boss unit's name and title, never a phase's `name` (the phase
/// banner shows it); a phase that sets a `form` shows "<name>, <form>" in place of the title.
#[test]
fn the_bar_keeps_the_units_name_and_shows_a_phase_form() {
    let (mut cx, mut g, mut b) = boss_arena(0, 50);
    let boss = boss_of(&b);
    let full = text(&b, "boss.name");
    let title = b.w.iter().find(|w| w.id == "boss.title").map(|w| w.text.clone());
    assert_eq!(full, s(&b).w.units[boss].name, "before the phase: the unit's name");
    // the `hp_below` phase fired (its name "Unbound"): the bar keeps the unit's name and title
    let sm = b.host.session.as_mut().unwrap();
    sm.phases.fired.insert("unbound".into(), (1, 1));
    assert_eq!(sm.phase_bar().unwrap().name, "Unbound", "the phase's name is the kit's read (the banner's)");
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(text(&b, "boss.name"), full, "not the phase's name");
    assert_eq!(b.w.iter().find(|w| w.id == "boss.title").map(|w| w.text.clone()), title, "the title kept");
    // the phase sets a form: "<name>, <form>", no title
    b.host.session.as_mut().unwrap().phases.phases.iter_mut().find(|p| p.id == "unbound").expect("the phase").form = "Unbound".into();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_eq!(text(&b, "boss.name"), format!("{full}, Unbound"));
    assert!(!has(&b, "boss.title"), "the form takes the title's place");
}

/// The Vane fight (sunk_ward `enc_evt_01`, the `boss_bar` scenario's) under the timeline on the Stage view.
fn vane_fight() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("stage");
    let mut b = BattleScreen::arena_with(&cx, Some("sunk_ward".into()), "enc_evt_01", Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

/// The tiles the bar must leave clear: in bounds and walkable, or holding a unit.
fn kept(w: &emberwake_battle::world::World, c: emberwake_battle::grid::V2) -> bool {
    w.grid.is_walkable(c) || (0..w.units.len()).any(|u| w.cells_of(u).contains(&c))
}

/// BU11, BU12 (PR7-59, `FB-368` (2)): on the Vane fight the grown bar (a hero targeting the boss) covers no unit and no
/// tile the plan can target: the board shifts down by the least that clears them (a kept tile then starts right under
/// the bar), at most `MAX_LIFT`; the tiles' widgets move with it. A board whose rows under the bar hold only walls and
/// no unit keeps its place.
#[test]
fn the_board_shifts_down_until_the_bar_covers_no_unit_or_open_tile() {
    use crate::battle::boss_bar::{board_lift, MAX_LIFT};
    use crate::battle::skill_menu::overlaps;
    use crate::stage::{Geom, View};
    use emberwake_battle::grid::{V2, GRID_H, GRID_W};
    let (mut cx, mut g, mut b) = vane_fight();
    let boss = boss_of(&b);
    target(&mut cx, &mut g, &mut b, boss);
    assert!(has(&b, "boss.rows"), "the grown bar");
    let bar = b.w.iter().find(|w| w.id == "boss.bar").expect("the bar").rect;
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let lift = b.board_lift;
    assert!((0..=MAX_LIFT).contains(&lift), "{lift}");
    let w = &s(&b).w;
    let tile = |c: V2| b.w.iter().find(|x| x.id == format!("tile.{}.{}", c.x, c.y)).expect("the tile widget").rect;
    let mut touching = false;
    for y in 0..GRID_H {
        for x in 0..GRID_W {
            let c = V2::new(x, y);
            let r = geo.cell_rect(c);
            assert_eq!(tile(c), Rect::new(r.x, r.y + lift, r.w, r.h), "{c}: the widget moves with the board");
            if kept(w, c) {
                assert!(!overlaps(tile(c), bar), "{c} under the bar {bar:?}");
                touching |= tile(c).y == bar.y + bar.h;
            }
        }
    }
    let covered = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).any(|c| kept(w, c) && overlaps(geo.cell_rect(c), bar));
    assert_eq!(lift > 0, covered, "a shift only when the bar would cover a kept tile");
    assert!(lift == 0 || touching, "the least shift: a kept tile right under the bar");
    // the rows under the bar all walls, no unit there: no shift
    let mut walled = w.clone();
    let wall = (0..GRID_W).map(|x| V2::new(x, 0)).find(|c| !walled.grid.is_walkable(*c)).expect("a wall tile on the top row");
    let mut rows: Vec<String> = (0..GRID_H).map(|y| (0..GRID_W).map(|x| walled.grid.glyph(V2::new(x, y))).collect()).collect();
    let g0 = walled.grid.glyph(wall);
    rows[0] = std::iter::repeat_n(g0, GRID_W as usize).collect();
    walled.grid.apply_board("", &rows, &indexmap::IndexMap::new());
    for u in 0..walled.units.len() {
        if walled.grid.positions[u].is_some_and(|p| p.y == 0) {
            walled.grid.positions[u] = None;
        }
    }
    assert!((0..GRID_W).all(|x| !walled.grid.is_walkable(V2::new(x, 0))), "the top row walled");
    let open_under = (0..GRID_H).flat_map(|y| (0..GRID_W).map(move |x| V2::new(x, y))).any(|c| kept(&walled, c) && overlaps(geo.cell_rect(c), bar));
    assert_eq!(board_lift(&walled, &geo) > 0, open_under, "no kept tile under the bar: no shift");
    assert_eq!(board_lift(w, &Geom::new(View::Flat, &cx.tuning.value)), 0, "the Flat view keeps its place");
}

/// PR7-75 (PT-40): the "Hostiles N" counter counts the boss with the panel's foes; the bar's HP read-out keeps the
/// phase's span and its hover gives both the span and the boss's whole HP, so the bar and the unit never disagree.
#[test]
fn the_counter_counts_the_boss_and_the_hp_hover_gives_both_numbers() {
    let (_cx, _g, b) = boss_arena(0, 50);
    let boss = boss_of(&b);
    let ses = s(&b);
    let name = |_: usize| String::new();
    let full = |_: usize| (String::new(), String::new());
    let none = |_: usize| None;
    let about = |_: &str| String::new();
    let inp = crate::battle_view::Inputs { sel: None, card_foe: None, round_ap: &[], plate_name: &name, full_name: &full, intent_of: &none, open_groups: &[], pinned: None, passive_about: &about };
    let m = crate::battle_view::model(ses, &inp);
    let active = ses.w.enemies.iter().filter(|e| ses.w.units[**e].is_active()).count();
    assert!(ses.w.enemies.contains(&boss) && active >= 1);
    assert_eq!(m.hostiles, format!("Hostiles {active}"), "the boss counts");
    let bar = m.boss.expect("the boss bar");
    let u = &ses.w.units[boss];
    assert_eq!(bar.whole, (u.hp, u.max_hp));
    assert_eq!(bar.hp_tip(), vec![format!("This phase: {}", bar.span_text()), format!("In all: {}/{}", u.hp, u.max_hp)]);
    let tip = b.w.iter().find(|w| w.id == "boss.hp").expect("the read-out").tip.iter().map(|t| t.0.clone()).collect::<Vec<_>>();
    assert_eq!(tip, bar.hp_tip(), "the read-out's hover");
    assert_eq!(bar.carried, (u.hp - ses.phase_bar().map_or(u.hp, |p| p.top)).max(0));
    // PR7-83 (PT-75): HP carried over from a phase that ended by time reads beside the span and on the hover
    let mut carried = bar.clone();
    carried.carried = 138;
    assert_eq!(carried.span_text(), format!("{}/{} +138", carried.span_left, carried.span));
    assert!(carried.hp_tip().last().is_some_and(|l| l.starts_with("Carried over: 138 HP")), "{:?}", carried.hp_tip());
}
