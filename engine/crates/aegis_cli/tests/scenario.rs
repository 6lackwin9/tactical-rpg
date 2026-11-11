//! `aegis scenario` on a fake link (INTEG1-E1; docs/specs/AGENTROLES1.md 4): the file rules and the driver's rules, no window.

use aegis_cli::scenario::{self, at_path, cond_holds, op_holds, parse, parse_cond, Cond, GameExit, Launcher, Link, Op, Options, Outcome, Plan, Process, Scenario, StartFailure, Status, Step};
use serde_json::{json, Value};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

// ---- the file ----

fn minimal(steps: Value) -> Value {
    json!({"id": "x", "steps": steps})
}

fn problems_of(v: Value) -> String {
    parse("x", &v).expect_err("refused").join("\n")
}

#[test]
fn a_minimal_file_takes_the_defaults() {
    let s = parse("x", &minimal(json!([{"step": "back"}]))).unwrap();
    assert_eq!(s.status, Status::Active);
    assert_eq!(s.launch.screen, "menu");
    assert_eq!(s.sizes, [(1280, 720)]);
    assert_eq!(s.timeout, Duration::from_secs(300));
    assert_eq!(s.title, "");
    assert_eq!(s.steps, [Step::Back]);
}

#[test]
fn each_step_parses_with_its_keys_and_defaults() {
    let s = parse(
        "x",
        &minimal(json!([
            {"step": "click", "id": "a"},
            {"step": "hover", "id": "a"},
            {"step": "key", "key": "enter"},
            {"step": "type", "text": ""},
            {"step": "drag", "id": "a", "to": "b"},
            {"step": "drag", "id": "a", "to": [1, 2]},
            {"step": "link", "body": {"cmd": "state"}},
            {"step": "wait", "until": {"widget": "a"}},
            {"step": "assert", "until": {"path": "screen", "eq": "menu"}},
            {"step": "shot", "name": "a-b_1"},
            {"step": "audit", "scales": [1.0]},
            {"step": "repeat", "times": 2, "steps": [{"step": "back"}]},
        ])),
    )
    .unwrap();
    assert!(matches!(&s.steps[2], Step::Key { times: 1, .. }));
    assert!(matches!(&s.steps[7], Step::Wait { timeout_ms: scenario::WAIT_MS_DEFAULT, .. }));
    assert!(matches!(&s.steps[9], Step::Shot { max_w: 1920, .. }));
    let Step::Audit { fail, .. } = &s.steps[10] else { panic!() };
    assert_eq!(fail, &["cut", "edge", "overlap", "spill"]);
}

#[test]
fn an_unknown_step_or_key_names_its_place() {
    let got = problems_of(minimal(json!([{"step": "back"}, {"step": "dance"}])));
    assert!(got.contains("step 2: unknown step 'dance'"), "{got}");
    let got = problems_of(minimal(json!([{"step": "click", "id": "a", "idd": "b"}])));
    assert!(got.contains("step 1: unknown key 'idd'"), "{got}");
    let got = problems_of(json!({"id": "x", "steps": [{"step": "back"}], "sized": []}));
    assert!(got.contains("scenario: unknown key 'sized'"), "{got}");
    let got = problems_of(minimal(json!([{"step": "repeat", "times": 2, "steps": [{"step": "back"}, {"step": "nope"}]}])));
    assert!(got.contains("step 1.2: unknown step 'nope'"), "{got}");
}

#[test]
fn a_bad_file_is_named_and_refused_in_every_field() {
    let cases: Vec<(Value, &str)> = vec![
        (json!({"id": "y", "steps": [{"step": "back"}]}), "is not the file's name"),
        (json!({"steps": [{"step": "back"}]}), "needs 'id'"),
        (json!({"id": "x", "steps": []}), "'steps' is a list"),
        (json!({"id": "x"}), "'steps' is a list"),
        (json!({"id": "x", "status": "waiting", "steps": [{"step": "back"}]}), "needs 'why'"),
        (json!({"id": "x", "status": "someday", "steps": [{"step": "back"}]}), "'status' is"),
        (json!({"id": "x", "launch": {"screen": "credits"}, "steps": [{"step": "back"}]}), "'screen' is menu, arena or runner"),
        (json!({"id": "x", "launch": {"map": 1}, "steps": [{"step": "back"}]}), "launch: unknown key"),
        (json!({"id": "x", "sizes": ["640x360"], "steps": [{"step": "back"}]}), "is under 1280x720"),
        (json!({"id": "x", "sizes": ["1280x719"], "steps": [{"step": "back"}]}), "is under 1280x720"),
        (json!({"id": "x", "sizes": ["99999x99999"], "steps": [{"step": "back"}]}), "is over"),
        (json!({"id": "x", "sizes": ["big"], "steps": [{"step": "back"}]}), "is not WxH"),
        (json!({"id": "x", "sizes": [], "steps": [{"step": "back"}]}), "'sizes' is a list"),
        (json!({"id": "x", "settings": {"mute": false}, "steps": [{"step": "back"}]}), "'mute' false is refused"),
        (json!({"id": "x", "settings": {"fullscreen": true}, "steps": [{"step": "back"}]}), "'fullscreen' belongs to the driver"),
        (json!({"id": "x", "settings": {"start_inactive": false}, "steps": [{"step": "back"}]}), "'start_inactive' belongs to the driver"),
        (json!({"id": "x", "settings": {"window_size": "1920x1080"}, "steps": [{"step": "back"}]}), "'window_size' belongs to the driver"),
        (json!({"id": "x", "timeout_s": 0, "steps": [{"step": "back"}]}), "'timeout_s' must be"),
        (json!({"id": "x", "timeout_s": 3601, "steps": [{"step": "back"}]}), "'timeout_s' must be"),
        (minimal(json!([{"step": "shot", "name": "../up"}])), "'name' becomes a file"),
        (minimal(json!([{"step": "link", "body": {"cmd": "quit"}}])), "the driver alone stops the game"),
        (minimal(json!([{"step": "link", "body": {}}])), "'body' is a request"),
        (minimal(json!([{"step": "drag", "id": "a"}])), "'to' is a widget id"),
        (minimal(json!([{"step": "audit", "scales": []}])), "'scales' is a list"),
        (minimal(json!([{"step": "audit", "scales": [3.0]}])), "'scales' is a list"),
        (minimal(json!([{"step": "audit", "scales": [1.0], "fail": ["typo"]}])), "'fail' is a list"),
        (minimal(json!([{"step": "wait", "until": {"path": "a", "eq": 1, "ne": 2}}])), "has one of eq"),
        (minimal(json!([{"step": "wait", "until": {"path": "a", "gt": "b"}}])), "'gt' needs a number"),
        (minimal(json!([{"step": "wait", "until": {"path": "a", "absent": false}}])), "'absent' is written as true"),
        (minimal(json!([{"step": "wait", "until": {"path": "a", "eq": 1, "source": "disk"}}])), "'source' is"),
        (minimal(json!([{"step": "wait", "until": {"all": []}}])), "'all' is a list"),
        (minimal(json!([{"step": "wait", "until": {"nope": 1}}])), "has 'path', 'widget' or 'all'"),
        (minimal(json!([{"step": "wait", "until": {"widget": "a"}, "timeout_ms": 0}])), "'timeout_ms' must be"),
        (minimal(json!([{"step": "wait"}])), "needs 'until'"),
        (minimal(json!([{"step": "perf", "name": "a"}])), "not built yet"),
    ];
    for (file, wanted) in cases {
        let got = problems_of(file.clone());
        assert!(got.contains(wanted), "{file}: wanted '{wanted}', got: {got}");
    }
}

#[test]
fn the_step_count_holds_after_repeat() {
    let at_limit = minimal(json!([{"step": "repeat", "times": 50, "steps": [{"step": "key", "key": "up", "times": 10}]}]));
    assert!(parse("x", &at_limit).is_ok());
    let over = minimal(json!([{"step": "repeat", "times": 50, "steps": [{"step": "key", "key": "up", "times": 10}, {"step": "back"}]}]));
    assert!(problems_of(over).contains("550 steps after repeat"));
    // nesting multiplies, and cannot overflow into a small number
    let mut deep = json!({"step": "back"});
    for _ in 0..4 {
        deep = json!({"step": "repeat", "times": 500, "steps": [deep]});
    }
    assert!(problems_of(minimal(json!([deep.clone()]))).contains("steps after repeat"));
    let too_deep = json!({"step": "repeat", "times": 1, "steps": [deep]});
    assert!(problems_of(minimal(json!([too_deep]))).contains("nests at most"));
}

// ---- conditions ----

fn cond(v: Value) -> Cond {
    let mut problems = Vec::new();
    let c = parse_cond(&v, "t", 0, &mut problems);
    assert!(problems.is_empty(), "{problems:?}");
    c
}

#[test]
fn a_path_reads_keys_and_list_places() {
    let v = json!({"state": {"units": [{"hp": 3}, {"hp": null}]}});
    assert_eq!(at_path(&v, "state.units.0.hp"), Some(&json!(3)));
    assert_eq!(at_path(&v, "state.units.1.hp"), None, "null counts as absent");
    assert_eq!(at_path(&v, "state.units.9.hp"), None);
    assert_eq!(at_path(&v, "state.nope"), None);
}

#[test]
fn each_op_holds_as_it_says() {
    let v = |x: Value| Some(x);
    assert!(op_holds(v(json!("a")).as_ref(), &Op::Eq(json!("a"))));
    assert!(!op_holds(v(json!("a")).as_ref(), &Op::Eq(json!("b"))));
    assert!(op_holds(v(json!(2)).as_ref(), &Op::Eq(json!(2.0))), "a number equals itself in any form");
    assert!(op_holds(v(json!("a")).as_ref(), &Op::Ne(json!("b"))));
    assert!(!op_holds(v(json!("a")).as_ref(), &Op::Ne(json!("a"))));
    assert!(op_holds(v(json!(3)).as_ref(), &Op::Gt(2.0)));
    assert!(!op_holds(v(json!(2)).as_ref(), &Op::Gt(2.0)));
    assert!(op_holds(v(json!(1)).as_ref(), &Op::Lt(2.0)));
    assert!(!op_holds(v(json!("3")).as_ref(), &Op::Gt(2.0)), "a text is no number");
    assert!(op_holds(v(json!(["a", "b"])).as_ref(), &Op::Has(json!("b"))));
    assert!(!op_holds(v(json!(["a"])).as_ref(), &Op::Has(json!("b"))));
    assert!(op_holds(v(json!({"k": 1})).as_ref(), &Op::Has(json!("k"))));
    assert!(op_holds(v(json!("hello")).as_ref(), &Op::Has(json!("ell"))));
    assert!(op_holds(None, &Op::Absent));
    assert!(!op_holds(v(json!(0)).as_ref(), &Op::Absent));
    // a mistyped path passes nothing but `absent`
    for op in [Op::Eq(json!(1)), Op::Ne(json!(1)), Op::Gt(0.0), Op::Lt(0.0), Op::Has(json!(1))] {
        assert!(!op_holds(None, &op), "{op:?}");
    }
}

#[test]
fn a_condition_reads_the_tree_the_game_or_the_widgets() {
    let tree = json!({"screen": "menu", "stack": ["menu"], "widgets": [{"id": "a"}]});
    let game = json!({"gold": 5});
    assert!(cond_holds(&cond(json!({"widget": "a"})), &tree, &game).is_ok());
    let missing = cond_holds(&cond(json!({"widget": "b"})), &tree, &game).unwrap_err();
    assert!(missing.contains("no widget 'b'"), "{missing}");
    assert!(cond_holds(&cond(json!({"path": "gold", "source": "game", "gt": 4})), &tree, &game).is_ok());
    assert!(cond_holds(&cond(json!({"path": "gold", "gt": 4})), &tree, &game).is_err(), "the tree has no gold");
    let all = cond(json!({"all": [{"path": "screen", "eq": "menu"}, {"path": "stack", "has": "battle"}]}));
    let why = cond_holds(&all, &tree, &game).unwrap_err();
    assert!(why.contains("stack is [\"menu\"]"), "the failure says what was there: {why}");
}

// ---- files ----

static NEXT: AtomicU32 = AtomicU32::new(0);

fn temp(name: &str) -> aegis_core::scratch::Scratch {
    let dir = aegis_core::scratch::Scratch::new(&format!("scenario_test_{}_{name}", NEXT.fetch_add(1, Ordering::Relaxed)));
    dir
}

#[test]
fn a_folder_checks_every_file_and_a_missing_one_is_empty() {
    let dir = temp("check");
    assert!(scenario::files_in(&dir.join("nope")).is_empty());
    std::fs::write(dir.join("good.json"), r#"{"id": "good", "steps": [{"step": "back"}]}"#).unwrap();
    std::fs::write(dir.join("bad.json"), r#"{"id": "bad", "steps": [{"step": "dance"}]}"#).unwrap();
    std::fs::write(dir.join("broken.json"), "{").unwrap();
    std::fs::write(dir.join("note.txt"), "not a scenario").unwrap();
    let files = scenario::files_in(&dir);
    assert_eq!(files.len(), 3);
    let problems = scenario::check_files(&files);
    assert_eq!(problems.len(), 2, "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("bad.json") && p.contains("step 1: unknown step 'dance'")), "{problems:?}");
    assert!(problems.iter().any(|p| p.contains("broken.json") && p.contains("bad JSON")), "{problems:?}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn an_oversized_file_is_refused_before_it_is_read() {
    let dir = temp("big");
    let path = dir.join("big.json");
    std::fs::write(&path, vec![b' '; scenario::FILE_MAX as usize + 1]).unwrap();
    assert!(scenario::load(&path).unwrap_err()[0].contains("at most"));
    std::fs::remove_dir_all(&dir).unwrap();
}

// ---- the fake game ----

/// What the fake link saw and how it answers; shared with the test.
#[derive(Default)]
struct Script {
    asked: Vec<Value>,
    frame: u64,
    /// `ui_tree` keys over the defaults, by how many trees were asked: (from which ask, keys).
    tree: Value,
    /// `tree` takes effect after this many asks of `ui_tree`.
    tree_after: usize,
    trees: usize,
    /// A request's cmd whose reply is `ok` false.
    refuse: Option<String>,
    /// From this request on (count) the link is dead.
    dies_at: Option<usize>,
    /// Frames stop coming after this many asks of `ui_tree`.
    stalls_after: Option<usize>,
    pid: u32,
    px: Value,
    audit_counts: Value,
    /// What the game wrote to stderr, and how long each stop gave it to leave.
    stderr: String,
    leaves: Vec<Duration>,
}

struct FakeLink(Rc<RefCell<Script>>);

impl Link for FakeLink {
    fn ask(&mut self, body: &Value) -> Result<Value, String> {
        let mut s = self.0.borrow_mut();
        s.asked.push(body.clone());
        if s.dies_at.is_some_and(|n| s.asked.len() >= n) {
            return Err("the link closed".into());
        }
        let cmd = body["cmd"].as_str().unwrap_or("").to_string();
        if s.refuse.as_deref() == Some(&cmd) {
            return Ok(json!({"ok": false, "error": "no widget"}));
        }
        Ok(match cmd.as_str() {
            "ui_tree" => {
                s.trees += 1;
                if s.stalls_after.is_none_or(|n| s.trees <= n) {
                    s.frame += 1;
                }
                let mut tree = json!({"ok": true, "screen": "menu", "stack": ["menu"], "canvas": [1280, 720], "frame": s.frame, "pid": s.pid, "px": s.px, "widgets": []});
                if s.trees > s.tree_after {
                    if let Some(extra) = s.tree.as_object() {
                        for (k, v) in extra.clone() {
                            tree[k] = v;
                        }
                    }
                }
                tree
            }
            "game" => json!({"gold": 7}),
            "screenshot" => {
                if let Some(path) = body["path"].as_str() {
                    std::fs::write(path, b"png").unwrap();
                }
                json!({"ok": true})
            }
            "text_audit" => json!({"ok": true, "screen": "menu", "counts": s.audit_counts}),
            _ => json!({"ok": true}),
        })
    }
}

struct FakeGame {
    log: Rc<RefCell<Vec<String>>>,
    script: Rc<RefCell<Script>>,
    pid: u32,
    /// How it ended, once the link died and this many looks later (a real process counts as ended a moment after).
    ended: Option<String>,
    late: u32,
    exit: GameExit,
}

impl Process for FakeGame {
    fn pid(&self) -> u32 {
        self.pid
    }
    fn exited(&mut self) -> Option<String> {
        let s = self.script.borrow();
        let dead = s.dies_at.is_some_and(|n| s.asked.len() >= n);
        if dead && self.late > 0 {
            self.late -= 1;
            return None;
        }
        self.ended.clone().filter(|_| dead)
    }
    fn stop(&mut self, leave: Duration) -> GameExit {
        self.log.borrow_mut().push(format!("stop {}", self.pid));
        self.script.borrow_mut().leaves.push(leave);
        self.exit.clone()
    }
    fn stderr(&self) -> String {
        self.script.borrow().stderr.clone()
    }
}

struct FakeLauncher {
    script: Rc<RefCell<Script>>,
    log: Rc<RefCell<Vec<String>>>,
    plans: Vec<Plan>,
    ended: Option<String>,
    refuse: Option<StartFailure>,
    /// The game started has another pid than the link names, for this many starts.
    foreign: u32,
    pid: u32,
    exit: GameExit,
}

impl Launcher for FakeLauncher {
    fn start(&mut self, plan: &Plan) -> Result<(Box<dyn Link>, Box<dyn Process>), StartFailure> {
        self.plans.push(plan.clone());
        if let Some(f) = self.refuse.take() {
            return Err(f);
        }
        self.pid += 1;
        self.log.borrow_mut().push(format!("start {}", self.pid));
        let pid = if self.foreign > 0 {
            self.foreign -= 1;
            self.pid + 1000
        } else {
            self.pid
        };
        self.script.borrow_mut().pid = pid;
        Ok((Box::new(FakeLink(self.script.clone())), Box::new(FakeGame { log: self.log.clone(), script: self.script.clone(), pid: self.pid, ended: self.ended.clone(), late: 3, exit: self.exit.clone() })))
    }
}

fn launcher() -> FakeLauncher {
    let script = Script { px: json!([1280, 720]), audit_counts: json!({"cut": 0, "edge": 0, "overlap": 0, "spill": 0}), ..Script::default() };
    FakeLauncher { script: Rc::new(RefCell::new(script)), log: Rc::default(), plans: Vec::new(), ended: None, refuse: None, foreign: 0, pid: 0, exit: GameExit { code: Some(0), killed: false } }
}

/// A real window: the frames are polled for. `no_window` is the run with none.
fn options(out: &Path) -> Options {
    Options { game_id: "somegame".into(), exe: PathBuf::from("no_such_exe"), data: None, out: out.to_path_buf(), slow: 1, size: None, sized_ms: 0, window: true, fps: None, dpr: None }
}

fn no_window(out: &Path) -> Options {
    Options { window: false, ..options(out) }
}

fn scenario_of(steps: Value) -> Scenario {
    parse("x", &minimal(steps)).unwrap()
}

fn run(steps: Value, l: &mut FakeLauncher, out: &Path) -> Outcome {
    scenario::run_size(&scenario_of(steps), (1280, 720), &options(out), l)
}

fn result_of(o: &Outcome) -> Value {
    serde_json::from_str(&std::fs::read_to_string(o.bundle.join("result.json")).unwrap()).unwrap()
}

/// The requests sent after the driver's start-up, without the frame polls.
fn actions(l: &FakeLauncher) -> Vec<String> {
    l.script.borrow().asked.iter().map(|a| a["cmd"].as_str().unwrap_or("").to_string()).filter(|c| c != "ui_tree").collect()
}

// ---- rule 1: frames, not sleeps ----

#[test]
fn each_action_waits_for_two_more_frames_before_the_next() {
    let out = temp("settle");
    let mut l = launcher();
    let o = run(json!([{"step": "click", "id": "a"}, {"step": "key", "key": "up", "times": 2}, {"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    let s = l.script.borrow();
    let cmds: Vec<(usize, &str)> = s.asked.iter().enumerate().map(|(i, a)| (i, a["cmd"].as_str().unwrap())).collect();
    let acts: Vec<usize> = cmds.iter().filter(|(_, c)| matches!(*c, "click" | "key" | "back")).map(|(i, _)| *i).collect();
    assert_eq!(acts.len(), 4, "{cmds:?}");
    for pair in acts.windows(2) {
        // a tree read right after the action, and two more frames: at least three trees, each one frame on the fake
        let trees = s.asked[pair[0] + 1..pair[1]].iter().filter(|a| a["cmd"] == "ui_tree").count();
        assert!(trees > scenario::SETTLE_FRAMES as usize, "only {trees} trees between actions: {cmds:?}");
    }
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_shot_after_a_wait_waits_for_the_frames_too() {
    let out = temp("shot");
    let wait = json!({"step": "wait", "until": {"path": "screen", "eq": "menu"}});
    // The trees a wait alone takes, from the start: what the shot adds comes on top.
    let mut alone = launcher();
    assert_eq!(run(json!([wait]), &mut alone, &out).code, 0);
    let trees_alone = alone.script.borrow().trees;
    let mut l = launcher();
    let o = run(json!([wait, {"step": "shot", "name": "menu"}]), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    let s = l.script.borrow();
    let at = s.asked.iter().position(|a| a["cmd"] == "screenshot").unwrap();
    let trees_before = s.asked[..at].iter().filter(|a| a["cmd"] == "ui_tree").count();
    assert!(trees_before >= trees_alone + scenario::SETTLE_FRAMES as usize, "{trees_before} trees, {trees_alone} for the wait");
    assert!(o.bundle.join("menu.png").is_file());
    drop(s);
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_tree_with_no_frame_count_fails_the_step() {
    let out = temp("nocount");
    let mut l = launcher();
    {
        let mut s = l.script.borrow_mut();
        s.tree = json!({"frame": null});
        s.tree_after = 3; // the start saw frames
    }
    let o = run(json!([{"step": "assert", "until": {"path": "screen", "eq": "menu"}}]), &mut l, &out);
    assert_eq!(o.code, 1, "{}", o.message);
    assert!(o.message.contains("no frame count"), "{}", o.message);
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_window_that_stops_drawing_fails_the_action_it_was_waiting_on() {
    let out = temp("stall");
    let mut l = launcher();
    l.script.borrow_mut().stalls_after = Some(3);
    let mut s = scenario_of(json!([{"step": "back"}]));
    s.timeout = Duration::from_millis(30);
    let o = scenario::run_size(&s, (1280, 720), &options(&out), &mut l);
    assert_eq!(o.code, 1, "{}", o.message);
    assert!(o.message.contains("drew no frame"), "{}", o.message);
    std::fs::remove_dir_all(&out).unwrap();
}

// ---- rule 2: a failure stops that size and leaves a bundle ----

#[test]
fn a_refused_step_stops_that_size_and_writes_the_bundle() {
    let out = temp("bundle");
    let mut l = launcher();
    {
        let mut s = l.script.borrow_mut();
        s.refuse = Some("click".into());
        s.stderr = "a line from the game".into();
    }
    let o = run(json!([{"step": "assert", "until": {"path": "screen", "eq": "menu"}}, {"step": "click", "id": "gone"}, {"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 1);
    assert_eq!(o.step.as_deref(), Some("2"));
    assert!(o.message.contains("refused") && o.message.contains("no widget"), "{}", o.message);
    assert!(!actions(&l).contains(&"back".to_string()), "nothing runs after the failed step");
    for file in ["result.json", "fail.png", "fail_tree.json", "stderr.txt"] {
        assert!(o.bundle.join(file).is_file(), "{file}");
    }
    assert!(std::fs::read_to_string(o.bundle.join("stderr.txt")).unwrap().contains("a line from the game"));
    let r = result_of(&o);
    assert_eq!((r["result"].clone(), r["exit"].clone(), r["step"].clone()), (json!("fail"), json!(1), json!("2")));
    assert_eq!(r["canvas"], json!([1280, 720]), "the real canvas is on record");
    assert_eq!(r["steps"].as_array().unwrap().len(), 1, "the steps that passed");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_failed_size_does_not_stop_the_next_and_a_second_run_clears_the_first() {
    let out = temp("sizes");
    let mut l = launcher();
    l.script.borrow_mut().refuse = Some("click".into());
    let mut s = scenario_of(json!([{"step": "click", "id": "gone"}]));
    s.sizes = vec![(1280, 720), (1920, 1080)];
    let got = scenario::run_scenario(&s, &options(&out), &mut l);
    assert_eq!(got.iter().map(|o| (o.size, o.code)).collect::<Vec<_>>(), [((1280, 720), 1), ((1920, 1080), 1)]);
    assert!(got[0].bundle.join("fail.png").is_file());
    l.script.borrow_mut().refuse = None;
    let again = scenario::run_scenario(&s, &options(&out), &mut l);
    assert_eq!(again[0].code, 0);
    assert!(!again[0].bundle.join("fail.png").is_file(), "a pass leaves no old failure beside it");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_folder_that_is_no_bundle_is_not_cleared() {
    let out = temp("keep");
    let folder = out.join("x").join("1280x720");
    std::fs::create_dir_all(&folder).unwrap();
    std::fs::write(folder.join("mine.txt"), "keep me").unwrap();
    let mut l = launcher();
    assert_eq!(run(json!([{"step": "back"}]), &mut l, &out).code, 0);
    assert!(folder.join("mine.txt").is_file());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn an_assert_and_a_wait_fail_on_what_was_there() {
    let out = temp("assert");
    let mut l = launcher();
    let o = run(json!([{"step": "assert", "until": {"path": "screen", "eq": "battle"}}]), &mut l, &out);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("screen is \"menu\""), "{}", o.message);
    let mut l = launcher();
    let started = Instant::now();
    let o = run(json!([{"step": "wait", "until": {"widget": "never"}, "timeout_ms": 20}]), &mut l, &out);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("timed out") && o.message.contains("no widget 'never'"), "{}", o.message);
    assert!(started.elapsed() < Duration::from_secs(5));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_wait_polls_until_the_state_comes() {
    let out = temp("wait");
    let mut l = launcher();
    {
        let mut s = l.script.borrow_mut();
        s.tree = json!({"screen": "battle"});
        s.tree_after = 8;
    }
    let o = run(json!([{"step": "wait", "until": {"path": "screen", "eq": "battle"}}, {"step": "assert", "until": {"path": "screen", "eq": "battle"}}]), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(l.script.borrow().trees > 8);
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn slow_stretches_a_wait() {
    let out = temp("slow");
    let mut l = launcher();
    let mut o = options(&out);
    o.slow = 20;
    let s = scenario_of(json!([{"step": "wait", "until": {"widget": "never"}, "timeout_ms": 10}]));
    let started = Instant::now();
    let got = scenario::run_size(&s, (1280, 720), &o, &mut l);
    assert_eq!(got.code, 1);
    assert!(started.elapsed() >= Duration::from_millis(200), "10 ms times 20: {:?}", started.elapsed());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn the_runs_timeout_ends_a_wait_that_asks_for_longer() {
    let out = temp("deadline");
    let mut l = launcher();
    let mut s = scenario_of(json!([{"step": "wait", "until": {"widget": "never"}, "timeout_ms": 600000}]));
    s.timeout = Duration::from_millis(30);
    let started = Instant::now();
    let o = scenario::run_size(&s, (1280, 720), &options(&out), &mut l);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("timeout_s ran out"), "{}", o.message);
    assert!(started.elapsed() < Duration::from_secs(5));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn the_audit_fails_on_the_named_kinds_only() {
    let out = temp("audit");
    let mut l = launcher();
    l.script.borrow_mut().audit_counts = json!({"cut": 2, "edge": 0, "overlap": 0, "spill": 0});
    let steps = json!([{"step": "audit", "scales": [1.0, 1.3], "fail": ["edge"]}]);
    assert_eq!(run(steps, &mut l, &out).code, 0, "a kind not named does not fail");
    assert_eq!(l.script.borrow().asked.iter().filter(|a| a["cmd"] == "text_audit").count(), 2, "one per scale");
    let o = run(json!([{"step": "audit", "scales": [1.0]}]), &mut l, &out);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("cut 2"), "{}", o.message);
    assert!(std::fs::read_dir(&o.bundle).unwrap().flatten().any(|e| e.file_name().to_string_lossy().starts_with("audit_")), "the audit reply is kept");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_repeat_ends_at_until_and_fails_when_its_turns_run_out() {
    let out = temp("repeat");
    let walk = |until: Value| json!([{"step": "repeat", "times": 3, "until": until, "steps": [{"step": "key", "key": "up"}]}]);
    let mut l = launcher();
    let o = run(walk(json!({"path": "screen", "eq": "menu"})), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert_eq!(actions(&l).iter().filter(|c| *c == "key").count(), 0, "already true: no turn runs");
    let mut l = launcher();
    let o = run(walk(json!({"path": "screen", "eq": "battle"})), &mut l, &out);
    assert_eq!(o.code, 1);
    assert_eq!(actions(&l).iter().filter(|c| *c == "key").count(), 3, "bounded");
    assert!(o.message.contains("3 turns"), "{}", o.message);
    let mut l = launcher();
    let o = run(json!([{"step": "repeat", "times": 3, "steps": [{"step": "key", "key": "up"}]}]), &mut l, &out);
    assert_eq!(o.code, 0);
    assert_eq!(actions(&l).iter().filter(|c| *c == "key").count(), 3, "no until: all its turns");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_repeated_step_names_its_turn() {
    let out = temp("turn");
    let mut l = launcher();
    let o = run(json!([{"step": "repeat", "times": 2, "steps": [{"step": "back"}, {"step": "assert", "until": {"widget": "never"}}]}]), &mut l, &out);
    assert_eq!(o.step.as_deref(), Some("1.2#1"));
    std::fs::remove_dir_all(&out).unwrap();
}

// ---- rule 3: port and profile ----

#[test]
fn the_scratch_profile_is_muted_windowed_and_keeps_the_focus_away() {
    let s = scenario_of(json!([{"step": "back"}]));
    let v = scenario::scratch_settings(&s, (1920, 1080));
    assert_eq!(v["mute"], json!(true));
    assert_eq!((v["fullscreen"].clone(), v["borderless"].clone()), (json!(false), json!(false)));
    assert_eq!(v["window_size"], json!("1920x1080"));
    assert_eq!(v["vsync"], json!(false));
    assert_eq!(v["record_playtests"], json!(true));
    assert_eq!(v["start_inactive"], json!(true));
    let mut s = s;
    s.settings = json!({"text_scale": 1.3, "map_zoom": 3}).as_object().unwrap().clone();
    let v = scenario::scratch_settings(&s, (1280, 720));
    assert_eq!((v["text_scale"].clone(), v["map_zoom"].clone(), v["mute"].clone()), (json!(1.3), json!(3), json!(true)), "merged over the profile");
}

#[test]
fn everything_the_game_writes_lies_under_the_scratch_folder() {
    let scratch = Path::new("scratch_root");
    let mut s = scenario_of(json!([{"step": "back"}]));
    s.launch.screen = "runner".into();
    s.launch.scene = Some("e1".into());
    s.launch.seed = Some(9);
    let p = scenario::plan(&s, (1280, 720), "somegame", Path::new("game.exe"), Some(Path::new("copy/data")), 4567, scratch);
    assert_eq!(p.appdata, scratch);
    assert!(p.settings_path.starts_with(scratch) && p.recordings.starts_with(scratch));
    for want in ["port=4567".to_string(), format!("saves={}", scratch.join("saves").display()), "screen=runner".into(), "scene=e1".into(), "seed=9".into(), "data=copy/data".into()] {
        assert!(p.args.contains(&want), "{want} in {:?}", p.args);
    }
    let dir = temp("write");
    let q = scenario::plan(&s, (1280, 720), "somegame", Path::new("game.exe"), None, 4567, &dir);
    scenario::write_scratch(&q).unwrap();
    let written: Value = serde_json::from_str(&std::fs::read_to_string(&q.settings_path).unwrap()).unwrap();
    assert_eq!(written, q.settings);
    assert!(dir.join("saves").is_dir());
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The driver writes where the game reads: the one place the two name the user folder.
#[test]
fn the_scratch_user_folder_is_where_the_engine_looks() {
    let appdata = std::env::var("APPDATA").map(PathBuf::from).unwrap_or_else(|_| PathBuf::from("."));
    assert_eq!(scenario::user_dir_under(&appdata, "somegame"), aegis_save::user_dir("somegame"));
}

#[test]
fn each_start_gets_a_free_port_of_its_own_and_a_real_one_is_never_used() {
    let out = temp("ports");
    let mut l = launcher();
    let mut s = scenario_of(json!([{"step": "back"}]));
    s.sizes = vec![(1280, 720), (1920, 1080)];
    assert!(scenario::run_scenario(&s, &options(&out), &mut l).iter().all(|o| o.code == 0));
    assert_eq!(l.plans.len(), 2);
    assert!(l.plans.iter().all(|p| p.port != 0 && p.args[0] == format!("port={}", p.port)));
    assert!(l.plans.iter().all(|p| p.appdata.starts_with(std::env::temp_dir())));
    assert_ne!(l.plans[0].appdata, l.plans[1].appdata, "each start its own folder");
    assert!(l.plans.iter().all(|p| !p.appdata.exists()), "scratch is gone after");
    // a port the OS gave is really free: this binds it
    let port = scenario::free_port().unwrap();
    drop(std::net::TcpListener::bind(("127.0.0.1", port)).unwrap());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_port_answering_for_another_process_is_left_alone_and_another_is_tried() {
    let out = temp("foreign");
    let mut l = launcher();
    l.foreign = 1;
    let o = run(json!([{"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert_eq!(l.plans.len(), 2, "a second try");
    let s = l.script.borrow();
    let quits: Vec<usize> = s.asked.iter().enumerate().filter(|(_, a)| a["cmd"] == "quit").map(|(i, _)| i).collect();
    let back = s.asked.iter().position(|a| a["cmd"] == "back").unwrap();
    assert_eq!(quits.len(), 1, "only its own game is told to quit: {:?}", s.asked);
    assert!(quits[0] > back);
    drop(s);
    assert_eq!(*l.log.borrow(), ["start 1", "stop 1", "start 2", "stop 2"], "and the process it did start is stopped");
    let mut l = launcher();
    l.foreign = scenario::PORT_TRIES;
    let o = run(json!([{"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 2, "no port of its own: could not start");
    assert_eq!(l.plans.len(), scenario::PORT_TRIES as usize);
    assert!(o.message.contains("answers for process"), "{}", o.message);
    assert!(!l.script.borrow().asked.iter().any(|a| a["cmd"] == "quit"));
    std::fs::remove_dir_all(&out).unwrap();
}

// ---- rule 4: stopping ----

#[test]
fn the_driver_stops_what_it_started_once_on_pass_and_fail_and_timeout() {
    let out = temp("stop");
    let cases: Vec<(Value, i32)> = vec![
        (json!([{"step": "back"}]), 0),
        (json!([{"step": "assert", "until": {"widget": "never"}}]), 1),
        (json!([{"step": "wait", "until": {"widget": "never"}, "timeout_ms": 5}]), 1),
    ];
    for (steps, code) in cases {
        let mut l = launcher();
        let o = run(steps, &mut l, &out);
        assert_eq!(o.code, code, "{}", o.message);
        assert_eq!(*l.log.borrow(), ["start 1", "stop 1"]);
        let s = l.script.borrow();
        assert_eq!(s.asked.iter().filter(|a| a["cmd"] == "quit").count(), 1, "the clean stop comes first");
        assert_eq!(s.asked.last().unwrap()["cmd"], "quit");
    }
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_game_that_ends_mid_run_fails_with_its_stderr_and_no_picture() {
    let out = temp("gone");
    let mut l = launcher();
    l.ended = Some("exit code 101; stderr: panicked at the fake".into());
    // The trees of the start and the click pass; the link dies at the tree after it.
    let mut before = launcher();
    assert_eq!(run(json!([{"step": "assert", "until": {"widget": "never"}}]), &mut before, &out).code, 1);
    let start_asks = before.script.borrow().asked.iter().position(|a| a["cmd"] != "ui_tree").unwrap() - 1;
    l.script.borrow_mut().dies_at = Some(start_asks + 2);
    let o = run(json!([{"step": "click", "id": "a"}, {"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 1, "{}", o.message);
    assert!(o.message.contains("the game ended mid-run") && o.message.contains("panicked at the fake"), "{}", o.message);
    assert!(!o.bundle.join("fail.png").exists());
    assert!(o.bundle.join("stderr.txt").is_file() && o.bundle.join("result.json").is_file());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_minimized_window_fails_the_run_rather_than_drawing_nothing() {
    let out = temp("min");
    // A real one reports [1, 1]; either side too small is enough.
    for px in [json!([1, 1]), json!([1, 720]), json!([1280, 1])] {
        let mut l = launcher();
        {
            let mut s = l.script.borrow_mut();
            s.tree = json!({"px": px});
            s.tree_after = 8;
        }
        let o = run(json!([{"step": "wait", "until": {"widget": "never"}, "timeout_ms": 5000}]), &mut l, &out);
        assert_eq!(o.code, 1, "{px}");
        assert!(o.message.contains("minimized"), "{px}: {}", o.message);
    }
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_clean_exit_is_on_record_and_a_bad_one_fails_a_run_whose_steps_passed() {
    let out = temp("exit");
    let steps = json!([{"step": "back"}]);
    let mut l = launcher();
    let o = run(steps.clone(), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert_eq!(result_of(&o)["game_exit"], json!({"code": 0, "killed": false, "panicked": false}));
    let cases = [
        (GameExit { code: Some(101), killed: false }, "", "exited with code 101"),
        (GameExit { code: None, killed: false }, "", "exited with code unknown"),
        (GameExit { code: Some(1), killed: true }, "", "did not leave"),
        (GameExit { code: Some(0), killed: false }, "thread 'audio' panicked at a.rs:1:1", "panicked at a.rs"),
    ];
    for (exit, stderr, wanted) in cases {
        let mut l = launcher();
        l.exit = exit.clone();
        l.script.borrow_mut().stderr = stderr.into();
        let o = run(steps.clone(), &mut l, &out);
        assert_eq!((o.code, o.step.as_deref()), (1, Some("stop")), "{exit:?}");
        assert!(o.message.contains(wanted), "{exit:?}: {}", o.message);
        let r = result_of(&o);
        assert_eq!((r["result"].clone(), r["game_exit"]["code"].clone(), r["game_exit"]["killed"].clone()), (json!("fail"), json!(exit.code), json!(exit.killed)));
    }
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_failed_step_keeps_its_own_message_when_the_exit_is_bad_too() {
    let out = temp("exit2");
    let mut l = launcher();
    l.exit = GameExit { code: Some(101), killed: false };
    let o = run(json!([{"step": "assert", "until": {"widget": "never"}}]), &mut l, &out);
    assert_eq!((o.code, o.step.as_deref()), (1, Some("1")));
    assert!(o.message.contains("no widget 'never'"), "{}", o.message);
    assert_eq!(result_of(&o)["game_exit"]["code"], json!(101));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn slow_stretches_the_time_a_game_has_to_leave() {
    let out = temp("leave");
    let mut l = launcher();
    let mut o = options(&out);
    o.slow = 3;
    assert_eq!(scenario::run_size(&scenario_of(json!([{"step": "back"}])), (1280, 720), &o, &mut l).code, 0);
    assert_eq!(l.script.borrow().leaves, [scenario::LEAVE * 3]);
    std::fs::remove_dir_all(&out).unwrap();
}

// ---- rule 5: exit codes ----

#[test]
fn exit_codes_say_pass_failed_or_could_not_start() {
    let out = temp("codes");
    let mut l = launcher();
    assert_eq!(run(json!([{"step": "back"}]), &mut l, &out).code, 0);
    let mut l = launcher();
    assert_eq!(run(json!([{"step": "assert", "until": {"widget": "never"}}]), &mut l, &out).code, 1);
    let mut l = launcher();
    l.refuse = Some(StartFailure { reason: "no program".into(), retry: false });
    let o = run(json!([{"step": "back"}]), &mut l, &out);
    assert_eq!(o.code, 2);
    assert_eq!(l.plans.len(), 1, "no second try when another port cannot help");
    let r = result_of(&o);
    assert_eq!((r["result"].clone(), r["message"].clone()), (json!("could not start"), json!("no program")));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_missing_program_could_not_start() {
    let out = temp("noexe");
    let mut real = scenario::GameLauncher { slow: 1 };
    let o = scenario::run_size(&scenario_of(json!([{"step": "back"}])), (1280, 720), &options(&out), &mut real);
    assert_eq!(o.code, 2);
    assert!(o.message.contains("no game program"), "{}", o.message);
    assert_eq!(result_of(&o)["result"], json!("could not start"));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_waiting_scenario_parses_and_never_runs() {
    let out = temp("waiting");
    let s = parse("x", &json!({"id": "x", "status": "waiting", "why": "needs its data", "steps": [{"step": "back"}]})).unwrap();
    assert_eq!(s.status, Status::Waiting("needs its data".into()));
    let mut l = launcher();
    assert!(scenario::run_scenario(&s, &options(&out), &mut l).is_empty());
    assert!(l.plans.is_empty());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn size_on_the_command_line_replaces_the_files_sizes() {
    let out = temp("size");
    let mut l = launcher();
    let mut s = scenario_of(json!([{"step": "back"}]));
    s.sizes = vec![(1280, 720), (1920, 1080)];
    let mut o = options(&out);
    o.size = Some((1600, 900));
    let got = scenario::run_scenario(&s, &o, &mut l);
    assert_eq!(got.len(), 1);
    assert_eq!(got[0].size, (1600, 900));
    assert_eq!(l.plans[0].settings["window_size"], json!("1600x900"));
    // a size the window did not reach (capped to the screen) is on record, and the run still counts
    assert_eq!(result_of(&got[0])["capped"], json!(true));
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_recording_in_the_scratch_folder_becomes_the_trail_report() {
    struct WithRecording(FakeLauncher);
    impl Launcher for WithRecording {
        fn start(&mut self, plan: &Plan) -> Result<(Box<dyn Link>, Box<dyn Process>), StartFailure> {
            let rec = aegis_host::playtest::Recording::new("r1", "playtest", Value::Null);
            rec.save(&plan.recordings).unwrap();
            self.0.start(plan)
        }
    }
    let out = temp("trail");
    let mut l = WithRecording(launcher());
    let s = scenario_of(json!([{"step": "back"}]));
    let o = scenario::run_size(&s, (1280, 720), &options(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(o.bundle.join("trail.json").is_file() && o.bundle.join("recording.json").is_file());
    assert!(result_of(&o)["trail"].is_object(), "the counts are in the result");
    std::fs::remove_dir_all(&out).unwrap();
}

// ---- the real link on a local socket ----

#[test]
fn the_tcp_link_answers_and_gives_up_on_silence_and_on_a_close() {
    use std::io::{BufRead, BufReader, Write};
    let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
    let port = listener.local_addr().unwrap().port();
    let server = std::thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        let mut writer = stream.try_clone().unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        let body: Value = serde_json::from_str(&line).unwrap();
        writeln!(writer, "{}", json!({"ok": true, "echo": body["cmd"]})).unwrap();
        // the second request is read and not answered, then the link closes
        line.clear();
        reader.read_line(&mut line).unwrap();
        std::thread::sleep(Duration::from_millis(150));
    });
    let mut link = scenario::TcpLink::connect(port, Duration::from_secs(5)).unwrap();
    assert_eq!(link.ask(&json!({"cmd": "ping"})).unwrap()["echo"], json!("ping"));
    link.patience(Duration::from_millis(30));
    assert!(link.ask(&json!({"cmd": "hang"})).unwrap_err().contains("no reply"));
    server.join().unwrap();
    link.patience(Duration::from_secs(2));
    assert!(link.ask(&json!({"cmd": "late"})).is_err(), "a closed link fails rather than hangs");
}

// ---- HEADLESS1: no window unless asked ----

#[test]
fn a_run_opens_no_window_unless_the_file_or_the_flag_asks() {
    let out = temp("modes");
    let steps = json!([{"step": "back"}]);
    let mut l = launcher();
    let o = scenario::run_size(&scenario_of(steps.clone()), (1920, 1080), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(l.plans[0].args.contains(&"headless=1920x1080".to_string()), "{:?}", l.plans[0].args);
    assert_eq!(result_of(&o)["window"], json!(false));

    // The file asks.
    let mut wanted = minimal(steps.clone());
    wanted["window"] = json!(true);
    let mut l = launcher();
    let o = scenario::run_size(&parse("x", &wanted).unwrap(), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(!l.plans[0].args.iter().any(|a| a.starts_with("headless=")), "{:?}", l.plans[0].args);
    assert_eq!(result_of(&o)["window"], json!(true));
    assert!(!actions(&l).contains(&"frames".to_string()), "a window's frames come by themselves");

    // The flag asks (`options` is what window=1 gives).
    let mut l = launcher();
    let o = run(steps.clone(), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(!l.plans[0].args.iter().any(|a| a.starts_with("headless=")));
    assert!(parse("x", &{
        let mut bad = minimal(steps);
        bad["window"] = json!("yes");
        bad
    })
    .is_err());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn the_rate_and_the_device_scale_reach_a_game_with_no_window_alone() {
    let out = temp("rate");
    let o = Options { fps: Some(30), dpr: Some("1.5".into()), ..no_window(&out) };
    assert_eq!(scenario::headless_args((1280, 720), &o), ["headless=1280x720", "fps=30", "dpr=1.5"]);
    assert_eq!(scenario::headless_args((1280, 720), &no_window(&out)), ["headless=1280x720"]);
    let mut l = launcher();
    assert_eq!(scenario::run_size(&scenario_of(json!([{"step": "back"}])), (1280, 720), &o, &mut l).code, 0);
    assert!(l.plans[0].args.contains(&"fps=30".to_string()));
    std::fs::remove_dir_all(&out).unwrap();
}

/// With no window frames are asked for: after each action, and before a shot, so the picture is whole with no waiting.
#[test]
fn with_no_window_each_action_and_shot_asks_for_its_frames() {
    let out = temp("steps");
    let mut l = launcher();
    let steps = json!([{"step": "click", "id": "a"}, {"step": "shot", "name": "menu"}]);
    let o = scenario::run_size(&scenario_of(steps), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    assert_eq!(actions(&l), ["click", "frames", "frames", "screenshot", "quit"]);
    let s = l.script.borrow();
    assert!(!s.asked.iter().any(|a| a["cmd"] == "step"), "a game's battle has its own step, which plays a turn");
    let asked = s.asked.iter().find(|a| a["cmd"] == "frames").unwrap();
    assert_eq!(asked["frames"], json!(scenario::SETTLE_FRAMES));
    drop(s);
    // A game that refuses them fails the step, rather than a shot of the frame before.
    let mut l = launcher();
    l.script.borrow_mut().refuse = Some("frames".into());
    let o = scenario::run_size(&scenario_of(json!([{"step": "click", "id": "a"}])), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 1);
    assert_eq!(o.step.as_deref(), Some("1"));
    std::fs::remove_dir_all(&out).unwrap();
}

/// The game refuses a blank frame (`aegis_ui::shell::save_png`): the step fails, and the run says so.
#[test]
fn a_shot_the_game_refuses_fails_the_step() {
    let out = temp("blank");
    let mut l = launcher();
    l.script.borrow_mut().refuse = Some("screenshot".into());
    let o = scenario::run_size(&scenario_of(json!([{"step": "shot", "name": "menu"}])), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 1, "{}", o.message);
    assert!(o.message.contains("the shot menu"), "{}", o.message);
    assert!(!o.bundle.join("menu.png").exists());
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_clip_goes_into_the_bundle_needs_no_window_and_is_bounded() {
    let out = temp("clip");
    let mut l = launcher();
    let steps = json!([{"step": "clip", "name": "fight", "frames": 5, "every_ms": 100, "max_w": 640}]);
    let o = scenario::run_size(&scenario_of(steps.clone()), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    let s = l.script.borrow();
    let asked = s.asked.iter().find(|a| a["cmd"] == "clip").unwrap();
    assert_eq!((asked["frames"].clone(), asked["every_ms"].clone(), asked["max_w"].clone()), (json!(5), json!(100), json!(640)));
    assert_eq!(asked["dir"], json!(o.bundle.join("fight").display().to_string()));
    drop(s);
    let mut l = launcher();
    let o = run(steps, &mut l, &out);
    assert_eq!(o.code, 1, "a window has no clip");
    assert!(o.message.contains("no window"), "{}", o.message);
    for bad in [
        json!({"step": "clip"}),
        json!({"step": "clip", "name": "a/b"}),
        json!({"step": "clip", "name": "c", "frames": 121}),
        json!({"step": "clip", "name": "c", "every_ms": 1}),
        json!({"step": "clip", "name": "c", "frames": 120, "every_ms": 2000}),
        json!({"step": "pause"}),
        json!({"step": "pause", "ms": 0}),
    ] {
        assert!(parse("x", &minimal(json!([bad]))).is_err(), "{bad}");
    }
    std::fs::remove_dir_all(&out).unwrap();
}

/// With no window a game leaves when nothing asks for a while, so a pause goes on asking.
#[test]
fn a_long_pause_keeps_asking() {
    let out = temp("pause_asks");
    let mut l = launcher();
    let ms = scenario::PAUSE_LOOK.as_millis() as u64 + 200;
    let o = scenario::run_size(&scenario_of(json!([{"step": "pause", "ms": 60}])), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    let short = l.script.borrow().trees;
    let mut l = launcher();
    let o = scenario::run_size(&scenario_of(json!([{"step": "pause", "ms": ms}])), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(l.script.borrow().trees > short, "no look in {ms} ms");
    // A game that died during it is noticed then, not at the end.
    let mut l = launcher();
    l.script.borrow_mut().dies_at = Some(3);
    l.ended = Some("exit code: 1".into());
    let o = scenario::run_size(&scenario_of(json!([{"step": "pause", "ms": ms}])), (1280, 720), &no_window(&out), &mut l);
    assert_ne!(o.code, 0);
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_clip_stays_inside_the_runs_timeout() {
    let out = temp("clip_time");
    let mut v = minimal(json!([{"step": "clip", "name": "c", "frames": 120, "every_ms": 100}]));
    v["timeout_s"] = json!(1);
    let mut l = launcher();
    let o = scenario::run_size(&parse("x", &v).unwrap(), (1280, 720), &no_window(&out), &mut l);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("timeout_s"), "{}", o.message);
    assert!(!actions(&l).contains(&"clip".to_string()), "never asked");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_file_that_cannot_run_as_written_is_a_problem() {
    let mut v = minimal(json!([{"step": "repeat", "times": 2, "steps": [{"step": "clip", "name": "c"}]}]));
    v["window"] = json!(true);
    assert!(parse("x", &v).unwrap_err().iter().any(|p| p.contains("clip")), "a clip needs no window, even inside a repeat");
    let mut v = minimal(json!([{"step": "back"}]));
    v["launch"] = json!({"screen": "runner", "rules": "r1"});
    assert!(parse("x", &v).unwrap_err().iter().any(|p| p.contains("rules")));
    v["launch"] = json!({"rules": "r1"});
    assert!(parse("x", &v).is_err(), "the default screen is not the arena");
    v["launch"] = json!({"screen": "runner", "ruleset": "r1"});
    assert!(parse("x", &v).unwrap_err().iter().any(|p| p.contains("rules")), "the old name is held to the same rule");
}

#[test]
fn a_run_opens_no_window_unless_the_key_says_so() {
    assert!(!scenario::window_asked(&keys(&[])).unwrap());
    assert!(!scenario::window_asked(&keys(&["window=0"])).unwrap());
    assert!(scenario::window_asked(&keys(&["window=1"])).unwrap());
    assert!(scenario::window_asked(&keys(&["window=2"])).is_err());
}

#[test]
fn a_pause_waits_and_never_past_the_runs_timeout() {
    let out = temp("pause");
    let mut l = launcher();
    let t0 = std::time::Instant::now();
    let o = run(json!([{"step": "pause", "ms": 60}]), &mut l, &out);
    assert_eq!(o.code, 0, "{}", o.message);
    assert!(t0.elapsed() >= Duration::from_millis(60));
    let mut short = minimal(json!([{"step": "pause", "ms": 5000}]));
    short["timeout_s"] = json!(1);
    let mut l = launcher();
    let t0 = std::time::Instant::now();
    let o = scenario::run_size(&parse("x", &short).unwrap(), (1280, 720), &options(&out), &mut l);
    assert_eq!(o.code, 1);
    assert!(o.message.contains("timeout_s"), "{}", o.message);
    assert!(t0.elapsed() < Duration::from_secs(4), "it did not sleep through");
    std::fs::remove_dir_all(&out).unwrap();
}

#[test]
fn a_launch_names_its_rules() {
    let mut v = minimal(json!([{"step": "back"}]));
    v["launch"] = json!({"screen": "arena", "scene": "e1", "rules": "r1"});
    let p = scenario::plan(&parse("x", &v).unwrap(), (1280, 720), "somegame", Path::new("game.exe"), None, 4567, Path::new("scratch_root"));
    assert!(p.args.contains(&"rules=r1".to_string()) && p.args.contains(&"scene=e1".to_string()), "{:?}", p.args);
    v["launch"]["rules"] = json!(3);
    assert!(parse("x", &v).is_err());
}

/// LAYER-ENG-KEYS (PR5-29a): a file with the launch keys' old names loads as the new ones, and the game is launched
/// with the new keys either way; a launch giving both names of one key is refused.
#[test]
fn the_old_launch_key_names_still_load() {
    let mut v = minimal(json!([{"step": "back"}]));
    v["launch"] = json!({"screen": "arena", "scene": "e1", "rules": "r1", "seed": 3});
    let new = parse("x", &v).unwrap();
    v["launch"] = json!({"screen": "arena", "encounter": "e1", "ruleset": "r1", "seed": 3});
    let old = parse("x", &v).unwrap();
    assert_eq!(old.launch, new.launch);
    assert_eq!((new.launch.scene.as_deref(), new.launch.rules.as_deref()), (Some("e1"), Some("r1")));
    let p = scenario::plan(&old, (1280, 720), "somegame", Path::new("game.exe"), None, 4567, Path::new("scratch_root"));
    assert!(p.args.contains(&"scene=e1".to_string()) && p.args.contains(&"rules=r1".to_string()), "{:?}", p.args);
    assert!(!p.args.iter().any(|a| a.starts_with("encounter=") || a.starts_with("ruleset=")), "the game hears only the new keys: {:?}", p.args);
    for (key, old) in scenario::LAUNCH_ALIASES {
        v["launch"] = json!({"screen": "arena", key: "a", old: "b"});
        let problems = parse("x", &v).unwrap_err();
        assert!(problems.iter().any(|p| p.contains(key) && p.contains(old) && p.contains("both")), "{key}/{old}: {problems:?}");
    }
    v["launch"] = json!({"screen": "arena", "fight": "e1"});
    assert!(parse("x", &v).unwrap_err().iter().any(|p| p.contains("unknown key 'fight'")));
}

// ---- the proof helper ----

fn keys(raw: &[&str]) -> aegis_cli::Args {
    aegis_cli::Args::parse(&raw.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn proof_shots_turn_a_short_list_into_steps() {
    let s = scenario::shots_scenario("menu", &keys(&[])).unwrap();
    assert_eq!(s.launch.screen, "menu");
    assert_eq!(s.steps, parse("menu", &json!({"id": "menu", "steps": [{"step": "shot", "name": "shot"}]})).unwrap().steps, "one shot when none is named");
    assert!(!s.window);

    let s = scenario::shots_scenario("fight", &keys(&["screen=arena", "scene=e1", "rules=r1", "seed=4", "do=screen:a_b,shot:start,key:a,widget:w,pause:50,click:c,hover:h,clip:play:6:100"])).unwrap();
    assert_eq!((s.launch.screen.as_str(), s.launch.scene.as_deref(), s.launch.rules.as_deref(), s.launch.seed), ("arena", Some("e1"), Some("r1"), Some(4)));
    // LAYER-ENG-KEYS: the old key names work as `shots` keys too; both names of one key are refused.
    let old = scenario::shots_scenario("fight", &keys(&["screen=arena", "encounter=e1", "ruleset=r1", "seed=4"])).unwrap();
    assert_eq!((old.launch.scene.as_deref(), old.launch.rules.as_deref()), (Some("e1"), Some("r1")));
    assert!(scenario::shots_scenario("fight", &keys(&["screen=arena", "scene=e1", "encounter=e2"])).is_err());
    assert!(scenario::shots_scenario("fight", &keys(&["screen=arena", "rules=r1", "ruleset=r2"])).is_err());
    let kinds = json!([
        {"step": "wait", "until": {"path": "screen", "eq": "a_b"}}, {"step": "shot", "name": "start"}, {"step": "key", "key": "a"}, {"step": "wait", "until": {"widget": "w"}},
        {"step": "pause", "ms": 50}, {"step": "click", "id": "c"}, {"step": "hover", "id": "h"}, {"step": "clip", "name": "play", "frames": 6, "every_ms": 100}
    ]);
    assert_eq!(s.steps, parse("fight", &json!({"id": "fight", "steps": kinds})).unwrap().steps);
    // A clip stands for a shot too.
    assert_eq!(scenario::do_steps("clip:c").unwrap().len(), 1);

    for bad in [vec!["do=key"], vec!["do=jump:up"], vec!["do=pause:soon"], vec!["seed=x"], vec!["do=clip:c:many"], vec!["screen=nowhere"], vec!["do=shot:a/b"]] {
        assert!(scenario::shots_scenario("x", &keys(&bad)).is_err(), "{bad:?}");
    }
    assert!(scenario::shots_scenario("a/b", &keys(&[])).is_err(), "the name becomes a folder");
}
