//! WEBREL1-E1b (`docs/specs/WEBREL1.md` 2.2): the shell's settings file goes through the user store (`aegis_save::store`)
//! like the saves: on the web store it lands at the key `aegis/<game id>/settings.json`, reads back over the defaults,
//! and a write past the quota is `save`'s error, not a panic.

use aegis_save::store::{with_store, KeyBackend, Keys, MemKeys, WEB_ROOT};
use aegis_ui::screen::Settings;
use serde_json::json;
use std::path::Path;
use std::rc::Rc;

#[test]
fn settings_save_and_load_through_the_web_store() {
    let s = Rc::new(Keys::new(MemKeys::with_quota(0)));
    let path = Path::new(WEB_ROOT).join("demo").join("settings.json");
    with_store(s.clone(), || {
        let mut set = Settings::load(path.clone(), json!({}));
        set.v["mute"] = json!(true);
        set.save().unwrap();
        assert_eq!(Settings::load(path.clone(), json!({})).v["mute"], json!(true), "read back over the defaults");
    });
    let text = s.backend.get("aegis/demo/settings.json").expect("the settings key");
    assert_eq!(serde_json::from_str::<serde_json::Value>(&text).unwrap()["mute"], json!(true));
}

#[test]
fn a_settings_write_past_the_quota_is_an_error() {
    let s = Rc::new(Keys::new(MemKeys::with_quota(16)));
    let path = Path::new(WEB_ROOT).join("demo").join("settings.json");
    let saved = with_store(s.clone(), || Settings::load(path, json!({})).save());
    assert!(saved.is_err_and(|e| e.contains("quota")), "the store's reason");
    assert!(s.backend.keys().is_empty(), "nothing written");
}
