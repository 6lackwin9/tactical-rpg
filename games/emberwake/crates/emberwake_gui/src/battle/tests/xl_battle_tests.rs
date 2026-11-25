//! PR7-63 (XL-BATTLE1, `FB-376` finding): the battle screen reads clean at Normal and Extra large text on the Vane
//! boss fight (sunk_ward `enc_evt_01`) and the bone-mill raid with a guest (`enc_evt_03`), on both views: no text cut
//! without its full words on hover, past the canvas, overlapping another or spilling out of its control.

use crate::battle::BattleScreen;
use crate::screens::{Cx, Screen};
use aegis_core::json::{self, s};
use aegis_ui::painter::TextRec;
use aegis_ui::ui::{Gfx, H, W};
use emberwake_rules::ruleset::TIMELINE;
use serde_json::{json, Value};

fn fight(encounter: &str, view: &str) -> (Cx, Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    if std::env::var("XL_SHOTS").is_ok() {
        // review shots with the game's art
        cx.art = aegis_ui::art::Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    }
    cx.settings.v["battle_view"] = json!(view);
    let mut b = BattleScreen::arena_with(&cx, Some("sunk_ward".into()), encounter, Some(TIMELINE.to_string()), "", false);
    assert!(b.host.session.is_some(), "the battle starts: {}", b.message);
    let mut g = crate::hud::hud_tests::gfx();
    b.tick(&mut cx);
    // the opening talk closed (its replies the first), so the screen under it is audited
    for _ in 0..40 {
        if b.talk.is_none() {
            break;
        }
        b.talk_reply(&mut cx, crate::talk::TalkAct::Advance);
        if b.talk.is_some() {
            b.talk_reply(&mut cx, crate::talk::TalkAct::Choose(0));
        }
    }
    assert!(b.talk.is_none(), "the opening talk closes");
    b.tick(&mut cx);
    if b.selected.is_none() {
        b.selected = b.session().w.allies.first().copied();
    }
    b.draw(&mut cx, &mut g, (-500, -500), true);
    (cx, g, b)
}

/// The audit of one frame at text size `scale`: (issues, texts drawn).
fn audit(b: &mut BattleScreen, cx: &mut Cx, g: &mut Gfx, scale: f32) -> (Value, Vec<TextRec>) {
    crate::hud::hud_tests::frame(b, cx, g, scale);
    let recs = crate::hud::hud_tests::frame(b, cx, g, scale);
    let boxes: Vec<(String, f32, f32, f32, f32)> = b
        .widgets()
        .iter()
        .filter(|w| !board_box(&s(json::dict(w), "id")))
        .map(|w| {
            let (x, y, ww, hh) = crate::hud::hud_tests::rect(w);
            (s(json::dict(w), "id"), x as f32, y as f32, ww as f32, hh as f32)
        })
        .collect();
    (aegis_ui::text_audit::issues(&recs, W as f32, H as f32, &boxes), recs)
}

/// A board cell or a unit on it: pictures, not text controls (a token's letter or a talk line over the board is not
/// held by them).
fn board_box(id: &str) -> bool {
    id.starts_with("tile.") || (id.starts_with("unit.") && id[5..].chars().all(|c| c.is_ascii_digit()))
}

/// The cut texts with no hover that gives their full words: no widget under the text's start whose hover lines (or
/// text, with a hover) hold them.
fn cut_without_hover(b: &BattleScreen, recs: &[TextRec]) -> Vec<String> {
    let ws = b.widgets();
    recs.iter()
        .filter(|r| !r.clipped && !r.covered && !r.transient)
        .filter_map(|r| r.cut.as_ref().map(|full| (r, full)))
        .filter(|(r, full)| {
            !ws.iter().any(|w| {
                let (x, y, ww, hh) = crate::hud::hud_tests::rect(w);
                let (px, py) = (r.x + 1.0, r.y + r.h / 2.0);
                let inside = px >= x as f32 && px <= (x + ww) as f32 && py >= y as f32 && py <= (y + hh) as f32;
                inside && json::aget(json::dict(w), "tip").iter().any(|t| t.as_str().is_some_and(|t| t.contains(full.as_str())))
            })
        })
        .map(|(_, full)| full.clone())
        .collect()
}

/// GUI audit (PR7-63): the Vane boss fight, the cage fight (two caged guests) and the bone-mill raid (a guest), on both
/// views at Normal and Extra large: nothing past the canvas, overlapping or spilling out of its control, and every cut
/// text hovers to its full words. `XL_SHOTS=<dir>` also writes each frame there for review.
#[test]
fn the_battle_reads_clean_at_normal_and_extra_large() {
    let mut problems = Vec::new();
    for enc in ["enc_evt_01", "enc_evt_02", "enc_evt_03"] {
        for view in ["stage", "flat"] {
            for (size, scale) in [("Normal", 1.0f32), ("Extra large", 1.3)] {
                let (mut cx, mut g, mut b) = fight(enc, view);
                let (v, recs) = audit(&mut b, &mut cx, &mut g, scale);
                if let Ok(dir) = std::env::var("XL_SHOTS") {
                    let _ = aegis_ui::shell::save_png(&g.canvas, &std::path::Path::new(&dir).join(format!("{enc}_{view}_{}.png", size.replace(' ', "_"))), 2000);
                }
                let c = &v["counts"];
                if c["edge"] != json!(0) || c["overlap"] != json!(0) || c["spill"] != json!(0) {
                    problems.push(format!("{enc} {view} {size}: {}", json!({"edge": v["edge"], "overlap": v["overlap"], "spill": v["spill"]})));
                }
                let bare = cut_without_hover(&b, &recs);
                if !bare.is_empty() {
                    problems.push(format!("{enc} {view} {size}: cut with no hover {bare:?}"));
                }
            }
        }
    }
    assert!(problems.is_empty(), "{problems:#?}");
}

/// At Normal the strip's lines keep their old places (the goal at 11, the protect line 6 below); at Extra large the
/// goal starts under the title's letters.
#[test]
fn the_strip_lines_follow_the_text_size() {
    use aegis_ui::ui::{BODY, SMALL};
    let mut g = crate::hud::hud_tests::gfx();
    let mut lines = |scale: f32| {
        g.fonts.scale = scale;
        let p = aegis_ui::painter::Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        crate::battle::draw::strip_lines(p.line_h(BODY), p.line_h(SMALL))
    };
    assert_eq!(lines(1.0), (11, 6));
    let (goal, step) = lines(1.3);
    assert!(goal > 11 && step > 6, "{goal} {step}");
}

/// XL-FIX2 (PR7-85, FB-455 a): the strip's title reads "Round 1 of N" up to Normal and "R 1/N" at Large and Extra
/// large, whole (never cut), hovering to the long form when short; the goal line under it is clear of it (no overlap
/// as the text audit reads one), and at Normal the goal keeps its place (11).
#[test]
fn the_strip_title_shortens_at_large_text_and_clears_the_goal() {
    for (size, scale) in aegis_ui::screen::TEXT_SIZES {
        let (mut cx, mut g, mut b) = fight("enc_evt_01", "stage");
        let recs = crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
        let (round, max) = (b.session().current_round.max(1), b.session().def.max_rounds);
        let long = format!("Round {round} of {max}");
        let want = if scale > 1.0 { format!("R {round}/{max}") } else { long.clone() };
        let title = recs.iter().find(|r| r.y == 3.0 && (r.text.starts_with("Round") || r.text.starts_with("R "))).unwrap_or_else(|| panic!("the title at {size}"));
        assert_eq!((title.text.as_str(), title.cut.is_none()), (want.as_str(), true), "at {size}");
        let w = b.widgets().into_iter().find(|w| w["id"] == "strip.title").expect("the title widget");
        assert_eq!(w["text"], json!(long), "the widget names the long form at {size}");
        let tip: Vec<String> = json::aget(json::dict(&w), "tip").iter().filter_map(|t| t.as_str().map(str::to_string)).collect();
        assert_eq!(tip.contains(&long), scale > 1.0, "the short title hovers to the long form at {size}: {tip:?}");
        let goal = b.widgets().into_iter().find(|w| w["id"] == "strip.goal").expect("the goal line");
        let (gx, gy, _, _) = crate::hud::hud_tests::rect(&goal);
        let line = recs.iter().find(|r| r.y == gy as f32 && r.x == gx as f32).expect("the goal's text");
        let shared = (title.y + title.h).min(line.y + line.h) - line.y;
        assert!(shared <= aegis_ui::text_audit::OVERLAP_Y * title.h.min(line.h), "at {size}: the goal at {gy} meets the title ({shared} shared)");
        if (scale - 1.0).abs() < 1e-6 {
            assert_eq!(gy, 11, "at Normal the goal keeps its place");
        }
    }
}

/// The reinforcement tag keeps its own cell: at Extra large "Boss above" takes a row under "Hostiles N" and the plates
/// start under it; at Normal it stays beside the count.
#[test]
fn the_boss_above_tag_takes_its_own_row_when_it_would_meet_the_count() {
    for (scale, below) in [(1.0f32, false), (1.3, true)] {
        let (mut cx, mut g, mut b) = fight("enc_evt_03", "stage");
        crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale);
        let tag = b.widgets().into_iter().find(|w| w["id"] == "foe.reinforce").expect("the tag");
        let (_, ty, _, _) = crate::hud::hud_tests::rect(&tag);
        assert_eq!(ty > 31, below, "at {scale}: tag at y {ty}");
    }
}

/// ENGINE-33 (XL-SWEEP1): a plate keeps "hp/max" while it fits beside the planned loss, and drops the maximum when the
/// two would run past the plate (the sweep's "120/120 -120" on a Hostiles row at Extra large).
#[test]
fn a_plate_drops_its_maximum_before_the_loss_runs_past_it() {
    use crate::battle::plates::hp_fit;
    let mut g = crate::hud::hud_tests::gfx();
    let mut fit = |scale: f32, incoming: i32, room: f32| {
        g.fonts.scale = scale;
        let mut p = aegis_ui::painter::Painter { c: &mut g.canvas, v: g.view, fonts: &mut g.fonts };
        hp_fit(&mut p, 120, 120, incoming, 0, room)
    };
    assert_eq!(fit(1.0, 0, 10.0), "120/120", "no loss: always the whole numbers");
    assert_eq!(fit(1.0, 120, 70.0), "120/120", "Normal: both fit");
    assert_eq!(fit(1.3, 120, 40.0), "120", "Extra large: the maximum goes first");
}

/// ENGINE-33 (XL-SWEEP1): on the docked card a foe's name stops short of the HP numbers under it at every text size
/// (the sweep's "Watch Crossbowman" over "136/136" at Extra large), on the Vane fight with each foe's card shown.
#[test]
fn the_card_name_stops_short_of_the_hp_numbers() {
    let mut met = 0;
    for (_, scale) in aegis_ui::screen::TEXT_SIZES {
        let (mut cx, mut g, mut b) = fight("enc_evt_01", "stage");
        let foes: Vec<usize> = b.session().w.enemies.clone();
        for f in foes {
            let Some(plate) = b.w.iter().find(|w| w.id == format!("foe.{f}")).map(|w| w.rect) else { continue };
            crate::hud::hud_tests::frame(&mut b, &mut cx, &mut g, scale as f32);
            b.draw(&mut cx, &mut g, (plate.x + plate.w / 2, plate.y + plate.h / 2), true);
            let (Some(name), Some(hp)) = (b.w.iter().find(|w| w.id == "card.name").map(|w| (w.rect, w.text.clone())), b.w.iter().find(|w| w.id == "card.hp").map(|w| w.rect)) else { continue };
            met += 1;
            let (n, text) = name;
            let rows_meet = n.y + n.h > hp.y && hp.y + hp.h > n.y;
            assert!(!rows_meet || n.x + n.w <= hp.x + 1, "at {scale}: {text:?} ({n:?}) runs into the HP ({hp:?})");
            if let Some(kept) = text.strip_suffix("...") {
                let w = b.w.iter().find(|w| w.id == "card.name").expect("the name");
                assert!(w.tip.iter().any(|t| t.0.starts_with(kept) && t.0.len() > kept.len()), "at {scale}: the cut name {text:?} hovers to its full words: {:?}", w.tip);
            }
        }
    }
    assert!(met > 0, "a card was shown");
}
