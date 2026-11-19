//! ENGINE-17 (SHOT-CLOCK1): the shot clock. Its own test binary: stepping is for the whole process.

use aegis_core::clock::{advance, step_on, stepped, Duration, Instant};

#[test]
fn stepped_time_moves_only_by_advance_and_wall_stays_real() {
    let before = Instant::now();
    assert!(!stepped());
    advance(Duration::from_secs(5));
    assert!(before.elapsed() < Duration::from_secs(5), "advance does nothing on the real clock");
    step_on();
    assert!(stepped());
    let a = Instant::now();
    let w = Instant::wall();
    std::thread::sleep(Duration::from_millis(20));
    assert_eq!(Instant::now(), a, "stepped time stands still");
    assert_eq!(a.elapsed(), Duration::ZERO);
    assert!(w.wall_elapsed() >= Duration::from_millis(20), "the wall clock still runs");
    advance(Duration::from_millis(66));
    assert_eq!(Instant::now() - a, Duration::from_millis(66));
    assert_eq!(a.elapsed(), Duration::from_millis(66));
    step_on();
    assert_eq!(Instant::now() - a, Duration::from_millis(66), "a second step_on changes nothing");
    let later = a + Duration::from_secs(1);
    assert_eq!(later.saturating_duration_since(a), Duration::from_secs(1));
    assert_eq!(a.saturating_duration_since(later), Duration::ZERO);
    assert_eq!(a.checked_duration_since(later), None);
    assert_eq!(later - Duration::from_secs(1), a);
    // The date is the shot clock's too: from STEP_EPOCH_S on, moving only with advance.
    let date = aegis_core::clock::system_now();
    let s = date.duration_since(aegis_core::clock::UNIX_EPOCH).unwrap().as_secs();
    assert!((aegis_core::clock::STEP_EPOCH_S..aegis_core::clock::STEP_EPOCH_S + 60).contains(&s), "{s}");
    std::thread::sleep(Duration::from_millis(5));
    assert_eq!(aegis_core::clock::system_now(), date);
    advance(Duration::from_secs(2));
    assert_eq!(aegis_core::clock::system_now().duration_since(date).unwrap(), Duration::from_secs(2));
    let raw: std::time::Instant = a.into();
    assert_eq!(Instant::from(raw), a);
}
