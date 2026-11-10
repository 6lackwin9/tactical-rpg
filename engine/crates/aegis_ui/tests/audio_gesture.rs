//! Audio on the first gesture (WEBREL1-E2, `Audio::unlock`): the shell calls `unlock` on every key or button press;
//! only the web build waits for it to open the device (checked in the browser by `tools/web/web_min_check.sh`). Here:
//! what holds on every target. An offline audio (no device wanted) stays silent and `unlock` changes nothing, and the
//! music wanted before the gesture is still the wanted track after it (music is a state, not a queued sound).

use aegis_ui::audio::Audio;

#[test]
fn offline_audio_stays_silent_through_unlock() {
    let mut a = Audio::offline(None);
    assert!(!a.live(), "offline audio has no device");
    a.unlock();
    a.unlock();
    assert!(!a.live(), "unlock opens nothing for an audio that wants no device");
}

#[test]
fn wanted_music_survives_the_gesture() {
    let mut a = Audio::offline(None);
    a.music(Some("music/theme.ogg"));
    a.unlock();
    assert_eq!(a.wanted_music().as_deref(), Some("music/theme.ogg"));
    a.music(None);
    a.unlock();
    assert_eq!(a.wanted_music(), None);
}
