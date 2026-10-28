//! WINDUP1-E (spec WINDUP1 section 2): a clips.json row may carry `loop_from` (a looping clip's intro plays once, then
//! frames `loop_from..frames` loop) and `strike` (where the clip starts after a held wind-up, at most `impact`);
//! `ClipPlayer::play_from` starts a shot at a frame and `ClipPlayer::set_base_from` starts the base at a frame, held
//! still after the intro when asked. A row without the fields plays exactly as before.

use aegis_core::assets::{read_json, ClipPlayer, Clips, Layers};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn body(row: Value) -> Value {
    json!({"frame": [48, 48], "palette": {}, "clips": [{"id": "idle", "frames": 2, "ms": [100, 100]}, row]})
}

/// The spec's E2 clip: 7 frames, the first 3 an intro.
const WINDUP_MS: [u32; 7] = [90, 110, 120, 180, 180, 180, 180];

fn windup() -> Clips {
    Clips::parse(&body(json!({"id": "windup", "frames": 7, "ms": WINDUP_MS, "loop_from": 3}))).unwrap()
}

fn sum(ms: &[u32]) -> u64 {
    ms.iter().map(|m| *m as u64).sum()
}

#[test]
fn e1_each_broken_field_names_itself() {
    let cases = [
        (json!({"id": "windup", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "loop_from": 2}), "clip 'windup': loop_from needs loop true."),
        (json!({"id": "windup", "frames": 4, "ms": [100, 100, 100, 100], "loop_from": 0}), "clip 'windup': loop_from must be 1 to 3."),
        (json!({"id": "windup", "frames": 4, "ms": [100, 100, 100, 100], "loop_from": 4}), "clip 'windup': loop_from must be 1 to 3."),
        (json!({"id": "windup", "frames": 4, "ms": [100, 100, 100, 100], "loop_from": "2"}), "clip 'windup': loop_from must be 1 to 3."),
        (json!({"id": "attack", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "strike": 4}), "clip 'attack': strike must be 0 to 3."),
        (json!({"id": "attack", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "strike": -1}), "clip 'attack': strike must be 0 to 3."),
        (json!({"id": "attack", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "strike": 3, "impact": 2}), "clip 'attack': strike must not come after impact."),
    ];
    for (row, want) in cases {
        assert_eq!(Clips::parse(&body(row.clone())).unwrap_err(), vec![want.to_string()], "{row}");
    }
    // Every problem is listed.
    let both = Clips::parse(&body(json!({"id": "attack", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "loop_from": 1, "strike": 9}))).unwrap_err();
    assert_eq!(both, vec!["clip 'attack': loop_from needs loop true.".to_string(), "clip 'attack': strike must be 0 to 3.".to_string()]);
    // The edges are allowed and read.
    let ok = Clips::parse(&body(json!({"id": "attack", "frames": 4, "ms": [100, 100, 100, 100], "loop": false, "strike": 2, "impact": 2}))).unwrap();
    assert_eq!((ok.clip("attack").unwrap().strike, ok.clip("attack").unwrap().loop_from), (2, 0));
    let ok = Clips::parse(&body(json!({"id": "windup", "frames": 4, "ms": [100, 100, 100, 100], "loop_from": 3, "strike": 0}))).unwrap();
    assert_eq!((ok.clip("windup").unwrap().loop_from, ok.clip("windup").unwrap().strike), (3, 0));
    // Absent fields read as 0.
    let plain = Clips::parse(&body(json!({"id": "attack", "frames": 3, "ms": [100, 100, 100], "loop": false}))).unwrap();
    assert_eq!((plain.clip("attack").unwrap().loop_from, plain.clip("attack").unwrap().strike), (0, 0));
}

#[test]
fn e2_the_intro_plays_once_then_the_loop_part_repeats() {
    let c = windup();
    let w = c.clip("windup").unwrap();
    let intro = sum(&WINDUP_MS[..3]);
    let span = sum(&WINDUP_MS[3..]);
    assert_eq!(w.intro_ms(), intro);
    assert_eq!(c.ms_before("windup", 3), intro);
    // The spec's points: frame 2 at 250 ms, 3 at the intro's end, 6 at 900, 3 again one loop later.
    assert_eq!(w.frame_at(250), (2, false));
    assert_eq!(w.frame_at(intro), (3, false));
    assert_eq!(w.frame_at(900), (6, false));
    assert_eq!(w.frame_at(intro + span), (3, false));
    assert_eq!(c.frame_at("windup", 1040).unwrap().frame, 3, "1040 ms: one loop after the intro");
    // The intro frames never show again.
    for t in (intro..intro + 3 * span).step_by(10) {
        assert!(w.frame_at(t).0 >= 3, "{t} ms");
    }
    // A player with it as the base shows the same.
    let mut p = ClipPlayer::default();
    p.set_base("windup", 1000);
    assert_eq!(p.frame(&c, 1000 + 250).unwrap().frame, 2);
    assert_eq!(p.frame(&c, 1000 + 900).unwrap().frame, 6);
}

#[test]
fn e3_play_from_starts_at_its_frame_and_ends_early() {
    let ms = [110u32, 160, 45, 140, 110];
    let c = Clips::parse(&body(json!({"id": "attack", "frames": 5, "ms": ms, "loop": false, "strike": 2, "impact": 3}))).unwrap();
    let a = c.clip("attack").unwrap();
    let skip = sum(&ms[..2]);
    let len = a.total_ms() - skip;
    let mut p = ClipPlayer::default();
    p.play_from("attack", 500, 2);
    assert_eq!(p.frame(&c, 500).unwrap().frame, 2, "the first frame shown is the start frame");
    assert_eq!(p.frame(&c, 500 + ms[2] as u64).unwrap().frame, 3, "then the clip runs on its own durations");
    // The impact lands the ms between the start frame and the impact frame after the start.
    assert_eq!(a.impact_ms().unwrap() - a.ms_before(a.strike), ms[2] as u64);
    assert_eq!(p.playing(&c, 500 + len - 1), "attack");
    assert_eq!(p.playing(&c, 500 + len), "", "over at the total less the skipped frames");
    assert_eq!(p.frame(&c, 500 + len).unwrap().clip, "idle", "then the base");
    // From frame 0 it is `play`.
    let (mut a0, mut b0) = (ClipPlayer::default(), ClipPlayer::default());
    a0.play("attack", 0);
    b0.play_from("attack", 0, 0);
    for t in (0..a.total_ms() + 200).step_by(5) {
        assert_eq!(a0.frame(&c, t), b0.frame(&c, t), "{t} ms");
    }
}

#[test]
fn e4_a_still_base_holds_its_loop_frame_after_the_intro() {
    let c = windup();
    let intro = c.clip("windup").unwrap().intro_ms();
    let mut p = ClipPlayer::default();
    p.set_base_from("windup", 0, 0, true);
    assert_eq!(p.frame(&c, 250).unwrap().frame, 2, "the intro still plays");
    for t in (intro..intro + 3000).step_by(7) {
        assert_eq!(p.frame(&c, t).unwrap().frame, 3, "{t} ms: held");
    }
    // Started at the loop frame and still: frame 3 at once and for good.
    let mut q = ClipPlayer::default();
    q.set_base_from("windup", 100, 3, true);
    for t in (100..3100).step_by(7) {
        assert_eq!(q.frame(&c, t).unwrap().frame, 3, "{t} ms");
    }
    // Started at the loop frame and moving: frame 3 at once, then the loop runs.
    let mut r = ClipPlayer::default();
    r.set_base_from("windup", 100, 3, false);
    assert_eq!(r.frame(&c, 100).unwrap().frame, 3);
    assert_eq!(r.frame(&c, 100 + WINDUP_MS[3] as u64).unwrap().frame, 4);
    // An unchanged base keeps its clock; only `still` follows.
    r.set_base_from("windup", 5000, 0, false);
    assert_eq!(r.frame(&c, 100 + WINDUP_MS[3] as u64).unwrap().frame, 4, "the clock did not restart");
    r.set_base_from("windup", 5000, 0, true);
    assert_eq!(r.frame(&c, 6000).unwrap().frame, 3, "still now: the loop frame holds");
    // A loop without an intro held still shows its frame 0.
    let mut i = ClipPlayer::default();
    i.set_base_from("idle", 0, 0, true);
    assert_eq!(i.frame(&c, 150).unwrap().frame, 0);
    // A shot over a still base plays as usual.
    p.play("idle", 4000);
    assert_eq!(p.frame(&c, 4150).unwrap().frame, 1);
}

/// Today's rule for a row without `loop_from`: a loop wraps at its total, a one-shot holds its last frame.
fn todays_frame(ms: &[u32], looped: bool, t: u64) -> u32 {
    let total = sum(ms);
    if total == 0 {
        return 0;
    }
    let t = if looped { t % total } else { t.min(total - 1) };
    let mut acc = 0;
    for (i, m) in ms.iter().enumerate() {
        acc += *m as u64;
        if t < acc {
            return i as u32;
        }
    }
    ms.len() as u32 - 1
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
fn e5_every_committed_body_parses_and_plays_its_frames_as_before() {
    let games = workspace_games();
    if games.is_empty() {
        eprintln!("no games/ folder with game.json and assets/: nothing to compare");
        return;
    }
    let (mut bodies, mut checked) = (0, 0);
    for game in games {
        let layers = Layers::for_game(&game);
        for key in layers.bodies() {
            let rel = aegis_core::assets::body_file(&key, "clips.json");
            let v = read_json(&layers, &rel).unwrap_or_else(|| panic!("{rel}: unreadable"));
            let clips = Clips::parse(&v).unwrap_or_else(|e| panic!("{rel}: {e:?}"));
            bodies += 1;
            for c in clips.clips.iter().filter(|c| c.loop_from == 0) {
                let (mut base, mut base_from, mut shot, mut shot_from) = (ClipPlayer::default(), ClipPlayer::default(), ClipPlayer::default(), ClipPlayer::default());
                base.set_base(&c.id, 0);
                base_from.set_base_from(&c.id, 0, 0, false);
                shot.play(&c.id, 0);
                shot_from.play_from(&c.id, 0, 0);
                for t in [0u64, 45, 130, 333, 900, 4_000] {
                    let want = todays_frame(&c.ms, c.looped, t);
                    assert_eq!(c.frame_at(t).0, want, "{key} {} at {t} ms", c.id);
                    if c.view == aegis_core::assets::View::Side {
                        assert_eq!(base.frame(&clips, t), base_from.frame(&clips, t), "{key} {} base at {t} ms", c.id);
                        assert_eq!(base.frame(&clips, t).unwrap().frame, want, "{key} {} base at {t} ms", c.id);
                        assert_eq!(shot.frame(&clips, t), shot_from.frame(&clips, t), "{key} {} shot at {t} ms", c.id);
                    }
                    checked += 1;
                }
            }
        }
    }
    eprintln!("{bodies} bodies, {checked} frames compared");
    assert!(bodies > 0, "the workspace's games hold bodies");
}
