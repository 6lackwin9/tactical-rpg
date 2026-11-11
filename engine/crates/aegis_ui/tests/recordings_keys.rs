//! LAYER-ENG-API (PR5-29c): the recordings browser names a recording's sessions genre-free. The session picker's old
//! widget id `recordings.battle` is read as `recordings.session` (`ID_ALIASES`, options included); a session's scene
//! is its first request's `scene`, else a command set's `encounter` (`SCENE_KEYS`); the drawn words stay as the pane's
//! pixel hashes pin them (`SESSION_LABELS`). The link side (a click on the old id) is the shell's
//! `the_session_picker_answers_its_new_and_old_ids`.

use aegis_host::playtest::Session;
use aegis_ui::recordings::{canonical_id, session_scene, ID_ALIASES, SCENE_KEYS, SESSION_LABELS};
use serde_json::{json, Value};

fn session(first: Value) -> Session {
    Session { requests: vec![first], checkpoints: Vec::new(), end: Value::Null }
}

#[test]
fn an_old_widget_id_and_its_options_read_as_the_new_id() {
    for (old, new) in ID_ALIASES {
        assert_eq!(canonical_id(old).as_deref(), Some(new));
        assert_eq!(canonical_id(&format!("{old}.opt.2")), Some(format!("{new}.opt.2")), "an option of the old picker");
        assert_eq!(canonical_id(new), None, "the new id is no alias");
        assert_eq!(canonical_id(&format!("{old}s")), None, "only the whole id or a dotted child");
    }
    for id in ["recordings.round", "recordings.play", "recordings.session.opt.0", ""] {
        assert_eq!(canonical_id(id), None, "{id}");
    }
}

#[test]
fn a_session_names_its_scene_by_scene_then_the_old_key() {
    assert_eq!(SCENE_KEYS[0], "scene", "the engine's own key comes first");
    for key in SCENE_KEYS {
        assert_eq!(session_scene(&session(json!({"cmd": "new", key: "ambush"}))), "ambush", "{key}");
    }
    let both: Value = SCENE_KEYS.iter().enumerate().map(|(i, k)| (k.to_string(), json!(format!("s{i}")))).collect::<serde_json::Map<_, _>>().into();
    assert_eq!(session_scene(&session(both)), "s0", "the first key found wins");
    assert_eq!(session_scene(&session(json!({"cmd": "new"}))), "?");
    assert_eq!(session_scene(&Session { requests: Vec::new(), checkpoints: Vec::new(), end: Value::Null }), "?");
}

/// The words are the pane's drawn text from before the rename; `theme_panes` pins their pixels, so a change here is a
/// theme re-record, named.
#[test]
fn the_drawn_session_words_are_the_pinned_ones() {
    assert_eq!((SESSION_LABELS.header, SESSION_LABELS.count), ("BATTLE", "battles"));
}
