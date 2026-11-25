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

/// WEBREL1-G1: the web build's audio arrives as a later bundle part, while the game already runs. A sound looked up
/// before it came is remembered as missing; after the next mount (`vfs::mount`, which the web start calls per part)
/// and the shell's tick it is looked up again and found. Natively the read seam reads the disk, so the file appears on
/// disk here and the mount only counts the generation.
#[test]
fn a_later_bundle_part_brings_the_sounds_remembered_missing() {
    let dir = aegis_core::scratch::Scratch::new("audio_late_part");
    let a = Audio::offline(Some(dir.to_path_buf()));
    a.music(Some("music/theme.ogg"));
    assert!(!a.has("sfx/hit.wav"), "not there yet");
    std::fs::create_dir_all(dir.join("sfx")).unwrap();
    std::fs::write(dir.join("sfx/hit.wav"), b"RIFF").unwrap();
    a.tick();
    assert!(!a.has("sfx/hit.wav"), "remembered as missing until a part is mounted");
    aegis_core::vfs::mount(aegis_core::vfs::Table::new());
    a.tick();
    assert!(a.has("sfx/hit.wav"), "looked up again after the mount");
    assert_eq!(a.wanted_music().as_deref(), Some("music/theme.ogg"), "the wanted track is still wanted");
}
