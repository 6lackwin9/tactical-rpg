//! DIR4-E (spec DIR4 section 3): direction views in the clip sheet. clips.json rows with `of`
//! and `view` (checks V1-V4), four-way `Facing::toward`, the row lookup with its fallback order
//! (`Clips::resolve_view`, `frame_at_view`, `ClipPlayer::frame_facing`) and `Art::played_facing`, which mirrors only
//! side rows. Expectations follow from the spec's rules and the fixture's own durations; the last test replays every
//! body of every game in the workspace through both the side-only and the four-way path.

use aegis_core::assets::{ClipPlayer, Clips, Facing, Layers, View};
use aegis_ui::art::Art;
use aegis_ui::canvas::{Image, Rgba};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

const F: u32 = 8;

/// (id, of, view, ms per frame, loop) of the fixture body, in row order (side clips first, then the view clips).
fn fixture() -> Vec<(&'static str, &'static str, &'static str, Vec<u32>, bool)> {
    vec![
        ("idle", "", "", vec![100, 100], true),
        ("walk", "", "", vec![80, 80], true),
        ("attack", "", "", vec![50, 60, 70], false),
        ("walk_up", "walk", "up", vec![80, 80], true),
        ("attack_down", "attack", "down", vec![50, 60, 70], false),
    ]
}

fn row_json(id: &str, of: &str, view: &str, ms: &[u32], looped: bool) -> Value {
    let mut r = json!({"id": id, "frames": ms.len(), "ms": ms, "loop": looped});
    if !of.is_empty() {
        r["of"] = json!(of);
        r["view"] = json!(view);
    }
    r
}

fn clips_json() -> Value {
    let rows: Vec<Value> = fixture().iter().map(|(id, of, view, ms, l)| row_json(id, of, view, ms, *l)).collect();
    json!({"frame": [F, F], "palette": {"main": "#0a0ac8"}, "clips": rows})
}

fn simple(rows: Value) -> Value {
    json!({"frame": [F, F], "clips": rows})
}

/// The colour of frame `n` of row `row` in the fixture sheet.
fn colour(row: usize, n: u32) -> [u8; 4] {
    [10 + row as u8 * 40, 10 + n as u8 * 40, 200, 255]
}

fn write_png(path: &Path, w: u32, h: u32, px: &[u8]) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    let file = std::fs::File::create(path).unwrap();
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(px).unwrap();
}

/// A scratch asset root with body `hero` (the fixture clips): every frame lopsided (a 3-pixel bar at x 1..3 and one
/// pixel at x 6, row 3) in its own colour, so a mirrored frame differs from the plain one.
fn body_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("aegis_ui_clip_views_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let rows = fixture();
    let cols = rows.iter().map(|r| r.3.len() as u32).max().unwrap();
    let (w, h) = (cols * F, rows.len() as u32 * F);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for (row, r) in rows.iter().enumerate() {
        for n in 0..r.3.len() as u32 {
            for x in [1u32, 2, 3, 6] {
                let (sx, sy) = (n * F + x, row as u32 * F + 3);
                let i = ((sy * w + sx) * 4) as usize;
                px[i..i + 4].copy_from_slice(&colour(row, n));
            }
        }
    }
    let dir = root.join("units/hero/base");
    write_png(&dir.join("sheet.png"), w, h, &px);
    std::fs::write(dir.join("clips.json"), clips_json().to_string()).unwrap();
    root
}

#[test]
fn view_rows_are_checked_and_side_only_bodies_parse_as_before() {
    let idle = row_json("idle", "", "", &[100], true);
    let walk = row_json("walk", "", "", &[80, 80], true);
    let bad = Clips::parse(&simple(json!([
        idle,
        walk,
        row_json("walk_left", "walk", "left", &[80, 80], true),
        {"id": "walk_down", "view": "down", "frames": 1, "ms": [80]},
        {"id": "walk_side", "of": "walk", "frames": 1, "ms": [80]},
        row_json("run_up", "run", "up", &[80], true),
        row_json("walk_up", "walk", "up", &[80, 80], true),
        row_json("walk_up_up", "walk_up", "up", &[80, 80], true),
        row_json("stride_up", "walk", "up", &[80, 80], true),
        row_json("walk_up", "walk", "up", &[80, 80], true),
    ])))
    .unwrap_err();
    assert_eq!(
        bad,
        [
            "clip 'walk_left': view must be down or up.",
            "clip 'walk_down': of and view go together.",
            "clip 'walk_side': of and view go together.",
            "clip 'walk_up' is listed twice.",
            "clip 'run_up': of 'run' is not a side clip of this body.",
            "clip 'walk_up_up': of 'walk_up' is not a side clip of this body.",
            "clip 'stride_up': a view clip's id must be 'walk_up'.",
        ],
        "V1 to V4 and the duplicate rule, one message each"
    );
    let idle_as_view = Clips::parse(&simple(json!([walk, row_json("idle", "walk", "up", &[80], true)]))).unwrap_err();
    assert_eq!(idle_as_view, ["clip 'idle': a view clip's id must be 'walk_up'.", "clips must include 'idle' (the token)."], "idle must be a side clip");
    let any_order = Clips::parse(&simple(json!([row_json("walk_up", "walk", "up", &[80, 80], true), idle, walk]))).unwrap();
    assert_eq!((any_order.row("walk_up"), any_order.view_row("walk", View::Up)), (Some(0), Some(0)), "view rows may come first");
    // Side clips only: every row is a side row and every lookup is today's `resolve`, in every view.
    let side_only = Clips::parse(&simple(json!([idle, walk, row_json("attack", "", "", &[50, 60], false)]))).unwrap();
    assert!(side_only.clips.iter().all(|c| c.of.is_empty() && c.view == View::Side));
    for id in ["idle", "walk", "attack", "cast"] {
        for view in [View::Side, View::Down, View::Up] {
            assert_eq!(side_only.resolve_view(id, view), side_only.resolve(id).map(|r| (r, true)), "{id} {view:?}");
        }
    }
}

#[test]
fn a_view_falls_back_to_its_side_clip_then_idle_in_that_view_then_side_idle() {
    let base = simple(json!([row_json("idle", "", "", &[100, 100], true), row_json("walk", "", "", &[80, 80], true), row_json("walk_up", "walk", "up", &[80, 80], true)]));
    let c = Clips::parse(&base).unwrap();
    let (idle, walk, walk_up) = (c.row("idle").unwrap(), c.row("walk").unwrap(), c.row("walk_up").unwrap());
    assert_eq!(c.resolve_view("walk", View::Up), Some((walk_up, false)), "the view row, not a side row");
    assert_eq!(c.resolve_view("walk", View::Down), Some((walk, true)), "no walk_down: the side walk");
    assert_eq!(c.resolve_view("walk", View::Side), Some((walk, true)));
    assert_eq!(c.resolve_view("cast", View::Up), Some((idle, true)), "no cast at all, no idle_up: the side idle");
    let mut with_idle_up = base.clone();
    with_idle_up["clips"].as_array_mut().unwrap().push(row_json("idle_up", "idle", "up", &[100, 100], true));
    let c = Clips::parse(&with_idle_up).unwrap();
    let idle_up = c.row("idle_up").unwrap();
    assert_eq!(c.resolve_view("cast", View::Up), Some((idle_up, false)), "idle in that view before side idle");
    assert_eq!(c.resolve_view("cast", View::Down), Some((idle, true)));
    assert_eq!(c.resolve_view("cast", View::Side), Some((idle, true)), "the side view never takes a view row");
    let f = c.frame_at_view("cast", View::Up, 150).unwrap();
    assert_eq!((f.clip.as_str(), f.frame, f.side), ("idle_up", 1, false), "an idle fallback animates on its own clock");
    assert!(Clips::default().resolve_view("walk", View::Up).is_none(), "no clips: nothing to draw");
}

#[test]
fn a_view_row_keeps_its_side_clips_clock() {
    let c = Clips::parse(&clips_json()).unwrap();
    for t in (0..400).step_by(7) {
        let (side, up) = (c.frame_at("walk", t).unwrap(), c.frame_at_view("walk", View::Up, t).unwrap());
        assert_eq!((up.clip.as_str(), up.side, side.side), ("walk_up", false, true));
        assert_eq!((up.frame, up.done), (side.frame, side.done), "walk at {t} ms");
    }
    // A turn in mid-clip: the player keeps the side id and its clock; the drawn row changes, the frame index does not.
    let attack: u64 = 50 + 60 + 70;
    let mut p = ClipPlayer::default();
    p.play("attack", 1_000);
    for t in [1_000, 1_049, 1_050, 1_115, 1_000 + attack - 1] {
        let (side, down, up) = (p.frame_facing(&c, t, View::Side).unwrap(), p.frame_facing(&c, t, View::Down).unwrap(), p.frame_facing(&c, t, View::Up).unwrap());
        assert_eq!((side.clip.as_str(), down.clip.as_str(), up.clip.as_str()), ("attack", "attack_down", "attack"), "no attack_up: the side attack");
        assert_eq!((down.frame, up.frame), (side.frame, side.frame), "at {t} ms");
        assert_eq!(p.playing(&c, t), "attack");
        assert_eq!(side, p.frame(&c, t).unwrap(), "frame is frame_facing in the side view");
    }
    assert_eq!(p.frame_facing(&c, 1_000 + attack, View::Down).unwrap().clip, "idle", "over: the base loop (no idle_down)");
    p.play_and_hold("attack", 2_000);
    let held = p.frame_facing(&c, 9_000, View::Down).unwrap();
    assert_eq!((held.clip.as_str(), held.frame, held.done), ("attack_down", 2, true), "a held shot keeps its view row's last frame");
    assert_eq!(p.held_for(&c, 2_000 + attack + 40), Some(40));
    p.stop();
    p.set_base("walk", 3_000);
    let f = p.frame_facing(&c, 3_090, View::Up).unwrap();
    assert_eq!((f.clip.as_str(), f.frame), ("walk_up", 1), "the base loop in its view");
}

#[test]
fn facing_turns_toward_the_larger_distance_and_a_tie_stays_side_on() {
    let right = Facing::default();
    assert_eq!(right, Facing { view: View::Side, left: false }, "default: side, right");
    let left = Facing::side(true);
    assert_eq!(left.toward(0, 1), Facing { view: View::Down, left: true }, "down keeps left");
    assert_eq!(right.toward(0, -2), Facing { view: View::Up, left: false }, "up");
    assert_eq!(left.toward(2, 1), Facing::side(false), "more across than down: side, right");
    assert_eq!(right.toward(-1, -1), Facing::side(true), "a diagonal tie: side, left");
    assert_eq!(right.toward(-3, 0), Facing::side(true));
    let up_left = Facing { view: View::Up, left: true };
    assert_eq!(up_left.toward(0, 0), up_left, "no offset: unchanged");
    assert!(up_left.toward(1, -3).left, "a steep step keeps the last left or right");
}

fn at(img: &Image, x: i32, y: i32) -> [u8; 4] {
    let i = ((y * img.w + x) * 4) as usize;
    [img.px[i], img.px[i + 1], img.px[i + 2], img.px[i + 3]]
}

#[test]
fn played_facing_mirrors_side_rows_only_and_its_outline_matches() {
    let root = body_root("played");
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let size = F as i32 * 3;
    let gold: Rgba = [244, 198, 84, 255];
    let mut p = ClipPlayer::default();
    p.set_base("walk", 0);
    let plain = |art: &mut Art, clip: &str, n: u32| art.body_clip("hero", "", clip, n, size).unwrap().clone();
    for (facing, clip, mirrored) in [
        (Facing::side(false), "walk", false),
        (Facing::side(true), "walk", true),
        (Facing { view: View::Up, left: true }, "walk_up", false),
        (Facing { view: View::Up, left: false }, "walk_up", false),
        (Facing { view: View::Down, left: true }, "walk", true),
        (Facing { view: View::Down, left: false }, "walk", false),
    ] {
        let (f, img) = art.played_facing("hero", "", &p, 90, size, facing).map(|(f, i)| (f, i.clone())).unwrap();
        assert_eq!((f.clip.as_str(), f.frame, facing.mirrors(&f)), (clip, 1, mirrored), "{facing:?}");
        let sheet = plain(&mut art, clip, 1);
        assert_eq!(img, if mirrored { sheet.mirrored() } else { sheet.clone() }, "{facing:?}: the sheet frame, mirrored only for a left side row");
        assert_ne!(sheet.mirrored(), sheet, "the fixture is lopsided, so mirroring shows");
        let row = art.clips("hero").unwrap().row(clip).unwrap();
        let x = if mirrored { size - 1 - (3 * 3 + 1) } else { 3 * 3 + 1 };
        assert_eq!(at(&img, x, 3 * 3 + 1), colour(row, 1), "{facing:?}: the bar pixel from the drawn side");
        let o = art.body_outline("hero", "", &f.clip, f.frame, size, gold, facing.mirrors(&f)).unwrap().clone();
        assert_eq!(o, img.outline(gold), "{facing:?}: the outline rings the frame as drawn");
    }
    std::fs::remove_dir_all(&root).unwrap();
    assert!(art.played_facing("hero", "", &p, 90, size, Facing { view: View::Up, left: true }).is_some(), "cached: the files are gone");
}

/// Every game folder of the workspace (holding game.json and assets/), without naming one.
fn workspace_games() -> Vec<PathBuf> {
    let games = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../games");
    let Ok(read) = std::fs::read_dir(&games) else { return Vec::new() };
    let mut out: Vec<PathBuf> = read.flatten().map(|e| e.path()).filter(|d| d.join("game.json").is_file() && d.join("assets").is_dir()).collect();
    out.sort();
    out
}

#[test]
fn every_committed_body_draws_as_before_through_played_facing() {
    let games = workspace_games();
    if games.is_empty() {
        eprintln!("no games/ folder with game.json and assets/: nothing to compare");
        return;
    }
    let size = 28;
    let gold: Rgba = [244, 198, 84, 255];
    let (mut bodies, mut frames) = (0, 0);
    for game in games {
        let mut art = Art::layered(Layers::for_game(&game));
        for key in art.layers().bodies() {
            let Some(clips) = art.clips(&key).cloned() else { continue };
            bodies += 1;
            let has_views = clips.clips.iter().any(|c| c.view != View::Side);
            let mut ids: Vec<String> = clips.clips.iter().filter(|c| c.view == View::Side).map(|c| c.id.clone()).collect();
            ids.push("no_such_clip".into());
            for id in &ids {
                let mut p = ClipPlayer::default();
                p.play(id, 0);
                for t in [0u64, 45, 130, 333, 900, 4_000] {
                    for left in [false, true] {
                        let (f0, i0) = art.played(&key, "", &p, t, size, left).map(|(f, i)| (f, i.clone())).unwrap();
                        let o0 = art.body_outline(&key, "", &f0.clip, f0.frame, size, gold, left).unwrap().clone();
                        // Today's side view always; the front and back too while the body has no view rows.
                        let views: &[View] = if has_views { &[View::Side] } else { &[View::Side, View::Down, View::Up] };
                        for &view in views {
                            let facing = Facing { view, left };
                            let (f1, i1) = art.played_facing(&key, "", &p, t, size, facing).map(|(f, i)| (f, i.clone())).unwrap();
                            assert_eq!((&f1, facing.mirrors(&f1)), (&f0, left), "{key} {id} at {t} ms, {facing:?}");
                            assert!(i1 == i0, "{key} {id} at {t} ms, {facing:?}: the frame differs");
                            let o1 = art.body_outline(&key, "", &f1.clip, f1.frame, size, gold, facing.mirrors(&f1)).unwrap();
                            assert!(*o1 == o0, "{key} {id} at {t} ms, {facing:?}: the outline differs");
                            frames += 1;
                        }
                    }
                }
            }
        }
    }
    eprintln!("{bodies} bodies, {frames} frames compared");
    assert!(bodies > 0, "the workspace's games hold bodies");
}
