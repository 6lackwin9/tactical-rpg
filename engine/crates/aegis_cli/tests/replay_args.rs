//! LAYER-ENG-KEYS (PR5-29a): `aegis replay` names a recording's session with `session=K`; the old `battle=K` still
//! works, and both at once is refused.

use aegis_cli::{replay_session_arg, Args};

fn keys(raw: &[&str]) -> Args {
    Args::parse(&raw.iter().map(|s| s.to_string()).collect::<Vec<_>>())
}

#[test]
fn a_replay_session_is_named_by_the_new_key_or_the_old_one() {
    assert_eq!(replay_session_arg(&keys(&["rec.json"])).unwrap(), None);
    assert_eq!(replay_session_arg(&keys(&["rec.json", "session=2"])).unwrap(), Some(2));
    assert_eq!(replay_session_arg(&keys(&["rec.json", "battle=2"])).unwrap(), Some(2));
    assert!(replay_session_arg(&keys(&["rec.json", "session=1", "battle=2"])).is_err());
    assert!(replay_session_arg(&keys(&["rec.json", "session=x"])).is_err());
}
