//! FX1-L1 (decision "Attack timing on the board" A): a clips.json row may mark its `impact` frame; the hit is timed at
//! the sum of the frames' ms before it. A row without one parses as before.

use aegis_core::assets::Clips;
use serde_json::json;

fn body(attack: serde_json::Value) -> serde_json::Value {
    json!({"frame": [48, 48], "palette": {}, "clips": [{"id": "idle", "frames": 2, "ms": [100, 100]}, attack]})
}

#[test]
fn an_impact_frame_is_read_and_timed_by_the_frames_before_it() {
    let ms = [110u32, 160, 45, 140, 110, 110];
    let c = Clips::parse(&body(json!({"id": "attack", "frames": 6, "ms": ms, "loop": false, "impact": 2}))).unwrap();
    let a = c.clip("attack").unwrap();
    assert_eq!(a.impact, Some(2));
    // The rule: the sum of the durations of the frames before the impact frame.
    let want: u64 = ms[..2].iter().map(|m| *m as u64).sum();
    assert_eq!(a.impact_ms(), Some(want));
    assert_eq!(a.ms_before(0), 0);
    assert_eq!(a.ms_before(6), a.total_ms(), "past the last frame: the whole clip");
}

#[test]
fn a_row_without_an_impact_has_none() {
    let c = Clips::parse(&body(json!({"id": "attack", "frames": 3, "ms": [100, 100, 100], "loop": false}))).unwrap();
    assert_eq!(c.clip("attack").unwrap().impact, None);
    assert_eq!(c.clip("attack").unwrap().impact_ms(), None);
    assert_eq!(c.clip("idle").unwrap().impact, None);
}

#[test]
fn an_impact_outside_the_clip_is_refused() {
    for bad in [json!(3), json!(-1), json!("2"), json!(1.5)] {
        let e = Clips::parse(&body(json!({"id": "attack", "frames": 3, "ms": [100, 100, 100], "loop": false, "impact": bad}))).unwrap_err();
        assert_eq!(e, vec!["clip 'attack': impact must be 0 to 2.".to_string()], "{bad}");
    }
    assert!(Clips::parse(&body(json!({"id": "attack", "frames": 3, "ms": [100, 100, 100], "loop": false, "impact": 2}))).is_ok(), "the last frame is allowed");
}
