//! FPSCAP1: the frame pacing rule, with no window: the cap holds under any wake rate, unlimited runs free, a screen that
//! has not said it is at rest keeps drawing, and a resting one still gets input, the agent link and its heartbeat.

use aegis_ui::pacing::{pace, FrameNeed, Pace, Pacer, HEARTBEAT, SETTLE};
use std::time::{Duration, Instant};

const CAP_60: Duration = Duration::from_micros(1_000_000 / 60);

/// Runs a loop for `span` that wakes every `wake` (the events of a flood), with each frame taking no time; the frames drawn.
fn frames_over(span: Duration, wake: Duration, cap: Option<Duration>, need: FrameNeed) -> u32 {
    let t0 = Instant::now();
    let mut pacer = Pacer::default();
    let (mut now, mut frames) = (t0, 0);
    while now < t0 + span {
        if pacer.decide(now, cap) == Pace::Redraw {
            pacer.framed(now, need);
            frames += 1;
        }
        now += wake;
    }
    frames
}

#[test]
fn the_cap_redraws_only_when_the_last_frame_is_old_enough() {
    let t = Instant::now();
    assert_eq!(pace(Some(t), t + CAP_60 / 2, Some(CAP_60), t), Pace::WaitUntil(t + CAP_60), "too soon: wait for the deadline");
    assert_eq!(pace(Some(t), t + CAP_60, Some(CAP_60), t), Pace::Redraw, "exactly old enough");
    assert_eq!(pace(Some(t), t + CAP_60 * 3, Some(CAP_60), t), Pace::Redraw, "long overdue");
    assert_eq!(pace(None, t, Some(CAP_60), t), Pace::Redraw, "the first frame waits for nothing");
}

#[test]
fn the_deadline_counts_from_the_last_frame_not_from_the_wake() {
    let t = Instant::now();
    // Whenever a wake comes, the wait names the same moment; a wait from `now` would slide with every event.
    let deadlines: Vec<Pace> = [1u32, 5, 10].iter().map(|ms| pace(Some(t), t + Duration::from_millis(*ms as u64), Some(CAP_60), t)).collect();
    assert!(deadlines.iter().all(|d| *d == Pace::WaitUntil(t + CAP_60)), "{deadlines:?}");
}

#[test]
fn a_flood_of_wakes_does_not_break_the_cap() {
    let second = Duration::from_secs(1);
    for (fps, cap) in [(30u32, Duration::from_micros(1_000_000 / 30)), (60, CAP_60)] {
        let frames = frames_over(second, Duration::from_micros(100), Some(cap), FrameNeed::Full);
        assert!(frames <= fps + 1, "cap {fps}: {frames} frames in a second");
        assert!(frames >= fps - fps / 10, "the cap also delivers: cap {fps}, {frames} frames");
    }
}

#[test]
fn unlimited_draws_at_every_wake() {
    let frames = frames_over(Duration::from_secs(1), Duration::from_millis(1), None, FrameNeed::Full);
    assert_eq!(frames, 1000);
}

#[test]
fn a_full_screen_never_rests() {
    // Longer than the settle and the heartbeat together, with a wake rate slower than the settle: still a frame each wake.
    let wake = SETTLE / 2;
    let frames = frames_over(HEARTBEAT * 3, wake, None, FrameNeed::Full);
    assert_eq!(frames, (HEARTBEAT * 3).as_micros() as u32 / wake.as_micros() as u32);
}

#[test]
fn an_idle_screen_rests_after_the_settle_until_the_heartbeat() {
    let t = Instant::now();
    let mut pacer = Pacer::default();
    assert_eq!(pacer.decide(t, None), Pace::Redraw, "the first frame comes unasked");
    pacer.framed(t, FrameNeed::Idle);
    assert_eq!(pacer.decide(t, None), Pace::WaitUntil(t + HEARTBEAT), "nothing to show");
    assert_eq!(pacer.decide(t + HEARTBEAT, None), Pace::Redraw, "the heartbeat still comes");
}

#[test]
fn input_wakes_a_resting_screen_for_the_settle_and_the_cap_still_holds() {
    let t = Instant::now();
    let mut pacer = Pacer::default();
    pacer.framed(t, FrameNeed::Idle);
    let at = t + Duration::from_millis(100);
    pacer.touch(at);
    assert_eq!(pacer.decide(at, None), Pace::Redraw, "input is shown at once");
    assert_eq!(pacer.decide(at + SETTLE - Duration::from_millis(1), None), Pace::Redraw, "and for the settle");
    assert_eq!(pacer.decide(at + SETTLE, None), Pace::WaitUntil(t + HEARTBEAT), "then it rests");
    // The last frame is what the cap counts from, however much input came since.
    pacer.framed(at, FrameNeed::Idle);
    pacer.touch(at + Duration::from_millis(1));
    assert_eq!(pacer.decide(at + Duration::from_millis(1), Some(CAP_60)), Pace::WaitUntil(at + CAP_60));
}

#[test]
fn a_screen_within_a_time_gets_a_frame_at_that_time_and_no_later_than_the_heartbeat() {
    let t = Instant::now();
    let soon = Duration::from_millis(66);
    let mut pacer = Pacer::default();
    pacer.framed(t, FrameNeed::Within(soon));
    assert_eq!(pacer.decide(t, None), Pace::WaitUntil(t + soon));
    assert_eq!(pacer.decide(t + soon, None), Pace::Redraw);
    pacer.framed(t, FrameNeed::Within(HEARTBEAT * 60));
    assert_eq!(pacer.decide(t, None), Pace::WaitUntil(t + HEARTBEAT), "the heartbeat limits any promise");
    // The cap wins over a shorter promise.
    pacer.framed(t, FrameNeed::Within(Duration::from_millis(1)));
    assert_eq!(pacer.decide(t + Duration::from_millis(1), Some(CAP_60)), Pace::WaitUntil(t + CAP_60));
}

#[test]
fn a_full_frame_after_rest_starts_the_settle_again() {
    let t = Instant::now();
    let mut pacer = Pacer::default();
    pacer.framed(t, FrameNeed::Idle);
    pacer.framed(t + HEARTBEAT, FrameNeed::Full);
    assert_eq!(pacer.decide(t + HEARTBEAT + SETTLE / 2, None), Pace::Redraw, "the screen stirred by itself at the heartbeat");
}

#[test]
fn a_slow_full_frame_after_rest_does_not_sleep_on_the_old_heartbeat() {
    let t = Instant::now();
    let mut pacer = Pacer::default();
    pacer.framed(t, FrameNeed::Idle);
    let full = t + Duration::from_millis(10);
    pacer.framed(full, FrameNeed::Full);
    // Its first frame took longer than the settle: the screen still wants frames, and must not wait for the menu's heartbeat
    assert_eq!(pacer.decide(full + SETTLE + Duration::from_millis(1), None), Pace::Redraw);
}

#[test]
fn the_screens_drawn_together_share_the_most_demanding_need() {
    let (a, b) = (Duration::from_millis(30), Duration::from_millis(60));
    let needs = [FrameNeed::Full, FrameNeed::Within(a), FrameNeed::Within(b), FrameNeed::Idle];
    for x in needs {
        assert_eq!(x.and(FrameNeed::Full), FrameNeed::Full);
        assert_eq!(FrameNeed::Full.and(x), FrameNeed::Full);
        assert_eq!(x.and(FrameNeed::Idle), x);
        assert_eq!(FrameNeed::Idle.and(x), x);
    }
    assert_eq!(FrameNeed::Within(a).and(FrameNeed::Within(b)), FrameNeed::Within(a));
    assert_eq!(FrameNeed::Within(b).and(FrameNeed::Within(a)), FrameNeed::Within(a));
}

/// A clock taking at most 100 ms a frame, as a battle's does, stays on wall time at the lowest headless rate with
/// each wake 16 ms late; at 10 fps it loses time, which is why that rate is refused. No real clock is read.
#[test]
fn the_lowest_headless_rate_keeps_a_100_ms_clock_on_wall_time() {
    fn clock_lost(fps: u32) -> Duration {
        let every = Duration::from_micros(1_000_000 / u64::from(fps));
        let (t0, mut pacer, mut lost) = (Instant::now(), Pacer::default(), Duration::ZERO);
        let (mut now, mut last) = (t0, t0);
        pacer.framed(t0, FrameNeed::Full);
        for _ in 0..1000 {
            match pacer.decide(now, Some(every)) {
                Pace::Redraw => {
                    let gap = now - last;
                    lost += gap.saturating_sub(Duration::from_millis(100));
                    last = now;
                    pacer.framed(now, FrameNeed::Full);
                }
                Pace::WaitUntil(at) => now = at + Duration::from_millis(16),
            }
        }
        lost
    }
    assert_eq!(clock_lost(aegis_ui::shell::headless::FPS_MIN), Duration::ZERO);
    assert!(clock_lost(10) > Duration::ZERO, "the test can see a lost clock");
}
