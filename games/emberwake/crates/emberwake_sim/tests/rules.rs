//! The sim's own rules (BAL2_SIM.md), checked without the Godot recordings: argument errors, target bands, the sweep
//! compositions, the campaign heal and a smoke run of three modes on the shipped data (no METRIC FAIL, no PLAN FAIL).

use emberwake_sim::args::SimArgs;
use emberwake_sim::campaign::{apply_heal, State};
use emberwake_sim::party::{compositions, sorted_key};
use emberwake_sim::runner::Runner;
use emberwake_sim::targets;
use emberwake_world::Data;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

fn argv(line: &str) -> Vec<String> {
    line.split_whitespace().map(String::from).collect()
}

#[test]
fn argument_errors_name_the_problem() {
    let d = data();
    let err = |line: &str| SimArgs::parse(&d, &argv(line)).err().unwrap_or_default();
    assert_eq!(err("duel 5 1 bad=1"), "unknown key 'bad'");
    assert_eq!(err("fly"), "unknown mode 'fly'");
    assert_eq!(err("duel 99"), "level must be 1..10 (got '99')");
    assert_eq!(err("stats nope"), "level must be 1..10 or all (got 'nope')");
    assert_eq!(err("duel 5 0"), "seeds must be a positive number (got '0')");
    assert_eq!(err("duel 5 1 cap=31"), "cap must be 1..30 (got '31')");
    assert!(err("party 5 1 comp=warrior,knight").starts_with("comp: unknown class 'knight'"));
    assert!(err("party 5 1 enc=nowhere").starts_with("enc: unknown encounter 'nowhere'"));
    assert!(err("duel 5 1 target=duel:inf-90").starts_with("target must be"), "inf is no number");
    let a = SimArgs::parse(&d, &argv("party 3")).unwrap();
    assert_eq!((a.level, a.seed_count, a.metrics), (3, 10, true), "party: 10 seeds and metrics by default");
    let a = SimArgs::parse(&d, &argv("role quick")).unwrap();
    assert_eq!((a.distance, a.seed_count), (4, 1), "role: distance 4; quick: one seed");
}

#[test]
fn target_bands() {
    let mut t = targets::defaults();
    assert!(targets::apply_override(&mut t, "swap_delta:-5-5"), "a leading minus belongs to lo");
    assert_eq!(t["swap_delta"], (-5.0, 5.0));
    assert_eq!(targets::band_text(&t, "swap_delta"), "+-5");
    assert_eq!(targets::band_text(&t, "clear_line"), "70-90");
    assert_eq!(targets::band_text(&t, "ranged_dist"), ">=2.5");
    assert_eq!(targets::band_text(&t, "aoe_allies_per_cast"), "<=0.3");
    assert_eq!(targets::band_text(&t, "tank_survival"), ">=60", "up to 100 % reads as at least");
    assert!(!targets::apply_override(&mut t, "nope:1-2"));
    assert!(!targets::apply_override(&mut t, "duel:70-30"), "lo above hi");
    assert_eq!(targets::mark(&t, "clear_line", 69.0), " X");
    assert_eq!(targets::mark(&t, "clear_line", 70.0), "");
}

#[test]
fn sweep_compositions() {
    let d = data();
    let count = |line: &str| {
        let a = SimArgs::parse(&d, &argv(line)).unwrap();
        compositions(&Runner::new(&d, &a))
    };
    assert_eq!(count("sweep").len(), 15, "4 of 6 classes");
    assert_eq!(count("sweep dupes=on").len(), 126, "multisets of 4 from 6");
    let trios = count("sweep size=3 lead=warrior");
    assert_eq!(trios.len(), 10, "the warrior plus 2 of the other 5");
    assert!(trios.iter().all(|c| c[0] == "warrior" && c.iter().filter(|x| *x == "warrior").count() == 1));
    let sample = count("sweep dupes=on sample=7");
    assert_eq!(sample.len(), 7);
    assert_eq!(sample, count("sweep dupes=on sample=7"), "the draw is seeded");
    assert_eq!(sorted_key(&["mage".into(), "warrior".into(), "priest".into()]), "warrior,priest,mage", "class order");
}

#[test]
fn campaign_heal_between_battles() {
    let state = || -> State { [("a".to_string(), (10, 40)), ("b".to_string(), (0, 30))].into_iter().collect() };
    let mut s = state();
    apply_heal(&mut s, "full");
    assert_eq!((s["a"], s["b"]), ((40, 40), (30, 30)), "full revives");
    let mut s = state();
    apply_heal(&mut s, "25");
    assert_eq!((s["a"], s["b"]), ((20, 40), (0, 30)), "a percent of max HP rounds up and skips the downed");
    let mut s = state();
    apply_heal(&mut s, "none");
    assert_eq!(s, state());
    let mut s = state();
    apply_heal(&mut s, "rally");
    assert!(s["b"].0 > 0 && s["a"].0 > 10, "rally: a downed hero gets up, the others heal");
}

#[test]
fn modes_run_clean_on_the_shipped_data() {
    let d = data();
    for line in ["duel 1 1", "party 1 1 enc=mob", "role 1 quick role=tank class=warrior"] {
        let out = emberwake_sim::run(&d, &argv(line));
        assert!(out[0].starts_with("ARGS mode "), "{line}: the header first");
        let checks = out.iter().find(|l| l.starts_with("metric checks: ")).unwrap_or_else(|| panic!("{line}: no check line in {out:?}"));
        assert!(checks.ends_with(", 0 METRIC FAIL"), "{line}: {checks}");
        assert!(out.iter().any(|l| l.starts_with("plan checks: 0 PLAN FAIL")), "{line}: {out:?}");
    }
}
