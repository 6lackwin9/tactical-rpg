//! ENGINE-62 (CRASH-SAFE1): the atomic write keeps each slot's previous readable save as `<file>.prev`; a damaged slot
//! is kept, named (`damage`) and read from that copy instead; deleting a slot takes its copy too.

use aegis_save::{damage, delete, peek_slot, prev_path, read, write, Format, Peek};
use serde_json::json;

const FMT: Format = Format { tag: "prev_test", version: 1, first_slot: 0, last_slot: 2, keys: &["gold"] };

fn gold(text: &str) -> i64 {
    aegis_save::open(&FMT, text).unwrap()["game_state"]["gold"].as_i64().unwrap()
}

#[test]
fn a_truncated_save_is_kept_and_reads_its_previous_copy() {
    let dir = aegis_core::scratch::Scratch::vacant("save_prev_copy");
    let path = aegis_save::path_for(&dir, 1);
    assert!(write(&FMT, &dir, 1, 0, || Ok(json!({"gold": 10}))).is_empty());
    assert!(!prev_path(&path).exists(), "a first save has nothing to keep");
    assert!(write(&FMT, &dir, 1, 0, || Ok(json!({"gold": 20}))).is_empty());
    assert_eq!(gold(&std::fs::read_to_string(prev_path(&path)).unwrap()), 10, "the save it replaced");
    assert_eq!(gold(&read(&dir, 1).unwrap()), 20);
    assert_eq!(damage(&FMT, &dir, 1), None);
    // Cut short (a crash mid-write outside the engine, a full disk): kept, named, and its copy reads instead.
    let text = std::fs::read_to_string(&path).unwrap();
    std::fs::write(&path, &text[..text.len() / 2]).unwrap();
    assert_eq!(damage(&FMT, &dir, 1).as_deref(), Some("the file is cut short or garbled"));
    assert_eq!(gold(&read(&dir, 1).unwrap()), 10, "the previous copy");
    match peek_slot(&FMT, &dir, 1) {
        (Peek::Found(e), Some(_)) => assert_eq!(e["game_state"]["gold"], json!(10)),
        _ => panic!("the header is the copy's, with the damage"),
    }
    assert!(path.exists(), "a damaged save is never deleted by a read");
    // A damaged save is not copied over the good copy by the next write.
    assert!(write(&FMT, &dir, 1, 0, || Ok(json!({"gold": 30}))).is_empty());
    assert_eq!(gold(&std::fs::read_to_string(prev_path(&path)).unwrap()), 10);
    // Another game's file in the slot is damage too.
    std::fs::write(&path, r#"{"format": "other", "version": 1}"#).unwrap();
    assert!(damage(&FMT, &dir, 1).unwrap().contains("format 'other'"));
    // The player's delete takes the copy too.
    delete(&dir, 1).unwrap();
    assert!(!path.exists() && !prev_path(&path).exists());
    assert!(matches!(peek_slot(&FMT, &dir, 1), (Peek::Missing, None)));
}

#[test]
fn a_rescue_save_is_keyed_beside_the_slots_and_goes_when_deleted() {
    let dir = aegis_core::scratch::Scratch::vacant("save_prev_rescue");
    assert!(aegis_save::write_keyed(&FMT, &dir, aegis_save::RESCUE_KEY, 0, || Ok(json!({"gold": 5}))).is_empty());
    assert!(aegis_save::exists_keyed(&dir, aegis_save::RESCUE_KEY));
    assert!((FMT.first_slot..=FMT.last_slot).all(|s| !aegis_save::exists(&dir, s)), "never a player's slot");
    aegis_save::delete_keyed(&dir, aegis_save::RESCUE_KEY).unwrap();
    assert!(!aegis_save::exists_keyed(&dir, aegis_save::RESCUE_KEY));
}
