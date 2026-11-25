//! STATUSVIS1-B (PR7-57, `docs/specs/STATUSVIS1.md` SV5-SV15, battle contract 6 and 9.3): each status shows where it
//! acts. The battle is the first catalog encounter under the timeline, its statuses given in memory (no shipped data
//! changes); expectations come from the kit's reads (`status_view`) and the status table, never a shipped number.

use crate::battle::{Action, BattleScreen};
use crate::battle_view::{tabs_of, PLATE_PASSIVES};
use crate::screens::{Cx, Key, Screen};
use aegis_ui::ui::Gfx;
use emberwake_battle::status_view::{self as sv, RowTarget};
use emberwake_rules::ruleset::TIMELINE;
use emberwake_rules::status;
use serde_json::json;

fn arena() -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.settings.v["battle_view"] = json!("flat");
    let encounter = cx.data.cat.encounters.keys().next().cloned().expect("a catalog encounter");
    let mut b = BattleScreen::arena_with(&cx, None, &encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    let g = crate::hud::hud_tests::gfx();
    (cx, g, b)
}

fn drawn(cx: &mut Cx, g: &mut Gfx, b: &mut BattleScreen) {
    b.draw(cx, g, (-500, -500), true);
}

/// The dev link's grant (`debug_status`), as the scenario gives statuses.
fn give(cx: &Cx, b: &mut BattleScreen, u: usize, st: &str, magnitude: i32) {
    let r = b.debug_status(cx, &json!({"cmd": "debug_status", "unit": u, "status": st, "magnitude": magnitude}));
    assert_eq!(r["ok"], json!(true), "{st} on {u}: {r}");
}

fn widget<'a>(b: &'a BattleScreen, id: &str) -> Option<&'a aegis_ui::ui::Widget<Action>> {
    b.w.iter().find(|w| w.id == id)
}

/// Section 3: the plate column orders by priority (a rule flag first, then reactive; ticks; the rest), each priority
/// by the rounds left as before.
#[test]
fn the_plate_column_orders_by_primitive_priority() {
    let (cx, _g, mut b) = arena();
    let h = b.session().w.allies[0];
    for (st, m) in [("weaken", 10), ("bleed", 0), ("riposte", 0), ("steadfast", 0)] {
        give(&cx, &mut b, h, st, m);
    }
    let tabs = tabs_of(&b.session().w, h, &[]);
    let order: Vec<&str> = tabs.iter().map(|t| t.status.as_str()).collect();
    assert_eq!(order.first(), Some(&"steadfast"), "the rule flag first: {order:?}");
    let prios: Vec<u8> = tabs.iter().map(|t| t.priority).collect();
    assert!(prios.windows(2).all(|p| p[0] <= p[1]), "by priority: {order:?} {prios:?}");
    for t in &tabs {
        assert_eq!(t.priority, sv::plate_priority(&t.status), "{}", t.status);
    }
    // a planned status comes last in its priority
    let tabs = tabs_of(&b.session().w, h, &["poison".to_string()]);
    let poison = tabs.iter().position(|t| t.status == "poison").expect("the planned tab");
    let bleed = tabs.iter().position(|t| t.status == "bleed").expect("bleed");
    assert!(poison > bleed && tabs[poison].planned);
}

/// SV6: the plate's loss is one number, the hits and the round-end ticks together, its hover each of them.
#[test]
fn hits_and_ticks_are_one_planned_loss_with_a_hover() {
    let (mut cx, mut g, mut b) = arena();
    let h = b.session().w.allies[0];
    give(&cx, &mut b, h, "bleed", 0);
    drawn(&mut cx, &mut g, &mut b);
    let s = b.session();
    let sched = crate::battle_view::round_schedule(s);
    let pl = sv::planned_loss(&s.w, &sched, h);
    assert!(pl.ticks.iter().any(|t| t.status == "bleed" && t.hp < 0), "the bleed ticks");
    let w = widget(&b, &format!("party.{h}.incoming")).expect("the loss number");
    assert_eq!(w.text, format!("-{}", pl.loss));
    assert_eq!(w.kind, "label", "a frameless hover");
    assert!(w.tip.iter().any(|t| t.0.starts_with("Round end, ") && t.0.contains(&status::display_name("bleed"))), "the tick on the hover: {:?}", w.tip);
    assert!(w.tip.iter().any(|t| t.0 == format!("Total -{}", pl.loss)));
    // the hover shows over the plate's card (lifted after it)
    let at = (w.rect.x + w.rect.w / 2, w.rect.y + w.rect.h / 2);
    let top = aegis_ui::ui::tip_widget(&b.w, at).expect("a card");
    assert_eq!(top.id, format!("party.{h}.incoming"));
}

/// SV5, SV15: a lock's blocks are one grey tile per run with the status icon; no plan there.
#[test]
fn a_lock_is_one_grey_tile_per_run_and_takes_no_plan() {
    for st in ["stun", "stagger"] {
        let (mut cx, mut g, mut b) = arena();
        let h = b.selected.expect("a hero");
        give(&cx, &mut b, h, st, 0);
        drawn(&mut cx, &mut g, &mut b);
        let runs = sv::lock_runs(&b.session().w, h);
        assert!(!runs.is_empty(), "{st} locks blocks");
        let shown: Vec<(String, i32, i32)> = b.strip_labels.iter().filter_map(|l| Some((l["locked"].as_str()?.to_string(), l["blocks"][0].as_i64()? as i32, l["blocks"][1].as_i64()? as i32))).collect();
        assert_eq!(shown, runs.iter().map(|r| (r.status.clone(), r.start, r.end)).collect::<Vec<_>>(), "{st}: one tile per run");
        let b1 = widget(&b, &format!("strip.block{}", runs[0].start)).expect("the run's block");
        assert_eq!(b1.action, None, "{st}: no menu on a locked block");
        assert!(b1.tip.iter().any(|t| t.0 == status::display_name(st)));
        let one = widget(&b, "strip.block6").map(|w| w.rect.w).unwrap_or(b1.rect.w);
        let span = runs[0].end - runs[0].start + 1;
        assert!(b1.rect.w >= one * span - span, "{st}: the run's first block holds the run");
    }
}

/// SV10, SV13: the loadout's two sockets sit centred under the portrait; an empty one is a dim socket with its own
/// hover, a slotted passive its icon with its name and what it does.
#[test]
fn two_passive_sockets_sit_under_the_portrait() {
    let (mut cx, mut g, mut b) = arena();
    let h = b.session().w.allies[0];
    let slotted: Option<String> = cx.data.cat.passives.values().find(|p| p.slot == "passive").map(|p| p.id.clone());
    {
        let s = b.host.session.as_mut().unwrap();
        assert!(s.w.units[h].deck_rule.is_some(), "the timeline has the deck rule");
        s.w.units[h].deck.passive = slotted.iter().cloned().collect();
    }
    drawn(&mut cx, &mut g, &mut b);
    let portrait = widget(&b, &format!("party.{h}.portrait")).expect("the portrait").rect;
    let a = widget(&b, &format!("party.{h}.passive.0")).expect("the slotted passive").clone();
    assert!(a.tip.len() >= 2, "name and what it does: {:?}", a.tip);
    let empty = widget(&b, &format!("party.{h}.socket.1")).expect("the empty socket").clone();
    assert!(widget(&b, &format!("party.{h}.socket.{PLATE_PASSIVES}")).is_none(), "two sockets only");
    let (l, r) = (a.rect.x - portrait.x, portrait.x + portrait.w - (empty.rect.x + empty.rect.w));
    assert!((l - r).abs() <= 1, "centred under the portrait: {l} {r}");
    assert!(a.rect.y >= portrait.y + portrait.h);
    // passives never enter the status column
    assert!(!b.w.iter().any(|w| w.id.starts_with(&format!("party.{h}.status.")) && slotted.as_ref().is_some_and(|p| w.text.contains(p.as_str()))));
}

/// SV8: a taunting unit wears the badge over its token (its hover names the unit foes must target), not the ring at
/// its feet.
#[test]
fn a_taunt_is_a_badge_over_the_unit() {
    let (mut cx, mut g, mut b) = arena();
    let e = b.session().w.enemies.iter().copied().find(|&e| b.session().w.grid.pos(e).is_some()).expect("a foe on the field");
    give(&cx, &mut b, e, "taunt", 0);
    drawn(&mut cx, &mut g, &mut b);
    let badge = widget(&b, &format!("board.taunt.{e}")).expect("the badge");
    let name = b.session().w.units[e].name.clone();
    assert!(badge.tip.iter().any(|t| t.0 == format!("Foes must target {name}")), "{:?}", badge.tip);
    let fx = b.tokens.anims.get(e).map(|a| a.status_fx.clone()).unwrap_or_default();
    assert!(!fx.contains(&"taunt"), "no feet ring: {fx:?}");
}

/// SV7, SV9: a planned hit's number on the card is the final one, tinted by its modifiers with the breakdown on its
/// hover; a riposting target shows the chip with no number, its hover the likely return.
#[test]
fn card_numbers_show_modifiers_and_the_riposte_chip() {
    let (mut cx, mut g, mut b) = arena();
    // the first hero, foe and attack the round takes as a pick
    let w = &b.session().w;
    let foes: Vec<usize> = w.enemies.iter().copied().filter(|&e| w.units[e].is_active() && w.grid.pos(e).is_some()).collect();
    let allies = w.allies.clone();
    let mut found = None;
    'all: for &h in &allies {
        let skills = b.send(&cx, json!({"cmd": "skills", "unit": h}));
        let attacks: Vec<String> = skills["skills"].as_array().into_iter().flatten().filter(|s| s["kind"] == json!("ATTACK") && s["reach"][1].as_i64() == Some(1)).filter_map(|s| s["skill"].as_str().map(String::from)).collect();
        for &f in &foes {
            for sk in &attacks {
                if b.send(&cx, json!({"cmd": "round_intent", "unit": h, "intent": "assault", "target": f}))["ok"] != json!(true) {
                    continue;
                }
                if b.send(&cx, json!({"cmd": "pick", "unit": h, "target": f, "pick": {"skill": sk}}))["ok"] == json!(true) {
                    // PR7-90 (PT-85): a hit the forecast reads from a tile next to the foe at its impact (the moves
                    // booked before it on the board as the round runs), so the riposte answers it
                    let s = b.session();
                    let sched = crate::battle_view::round_schedule(s);
                    let next_to = sched.iter().filter(|a| a.unit == h && !a.cancelled && emberwake_battle::intent::hit_on(&s.w, &sched, a, f).is_some_and(|x| x.fails.is_none())).any(|a| {
                        let sc = emberwake_battle::intent::scene_at(&s.w, &sched, a, f);
                        sc.from.dist(sc.at) == 1
                    });
                    if next_to {
                        found = Some((h, f));
                        break 'all;
                    }
                    b.send(&cx, json!({"cmd": "clear", "unit": h}));
                }
            }
        }
    }
    let (h, f) = found.expect("a hero can plan a melee hit on a foe");
    give(&cx, &mut b, h, "empower", 30);
    give(&cx, &mut b, f, "riposte", 0);
    b.selected = Some(h);
    drawn(&mut cx, &mut g, &mut b);
    let reads = sv::loss_reads(&b.session().w, &crate::battle_view::round_schedule(b.session()), f);
    let read = reads.iter().find(|r| r.parts.sign() > 0).expect("an empowered hit");
    let num = b.w.iter().find(|w| w.id.starts_with("card.row.") && w.id.ends_with(".number")).expect("a card number");
    assert_eq!(num.kind, "label");
    assert!(num.tip.iter().any(|t| t.0.starts_with(&status::display_name("empower"))), "the modifier on its hover: {:?}", num.tip);
    assert!(num.tip.iter().any(|t| t.0 == format!("Base {} HP", read.parts.base)), "the base: {:?}", num.tip);
    let chip = b.w.iter().find(|w| w.id.starts_with("card.row.") && w.id.ends_with(".riposte")).expect("the riposte chip");
    assert_eq!(chip.text, "Riposte", "no number on the chip");
    let back = read.counter.expect("a melee hit draws the counter");
    assert!(chip.tip.iter().any(|t| t.0.contains(&format!("about {back} HP"))), "the likely return: {:?}", chip.tip);
}

/// SV13-SV15: the full timeline view is the calm one: tiles with icons, a lock one grey tile per run, each row's one
/// target in its aside square.
#[test]
fn the_full_view_shows_tiles_locks_and_one_target_a_row() {
    let (mut cx, mut g, mut b) = arena();
    let h = b.session().w.allies[0];
    give(&cx, &mut b, h, "stun", 0);
    b.key(&mut cx, Key::Char('t'));
    drawn(&mut cx, &mut g, &mut b);
    assert!(b.popup.is_none(), "T opens no skill menu");
    let t = b.tv_shown.clone().expect("the view is open");
    assert!(t.tiles, "the calm tiles");
    let track = t.sections.iter().flat_map(|x| &x.tracks).find(|x| x.key == crate::timeline_view::track_key(h)).expect("the hero's track");
    let runs = sv::lock_runs(&b.session().w, h);
    let locks: Vec<(i32, i32)> = track.segments.iter().filter(|s| s.key.starts_with("lock")).map(|s| (s.start, s.end)).collect();
    assert_eq!(locks, runs.iter().map(|r| (r.start, r.end)).collect::<Vec<_>>(), "one grey tile per run");
    assert!(track.segments.iter().all(|s| s.parts.is_empty()), "no wind-up parts in the calm view");
    assert!(t.sections.iter().flat_map(|x| &x.tracks).all(|x| x.aside && x.marks.iter().all(|m| m.glyph != "!")), "an aside square a row, no per-blow ticks");
    let aims = crate::timeline_view::row_aims(b.session(), &crate::timeline_view::Opts { viewed: b.session().current_round, reveal_all: false, foresight_off: false, show_tells: true, layers: crate::battle::Layers::of(&json!(null)), selected: None, block: None, shown: None, playhead: None });
    for x in t.sections.iter().flat_map(|x| &x.tracks) {
        let w = widget(&b, &format!("timeline.target.{}", x.key)).unwrap_or_else(|| panic!("{}: its target square", x.key));
        match aims.get(&x.key).map(|a| a.target) {
            Some(RowTarget::None) | None => assert_eq!(w.text, "No target"),
            Some(_) => assert!(w.text.starts_with("Target"), "{}: {}", x.key, w.text),
        }
    }
}

/// SV11: the AP row follows the rules and changes none; a lock takes blocks, and the row shows the AP the rules leave
/// (the same row when the lock costs no AP, the new AP when it does).
#[test]
fn the_ap_row_follows_the_rules_under_a_lock() {
    let (mut cx, mut g, mut b) = arena();
    let h = b.selected.expect("a hero");
    drawn(&mut cx, &mut g, &mut b);
    let line = |b: &BattleScreen| widget(b, "strip.ap_line").map(|w| w.text.clone()).expect("the AP line");
    let before = (line(&b), emberwake_battle::round_intent_plan::ap_forecast(&b.session().w, &b.session().plan, h).now);
    give(&cx, &mut b, h, "stun", 0);
    drawn(&mut cx, &mut g, &mut b);
    let now = emberwake_battle::round_intent_plan::ap_forecast(&b.session().w, &b.session().plan, h).now;
    if now == before.1 {
        assert_eq!(line(&b), before.0, "a lock that costs no AP leaves the row");
    } else {
        assert!(line(&b).starts_with(&format!("AP {now}/")), "the AP the rules leave: {}", line(&b));
    }
}

/// SV5 with contract 7: the menu key over a locked block opens no menu (as its click); only what the player's map
/// turns into Tab is the menu key (a rebind moves it; T or any other key opens nothing); off the strip the key still
/// opens the menu on the block the next pick takes.
#[test]
fn the_menu_key_opens_nothing_on_a_locked_block() {
    use aegis_ui::controls::{Controls, Input, Pad, SETTINGS_KEY};
    let (mut cx, mut g, mut b) = arena();
    let h = b.selected.expect("a hero");
    give(&cx, &mut b, h, "stun", 0);
    drawn(&mut cx, &mut g, &mut b);
    let run = sv::lock_runs(&b.session().w, h).first().cloned().expect("the stun's run");
    let r = widget(&b, &format!("strip.block{}", run.start)).expect("the locked block").rect;
    b.mouse = (r.x + r.w / 2, r.y + r.h / 2);
    assert_eq!(b.strip_block_under_mouse().map(|x| x.0), Some(run.start));
    b.capture(&mut cx, Input::Key(Key::Tab));
    assert!(b.popup.as_ref().and_then(|p| p.menu.as_ref()).is_none(), "no menu on a locked block");
    b.strip_menu(run.end);
    assert!(b.popup.is_none(), "nor on the run's last block");
    assert!(!b.capture(&mut cx, Input::Key(Key::Char('t'))), "T is not the menu key");
    assert!(b.popup.is_none());
    // a rebind: Pad X gives Tab; over the lock it opens nothing, off the strip it opens the menu
    let (mut map, _) = Controls::from_settings(&cx.settings.v);
    map.bind_pad(Key::Tab, Pad::West);
    cx.settings.v[SETTINGS_KEY] = map.to_json();
    b.capture(&mut cx, Input::Pad(Pad::West));
    assert!(b.popup.is_none(), "the rebound key opens nothing over the lock");
    b.mouse = (-500, -500);
    assert!(!b.capture(&mut cx, Input::Pad(Pad::North)), "pad north no longer gives Tab");
    assert!(b.capture(&mut cx, Input::Pad(Pad::West)));
    let p = b.popup.as_ref().filter(|p| p.menu.is_some()).expect("off the strip the rebound key opens the menu");
    let at = p.block.map(|x| x.0).expect("on a block");
    assert!(sv::run_at(&sv::lock_runs(&b.session().w, h), at).is_none(), "past the lock: block {at}");
}

/// SV14 with TV4: a masked enemy's row target is its round state's target only (what its masked note names), never a
/// shape read from its secret actions (no area ring, no ground patch the party does not know); revealed, the row reads
/// its plan.
#[test]
fn a_masked_enemy_row_shows_only_its_round_target() {
    use emberwake_battle::round_intent::RoundTarget;
    let (_cx, _g, b) = arena();
    let s = b.session();
    let opts = |reveal_all: bool| crate::timeline_view::Opts { viewed: s.current_round, reveal_all, foresight_off: false, show_tells: true, layers: crate::battle::Layers::of(&json!(null)), selected: None, block: None, shown: None, playhead: None };
    let aims = crate::timeline_view::row_aims(s, &opts(false));
    let tells = s.enemy_intents();
    let mut masked = 0;
    for &e in s.w.enemies.iter().filter(|&&e| s.w.units[e].is_active()) {
        if tells.iter().find(|t| t.unit == e).is_some_and(|t| t.revealed) {
            continue;
        }
        masked += 1;
        let aim = aims.get(&crate::timeline_view::track_key(e)).expect("its aim");
        assert!(!aim.area, "unit {e}: no area shape from its secret plan");
        let want = match s.w.round_states.get(&e).and_then(|r| r.target) {
            Some(RoundTarget::Unit(t)) if s.w.enemies.contains(&t) => RowTarget::Ally(t),
            Some(RoundTarget::Unit(t)) => RowTarget::Foe(t),
            Some(RoundTarget::Cell(c)) => RowTarget::Ground(c),
            Some(RoundTarget::Myself) => RowTarget::Myself,
            None => RowTarget::None,
        };
        assert_eq!(aim.target, want, "unit {e}: its round state's target");
    }
    assert!(masked > 0, "an enemy plans in secret");
    // revealed (the Gallery's Reveal), the rows read the plan
    let open = crate::timeline_view::row_aims(s, &opts(true));
    for &e in s.w.enemies.iter().filter(|&&e| s.w.units[e].is_active()) {
        let planned = emberwake_battle::intent::planned_schedule(&s.w, &s.resolver.scheduled, &s.plan);
        assert_eq!(open[&crate::timeline_view::track_key(e)], sv::row_aim(&s.w, &planned, e), "unit {e}");
    }
}

/// PR7-75 (PT-48): a battle-long status (the `BATTLE_ROUNDS` sentinel, counted down) lists as lasting, never as the
/// number; a real count still shows.
#[test]
fn a_battle_long_status_shows_no_sentinel_number() {
    let (cx, _g, mut b) = arena();
    let h = b.session().w.allies[0];
    give(&cx, &mut b, h, "weaken", 10);
    let st = &mut b.host.session.as_mut().expect("a session").w.units[h].st;
    st.rounds_left.insert("weaken".into(), emberwake_battle::trigger::BATTLE_ROUNDS - 1);
    let line = tabs_of(&b.session().w, h, &[]).into_iter().find(|t| t.status == "weaken").expect("the weaken tab").line();
    assert!(line.ends_with(": lasts") && !line.contains("998"), "{line}");
    b.host.session.as_mut().expect("a session").w.units[h].st.rounds_left.insert("weaken".into(), 3);
    let line = tabs_of(&b.session().w, h, &[]).into_iter().find(|t| t.status == "weaken").expect("the weaken tab").line();
    assert!(line.ends_with(": 3 rounds"), "{line}");
    assert!(emberwake_battle::trigger::lasts_battle(emberwake_battle::trigger::BATTLE_ROUNDS - 30) && !emberwake_battle::trigger::lasts_battle(99));
}
