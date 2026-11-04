mod pacing_tests;
mod windup_tests;
mod arena_rules_tests;
mod status_tests;
mod input1_tests;
mod overlay_tests;
mod tells_tests;
mod slice5_tests;
mod slice6_tests;
mod hook_marks_tests;
mod meter_guard_tests;
mod push_forecast_tests;
mod hooks_audit_tests;
mod hold_screen_tests;
mod talk_screen_tests;
mod visclar_tests;
mod timeline_view_tests;
mod playback_tests;
mod script_screen_tests;
mod portrait_panel_tests;
mod portrait_timeline_tests;
mod strip_block_tests;

use crate::battle::flow::{boss_fight, sound_of};
use crate::battle::{Action, BattleScreen};
use crate::battle::tokens::{action_clips, token_frame, token_outline, turn, Tokens};
use crate::fx;
use crate::screens::Screen;
use aegis_ui::fx::Layer;
use crate::stage::{Geom, View};
use emberwake_battle::grid::GRID_H;
use emberwake_battle::command::CmdKind;
use emberwake_battle::grid::{V2, NO_CELL};
use aegis_core::assets::{ClipPlayer, Clips, Facing, Layers, View as Look};
use aegis_ui::art::Art;
use emberwake_battle::session::{EncounterDef, Phase, Session};
use emberwake_host::{Battle, Host};
use emberwake_rules::Stance;
use emberwake_world::battle::WorldRoster;
use emberwake_world::Data;
use serde_json::json;
use std::sync::Arc;

#[test]
fn an_action_picks_the_rig_clip_of_its_skill_kind_and_weapon() {
    assert_eq!(action_clips("shield_bash", "attack", "sword"), ["shield_bash", "attack"], "a skill with its own clip first");
    assert_eq!(action_clips("cyclone_kick", "area", ""), ["sweep_kick", "sweep_kick", "kick", "jab", "attack"]);
    assert_eq!(action_clips("strike", "attack", "fist"), ["jab", "attack"], "fists jab");
    assert_eq!(action_clips("caustic_vomit", "attack", ""), ["spit", "jab", "attack"], "the crawler spits, other bodies fall back");
    assert_eq!(action_clips("strike", "attack", "bow"), ["attack"]);
    assert_eq!(action_clips("volley", "area", "bow"), ["attack"]);
    assert_eq!(action_clips("mend", "support", "mace"), ["cast"]);
    assert!(action_clips("guard", "", "sword").is_empty(), "guards play through the stance, moves through the slide");
}

/// The rig's sheets face right: a token facing left (FACE1) shows the sheet frame mirrored, its outline with it; one
/// facing right the sheet frame as drawn. A scratch body whose frame is lopsided shows the difference.
#[test]
fn a_token_facing_left_is_the_sheet_frame_mirrored_and_one_facing_right_is_not() {
    const F: u32 = 8;
    let root = std::env::temp_dir().join(format!("emberwake_foe_facing_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("units/brawler/goblin");
    std::fs::create_dir_all(&dir).unwrap();
    // A bar on the left of the 8x8 frame (x 1..3, the back) and one pixel on the right (x 6, the face).
    let mut px = vec![0u8; (F * F * 4) as usize];
    for x in [1u32, 2, 3, 6] {
        let i = ((4 * F + x) * 4) as usize;
        px[i..i + 4].copy_from_slice(&[90, 140, 60, 255]);
    }
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
    std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": [{"id": "idle", "frames": 1, "ms": [100]}]}).to_string()).unwrap();
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let (body, p, size) = ("brawler/goblin", ClipPlayer::default(), 3 * F as i32);
    let sheet = art.body_clip(body, "", "idle", 0, size).unwrap().clone();
    let left = token_frame(&mut art, body, "", &p, 0, size, Facing::side(true)).unwrap().1.clone();
    let right = token_frame(&mut art, body, "", &p, 0, size, Facing::side(false)).unwrap().1.clone();
    assert_eq!(left, sheet.mirrored(), "facing left: the sheet frame mirrored");
    assert_eq!(right, sheet, "facing right: as the sheet faces");
    assert_ne!(left, right);
    let f = token_frame(&mut art, body, "", &p, 0, size, Facing::side(true)).unwrap().0;
    let gold = [244, 198, 84, 255];
    let left_ring = token_outline(&mut art, body, "", &f, size, gold, Facing::side(true)).unwrap().clone();
    assert_eq!(left_ring, sheet.mirrored().outline(gold), "the selection outline follows the mirrored frame");
    assert_eq!(token_outline(&mut art, body, "", &f, size, gold, Facing::side(false)).unwrap().clone(), sheet.outline(gold));
    let _ = std::fs::remove_dir_all(&root);
}

/// DIR4-G: a scratch body `units/<key>` with clips `ids` (one 8 x 8 frame each, 100 ms; `hurt` and `death` play
/// once), each `<of>_<view>` id whose side clip is listed a view row of it; every row a lopsided bar (so a mirror
/// shows) in its own colour (so the rows differ).
fn scratch_body(root: &std::path::Path, key: &str, ids: &[&str]) {
    const F: u32 = 8;
    let dir = root.join("units").join(key);
    std::fs::create_dir_all(&dir).unwrap();
    let mut px = vec![0u8; (F * F * ids.len() as u32 * 4) as usize];
    for row in 0..ids.len() as u32 {
        for x in [1u32, 2, 3, 6] {
            let i = (((row * F + 4) * F + x) * 4) as usize;
            px[i..i + 4].copy_from_slice(&[40 + 20 * row as u8, 140, 60, 255]);
        }
    }
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), F, F * ids.len() as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
    let clips: Vec<_> = ids.iter().map(|id| {
        let mut c = json!({"id": id, "frames": 1, "ms": [100], "loop": !["hurt", "death"].contains(id)});
        for v in ["up", "down"] {
            if let Some(of) = id.strip_suffix(&format!("_{v}")).filter(|of| ids.contains(of)) {
                c["of"] = json!(of);
                c["view"] = json!(v);
            }
        }
        c
    }).collect();
    std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": clips}).to_string()).unwrap();
}

/// DIR4-G: a scratch tree with a body with front and back views (`test/views`) and one with side clips only
/// (`test/side`, as every body before DIR4-R1/R2/R3), and its `Art`.
fn view_bodies(tag: &str) -> (std::path::PathBuf, Art) {
    let root = std::env::temp_dir().join(format!("emberwake_dir4g_{tag}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    scratch_body(&root, "test/views", &["idle", "walk", "hurt", "idle_down", "idle_up", "walk_up"]);
    scratch_body(&root, "test/side", &["idle", "walk", "hurt"]);
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    for body in ["test/views", "test/side"] {
        assert_eq!(art.body_problems(body), Vec::<String>::new(), "the scratch body {body} loads");
    }
    (root, art)
}

/// DIR4-G check 1: a unit whose target is two tiles straight below faces down and draws `idle_down` (never
/// mirrored) when its body has it, its side `idle` (mirrored by its left/right) otherwise.
#[test]
fn a_target_straight_below_draws_idle_down_when_the_body_has_it_and_side_idle_otherwise() {
    let (root, mut art) = view_bodies("below");
    let size = 24;
    let mut p = ClipPlayer::default();
    p.set_base("idle", 0);
    for left in [false, true] {
        let f = turn(Facing::side(left), V2 { x: 5, y: 2 }, V2 { x: 5, y: 4 });
        assert_eq!(f, Facing { view: Look::Down, left }, "two tiles below: down, keeping left/right");
        let (fr, img) = token_frame(&mut art, "test/views", "", &p, 0, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
        assert_eq!((fr.clip.as_str(), fr.side), ("idle_down", false));
        assert_eq!(img, art.body_clip("test/views", "", "idle_down", 0, size).unwrap().clone(), "a down row is never mirrored");
        let (fr, img) = token_frame(&mut art, "test/side", "", &p, 0, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
        assert_eq!((fr.clip.as_str(), fr.side), ("idle", true), "no down view: the side idle");
        assert_eq!(img, art.body_clip_flipped("test/side", "", "idle", 0, size, left).unwrap().clone(), "mirrored by the kept left/right");
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// Unit `h` of `host` steps to `to` at `now` (a step the tokens see).
fn slide_to(host: &mut Host, t: &mut Tokens, h: usize, to: V2, now: u64) {
    let s = host.session.as_mut().unwrap();
    s.w.grid.positions[h] = Some(to);
    t.catch_up(s, now, false, 300, true);
}

/// DIR4-G check 2 (G2): a slide plays its walk in the step's facing: (5, 4) to (5, 2) `walk_up`, then `idle_up`
/// once the slide ends; (5, 4) to (7, 5) the side walk as drawn; (5, 4) to (3, 3) the side walk mirrored. A body
/// without the up view walks side-on.
#[test]
fn a_slide_plays_walk_up_or_the_side_walk_mirrored_by_its_direction() {
    let (root, mut art) = view_bodies("slide");
    let size = 24;
    let mut host = skirmish(3);
    let mut t = Tokens::default();
    t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
    let h = host.session.as_ref().unwrap().w.allies[0];
    let mut now = 0;
    let mut go = |host: &mut Host, t: &mut Tokens, x: i32, y: i32| {
        now += 1000;
        slide_to(host, t, h, V2 { x, y }, now);
        now
    };
    let start = V2 { x: 5, y: 4 };
    go(&mut host, &mut t, start.x, start.y);
    let at = go(&mut host, &mut t, 5, 2);
    let a = t.anims[h].clone();
    assert_eq!(a.slide, Some((start, at)));
    assert_eq!(a.faces(at + 10, 300).view, Look::Up);
    let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, at + 10, size, a.faces(at + 10, 300)).map(|(fr, i)| (fr, i.clone())).unwrap();
    assert_eq!((fr.clip.as_str(), fr.side), ("walk_up", false), "(5, 4) to (5, 2) walks up");
    assert_eq!(img, art.body_clip("test/views", "", "walk_up", 0, size).unwrap().clone());
    let fr = token_frame(&mut art, "test/side", "", &a.player, at + 10, size, a.faces(at + 10, 300)).unwrap().0;
    assert_eq!((fr.clip.as_str(), fr.side), ("walk", true), "no up view: the side walk");
    let fr = token_frame(&mut art, "test/views", "", &a.player, at + 400, size, a.faces(at + 400, 300)).unwrap().0;
    assert_eq!(fr.clip, "idle_up", "the slide over: it stands facing up");
    for ((x, y), mirrored) in [((7, 5), false), ((3, 3), true)] {
        go(&mut host, &mut t, start.x, start.y);
        let at = go(&mut host, &mut t, x, y);
        let a = t.anims[h].clone();
        let f = a.faces(at + 10, 300);
        assert_eq!(f, Facing::side(mirrored), "(5, 4) to ({x}, {y})");
        let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, at + 10, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
        assert_eq!((fr.clip.as_str(), fr.side), ("walk", true), "(5, 4) to ({x}, {y}) walks side-on");
        assert_eq!(img, art.body_clip_flipped("test/views", "", "walk", 0, size, mirrored).unwrap().clone(), "(5, 4) to ({x}, {y}) mirrored: {mirrored}");
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// DIR4-G check 4 (G4): a unit facing up that is hit plays the side `hurt` with its last left/right (mirrored when
/// left), its outline with it, then shows `idle_up` again.
#[test]
fn a_hit_while_facing_up_plays_the_side_hurt_then_idle_up_again() {
    let (root, mut art) = view_bodies("hurt");
    let size = 24;
    let mut host = skirmish(3);
    let mut t = Tokens::default();
    t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
    let h = host.session.as_ref().unwrap().w.allies[0];
    for (k, (x, y)) in [(5, 4), (4, 4), (4, 2)].into_iter().enumerate() {
        slide_to(&mut host, &mut t, h, V2 { x, y }, 1000 * (k as u64 + 1));
    }
    assert_eq!(t.anims[h].rest(), Facing { view: Look::Up, left: true }, "a step left, then up");
    let hit = 10_000;
    {
        let s = host.session.as_mut().unwrap();
        let u = &mut s.w.units[h];
        (u.guard, u.stance) = (0.0, Stance::Neutral);
        t.catch_up(s, hit - 1, false, 300, true);
        s.w.units[h].hp -= 1;
        t.catch_up(s, hit, false, 300, true);
    }
    let a = t.anims[h].clone();
    let f = a.faces(hit + 10, 300);
    let (fr, img) = token_frame(&mut art, "test/views", "", &a.player, hit + 10, size, f).map(|(fr, i)| (fr, i.clone())).unwrap();
    assert_eq!((fr.clip.as_str(), fr.side), ("hurt", true), "no hurt_up: the side hurt");
    assert_eq!(img, art.body_clip_flipped("test/views", "", "hurt", 0, size, true).unwrap().clone(), "with the last left");
    let gold = [244, 198, 84, 255];
    assert_eq!(token_outline(&mut art, "test/views", "", &fr, size, gold, f).unwrap().clone(), img.outline(gold), "the outline follows the side frame");
    let fr = token_frame(&mut art, "test/views", "", &a.player, hit + 500, size, a.faces(hit + 500, 300)).unwrap().0;
    assert_eq!((fr.clip.as_str(), fr.side), ("idle_up", false), "after the hit: idle_up again");
    let _ = std::fs::remove_dir_all(&root);
}

/// DIR4-G check 5: a body without view clips (the scratch side body, and every shipped body of the skirmish that
/// has none) draws in any facing exactly as FACE1 did: `Art::played` mirrored when left, its outline the same.
#[test]
fn a_body_without_view_clips_draws_exactly_as_face1() {
    let (root, mut scratch) = view_bodies("side");
    let host = skirmish(3);
    let s = host.session.as_ref().unwrap();
    let mut shipped = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let bodies: Vec<String> = s.art.iter().filter_map(|chain| shipped.first_body(&emberwake_content::art::body_chain(chain))).collect();
    let gold = [244, 198, 84, 255];
    let facings: Vec<Facing> = [Look::Side, Look::Down, Look::Up].into_iter().flat_map(|view| [false, true].map(|left| Facing { view, left })).collect();
    let check = |art: &mut Art, body: &str| {
        for clip in ["idle", "walk", "hurt"] {
            let mut p = ClipPlayer::default();
            p.set_base("idle", 0);
            p.play(clip, 0);
            for &f in &facings {
                let (fr, img) = token_frame(art, body, "", &p, 10, 40, f).map(|(fr, i)| (fr, i.clone())).unwrap();
                let (was, old) = art.played(body, "", &p, 10, 40, f.left).map(|(fr, i)| (fr, i.clone())).unwrap();
                assert_eq!((fr.clip.as_str(), fr.frame, &img), (was.clip.as_str(), was.frame, &old), "{body} {clip} in {f:?}");
                let ring = token_outline(art, body, "", &fr, 40, gold, f).unwrap().clone();
                assert_eq!(&ring, art.body_outline(body, "", &was.clip, was.frame, 40, gold, f.left).unwrap(), "{body} {clip} outline in {f:?}");
            }
        }
    };
    check(&mut scratch, "test/side");
    let mut checked = 0;
    for body in &bodies {
        if shipped.clips(body).is_some_and(|c| c.clips.iter().all(|c| c.view == Look::Side)) {
            check(&mut shipped, body);
            checked += 1;
        }
    }
    println!("shipped bodies without views checked: {checked} of {}", bodies.len());
    let _ = std::fs::remove_dir_all(&root);
}

/// Every rig clip (SPRITE1 section 2), one frame each, looping as the export writes them.
fn rig_clips() -> Clips {
    let ids = ["idle", "walk", "attack", "jab", "sweep_kick", "kick", "shield_bash", "haymaker", "hurt", "death", "defend", "idle_guarded", "hurt_guarded"];
    let clips: Vec<_> = ids.iter().map(|id| {
        let looped = ["idle", "walk", "idle_guarded"].contains(id);
        json!({"id": id, "frames": 1, "ms": [100], "loop": looped})
    }).collect();
    Clips::parse(&json!({"frame": [48, 48], "clips": clips})).unwrap()
}

/// SPRITE1-U: played step by step, every token follows its unit: a move slides and walks, a hit plays hurt (guarded
/// while it guards), a fall plays death, the resolved attack queues its actor's clip; skip plays nothing.
#[test]
fn tokens_follow_the_battle_step_by_step() {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
    assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3}))["ok"], json!(true));
    let clips = rig_clips();
    let (mut t, mut skip) = (Tokens::default(), Tokens::default());
    let mut now = 0u64;
    t.catch_up(host.session.as_ref().unwrap(), now, false, 300, true);
    skip.catch_up(host.session.as_ref().unwrap(), now, true, 300, true);
    assert!(t.anims.iter().all(|a| a.player.playing(&clips, now).is_empty() && a.slide.is_none()), "a new battle plays nothing");
    let (mut moves, mut hits, mut falls, mut attacks) = (0, 0, 0, 0);
    for _ in 0..600 {
        let s = host.session.as_ref().unwrap();
        if s.combat_over {
            break;
        }
        if s.phase == Phase::Replay {
            if !s.can_advance_round() {
                break;
            }
            host.handle(&json!({"cmd": "advance"}));
        }
        if host.session.as_ref().unwrap().is_planning() {
            host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
        }
        let before: Vec<_> = t.anims.iter().map(|a| a.seen.clone()).collect();
        let popped = host.session.as_ref().unwrap().last_popped;
        assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
        now += 1000;
        let s = host.session.as_ref().unwrap();
        t.catch_up(s, now, false, 300, true);
        skip.catch_up(s, now, true, 300, true);
        for (i, u) in s.w.units.iter().enumerate() {
            let (a, was) = (&t.anims[i], &before[i]);
            let guarded = u.guard > 0.0 || u.stance == Stance::Defensive;
            let playing = a.player.playing(&clips, now);
            if was.alive && !u.is_alive() {
                falls += 1;
                assert_eq!((a.fell, playing), (Some(now), "death"), "{} fell", u.name);
            } else if u.is_alive() && u.hp < was.hp {
                hits += 1;
                assert_eq!(playing, if guarded { "hurt_guarded" } else { "hurt" }, "{} was hit", u.name);
            } else if u.is_alive() && guarded && !was.guarded {
                assert_eq!(playing, "defend", "{} raised a guard", u.name);
            } else if was.pos.is_some() && s.w.grid.pos(i).is_some() && was.pos != s.w.grid.pos(i) {
                moves += 1;
                assert_eq!((a.slide, playing), (Some((was.pos.unwrap(), now)), "walk"), "{} moved", u.name);
            }
            assert_eq!(a.player.base(), if guarded { "idle_guarded" } else { "idle" });
            let k = &skip.anims[i];
            assert!(k.player.playing(&clips, now).is_empty() && k.slide.is_none() && k.want.is_none(), "skip: the end state at once");
            assert_eq!(k.fell.is_some(), !u.is_alive());
        }
        if let Some((_, _, idx)) = s.last_popped.filter(|p| Some(*p) != popped) {
            let act = &s.resolver.scheduled[idx];
            let attack = matches!(act.command.kind, emberwake_battle::command::CmdKind::Attack { .. } | emberwake_battle::command::CmdKind::Area { .. });
            if attack && !act.cancelled && s.w.units[act.unit].is_alive() {
                attacks += 1;
                let want = t.anims[act.unit].want.as_ref().expect("the attacker queues its clip");
                assert_eq!(want.1, now);
                assert!(want.0.last() == Some(&"attack"), "{:?} ends with the generic attack", want.0);
            }
        }
    }
    assert!(moves > 0 && hits > 0 && falls > 0 && attacks > 0, "moves {moves}, hits {hits}, falls {falls}, attacks {attacks}");
}

/// A goblin_skirmish battle through the command host (the data's own encounter; no number of it is assumed).
fn skirmish(seed: i64) -> Host {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let mut host = Host::new(Battle::with_data(Arc::new(Data::open(&dir).expect("data").0)));
    assert_eq!(host.handle(&json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": seed}))["ok"], json!(true));
    host
}

/// DIR4-G (spec 4, check 3): a unit looks up or down at a target more vertical than horizontal (keeping its
/// left/right), sideways at anything else (a diagonal tie keeps the side view); its own tile keeps the facing.
#[test]
fn facing_turns_four_ways_toward_the_target_and_a_diagonal_keeps_the_side_view() {
    let at = V2 { x: 5, y: 4 };
    let (right, left) = (Facing::side(false), Facing::side(true));
    assert_eq!(turn(left, at, V2 { x: 5, y: 6 }), Facing { view: Look::Down, left: true }, "two tiles straight below");
    assert_eq!(turn(right, at, V2 { x: 5, y: 2 }), Facing { view: Look::Up, left: false }, "two tiles straight above");
    assert_eq!(turn(left, at, V2 { x: 6, y: 5 }), right, "the diagonal (+1, +1): side, right");
    assert_eq!(turn(right, at, V2 { x: 4, y: 3 }), left, "the diagonal (-1, -1): side, left");
    assert_eq!(turn(right, at, V2 { x: 3, y: 4 }), left, "a target on the left");
    assert_eq!(turn(left, at, V2 { x: 7, y: 5 }), right, "a target more right than down");
    let up = Facing { view: Look::Up, left: true };
    assert_eq!(turn(up, at, at), up, "its own tile: unchanged");
}

/// FACE1 and DIR4-G G2: a new battle faces the party right and the foes left (side view); a step turns a unit
/// toward it: up or down on a vertical step (keeping its left/right), sideways by its x otherwise.
#[test]
fn a_new_battle_faces_the_party_right_and_a_vertical_step_faces_up_or_down() {
    let mut host = skirmish(3);
    let mut t = Tokens::default();
    t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
    let s = host.session.as_ref().unwrap();
    for i in 0..s.w.units.len() {
        assert_eq!(t.anims[i].facing, Facing::side(!s.w.allies.contains(&i)), "{} at the start", s.w.units[i].name);
    }
    let h = s.w.allies[0];
    let up = if s.w.grid.pos(h).expect("the hero stands on the grid").y > 0 { -1 } else { 1 };
    let (go, back) = if up < 0 { (Look::Up, Look::Down) } else { (Look::Down, Look::Up) };
    let mut walk = |t: &mut Tokens, dx: i32, dy: i32| {
        let s = host.session.as_mut().unwrap();
        let at = s.w.grid.pos(h).unwrap();
        s.w.grid.positions[h] = Some(V2 { x: at.x + dx, y: at.y + dy });
        t.catch_up(s, 0, false, 300, true);
        assert_eq!(t.anims[h].step, t.anims[h].facing, "the slide shows the step's facing");
        t.anims[h].facing
    };
    assert_eq!(walk(&mut t, 0, up), Facing { view: go, left: false }, "a vertical step faces that way and keeps right");
    assert_eq!(walk(&mut t, -1, 0), Facing::side(true), "a step left turns left");
    assert_eq!(walk(&mut t, 0, -up), Facing { view: back, left: true }, "a vertical step keeps left");
    assert_eq!(walk(&mut t, -1, up), Facing::side(true), "a diagonal step goes by its x");
    assert_eq!(walk(&mut t, 1, 0), Facing::side(false), "a step right turns right");
}

/// FACE1 and DIR4-G G3: a unit that resolves an attack turns toward its target: left when the target stands left
/// of it, right when it stands right (a foe too), up at a target straight above keeping its left/right; skip turns
/// it the same.
#[test]
fn an_attacker_faces_its_target_on_either_side() {
    let mut host = skirmish(3);
    assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
    assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
    let (mut t, mut skip) = (Tokens::default(), Tokens::default());
    let attacks: Vec<(usize, usize, usize)> = {
        let s = host.session.as_ref().unwrap();
        s.resolver.scheduled.iter().enumerate().filter_map(|(idx, a)| match a.command.kind {
            CmdKind::Attack { target: Some(tg), .. } if !a.cancelled && tg != a.unit && s.w.grid.pos(tg).is_some() => Some((idx, a.unit, tg)),
            _ => None,
        }).collect()
    };
    let mut sides = (false, false);
    let mut block = 0;
    for (idx, actor, target) in attacks {
        let tp = host.session.as_ref().unwrap().w.grid.pos(target).unwrap();
        for (dx, dy, keep) in [(1, 0, None), (-1, 0, None), (0, 1, Some(true)), (0, 1, Some(false))] {
            let s = host.session.as_mut().unwrap();
            s.w.grid.positions[actor] = Some(V2 { x: tp.x + dx, y: tp.y + dy });
            t.catch_up(s, 0, false, 300, true);
            skip.catch_up(s, 0, true, 300, true);
            if let Some(k) = keep {
                (t.anims[actor].facing, skip.anims[actor].facing) = (Facing::side(k), Facing::side(k));
            }
            block += 1;
            s.last_popped = Some((s.current_round, block, idx));
            t.catch_up(s, 0, false, 300, true);
            skip.catch_up(s, 0, true, 300, true);
            let want = match keep {
                Some(k) => Facing { view: Look::Up, left: k },
                None => Facing::side(dx > 0),
            };
            assert_eq!(t.anims[actor].rest(), want, "{} attacks from {dx},{dy} of its target", s.w.units[actor].name);
            assert_eq!(skip.anims[actor].rest(), want, "skip: the same facing");
        }
        if host.session.as_ref().unwrap().w.allies.contains(&actor) {
            sides.0 = true;
        } else {
            sides.1 = true;
        }
    }
    assert!(sides.0 && sides.1, "a hero's and a foe's attack were turned both ways: {sides:?}");
}

/// FACE1 (user 2026-10-23): with the enemy tells shown an enemy faces its intent's target while the party plans; with
/// them hidden it keeps its facing through the planning and the confirm (until it acts). Heroes show their plan
/// either way. An enemy is set left of its intent's target so its intent would turn it away from its start facing.
#[test]
fn hidden_tells_keep_the_enemies_facing_while_planning() {
    let mut host = skirmish(3);
    assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
    let (e, target) = {
        let s = host.session.as_ref().unwrap();
        s.enemy_intents().into_iter().find_map(|r| r.line.target.filter(|t| s.w.grid.pos(*t).is_some()).map(|t| (r.unit, t))).expect("an enemy intends a unit")
    };
    let s = host.session.as_mut().unwrap();
    let (tp, home) = (s.w.grid.pos(target).unwrap(), s.w.grid.positions[e]);
    s.w.grid.positions[e] = Some(V2 { x: tp.x - 1, y: tp.y });
    let (mut shown, mut hidden) = (Tokens::default(), Tokens::default());
    shown.catch_up(s, 0, false, 300, true);
    hidden.catch_up(s, 0, false, 300, false);
    assert_eq!(shown.anims[e].rest(), Facing::side(false), "tells shown: {} faces its intent's target on its right", s.w.units[e].name);
    assert_eq!(hidden.anims[e].rest(), Facing::side(true), "tells hidden: {} keeps its start facing (left)", s.w.units[e].name);
    for &h in &s.w.allies {
        assert_eq!(shown.anims[h].rest(), hidden.anims[h].rest(), "{} shows its plan either way", s.w.units[h].name);
    }
    let hero_planned = s.w.allies.iter().any(|&h| shown.anims[h].planned.is_some());
    assert!(hero_planned, "the aggressive preset gives a hero a target");
    // Back to its own tile (the test tile may be taken) so the plan confirms; the facing shown is kept. The tokens
    // take that tile as seen, so the teleport is no step.
    s.w.grid.positions[e] = home;
    (shown.anims[e].seen.pos, hidden.anims[e].seen.pos) = (home, home);
    assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
    let s = host.session.as_ref().unwrap();
    shown.catch_up(s, 0, false, 300, true);
    hidden.catch_up(s, 0, false, 300, false);
    assert_eq!(shown.anims[e].rest(), Facing::side(false), "tells shown: the confirm keeps the facing it showed");
    assert_eq!(hidden.anims[e].rest(), Facing::side(true), "tells hidden: still its own facing after the confirm");
}

/// FACE1 and DIR4-G, played step by step (rule-derived over several seeds): after every step each unit faces its
/// step turned toward the target of the action it just resolved (`turn`: four ways; no step or no target keeps
/// the facing); while the party plans an enemy faces its tell's target. The skip setting ends every step in the
/// same facing (G5).
#[test]
fn units_face_their_targets_and_steps_and_skip_agrees() {
    let (mut turned_left, mut turned_right, mut turned_vertical) = (0, 0, 0);
    for seed in 1..=4 {
        let mut host = skirmish(seed);
        let (mut t, mut skip) = (Tokens::default(), Tokens::default());
        t.catch_up(host.session.as_ref().unwrap(), 0, false, 300, true);
        skip.catch_up(host.session.as_ref().unwrap(), 0, true, 300, true);
        let mut now = 0u64;
        for _ in 0..600 {
            let s = host.session.as_ref().unwrap();
            if s.combat_over {
                break;
            }
            if s.phase == Phase::Replay {
                if !s.can_advance_round() {
                    break;
                }
                host.handle(&json!({"cmd": "advance"}));
            }
            if host.session.as_ref().unwrap().is_planning() {
                host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
                let s = host.session.as_ref().unwrap();
                t.catch_up(s, now, false, 300, true);
                skip.catch_up(s, now, true, 300, true);
                for r in s.enemy_intents() {
                    let to = r.line.target.and_then(|u| s.w.grid.pos(u)).unwrap_or(r.line.cell);
                    if let (Some(from), true) = (s.w.grid.pos(r.unit), to != NO_CELL) {
                        let want = turn(t.anims[r.unit].facing, from, to);
                        assert_eq!(t.anims[r.unit].rest(), want, "{} shows its tell's target", s.w.units[r.unit].name);
                    }
                }
            }
            let before: Vec<(Facing, Option<V2>)> = t.anims.iter().map(|a| (a.rest(), a.seen.pos)).collect();
            let popped = host.session.as_ref().unwrap().last_popped;
            assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
            now += 1000;
            let s = host.session.as_ref().unwrap();
            t.catch_up(s, now, false, 300, true);
            skip.catch_up(s, now, true, 300, true);
            let acted = s.last_popped.filter(|p| Some(*p) != popped && p.0 == s.current_round).map(|(_, _, idx)| &s.resolver.scheduled[idx]).filter(|a| !a.cancelled && s.w.units[a.unit].is_alive());
            for i in 0..s.w.units.len() {
                let aim = acted.filter(|a| a.unit == i).and_then(|a| match &a.command.kind {
                    CmdKind::Attack { target, .. } => target.and_then(|u| s.w.grid.pos(u).or(before[u].1)),
                    CmdKind::Area { aim, .. } => Some(*aim),
                    CmdKind::Support { target, .. } | CmdKind::Item { target, .. } => s.w.grid.pos(*target).or(before[*target].1),
                    _ => None,
                });
                let stepped = before[i].1.zip(s.w.grid.pos(i)).map_or(before[i].0, |(from, to)| turn(before[i].0, from, to));
                let aimed = s.w.grid.pos(i).zip(aim.filter(|c| *c != NO_CELL));
                let want = aimed.map_or(stepped, |(from, to)| turn(stepped, from, to));
                assert_eq!(t.anims[i].rest(), want, "seed {seed}: {} after the step", s.w.units[i].name);
                assert_eq!(skip.anims[i].rest(), want, "seed {seed}: skip ends {} in the same facing", s.w.units[i].name);
                if let Some((from, to)) = aimed.filter(|(from, to)| from != to) {
                    match turn(stepped, from, to) {
                        Facing { view: Look::Side, left: true } => turned_left += 1,
                        Facing { view: Look::Side, left: false } => turned_right += 1,
                        _ => turned_vertical += 1,
                    }
                }
            }
        }
    }
    assert!(turned_left > 0 && turned_right > 0 && turned_vertical > 0, "actions turned left {turned_left}, right {turned_right}, up or down {turned_vertical}");
}

/// The Test Arena on `goblin_ambush` (the forest clearing) with the game's art, in battle view `view`.
pub(super) fn ambush(view: serde_json::Value) -> (crate::screens::Cx, aegis_ui::ui::Gfx, BattleScreen) {
    let mut cx = crate::hud::hud_tests::game_cx();
    cx.art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    cx.app.encounter = "goblin_ambush".into();
    cx.app.seed = Some(1);
    if !view.is_null() {
        cx.settings.v["battle_view"] = view;
    }
    let b = BattleScreen::sandbox(&cx);
    assert!(b.host.session.is_some(), "the ambush starts: {}", b.message);
    (cx, crate::hud::hud_tests::gfx(), b)
}

/// DERIVE1 4.4: the selected hero's panel and every unit's hover card list the catalog derived stats shown in
/// battle (by the battle's ruleset), the panel's line hovering to the formula and breakdown; with no such row the
/// panel and the cards are as before.
#[test]
fn the_hero_panel_lists_battle_derived_stats_with_their_breakdown() {
    for shown in [false, true] {
        let mut cx = crate::hud::hud_tests::game_cx();
        let mut data = (*cx.app.data).clone();
        data.edit_cat(|cat| {
            cat.derived_stats.clear();
            if shown {
                let r = emberwake_data::DerivedStatDef::from_obj(json!({"id": "nerve", "name": "Nerve", "formula": "KNOW * 2", "show_on": ["battle"]}).as_object().unwrap()).unwrap();
                cat.derived_stats.insert(r.id.clone(), r);
            }
        });
        cx.app.data = Arc::new(data);
        cx.app.encounter = "goblin_ambush".into();
        cx.app.seed = Some(1);
        let mut b = BattleScreen::sandbox(&cx);
        let mut g = crate::hud::hud_tests::gfx();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let s = b.host.session.as_ref().unwrap();
        let h = b.selected.expect("a hero is selected");
        let card = b.w.iter().find(|w| w.id == format!("party.{h}")).unwrap();
        let line = b.w.iter().find(|w| w.id == "hero.derived");
        if !shown {
            assert!(line.is_none() && !card.tip.iter().any(|t| t.0.starts_with("Nerve")));
            continue;
        }
        let v = emberwake_rules::derived::value(&s.w.cat, &s.w.ruleset.id, &s.w.units[h], "nerve").unwrap();
        let line = line.expect("the derived line");
        assert_eq!(line.text, format!("Nerve {v}"));
        assert_eq!(line.tip[0].0, format!("Nerve {v}"));
        assert!(line.tip[1].0.starts_with("Formula: KNOW * 2. KNOW "), "{:?}", line.tip);
        assert!(card.tip.iter().any(|t| t.0 == format!("Nerve {v}")), "the hover card: {:?}", card.tip);
    }
}

fn rect_of(b: &BattleScreen, id: &str) -> aegis_ui::canvas::Rect {
    b.w.iter().find(|w| w.id == id).unwrap_or_else(|| panic!("no widget {id}")).rect
}

/// STAGE1 check 10: the Battle view setting picks the board: Flat keeps today's 32 x 32 cells at (128, 30) and a
/// unit's widget is its cell; Stage (also an unknown value and the default) the 32 x 24 cells from (128, 82).
#[test]
fn the_battle_view_setting_picks_the_stage_or_the_flat_board() {
    use aegis_ui::canvas::Rect;
    for (view, flat) in [(json!("flat"), true), (json!("stage"), false), (json!("sideways"), false), (serde_json::Value::Null, false)] {
        let (mut cx, mut g, mut b) = ambush(view.clone());
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let (cell, last) = (rect_of(&b, "tile.0.0"), rect_of(&b, "tile.11.8"));
        if flat {
            assert_eq!((cell, last), (Rect::new(128, 30, 32, 32), Rect::new(480, 286, 32, 32)), "{view}");
            let s = b.host.session.as_ref().unwrap();
            let h = s.w.allies[0];
            let p = s.w.grid.pos(h).unwrap();
            assert_eq!(rect_of(&b, &format!("unit.{h}")), Rect::new(128 + p.x * 32, 30 + p.y * 32, 32, 32), "a unit's widget is its cell");
        } else {
            assert_eq!((cell, last), (Rect::new(128, 82, 32, 24), Rect::new(480, 274, 32, 24)), "{view}");
        }
        assert_eq!(b.w.iter().filter(|w| w.id.starts_with("tile.")).count(), 12 * 9, "every cell keeps its widget");
    }
}

/// STAGE1 check 6: on the Stage the hero's topmost solid pixel (his helmet, above his cell) clicks him; the top-left
/// 2 x 2 px of a free cell north of a unit click that cell; where two units' rects overlap the southern one wins;
/// the backdrop and the front face hold no cell.
#[test]
fn stage_clicks_reach_heads_free_cells_and_the_unit_in_front() {
    let (mut cx, mut g, mut b) = ambush(json!("stage"));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let (h, pos, body, palette) = {
        let s = b.host.session.as_ref().unwrap();
        let h = s.w.allies[0];
        (h, s.w.grid.pos(h).unwrap(), cx.art.first_body(&crate::battle::tokens::body_keys(&s.art[h], &s.w.units[h])).expect("the hero has a body"), s.palettes.get(h).cloned().unwrap_or_default())
    };
    assert_eq!(h, 0, "the first hero is unit 0");
    // before the first change the tokens have no animations yet and a hero faces right (as the draw does)
    let left = b.tokens.anims.get(h).is_some_and(|a| a.rest().left);
    let bb = cx.art.body_bounds(&body, &palette, geo.token_px, left).expect("solid pixels");
    let frame = cx.art.body_clip_flipped(&body, &palette, "idle", 0, geo.token_px, left).unwrap().clone();
    let top_x = (bb.x..bb.x + bb.w).find(|x| frame.px[((bb.y * frame.w + x) * 4 + 3) as usize] >= aegis_ui::canvas::OUTLINE_ALPHA).unwrap();
    let tr = geo.token_rect(pos, 0, 0);
    let helmet = (tr.x + top_x, tr.y + bb.y);
    assert!(helmet.1 < geo.cell_rect(pos).y, "the helmet is above the hero's cell");
    assert_eq!(aegis_ui::ui::hit(&b.w, helmet), Some(Action::UnitClick(h)), "the helmet pixel clicks the hero");
    // A free cell north of a unit.
    let free = {
        let s = b.host.session.as_ref().unwrap();
        s.w.grid.positions.iter().flatten().map(|p| V2 { x: p.x, y: p.y - 1 }).find(|c| s.w.grid.is_valid(*c) && s.w.occupant(*c).is_none()).expect("a free cell north of a unit")
    };
    let r = geo.cell_rect(free);
    for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
        assert_eq!(aegis_ui::ui::hit(&b.w, (r.x + dx, r.y + dy)), Some(Action::TileClick(free)), "the corner of free cell {free}");
    }
    for pt in [(300, 50), (300, 305)] {
        assert!(!matches!(aegis_ui::ui::hit(&b.w, pt), Some(Action::TileClick(_))), "{pt:?} holds no cell");
    }
    // Two units in one column: the southern one wins where their rects overlap.
    let (north, south) = {
        let s = b.host.session.as_mut().unwrap();
        let other = (0..s.w.units.len()).find(|i| *i != h && s.w.units[*i].is_alive() && s.w.grid.pos(*i).is_some()).expect("another unit");
        let below = if pos.y + 1 < GRID_H { V2 { x: pos.x, y: pos.y + 1 } } else { V2 { x: pos.x, y: pos.y - 1 } };
        s.w.grid.positions[other] = Some(below);
        if below.y > pos.y { (h, other) } else { (other, h) }
    };
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let (rn, rs) = (rect_of(&b, &format!("unit.{north}")), rect_of(&b, &format!("unit.{south}")));
    let m = rn.intersect(rs);
    assert!(m.w > 0 && m.h > 0, "the southern unit's head reaches into the northern one's rect");
    assert_eq!(aegis_ui::ui::hit(&b.w, (m.x + m.w / 2, m.y + m.h / 2)), Some(Action::UnitClick(south)), "the unit in front wins");
    assert_eq!(aegis_ui::ui::hit(&b.w, (rn.x + rn.w / 2, rn.y + 1)), Some(Action::UnitClick(north)), "the unit behind still clicks on its head");
}

/// DIR4-HB: on the Stage a unit facing down or up is hit by its front or back `idle` frame (`body_bounds_facing`;
/// the side idle for a body without that view row): its widget rect is its cell joined with that frame's solid
/// bounds, and the frame's topmost solid pixel clicks it.
#[test]
fn stage_clicks_follow_a_unit_facing_up_or_down() {
    use aegis_ui::canvas::{Rect, OUTLINE_ALPHA};
    let (mut cx, mut g, mut b) = ambush(json!("stage"));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let (h, pos, body, palette) = {
        let s = b.host.session.as_ref().unwrap();
        let h = s.w.allies[0];
        (h, s.w.grid.pos(h).unwrap(), cx.art.first_body(&crate::battle::tokens::body_keys(&s.art[h], &s.w.units[h])).expect("the hero has a body"), s.palettes.get(h).cloned().unwrap_or_default())
    };
    let viewed = [Look::Down, Look::Up].map(|view| cx.art.clips(&body).and_then(|c| c.view_row("idle", view).map(|r| c.clips[r].id.clone())));
    assert!(viewed.iter().any(|v| v.is_some()), "the hero's body has a front or back idle (DIR4-R2)");
    // before the first change the tokens have no animations yet: catch them up to the session
    let now = b.clock.now();
    b.tokens.catch_up(b.host.session.as_ref().unwrap(), now, true, 300, false);
    for (view, row) in [Look::Down, Look::Up].into_iter().zip(viewed) {
        for left in [false, true] {
            let f = Facing { view, left };
            {
                let a = &mut b.tokens.anims[h];
                a.facing = f;
                a.planned = None;
                a.slide = None;
            }
            b.draw(&mut cx, &mut g, (-500, -500), true);
            assert_eq!(b.tokens.anims[h].rest(), f, "the hero keeps the facing set");
            let (clip, flip) = match &row {
                Some(id) => (id.clone(), false),
                None => ("idle".to_string(), left),
            };
            let bb = cx.art.body_bounds_facing(&body, &palette, geo.token_px, f).expect("solid pixels");
            let frame = cx.art.body_clip_flipped(&body, &palette, &clip, 0, geo.token_px, flip).unwrap().clone();
            assert_eq!(frame.opaque_bounds(OUTLINE_ALPHA), Some(bb), "{f:?}: the bounds of {clip}");
            let tr = geo.token_rect(pos, 0, 0);
            let want = crate::stage::unit_rect(geo.cell_rect(pos), Rect::new(tr.x + bb.x, tr.y + bb.y, bb.w, bb.h));
            assert_eq!(rect_of(&b, &format!("unit.{h}")), want, "{f:?}: the widget follows the view's silhouette");
            let top_x = (bb.x..bb.x + bb.w).find(|x| frame.px[((bb.y * frame.w + x) * 4 + 3) as usize] >= OUTLINE_ALPHA).unwrap();
            assert_eq!(aegis_ui::ui::hit(&b.w, (tr.x + top_x, tr.y + bb.y)), Some(Action::UnitClick(h)), "{f:?}: the top of {clip} clicks the hero");
        }
    }
}

#[test]
fn a_boss_tier_enemy_makes_a_boss_fight() {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    let d = Data::open(&dir).expect("data").0;
    let fight = |id: &str| {
        let def = EncounterDef::from_obj(d.resolve("ashlands", "encounters", id));
        boss_fight(&Session::start(d.cat.clone(), def, &WorldRoster { d: &d, world: "ashlands".into() }))
    };
    assert!(fight("ashmaw_stand"));
    assert!(!fight("courtyard_skirmish"));
}

#[test]
fn log_lines_pick_their_sounds() {
    assert_eq!(sound_of("[Attack] Rook struck Goblin for 7 (HP 3/10)"), Some("hit"));
    assert_eq!(sound_of("[Attack] Rook struck Goblin for 14 CRIT (HP 0/10)"), Some("crit"));
    assert_eq!(sound_of("[Attack] Rook struck Goblin for 0 (HP 10/10)"), Some("mitigated"));
    assert_eq!(sound_of("[Attack Miss] Rook: target no longer in combat"), Some("miss"));
    assert_eq!(sound_of("[Area Miss] Mage hit nobody"), Some("miss"));
    assert_eq!(sound_of("[Status] Hex -> Goblin (inflicts poison)"), Some("debuff"));
    assert_eq!(sound_of("[Status] Hex -> Goblin (resisted poison)"), None);
    assert_eq!(sound_of("[Status] Bard -> Rook (inflicts haste)"), None);
    assert_eq!(sound_of("[Defend] Rook raises guard (40% less damage)"), Some("defend"));
    assert_eq!(sound_of("[Defend] Rook's guard was broken"), None);
    assert_eq!(sound_of("[Support] Cleric used Mend on Rook: healed 6"), Some("heal"));
    assert_eq!(sound_of("[Move] Rook moved 2 of 2 tiles to (3, 4)"), Some("move_step"));
    assert_eq!(sound_of("[Rest] Rook AP 3/4"), None);
}

/// FX1-L1: plays the ambush step by step (the party on the aggressive preset) until a single step leaves a pending
/// impact `want` accepts; earlier impacts fire before the next step (rule F). False when the battle ends first.
pub(super) fn step_until(cx: &mut crate::screens::Cx, b: &mut BattleScreen, want: impl Fn(&BattleScreen, &crate::fx::Impact) -> bool) -> bool {
    for _ in 0..600 {
        let (over, phase, planning, can) = {
            let s = b.host.session.as_ref().unwrap();
            (s.combat_over, s.phase, s.is_planning(), s.can_advance_round())
        };
        if over || (phase == Phase::Replay && !can) {
            return false;
        }
        if phase == Phase::Replay {
            b.send(cx, json!({"cmd": "advance"}));
            continue;
        }
        if planning {
            b.send(cx, json!({"cmd": "preset", "preset": "aggressive"}));
        }
        b.flush(cx);
        b.floats.clear();
        b.flashes.clear();
        b.send(cx, json!({"cmd": "step"}));
        if b.impacts.last().is_some_and(|i| want(b, i)) {
            return true;
        }
    }
    false
}

/// The clips of unit `i`'s body (the game's art).
fn unit_clips(cx: &mut crate::screens::Cx, b: &BattleScreen, i: usize) -> Clips {
    let s = b.host.session.as_ref().unwrap();
    let body = cx.art.first_body(&crate::battle::tokens::body_keys(&s.art[i], &s.w.units[i])).expect("a body");
    cx.art.clips(&body).unwrap().clone()
}

/// FX1-L1 L3 (rule I): between the pop and the impact the struck unit's board HP is the old one, no float, flash or
/// hurt clip shows and the step's sounds wait; at the impact all of them start, the board HP is the battle's and
/// the hit-stop holds `hit_stop_ms`. The impact of a blow is at its attack clip's `impact` frame (decision "Attack
/// timing on the board" A).
#[test]
fn a_hit_shows_on_the_board_at_its_impact_not_at_the_pop() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    let found = step_until(&mut cx, &mut b, |b, i| {
        let s = b.host.session.as_ref().unwrap();
        i.struck.iter().any(|&u| s.w.units[u].is_alive())
    });
    assert!(found, "the ambush has a hit");
    b.resolve_impacts(&mut cx);
    let i = b.impacts[0].clone();
    let at = i.at.expect("resolved");
    let target = *i.struck.iter().find(|&&u| b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
    let (hp_now, n) = { let s = b.host.session.as_ref().unwrap(); (s.w.units[target].hp, s.w.units.len()) };
    // before the impact
    let (shown_hp, alive) = b.tokens.shown(target, &b.host.session.as_ref().unwrap().w.units[target], n);
    assert!(alive && shown_hp > hp_now, "the board keeps the HP before the step ({shown_hp} vs {hp_now})");
    assert!(b.floats.is_empty() && b.flashes.is_empty(), "no float or flash before the impact");
    assert!(!i.sounds.is_empty() && !i.floats.is_empty(), "the step's sounds and floats wait in the impact");
    let clips = unit_clips(&mut cx, &b, target);
    assert!(!b.tokens.anims[target].player.playing(&clips, b.clock.now()).starts_with("hurt"), "no hurt clip yet");
    assert!(at >= i.t0, "the impact comes after t0");
    if i.delivery == crate::fx::Delivery::Melee {
        let actor_clips = unit_clips(&mut cx, &b, i.actor);
        let clip = i.clips.iter().copied().find(|c| actor_clips.clip(c).is_some());
        let want = clip.and_then(|c| actor_clips.clip(c).unwrap().impact_ms()).unwrap_or(cx.tuning.value.melee_hit_ms);
        assert_eq!(at, i.t0 + want, "a blow lands at its clip's impact frame");
    }
    // the impact
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    assert!(b.impacts.is_empty());
    let n = b.host.session.as_ref().unwrap().w.units.len();
    assert_eq!(b.tokens.shown(target, &b.host.session.as_ref().unwrap().w.units[target], n).0, hp_now, "the board HP is the battle's");
    assert!(b.floats.iter().any(|f| f.unit == target) && b.flashes.iter().any(|f| f.0 == target));
    assert!(b.tokens.anims[target].player.playing(&clips, at).starts_with("hurt"), "the hurt clip from the impact");
    assert_eq!(b.clock.hold, cx.tuning.value.hit_stop_ms as f64, "the hit-stop");
}

/// FX1-L1 L7 (4.2 step 1): an actor that moved in the step it acts (a dash, a charge, an Engage carry) starts its
/// attack clip at the end of its slide, the walk showing until then; one that stood starts it at the pop.
#[test]
fn a_dash_starts_its_attack_after_the_slide() {
    let mut host = skirmish(3);
    assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
    assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
    let mut art = Art::new(Some(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")));
    let s = host.session.as_mut().unwrap();
    let (idx, actor, target) = s.resolver.scheduled.iter().enumerate().find_map(|(idx, a)| match a.command.kind {
        CmdKind::Attack { target: Some(tg), .. } if !a.cancelled && tg != a.unit && s.w.grid.pos(tg).is_some() => Some((idx, a.unit, tg)),
        _ => None,
    }).expect("an attack");
    let body = art.first_body(&crate::battle::tokens::body_keys(&s.art[actor], &s.w.units[actor])).expect("a body");
    let clips = art.clips(&body).unwrap().clone();
    let tp = s.w.grid.pos(target).unwrap();
    let dir = if tp.x < emberwake_battle::grid::GRID_W / 2 { 1 } else { -1 };
    let walk = 300;
    let mut t = Tokens::default();
    s.w.grid.positions[actor] = Some(V2 { x: tp.x + 2 * dir, y: tp.y });
    t.catch_up(s, 0, false, walk, true);
    // the step: it closes in and its attack pops
    s.w.grid.positions[actor] = Some(V2 { x: tp.x + dir, y: tp.y });
    s.last_popped = Some((s.current_round, 1, idx));
    t.catch_up(s, 1000, false, walk, true);
    assert_eq!(t.anims[actor].want.as_ref().map(|w| w.1), Some(1000 + walk), "t0 is the slide's end");
    crate::battle::board::start_wanted(&mut t, &mut art, actor, Some(&body), 1000 + walk - 1);
    assert_eq!(t.anims[actor].player.playing(&clips, 1000 + walk - 1), "walk", "the walk until then");
    crate::battle::board::start_wanted(&mut t, &mut art, actor, Some(&body), 1000 + walk);
    assert!(t.anims[actor].want.is_none());
    let playing = t.anims[actor].player.playing(&clips, 1000 + walk).to_string();
    assert!(!playing.is_empty() && playing != "walk", "the attack clip from t0: {playing}");
    // standing still, the next pop starts it at once
    s.last_popped = Some((s.current_round, 2, idx));
    t.catch_up(s, 5000, false, walk, true);
    assert_eq!(t.anims[actor].want.as_ref().map(|w| w.1), Some(5000));
}

/// INPUT1 MR12 (`dash_tests`): a unit moved by a dash (its command's start this block made the `MoveKind::Dash`
/// board event) slides in `DASH_MS` (a walk takes the tuning's walk), playing the rig's `dash` clip when the body
/// has one, else its walk; under skip animations it stands on the position at once.
#[test]
fn dash_tests_a_dash_plays_its_clip_faster_than_a_walk() {
    let root = std::env::temp_dir().join(format!("emberwake_dash_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    scratch_body(&root, "test/dash", &["idle", "walk", "dash", "hurt"]);
    scratch_body(&root, "test/side", &["idle", "walk", "hurt"]);
    let mut art = Art::layered(Layers::of(vec![root.clone()]));
    let mut host = skirmish(3);
    assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
    let s = host.session.as_mut().unwrap();
    let (idx, actor) = s.resolver.scheduled.iter().enumerate().find(|(_, a)| !a.cancelled && s.w.grid.pos(a.unit).is_some()).map(|(i, a)| (i, a.unit)).expect("an action");
    let walk = 300;
    let here = s.w.grid.pos(actor).unwrap();
    let next = [V2::new(1, 0), V2::new(-1, 0), V2::new(0, 1), V2::new(0, -1)].into_iter().map(|d| here + d).find(|c| s.w.grid.is_valid(*c) && s.w.occupant(*c).is_none()).expect("a free neighbour");
    for (body, clip) in [("test/dash", "dash"), ("test/side", "walk")] {
        let clips = art.clips(body).unwrap().clone();
        s.w.grid.positions[actor] = Some(here);
        s.resolver.scheduled[idx].command.moved = None;
        let mut t = Tokens::default();
        t.catch_up(s, 0, false, walk, true);
        // a plain walk slides the walk's time
        s.w.grid.positions[actor] = Some(next);
        t.catch_up(s, 100, false, walk, true);
        assert_eq!((t.anims[actor].slide, t.anims[actor].slide_ms(walk)), (Some((here, 100)), walk), "{body}: a walk");
        // the dash back
        s.resolver.scheduled[idx].start = s.current_block;
        s.resolver.scheduled[idx].command.moved = Some(emberwake_battle::command::MoveKind::Dash);
        s.w.grid.positions[actor] = Some(here);
        t.catch_up(s, 1000, false, walk, true);
        assert_eq!((t.anims[actor].slide, t.anims[actor].slide_ms(walk)), (Some((next, 1000)), crate::battle::DASH_MS), "{body}: a dash runs DASH_MS");
        crate::battle::board::start_wanted(&mut t, &mut art, actor, Some(body), 1000);
        assert_eq!(t.anims[actor].player.playing(&clips, 1000 + crate::battle::DASH_MS - 1), clip, "{body}: the clip over the dash");
        assert_eq!(t.anims[actor].player.playing(&clips, 1000 + crate::battle::DASH_MS), "", "{body}: over after DASH_MS");
        // skip animations: on the position at once
        s.w.grid.positions[actor] = Some(next);
        t.catch_up(s, 5000, true, walk, true);
        assert!(t.anims[actor].slide.is_none_or(|(_, t0)| t0 != 5000) && t.anims[actor].seen.pos == Some(next), "{body}: instant");
    }
    let _ = std::fs::remove_dir_all(&root);
}

/// FX1-L1 L4: a unit killed by the step is drawn standing until the impact, then plays `death` from it.
#[test]
fn a_fall_waits_for_its_impact() {
    let (mut cx, _g, mut b) = ambush(json!("flat"));
    let found = step_until(&mut cx, &mut b, |b, i| {
        let s = b.host.session.as_ref().unwrap();
        i.struck.iter().any(|&u| !s.w.units[u].is_alive())
    });
    assert!(found, "the ambush has a fall");
    b.resolve_impacts(&mut cx);
    let i = b.impacts[0].clone();
    let n = b.host.session.as_ref().unwrap().w.units.len();
    let dead = *i.struck.iter().find(|&&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive()).unwrap();
    assert!(b.tokens.shown(dead, &b.host.session.as_ref().unwrap().w.units[dead], n).1, "standing until the impact");
    assert_eq!(b.tokens.anims[dead].fell, None);
    let at = i.at.unwrap();
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    let clips = unit_clips(&mut cx, &b, dead);
    assert!(!b.tokens.shown(dead, &b.host.session.as_ref().unwrap().w.units[dead], n).1);
    assert_eq!((b.tokens.anims[dead].fell, b.tokens.anims[dead].player.playing(&clips, at)), (Some(at), "death"));
}

/// FX1-L1 L5 (rule F) and L11: the next request fires a pending impact first; `resolve` shows its end state with
/// no pending impact; under skip animations a step shows at once with no pending impact and no hit-stop.
#[test]
fn a_later_request_flushes_and_skip_shows_at_once() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
    let first = b.impacts[0].clone();
    b.send(&cx, json!({"cmd": "step"}));
    assert!(first.struck.iter().all(|u| b.flashes.iter().any(|f| f.0 == *u)), "the first impact fired before the next step");
    assert!(b.impacts.len() <= 1, "at most the second step's impact pends");
    b.send(&cx, json!({"cmd": "resolve"}));
    assert!(b.impacts.is_empty(), "resolve: the end state, no pending impact");
    let s = b.host.session.as_ref().unwrap();
    let n = s.w.units.len();
    assert!((0..n).all(|i| b.tokens.shown(i, &s.w.units[i], n) == (s.w.units[i].hp, s.w.units[i].is_alive())), "the board shows the end state");
    // skip animations
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    cx.settings.v["skip_anim"] = json!(true);
    let hold = b.clock.hold;
    let mut hits = 0;
    for _ in 0..200 {
        let s = b.host.session.as_ref().unwrap();
        if s.combat_over || s.phase == Phase::Replay {
            break;
        }
        if s.is_planning() {
            b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
        }
        b.flashes.clear();
        b.send(&cx, json!({"cmd": "step"}));
        assert!(b.impacts.is_empty(), "skip: no pending impact");
        hits += b.flashes.len();
    }
    assert!(hits > 0, "skip: the hits flash at once");
    assert_eq!(b.clock.hold, hold, "skip: no hit-stop");
}

/// FX1-L1 L6 (rule P): autoplay sends no step while an impact pends, nor within `impact_gap_ms` of its firing.
#[test]
fn autoplay_waits_for_the_impact_and_its_gap() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
    b.resolve_impacts(&mut cx);
    let sent = b.host.history.len();
    (b.playing, b.next_at) = (true, std::time::Instant::now());
    b.clock_at = std::time::Instant::now();
    b.tick(&mut cx);
    assert_eq!(b.host.history.len(), sent, "no step while the impact pends");
    b.clock.ms = b.impacts[0].at.unwrap() as f64;
    b.fire_due(&mut cx);
    let now = std::time::Instant::now();
    assert!(!b.impacts_done(now), "the gap after the impact");
    assert!(b.impacts_done(now + std::time::Duration::from_millis(cx.tuning.value.impact_gap_ms)), "then the next step may go");
}

/// FX1-L1 L14: effects are presentation only: the same requests give the same battle log and HP with skip
/// animations on and off.
#[test]
fn effects_never_change_the_battle() {
    let run = |skip: bool| {
        let (mut cx, _g, mut b) = ambush(json!("stage"));
        cx.settings.v["skip_anim"] = json!(skip);
        for _ in 0..60 {
            let s = b.host.session.as_ref().unwrap();
            if s.combat_over || (s.phase == Phase::Replay && !s.can_advance_round()) {
                break;
            }
            if s.phase == Phase::Replay {
                b.send(&cx, json!({"cmd": "advance"}));
            } else {
                if s.is_planning() {
                    b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
                }
                b.send(&cx, json!({"cmd": "step"}));
            }
        }
        let s = b.host.session.as_ref().unwrap();
        (s.resolver.log.clone(), s.w.units.iter().map(|u| u.hp).collect::<Vec<_>>())
    };
    assert_eq!(run(false), run(true));
}

/// The canvas's RGBA bytes at logical (`x`, `y`) (the test canvas is at scale 1).
fn pixel(g: &aegis_ui::ui::Gfx, x: i32, y: i32) -> [u8; 4] {
    let k = ((y * g.canvas.w + x) * 4) as usize;
    [g.canvas.px[k], g.canvas.px[k + 1], g.canvas.px[k + 2], g.canvas.px[k + 3]]
}

/// FX1-L1 slice 2 (4.2 steps 2-3, 4.4): the pop puts the delivery on the live list (the trail of a blow, the
/// projectile of a shot) from `t0`; the impact adds the row's ring, flash and sparks at the impact; the effects draw
/// on the board in both views (a frame with them differs from the same frame without).
#[test]
fn an_action_pushes_its_delivery_at_the_pop_and_its_impact_effects_at_the_impact() {
    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = ambush(json!(view));
        assert!(step_until(&mut cx, &mut b, |_, i| i.hit() && (i.style.swing.is_some() || i.style.shot.is_some())), "{view}: a hit with a delivery");
        b.resolve_impacts(&mut cx);
        let i = b.impacts[0].clone();
        let at = i.at.unwrap();
        let want = if i.style.swing.is_some() { "trail" } else { "projectile" };
        assert!(b.fx.kinds(b.clock.now()).contains(&want), "{view}: the {want} from the pop");
        assert!(b.fx.live.iter().all(|e| e.start >= i.t0 && e.start <= at), "{view}: the delivery between t0 and the impact");
        b.clock.ms = at as f64;
        b.fire_due(&mut cx);
        let landed: Vec<&str> = b.fx.live.iter().filter(|e| e.start == at).map(|e| e.kind()).collect();
        let rows = [(i.style.hit.ring.is_some(), "ring"), (i.style.hit.flash.is_some(), "flash"), (i.style.hit.sparks.is_some(), "sparks")];
        for (has, kind) in rows {
            assert_eq!(landed.contains(&kind), has && (kind != "sparks" || !i.mods.miss), "{view}: {kind} at the impact ({landed:?})");
        }
        // the effects draw: the frame with them differs from the frame without
        b.clock.ms = (at + 20) as f64;
        b.floats.clear();
        b.flashes.clear();
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let with = g.canvas.px.clone();
        let live = std::mem::take(&mut b.fx);
        b.draw(&mut cx, &mut g, (-500, -500), true);
        assert_ne!(with, g.canvas.px, "{view}: the effects show");
        b.fx = live;
    }
}

/// FX1-L1 L9 (4.5) and L11: at its impact a 3-block skill that hit starts `heavy`, a 2-block one does not, a crit
/// alone starts none, a break starts `heavy`; under reduce motion none starts (the effects and the hit-stop stay).
#[test]
fn heavy_hits_and_breaks_shake_the_board_at_their_impact() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a hit");
    b.resolve_impacts(&mut cx);
    let i = b.impacts.remove(0);
    let at = i.at.unwrap();
    let heavy = cx.tuning.value.shakes.get("heavy").unwrap().clamped(cx.tuning.value.shake_limits());
    let fire = |b: &mut BattleScreen, cx: &crate::screens::Cx, blocks: i32, broke: bool, crit: bool| {
        b.camera.stop();
        let mut j = i.clone();
        (j.blocks, j.broke, j.mods.crit) = (blocks, broke, crit);
        b.fire(cx, j, at);
        b.camera.shake.map(|s| (s.0, s.1))
    };
    assert_eq!(fire(&mut b, &cx, 3, false, false), Some((heavy, at)), "a Heavy hit");
    assert_eq!(fire(&mut b, &cx, 2, false, false), None, "a 2-block hit");
    assert_eq!(fire(&mut b, &cx, 1, false, true), None, "a crit");
    assert_eq!(fire(&mut b, &cx, 1, true, false), Some((heavy, at)), "a break");
    cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(true);
    b.fx.clear();
    b.clock.hold = 0.0;
    assert_eq!(fire(&mut b, &cx, 3, true, false), None, "reduce motion: no shake");
    assert!(!b.fx.is_empty() && b.clock.hold > 0.0, "reduce motion keeps the effects and the hit-stop");
}

/// FX1-L1 L9 (4.5 row 3) and L11: a newly fired phase whose effects hold `shake` `tremor` starts it at once, also
/// under skip animations; a phase already seen does not start it again; reduce motion starts none.
#[test]
fn a_fired_phase_with_a_shake_starts_it_at_once() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    cx.settings.v["skip_anim"] = json!(true);
    let fire_phase = |b: &mut BattleScreen, id: &str| {
        let s = b.host.session.as_mut().unwrap();
        let effect: aegis_core::value::Obj = serde_json::from_value(json!({"type": emberwake_data::SHAKE_EFFECT, "preset": "tremor"})).unwrap();
        s.phases.phases.push(emberwake_battle::encounter::PhaseDef { id: id.into(), name: String::new(), trigger: Default::default(), effects: vec![effect] });
        s.phases.fired.insert(id.into(), (s.current_round, s.current_block));
    };
    fire_phase(&mut b, "fx_tremor");
    b.after_change(&cx);
    let tremor = cx.tuning.value.shakes.get("tremor").unwrap().clamped(cx.tuning.value.shake_limits());
    assert_eq!(b.camera.shake.map(|s| (s.0, s.1)), Some((tremor, b.clock.now())), "tremor at once, under skip too");
    b.camera.stop();
    b.after_change(&cx);
    assert_eq!(b.camera.shake, None, "a phase starts its shake once");
    cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(true);
    fire_phase(&mut b, "fx_tremor_2");
    b.after_change(&cx);
    assert_eq!(b.camera.shake, None, "reduce motion");
}

/// FX1-L2 M6 (4.4.4, 4.1): a unit the impact fells raises its dust ring and dust at its foot point on the ground
/// layer, which the Stage draws (pass Arrows) before the feet pass; under skip animations none.
#[test]
fn a_fall_raises_dust_on_the_ground_at_its_impact() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |b, i| i.struck.iter().any(|&u| !b.host.session.as_ref().unwrap().w.units[u].is_alive())), "a fall");
    b.resolve_impacts(&mut cx);
    let i = b.impacts[0].clone();
    let at = i.at.unwrap();
    let dead = *i.struck.iter().find(|&&u| !i.after[u].alive).unwrap();
    let geo = Geom::new(View::Stage, &cx.tuning.value);
    let foot = crate::battle::board::foot_point(&geo, b.host.session.as_ref().unwrap().w.grid.pos(dead).unwrap());
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    let dust: Vec<_> = b.fx.live.iter().filter(|e| e.layer == Layer::Ground && e.start == at && matches!(e.prim, aegis_ui::fx::Prim::Ring(r) if r.at == foot && r.r1 == 10.0)).collect();
    assert_eq!(dust.len(), 1, "the dust ring at the fallen unit's foot");
    let order = crate::stage::passes();
    let pos = |p: crate::stage::Pass| order.iter().position(|q| *q == p).unwrap();
    assert!(pos(crate::stage::Pass::Arrows) < pos(crate::stage::Pass::Feet), "the ground layer draws before the feet pass");
    // skip animations: no effects at the impact
    b.fx.clear();
    cx.settings.v["skip_anim"] = json!(true);
    b.fire(&cx, i, at);
    assert!(b.fx.is_empty());
}

/// FX1-L2 M4 (4.4.4, 4.2): a support's impact rises the gold motes from its target's foot point; its heal float
/// waits for the impact.
#[test]
fn mend_motes_rise_at_the_targets_foot_and_its_float_waits_for_the_impact() {
    let (mut cx, _g, mut b) = ambush(json!("flat"));
    assert!(step_until(&mut cx, &mut b, |_, i| i.hit()), "a step with feedback");
    b.resolve_impacts(&mut cx);
    let mut i = b.impacts.remove(0);
    let target = i.struck[0];
    let cell = b.host.session.as_ref().unwrap().w.grid.pos(target).unwrap();
    (i.style, i.area, i.to, i.struck, i.floats) = (fx::MEND, false, cell, Vec::new(), vec![(target, "+5".into(), cx.tuning.value.heal_color)]);
    let at = i.at.unwrap();
    b.floats.clear();
    b.impacts.push(i);
    b.fire_due(&mut cx);
    assert!(b.floats.is_empty(), "no heal float before the impact");
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    assert!(b.floats.iter().any(|f| f.unit == target && f.text == "+5"), "the heal float at the impact");
    let foot = crate::battle::board::foot_point(&Geom::new(View::Flat, &cx.tuning.value), cell);
    let motes = b.fx.live.iter().find_map(|e| match e.prim {
        aegis_ui::fx::Prim::Sparks(s) if e.start == at => Some(s),
        _ => None,
    });
    assert_eq!(motes.map(|s| (s.at, s.cols[0], s.ms)), Some((foot, fx::GOLD, 500)), "the gold motes at its foot");
}

/// FX1-L2 M7 (4.3): a missed attack's target steps 3 px away from its attacker and back at the impact; not under
/// reduce motion nor skip animations; a missed shot flies past it.
#[test]
fn a_missed_target_sidesteps_unless_reduce_motion() {
    let (mut cx, mut g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |b, i| !i.area && i.target.is_some_and(|u| b.host.session.as_ref().unwrap().w.units[u].is_alive() && u != i.actor)), "an attack on a standing unit");
    b.resolve_impacts(&mut cx);
    let mut i = b.impacts.remove(0);
    i.mods = fx::Mods { miss: true, ..fx::Mods::default() };
    let (u, at) = (i.target.unwrap(), i.at.unwrap());
    for (reduce, want) in [(true, false), (false, true)] {
        cx.settings.v[aegis_ui::vignette::REDUCE_MOTION] = json!(reduce);
        b.sidesteps.clear();
        b.fire(&cx, i.clone(), at);
        assert_eq!(b.sidesteps.iter().any(|s| s.0 == u && s.1 == at), want, "reduce motion {reduce}");
    }
    assert_ne!(fx::sidestep_of(&b.sidesteps, u, at + fx::SIDESTEP_OUT_MS), (0, 0), "3 px out at 80 ms");
    // the sidestep moves the target's sprite on the board
    b.clock.ms = (at + fx::SIDESTEP_OUT_MS) as f64;
    b.fx.clear();
    b.floats.clear();
    b.flashes.clear();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let with = g.canvas.px.clone();
    b.sidesteps.clear();
    b.draw(&mut cx, &mut g, (-500, -500), true);
    assert_ne!(with, g.canvas.px, "the target is drawn aside");
    cx.settings.v["skip_anim"] = json!(true);
    b.fire(&cx, i, at);
    assert!(b.sidesteps.is_empty(), "skip animations: no sidestep");
}

/// FX1-L2 M8 (4.8): the agent link's `state.fx` counts the live effects and pending impacts with their kinds and the
/// camera offset, and goes to 0 after the last impact plus its longest effect.
#[test]
fn the_agent_link_reports_the_effects_until_they_end() {
    let (mut cx, _g, mut b) = ambush(json!("stage"));
    assert!(step_until(&mut cx, &mut b, |_, i| i.hit() && (i.style.swing.is_some() || i.style.shot.is_some())), "a hit with a delivery");
    b.resolve_impacts(&mut cx);
    let st = b.state(&cx)["fx"].clone();
    assert_eq!(st["pending"], json!(1));
    assert!(st["live"].as_u64().unwrap() >= 1 && st["kinds"].as_array().is_some_and(|k| !k.is_empty()), "{st}");
    assert_eq!(st["shake"], json!([0, 0]));
    let at = b.impacts[0].at.unwrap();
    b.clock.ms = at as f64;
    b.fire_due(&mut cx);
    let st = b.state(&cx)["fx"].clone();
    assert_eq!(st["pending"], json!(0));
    assert_eq!(st["live"].as_u64().unwrap() as usize, b.fx.kinds(at).len());
    b.clock.ms = b.fx.busy_until().unwrap() as f64;
    b.clock.hold = 0.0;
    let st = b.state(&cx)["fx"].clone();
    assert_eq!((st["live"].clone(), st["pending"].clone(), st["kinds"].clone()), (json!(0), json!(0), json!([])), "all done");
}

/// FX1-L1 L12 (4.1): in both views the camera's offset moves a board pixel and leaves the planning strip and the
/// hostile cards unchanged; the `unit.<i>` and `tile.<x>.<y>` hit rects do not move.
#[test]
fn the_camera_shakes_the_board_only() {
    use aegis_ui::canvas::Rect;
    for view in ["stage", "flat"] {
        let (mut cx, mut g, mut b) = ambush(json!(view));
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let before = g.canvas.px.clone();
        let rects = |b: &BattleScreen| b.w.iter().filter(|w| w.id.starts_with("unit.") || w.id.starts_with("tile.")).map(|w| (w.id.clone(), w.rect)).collect::<Vec<_>>();
        let hit_rects = rects(&b);
        let now = b.clock.now();
        let quake = cx.tuning.value.shakes.get("quake").unwrap();
        assert!(b.camera.start(quake, now, 1, cx.tuning.value.shake_limits(), false));
        assert_ne!(b.camera.offset_at(now), (0, 0), "{view}: the shake moves the camera now");
        b.draw(&mut cx, &mut g, (-500, -500), true);
        let at = |buf: &[u8], x: i32, y: i32| {
            let k = ((y * g.canvas.w + x) * 4) as usize;
            [buf[k], buf[k + 1], buf[k + 2], buf[k + 3]]
        };
        let board = crate::stage::INTERIOR;
        let moved = (board.y..board.y + board.h).any(|y| (board.x..board.x + board.w).any(|x| at(&before, x, y) != pixel(&g, x, y)));
        assert!(moved, "{view}: a board pixel moved");
        for r in [Rect::new(2, 2, aegis_ui::ui::W - 4, 24), Rect::new(aegis_ui::ui::W - 126, 28, 124, 290)] {
            let same = (r.y..r.y + r.h).all(|y| (r.x..r.x + r.w).all(|x| at(&before, x, y) == pixel(&g, x, y)));
            assert!(same, "{view}: the HUD at {r:?} does not shake");
        }
        assert_eq!(rects(&b), hit_rects, "{view}: the hit rects stay");
    }
}
mod visclar_c_tests;
