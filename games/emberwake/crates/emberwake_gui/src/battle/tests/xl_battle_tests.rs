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
