//! WINDUP1-L, the acceptance of WINDUP1.md 6.3: scratch bodies and synthetic `Clips` with the section 4 rows (no shipped
//! body has them yet); the battles are the data's own encounters and every expectation comes from a rule (W1, the clip
//! rows), never from a number of the data.

use crate::battle::board::{start_wanted, windup_outline};
use crate::battle::{BattleScreen, AMBER};
use crate::battle::tokens::{command_pose, item_pose, plan_pose, skill_pose, Pose, PoseSet, Tokens};
use crate::screens::Screen;
use crate::tuning::Tuning;
use aegis_core::assets::{body_dir, Clips, Layers};
use aegis_ui::art::Art;
use emberwake_battle::command::CmdKind;
use emberwake_battle::plan::PlanStep;
use emberwake_battle::session::Session;
use emberwake_host::{Battle, Host};
use emberwake_world::Data;
use serde_json::{json, Value};
use std::sync::Arc;

const WALK: u64 = 300;

/// Section 4's rows (frames, ms, `loop_from` as the spec), an `attack` with its own wind-up frames (`strike` 2,
/// `impact` 3) and the rows every body has; `drop` leaves rows out (a body without them).
fn rows(drop: &[&str]) -> Value {
    let windup = [90, 110, 120, 180, 180, 180, 180];
    let all = vec![
        json!({"id": "idle", "frames": 2, "ms": [200, 200], "loop": true}),
        json!({"id": "walk", "frames": 1, "ms": [100], "loop": true}),
        json!({"id": "idle_guarded", "frames": 1, "ms": [100], "loop": true}),
        json!({"id": "attack", "frames": 5, "ms": [80, 90, 100, 120, 140], "loop": false, "strike": 2, "impact": 3}),
        json!({"id": "hurt", "frames": 1, "ms": [100], "loop": false}),
        json!({"id": "hurt_guarded", "frames": 1, "ms": [100], "loop": false}),
        json!({"id": "defend", "frames": 1, "ms": [100], "loop": false}),
        json!({"id": "death", "frames": 1, "ms": [100], "loop": false}),
        json!({"id": "windup", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
        json!({"id": "windup_heavy", "frames": 7, "ms": [100, 120, 140, 160, 160, 160, 160], "loop": true, "loop_from": 3}),
        json!({"id": "windup_buff", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
        json!({"id": "windup_debuff", "frames": 7, "ms": windup, "loop": true, "loop_from": 3}),
        json!({"id": "stagger", "frames": 5, "ms": [60, 90, 110, 120, 160], "loop": false}),
        json!({"id": "dazed", "frames": 8, "ms": [150, 150, 150, 150, 150, 150, 150, 150], "loop": true}),
    ];
    Value::Array(all.into_iter().filter(|r| !drop.contains(&r["id"].as_str().unwrap_or(""))).collect())
}

fn clips(drop: &[&str]) -> Clips {
    Clips::parse(&json!({"frame": [8, 8], "clips": rows(drop)})).expect("the scratch rows parse")
}

/// The humanoid set (every row) and the creature set (A7: `windup`, `stagger`, `dazed` only).
fn humanoid() -> PoseSet {
    PoseSet::of(&clips(&[]))
}

fn creature() -> PoseSet {
    PoseSet::of(&clips(&["windup_heavy", "windup_buff", "windup_debuff"]))
}

/// A scratch body `units/<key>` with `rows` (8 x 8 frames, each a bar in its row's colour).
fn write_body(root: &std::path::Path, key: &str, rows: &Value) {
    const F: u32 = 8;
    let dir = root.join(body_dir(key));
    std::fs::create_dir_all(&dir).unwrap();
    let list = rows.as_array().unwrap();
    let cols = list.iter().map(|r| r["frames"].as_u64().unwrap() as u32).max().unwrap();
    let (w, h) = (cols * F, list.len() as u32 * F);
    let mut px = vec![0u8; (w * h * 4) as usize];
    for (row, r) in list.iter().enumerate() {
        for f in 0..r["frames"].as_u64().unwrap() as u32 {
            for x in [1u32, 2, 3, 6] {
                let i = (((row as u32 * F + 4) * w + f * F + x) * 4) as usize;
                px[i..i + 4].copy_from_slice(&[30 + 12 * row as u8, 40 + 20 * f as u8, 60, 255]);
            }
        }
    }
    let mut enc = png::Encoder::new(std::io::BufWriter::new(std::fs::File::create(dir.join("sheet.png")).unwrap()), w, h);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().unwrap().write_image_data(&px).unwrap();
    std::fs::write(dir.join("clips.json"), json!({"frame": [F, F], "clips": rows}).to_string()).unwrap();
}

fn data() -> Arc<Data> {
    let dir = Data::find_data_dir(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Arc::new(Data::open(&dir).expect("data").0)
}

/// The data's `goblin_skirmish` (classic) or, with `ruleset`, under that ruleset.
fn battle(ruleset: Option<&str>) -> Host {
    let mut host = Host::new(Battle::with_data(data()));
    let mut req = json!({"cmd": "new", "encounter": "goblin_skirmish", "seed": 3});
    if let Some(r) = ruleset {
        req["ruleset"] = json!(r);
    }
    assert_eq!(host.handle(&req)["ok"], json!(true));
    host
}

/// Plans on the aggressive preset and confirms; the first live action over one block that starts at block 1 with a
/// wind-up pose (W1, W1a): (its index, its unit, its pose).
fn confirmed(host: &mut Host) -> (usize, usize, Pose) {
    assert_eq!(host.handle(&json!({"cmd": "preset", "preset": "aggressive"}))["ok"], json!(true));
    assert_eq!(host.handle(&json!({"cmd": "confirm"}))["ok"], json!(true));
    let s = host.session.as_ref().unwrap();
    s.resolver
        .scheduled
        .iter()
        .enumerate()
        .find_map(|(idx, a)| {
            let pose = command_pose(s, &a.command);
            let attack = matches!(a.command.kind, CmdKind::Attack { .. } | CmdKind::Area { .. });
            (attack && !a.cancelled && a.start == 1 && a.impact > a.start && pose.held()).then_some((idx, a.unit, pose))
        })
        .expect("a block-1 attack over one block")
}

/// Tokens whose every unit has the pose set `ps`.
fn tokens(s: &Session, ps: &PoseSet) -> Tokens {
    Tokens { poses: vec![ps.clone(); s.w.units.len()], ..Tokens::default() }
}

/// 6.3.1 and 8: rule W1 gives each command its pose and the bodies' fallbacks pick the clip.
#[test]
fn rule_w1_gives_each_command_its_pose_and_bodies_fall_back() {
    let d = data();
    let cat = &d.cat;
    let pose = |id: &str| skill_pose(cat.skill(id).unwrap_or_else(|| panic!("the skill {id}")));
    assert_eq!(pose("strike"), Pose::Attack);
    assert_eq!((pose("tl_heavy_cleave"), pose("tl_overhead_smash")), (Pose::Heavy, Pose::Heavy));
    assert_eq!((pose("mend"), pose("focus")), (Pose::Buff, Pose::Buff));
    assert_eq!((pose("active_guard"), pose("move")), (Pose::None, Pose::None));
    let mut hex = cat.skill("strike").unwrap().clone();
    hex.power_mult = 0.0;
    assert_eq!(skill_pose(&hex), Pose::Debuff, "an attack with power_mult 0 is a debuff");
    for it in cat.items.values().filter(|i| i.damage > 0 || i.heal > 0) {
        assert_eq!(item_pose(it), if it.aims_at_foes() { Pose::Debuff } else { Pose::Buff }, "{}", it.id);
    }
    let (h, c, bare) = (humanoid(), creature(), PoseSet::default());
    assert_eq!([h.clip(Pose::Heavy), h.clip(Pose::Debuff), h.clip(Pose::Buff), h.clip(Pose::Dazed)], [Some("windup_heavy"), Some("windup_debuff"), Some("windup_buff"), Some("dazed")]);
    assert_eq!([c.clip(Pose::Heavy), c.clip(Pose::Debuff), c.clip(Pose::Attack)], [Some("windup"); 3], "no heavy or debuff wind-up: the windup");
    assert_eq!(c.clip(Pose::Buff), None, "a creature on Support idles");
    assert!([Pose::Attack, Pose::Heavy, Pose::Buff, Pose::Debuff, Pose::Dazed, Pose::None].iter().all(|p| bare.clip(*p).is_none()), "a body without the rows shows no pose");
    assert_eq!((h.clip(Pose::Guard), bare.clip(Pose::Guard)), (Some("idle_guarded"), None), "a body without the rows stands as before");
}

/// 6.3.2, 6.3.8, 6.3.11 (classic): from the confirm the actor of a block-1 action over one block loops its wind-up
/// from frame 0 (frame `loop_from` once the intro has run); at its pop its attack starts at the `strike` frame at t0.
/// A body without the rows idles and strikes from frame 0, as before.
#[test]
fn a_two_block_action_winds_up_from_the_confirm_and_strikes_from_its_strike_frame() {
    let mut host = battle(None);
    let (idx, u, pose) = confirmed(&mut host);
    let k = clips(&[]);
    let ps = humanoid();
    let clip = ps.clip(pose).unwrap();
    let (mut t, mut bare) = (tokens(host.session.as_ref().unwrap(), &ps), Tokens::default());
    t.catch_up(host.session.as_ref().unwrap(), 1000, false, WALK, true);
    bare.catch_up(host.session.as_ref().unwrap(), 1000, false, WALK, true);
    let a = &t.anims[u];
    assert_eq!((a.player.base(), a.windup.map(|w| w.idx)), (clip, Some(idx)), "the wind-up from the confirm");
    let intro = k.ms_before(clip, 3);
    assert_eq!(a.player.frame(&k, 1000).unwrap().frame, 0);
    assert_eq!(a.player.frame(&k, 1000 + intro - 1).unwrap().frame, 2);
    assert_eq!(a.player.frame(&k, 1000 + intro).unwrap().frame, 3, "the hold from loop_from");
    assert_eq!(bare.anims[u].player.base(), "idle", "no rows: idle as before");
    // step until the action pops
    let root = aegis_core::scratch::Scratch::vacant("emberwake_windup_strike");
    write_body(&root, "test/windup", &rows(&[]));
    write_body(&root, "test/bare", &rows(&["windup", "windup_heavy", "windup_buff", "windup_debuff", "stagger", "dazed"]));
    let mut art = Art::layered(Layers::of(vec![root.to_path_buf()]));
    // play on until an action over one block pops live (its wind-up shown) and check its strike
    let mut now = 1000;
    let mut struck = None;
    for _ in 0..600 {
        let (over, phase, planning) = { let s = host.session.as_ref().unwrap(); (s.combat_over, s.phase, s.is_planning()) };
        if over {
            break;
        }
        if phase == emberwake_battle::session::Phase::Replay {
            host.handle(&json!({"cmd": "advance"}));
            continue;
        }
        if planning {
            host.handle(&json!({"cmd": "preset", "preset": "aggressive"}));
        }
        assert_eq!(host.handle(&json!({"cmd": "step"}))["ok"], json!(true));
        now += 1000;
        let s = host.session.as_ref().unwrap();
        let shown: Vec<Option<(i32, usize)>> = t.anims.iter().map(|a| a.windup.map(|w| (w.round, w.idx))).collect();
        t.catch_up(s, now, false, WALK, true);
        bare.catch_up(s, now, false, WALK, true);
        assert!(bare.anims.iter().all(|a| a.windup.is_none() && !a.player.base().starts_with("windup")), "no rows: no wind-up");
        let Some((round, _, idx)) = s.last_popped else { continue };
        let act = &s.resolver.scheduled[idx];
        if round == s.current_round && !act.cancelled && s.w.units[act.unit].is_alive() && t.anims[act.unit].want.is_some() && shown[act.unit] == Some((round, idx)) {
            struck = Some(act.unit);
            break;
        }
    }
    let u = struck.expect("an attack over one block pops live after its wind-up");
    let a = &t.anims[u];
    assert!(a.strike && a.windup.is_none(), "the strike follows the shown wind-up, which ends");
    let t0 = a.want.as_ref().unwrap().1;
    start_wanted(&mut t, &mut art, u, Some("test/windup"), t0);
    let f = t.anims[u].player.frame(&k, t0).unwrap();
    assert_eq!((f.clip.as_str(), f.frame), ("attack", k.clip("attack").unwrap().strike), "the attack from its strike frame at t0");
    assert!(!bare.anims[u].strike);
    let t0 = bare.anims[u].want.as_ref().unwrap().1;
    start_wanted(&mut bare, &mut art, u, Some("test/bare"), t0);
    assert_eq!(bare.anims[u].player.frame(&k, t0).unwrap().frame, 0, "no wind-up shown: the attack from frame 0");
    let _ = std::fs::remove_dir_all(&root);
}

/// 6.3.3 (W1a): an action whose span is one block shows no wind-up and strikes from frame 0.
#[test]
fn a_one_block_action_shows_no_windup_and_strikes_from_frame_0() {
    let mut host = battle(None);
    let (idx, u, _) = confirmed(&mut host);
    let s = host.session.as_mut().unwrap();
    s.resolver.scheduled[idx].impact = s.resolver.scheduled[idx].start;
    let mut t = tokens(s, &humanoid());
    t.catch_up(s, 1000, false, WALK, true);
    assert!(t.anims[u].windup.is_none() && !t.anims[u].player.base().starts_with("windup"), "one block: no wind-up");
    s.resolver.scheduled[idx].landed = true;
    s.last_popped = Some((s.current_round, 1, idx));
    t.catch_up(s, 2000, false, WALK, true);
    assert!(t.anims[u].want.is_some() && !t.anims[u].strike, "the attack from frame 0");
}

/// The first unit showing a wind-up after the confirm (`tokens` with every body's rows) at clock 1000.
fn winding(host: &mut Host) -> (Tokens, usize, usize) {
    let (idx, u, _) = confirmed(host);
    let s = host.session.as_ref().unwrap();
    let mut t = tokens(s, &humanoid());
    t.catch_up(s, 1000, false, WALK, true);
    assert!(t.anims[u].windup.is_some());
    (t, idx, u)
}

/// 6.3.4 (W6, WQ2): a break mid wind-up plays `stagger` at the breaking impact instead of `hurt`, the wind-up ends
/// then (not before) and nothing strikes; a guarding unit's break plays `stagger` too.
#[test]
fn a_break_staggers_at_its_impact_instead_of_a_hurt_and_never_strikes() {
    let mut host = battle(None);
    let (mut t, idx, u) = winding(&mut host);
    let k = clips(&[]);
    let s = host.session.as_mut().unwrap();
    s.resolver.scheduled[idx].cancelled = true;
    s.resolver.scheduled[idx].broken = true;
    s.w.poise.entry(u).or_default().broken = true;
    s.w.units[u].hp -= 1;
    let after = t.follow(s, 2000, false, WALK, true, true);
    assert!(t.anims[u].windup.is_some() && t.anims[u].player.base().starts_with("windup"), "the wind-up holds until the impact");
    t.react(&after, &[], 2100, false);
    let a = &t.anims[u];
    assert_eq!(a.player.playing(&k, 2100), "stagger", "the stagger, not hurt");
    assert!(a.windup.is_none() && !a.player.base().starts_with("windup") && a.want.is_none(), "the wind-up ended, no strike");
    // a guarding unit that is not winding up
    let g = (0..s.w.units.len()).find(|&i| i != u && t.anims[i].windup.is_none() && s.w.units[i].is_alive()).expect("another unit");
    s.w.units[g].guard = 1.0;
    t.catch_up(s, 3000, false, WALK, true);
    s.w.poise.entry(g).or_default().broken = true;
    s.w.units[g].hp -= 1;
    t.catch_up(s, 4000, false, WALK, true);
    assert_eq!(t.anims[g].player.playing(&k, 4000), "stagger", "a guard's break staggers too");
}

/// 6.3.5 (W7): the Staggered status landing plays `stagger` once, the base is `dazed` while it lasts, then `idle`.
#[test]
fn the_staggered_status_reels_then_stands_dazed_until_it_ends() {
    let mut host = battle(None);
    let (mut t, _, u) = winding(&mut host);
    let k = clips(&[]);
    let s = host.session.as_mut().unwrap();
    let v = (0..s.w.units.len()).find(|&i| i != u && t.anims[i].windup.is_none() && s.w.units[i].is_alive() && s.w.units[i].guard <= 0.0 && s.w.units[i].stance != emberwake_rules::Stance::Defensive).expect("a unit standing idle");
    s.w.units[v].st.pending.insert("stagger".into());
    t.catch_up(s, 2000, false, WALK, true);
    assert_eq!((t.anims[v].player.playing(&k, 2000), t.anims[v].player.base()), ("stagger", "dazed"));
    t.catch_up(s, 3000, false, WALK, true);
    assert_eq!((t.anims[v].player.playing(&k, 3000), t.anims[v].player.base()), ("", "dazed"), "once; dazed while it lasts");
    s.w.units[v].st.pending.shift_remove("stagger");
    t.catch_up(s, 4000, false, WALK, true);
    assert_eq!(t.anims[v].player.base(), "idle", "idle once it is gone");
}

/// 6.3.6 (W8): an actor that falls mid wind-up plays `death`, no `stagger`, even when broken by the same hit.
#[test]
fn an_actor_that_falls_mid_windup_plays_death_not_stagger() {
    let mut host = battle(None);
    let (mut t, idx, u) = winding(&mut host);
    let k = clips(&[]);
    let s = host.session.as_mut().unwrap();
    s.resolver.scheduled[idx].cancelled = true;
    s.w.poise.entry(u).or_default().broken = true;
    s.w.units[u].hp = 0;
    t.catch_up(s, 2000, false, WALK, true);
    assert_eq!(t.anims[u].player.playing(&k, 2000), "death");
    assert!(t.anims[u].windup.is_none() && t.anims[u].fell == Some(2000));
}

/// 6.3.7 (section 5, timeline): while planning a Guard intent stands guarded, an Assault with a Heavy Cleave opener in
/// the heavy wind-up, Support in the buff wind-up, Staggered dazed; an enemy shows its tell's pose while the tells
/// show (a creature body: its `windup`) and idles while they are hidden (WQ1).
#[test]
fn planning_shows_each_units_round_pose() {
    let mut host = battle(Some("timeline"));
    let s = host.session.as_mut().unwrap();
    let ps = humanoid();
    let mut t = tokens(s, &ps);
    for &e in &s.w.enemies.clone() {
        t.poses[e] = creature();
    }
    let h = s.w.allies.iter().copied().find(|&a| s.w.units[a].is_alive() && s.w.grid.pos(a).is_some()).expect("a hero");
    let foe = s.w.enemies.iter().copied().find(|&e| s.w.grid.pos(e).is_some()).expect("a foe");
    let cleave = s.w.cat.skills.iter().position(|k| k.id == "tl_heavy_cleave").expect("tl_heavy_cleave");
    s.set_round_intent(h, "guard").expect("guard");
    t.catch_up(s, 0, false, WALK, true);
    assert_eq!(t.anims[h].player.base(), "idle_guarded", "Guard");
    s.set_round_intent(h, "assault").expect("assault");
    s.plan.set_steps(h, vec![PlanStep::engage(cleave, foe)]);
    t.catch_up(s, 100, false, WALK, true);
    assert_eq!((t.anims[h].player.base(), t.anims[h].pose), ("windup_heavy", Pose::Heavy), "Assault, Heavy Cleave opener");
    t.catch_up(s, 200, false, WALK, true);
    assert_eq!(t.anims[h].player.frame(&clips(&[]), 200).unwrap().frame, t.anims[h].player.frame(&clips(&[]), 100).unwrap().frame + 1, "an unchanged pose keeps its clock");
    s.set_round_intent(h, "support").expect("support");
    s.plan.set_steps(h, Vec::new());
    t.catch_up(s, 300, false, WALK, true);
    assert_eq!(t.anims[h].player.base(), "windup_buff", "Support");
    s.w.units[h].st.active.insert("stagger".into());
    t.catch_up(s, 400, false, WALK, true);
    assert_eq!(t.anims[h].player.base(), "dazed", "Staggered");
    s.w.units[h].st.active.shift_remove("stagger");
    // the enemies: their tell's pose with the tells shown, idle with them hidden
    let shown: Vec<(usize, Pose)> = s.w.enemies.iter().map(|&e| (e, plan_pose(s, e, true, &[]))).collect();
    let (e, p) = shown.iter().copied().find(|(_, p)| matches!(p, Pose::Attack | Pose::Heavy | Pose::Debuff)).expect("an enemy about to attack");
    t.catch_up(s, 500, false, WALK, true);
    assert_eq!((t.anims[e].player.base(), p.held()), ("windup", true), "an attacking enemy with the tells shown");
    t.catch_up(s, 600, false, WALK, false);
    assert!(s.w.enemies.iter().all(|&e| t.anims[e].player.base() == "idle" || t.anims[e].player.base() == "idle_guarded"), "the tells hidden: no pose");
}

/// 6.3.11 (classic, no intents): the planning pose comes from the first command: a Defend first stands guarded.
#[test]
fn classic_planning_poses_come_from_the_first_command() {
    let mut host = battle(None);
    let s = host.session.as_mut().unwrap();
    let mut t = tokens(s, &humanoid());
    let h = s.w.allies[0];
    let defend = s.w.cat.skills.iter().position(|k| emberwake_battle::skill::kind(k) == emberwake_battle::skill::Kind::Defend).expect("a defend skill");
    let strike = s.w.cat.skills.iter().position(|k| k.id == "strike").expect("strike");
    let foe = s.w.enemies[0];
    s.plan.set_steps(h, vec![PlanStep::self_skill(defend), PlanStep::engage(strike, foe)]);
    t.catch_up(s, 0, false, WALK, true);
    assert_eq!(t.anims[h].player.base(), "idle_guarded", "a Defend first");
    s.plan.set_steps(h, vec![PlanStep::engage(strike, foe)]);
    t.catch_up(s, 100, false, WALK, true);
    assert_eq!(t.anims[h].player.base(), "windup", "an attack first");
    assert!(windup_outline(&t.anims[h], &Tuning::default()).is_none(), "no amber outline while planning (WQ5)");
}

/// 6.3.9 (6.1): skip animations starts a wind-up at its `loop_from` and plays no `stagger` (the dazed base still
/// shows); reduce motion plays the intro and then holds frame `loop_from`.
#[test]
fn skip_starts_the_hold_and_reduce_motion_holds_it_still() {
    let k = clips(&[]);
    let mut host = battle(None);
    let (idx, u, pose) = confirmed(&mut host);
    let s = host.session.as_mut().unwrap();
    let ps = humanoid();
    let clip = ps.clip(pose).unwrap();
    let mut skip = tokens(s, &ps);
    skip.catch_up(s, 1000, true, WALK, true);
    assert_eq!(skip.anims[u].player.frame(&k, 1000).unwrap().frame, 3, "skip: the hold at once");
    let mut still = tokens(s, &ps);
    still.still = true;
    still.catch_up(s, 1000, false, WALK, true);
    let intro = k.ms_before(clip, 3);
    assert_eq!(still.anims[u].player.frame(&k, 1000 + intro - 1).unwrap().frame, 2, "reduce motion: the intro plays");
    for dt in [0, 300, 900, 2000] {
        assert_eq!(still.anims[u].player.frame(&k, 1000 + intro + dt).unwrap().frame, 3, "reduce motion: the hold stands still");
    }
    s.resolver.scheduled[idx].cancelled = true;
    s.w.poise.entry(u).or_default().broken = true;
    s.w.units[u].st.active.insert("stagger".into());
    skip.catch_up(s, 2000, true, WALK, true);
    assert_eq!((skip.anims[u].player.playing(&k, 2000), skip.anims[u].player.base()), ("", "dazed"), "skip: no stagger, the dazed base");
}

/// 6.3.10 (W9, WQ5): the amber outline only while a wind-up shows (`windup_outline_alpha`, 0 off).
#[test]
fn the_amber_outline_shows_only_during_a_windup() {
    let mut host = battle(None);
    let (t, _, u) = winding(&mut host);
    let tu = Tuning::default();
    assert_eq!(windup_outline(&t.anims[u], &tu), Some([AMBER[0], AMBER[1], AMBER[2], tu.windup_outline_alpha]));
    let idle = t.anims.iter().find(|a| a.windup.is_none()).expect("a unit not winding up");
    assert_eq!(windup_outline(idle, &tu), None);
    assert_eq!(windup_outline(&t.anims[u], &Tuning { windup_outline_alpha: 0, ..Tuning::default() }), None, "0: off");
}

/// The Test Arena on `goblin_ambush` whose every unit's body is a scratch body with the section 4 rows (layered
/// over the game's art).
fn windup_arena(tag: &str) -> (crate::screens::Cx, aegis_ui::ui::Gfx, BattleScreen, aegis_core::scratch::Scratch) {
    let mut cx = crate::hud::hud_tests::game_cx();
    let game = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    cx.art = Art::new(Some(game.clone()));
    cx.app.encounter = "goblin_ambush".into();
    cx.app.seed = Some(1);
    cx.settings.v["battle_view"] = json!("flat");
    let b = BattleScreen::sandbox(&cx);
    let root = aegis_core::scratch::Scratch::vacant(&format!("emberwake_windup_{tag}"));
    let s = b.host.session.as_ref().unwrap();
    for (i, u) in s.w.units.iter().enumerate() {
        if let Some(key) = cx.art.first_body(&crate::battle::tokens::body_keys(&s.art[i], u)) {
            write_body(&root, &key, &rows(&[]));
        }
    }
    let mut layers = Layers::for_game(&game);
    layers.roots.insert(0, root.to_path_buf());
    cx.art = Art::layered(layers);
    (cx, crate::hud::hud_tests::gfx(), b, root)
}

/// 6.3.2 (W4, the screen): a strike from a held wind-up lands `ms_before(impact) - ms_before(strike)` after t0.
#[test]
fn a_strike_from_the_pose_lands_its_strike_to_impact_ms_after_t0() {
    let (mut cx, _g, mut b, root) = windup_arena("impact");
    assert!(b.learn_poses(&mut cx));
    b.sync_anims(&cx);
    assert!(b.tokens.poses.iter().any(|p| p.has("windup")), "the bodies' rows are known");
    let found = crate::battle::tests::step_until(&mut cx, &mut b, |_, i| i.strike && i.delivery == crate::fx::Delivery::Melee);
    assert!(found, "a melee strike from a wind-up");
    b.resolve_impacts(&mut cx);
    let i = b.impacts.last().unwrap().clone();
    let k = clips(&[]);
    let c = k.clip("attack").unwrap();
    assert_eq!(i.at, Some(i.t0 + c.impact_ms().unwrap() - c.ms_before(c.strike)), "from the strike frame to the impact frame");
    let _ = std::fs::remove_dir_all(&root);
}

/// 6.3.12 (6.2): the agent link's `state.tokens` names, per unit, the clip drawn, the base, the pose and the wind-up.
#[test]
fn the_agent_link_tokens_match_the_drawn_clips() {
    let (mut cx, mut g, mut b, root) = windup_arena("state");
    b.learn_poses(&mut cx);
    b.send(&cx, json!({"cmd": "preset", "preset": "aggressive"}));
    b.send(&cx, json!({"cmd": "confirm"}));
    b.draw(&mut cx, &mut g, (-500, -500), true);
    let st = b.state(&cx)["tokens"].clone();
    let s = b.host.session.as_ref().unwrap();
    let list = st.as_array().expect("tokens");
    assert_eq!(list.len(), s.w.units.len());
    let now = b.clock.now();
    let mut winding = 0;
    for (i, e) in list.iter().enumerate() {
        let a = &b.tokens.anims[i];
        let body = cx.art.first_body(&crate::battle::tokens::body_keys(&s.art[i], &s.w.units[i])).expect("a body");
        let drawn = a.player.frame(cx.art.clips(&body).unwrap(), now).unwrap().clip;
        assert_eq!((e["unit"].as_u64(), e["clip"].as_str(), e["base"].as_str(), e["pose"].as_str()), (Some(i as u64), Some(drawn.as_str()), Some(a.player.base()), Some(a.pose.id())));
        match a.windup {
            Some(w) => {
                winding += 1;
                assert_eq!(e["windup"], json!([w.blocks.0, w.blocks.1]));
                assert!(e["pose"].as_str().is_some_and(|p| !p.is_empty()) && e["base"].as_str().is_some_and(|c| c.starts_with("windup")));
            }
            None => assert!(e["windup"].is_null()),
        }
    }
    assert!(winding > 0, "some unit winds up after the confirm");
    let _ = std::fs::remove_dir_all(&root);
}
