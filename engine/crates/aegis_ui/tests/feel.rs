//! ENGINE-45 (FEEL-PROBE1): the feel probe's sums (`aegis_ui::feel`), on times given here: an input is done by the first
//! frame that started after it arrived, its handler time is its own, and the report flags what the row's bars name.

use aegis_ui::feel::{is_stall, log_line, report, Probe, Sample, Stall, FRAME_FLAG_MS, INPUT_FLAG_MS};
use serde_json::json;

fn on() -> Probe {
    let mut p = Probe::default();
    p.on = true;
    p
}

#[test]
fn an_input_is_done_by_the_first_frame_that_started_after_it() {
    let mut p = on();
    p.input("key right", 10.0);
    p.state(12.5);
    // a frame already running when it came does not show it
    p.frame(9.0, 20.0);
    assert!(p.last.is_none(), "{:?}", p.last);
    p.frame(24.0, 30.0);
    let s = p.last.clone().unwrap();
    assert_eq!((s.kind.as_str(), s.to_state, s.to_frame, s.to_present, s.frame), ("key right", 2.5, 14.0, 20.0, 6.0));
    assert_eq!(s.spent_in(), "wait", "most of it waited for the frame");
}

#[test]
fn two_inputs_before_one_frame_both_end_with_it() {
    let mut p = on();
    p.input("click a", 0.0);
    p.state(1.0);
    p.input("click b", 2.0);
    p.state(40.0);
    p.frame(41.0, 45.0);
    let (samples, frames) = p.take();
    assert_eq!(samples.iter().map(|s| (s.kind.as_str(), s.to_state, s.to_present)).collect::<Vec<_>>(), vec![("click a", 1.0, 45.0), ("click b", 38.0, 43.0)]);
    assert_eq!(samples[1].spent_in(), "handler");
    assert_eq!(frames, vec![4.0]);
    assert!(p.take().0.is_empty(), "take clears");
}

#[test]
fn off_it_notes_nothing() {
    let mut p = Probe::default();
    p.input("key up", 0.0);
    p.state(1.0);
    p.frame(2.0, 3.0);
    assert!(p.last.is_none() && p.take().1.is_empty());
}

#[test]
fn the_overlay_reads_the_last_frame_the_worst_of_a_second_and_the_last_input() {
    let mut p = on();
    p.frame(0.0, 40.0);
    p.input("pad a", 1500.0);
    p.frame(1600.0, 1612.0);
    p.frame(2000.0, 2005.0);
    assert_eq!(p.worst_recent(2005.0), 12.0, "the 40 ms frame is over a second old");
    assert_eq!(p.overlay_text(2005.0), "frame 5.0 ms  worst 1s 12.0 ms  input 112.0 ms (pad a)");
    assert_eq!(Probe::default().overlay_text(0.0), "frame 0.0 ms  worst 1s 0.0 ms  input -");
}

#[test]
fn the_report_and_the_log_flag_slow_inputs_and_frames() {
    let fast = Sample { kind: "key up".into(), to_state: 0.5, to_frame: 3.0, to_present: 8.0, frame: 5.0, tick: 1.0, draw: 3.0, stall: false };
    let slow = Sample { kind: "click battle.confirm".into(), to_state: 90.0, to_frame: 91.0, to_present: INPUT_FLAG_MS + 20.0, frame: 29.0, tick: 2.0, draw: 25.0, stall: false };
    let r = report(&[fast.clone(), slow.clone()], &[5.0, FRAME_FLAG_MS + 1.0, 6.0]);
    assert_eq!(r["to_present"]["worst"], serde_json::json!(INPUT_FLAG_MS + 20.0));
    assert_eq!(r["to_state"]["p50"], serde_json::json!(0.5));
    assert_eq!(r["frames_over"], serde_json::json!(1));
    let flagged = r["flagged_inputs"].as_array().unwrap();
    assert_eq!((flagged.len(), flagged[0]["input"].clone(), flagged[0]["spent_in"].clone()), (1, serde_json::json!("click battle.confirm"), serde_json::json!("handler")));
    assert!(log_line(1234.0, &slow).ends_with("\thandler\tOVER") && log_line(1234.0, &slow).starts_with("1.234\tclick battle.confirm"));
    assert!(!log_line(0.0, &fast).contains("OVER"));
    assert!(log_line(0.0, &fast).contains("frame 5.0 ms (tick 1.0, draw 3.0)"), "{}", log_line(0.0, &fast));
}

/// ENGINE-60 (PLAN-START2): a frame over the flag that ran on the CPU for under half its wall time is a stall: kept apart,
/// in no statistic, and the input it showed is listed as stalled; a slow frame that ran is still a frame, and a frame
/// with no CPU time given is never a stall.
#[test]
fn a_frame_mostly_off_the_cpu_is_a_stall_kept_out_of_the_report() {
    assert!(is_stall(FRAME_FLAG_MS + 7.0, 10.0) && !is_stall(FRAME_FLAG_MS + 7.0, 30.0) && !is_stall(20.0, 1.0));
    let mut p = on();
    p.input("click a", 0.0);
    p.cpu = Some(10.0);
    p.frame(1.0, 41.0);
    p.cpu = Some(38.0);
    p.frame(50.0, 90.0);
    p.frame(100.0, 140.0);
    p.frame(150.0, 160.0);
    let (samples, frames) = p.take();
    assert_eq!(frames, vec![40.0, 40.0, 10.0], "the stall is in no frame time; a slow frame that ran, or with no CPU time, is");
    assert!(samples[0].stall);
    assert_eq!(p.take_stalls(), vec![Stall { start: 1.0, end: 41.0, ms: 40.0, cpu: 10.0 }]);
    assert!(p.take_stalls().is_empty(), "taken");
    let r = report(&samples, &frames);
    assert_eq!(r["to_present"]["count"], json!(0), "the stalled input is in no statistic");
    assert_eq!(r["inputs"][0]["stall"], json!(true), "but listed");
}

/// ENGINE-60: an underrun heard during a stalled frame, or within two output buffers after it, is the stall's.
#[test]
fn underruns_during_a_stall_are_the_stalls() {
    let audio = json!({"buffer_ms": 10.0, "underrun_ms": [5.0, 60.0, 200.0]});
    // the replay started 100 probe ms in; the stall ran from 140 to 150
    let stalls = vec![Stall { start: 140.0, end: 150.0, ms: 40.0, cpu: 5.0 }];
    let (stalled, real) = aegis_ui::shell::headless::stalled_underruns(&audio, &stalls, 100.0);
    assert_eq!((stalled, real), (vec![60.0], vec![5.0, 200.0]));
}

/// ENGINE-61: a late mix that ran mostly off the audio thread's CPU is a stall; one too slow for its buffer is not.
#[test]
fn a_late_mix_mostly_off_the_cpu_is_a_stall() {
    use aegis_ui::audio_out::mix_stalled;
    assert!(mix_stalled(30.0, 1.0), "the thread stopped");
    assert!(!mix_stalled(30.0, 25.0), "the mix itself was too slow");
}
