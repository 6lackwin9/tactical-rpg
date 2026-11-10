//! WEBREL1-E1b (`docs/specs/WEBREL1.md` 2.1): the read seam. Native reads pass straight to `std::fs` (same bytes, same
//! error kinds, the same entries as `read_dir`); the web `Table` (an in-memory path -> bytes map with derived folders)
//! answers the same six questions, tested here natively.

use aegis_core::scratch::Scratch;
use aegis_core::vfs::{self, key, Table};
use std::collections::BTreeSet;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

#[test]
fn native_reads_are_std_fs_reads() {
    let dir = Scratch::new("vfs_native");
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    std::fs::write(dir.join("a.json"), "{\"a\": 1}").unwrap();
    std::fs::write(dir.join("sub/b.bin"), [0u8, 159, 146, 150]).unwrap();
    assert_eq!(vfs::read(dir.join("sub/b.bin")).unwrap(), std::fs::read(dir.join("sub/b.bin")).unwrap());
    assert_eq!(vfs::read_to_string(dir.join("a.json")).unwrap(), "{\"a\": 1}");
    assert_eq!(vfs::read_to_string(dir.join("sub/b.bin")).unwrap_err().kind(), std::fs::read_to_string(dir.join("sub/b.bin")).unwrap_err().kind(), "not UTF-8");
    assert_eq!(vfs::read(dir.join("none")).unwrap_err().kind(), ErrorKind::NotFound);
    assert!(vfs::is_file(dir.join("a.json")) && !vfs::is_file(dir.join("sub")) && !vfs::is_file(dir.join("none")));
    assert!(vfs::is_dir(dir.join("sub")) && !vfs::is_dir(dir.join("a.json")));
    assert!(vfs::exists(dir.join("sub")) && vfs::exists(dir.join("a.json")) && !vfs::exists(dir.join("none")));
    let listed: BTreeSet<PathBuf> = vfs::list_dir(&*dir).unwrap().into_iter().collect();
    let std_listed: BTreeSet<PathBuf> = std::fs::read_dir(&*dir).unwrap().flatten().map(|e| e.path()).collect();
    assert_eq!(listed, std_listed, "the entries read_dir gives, as full paths");
    assert_eq!(vfs::list_dir(dir.join("none")).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_table_key_is_the_normalised_path() {
    assert_eq!(key(Path::new("data/catalog/units.json")), "data/catalog/units.json");
    assert_eq!(key(Path::new("./data/./worlds/../catalog/units.json")), "data/catalog/units.json");
    assert_eq!(key(Path::new("/game/assets/ui/theme.json")), "/game/assets/ui/theme.json");
    assert_eq!(key(&Path::new("game").join("assets").join("a.png")), "game/assets/a.png");
}

fn table() -> Table {
    let mut t = Table::new();
    t.insert("game/game.json", b"{\"kit\": \"k\"}".to_vec());
    t.insert("game/assets/ui/theme.json", b"{}".to_vec());
    t.insert("game/assets/ui/frames/panel.png", vec![137, 80, 78, 71]);
    t.insert("game/assets/ui/frames/panel.json", b"{}".to_vec());
    t.insert("game/assets/units/a/base/clips.json", b"{}".to_vec());
    t
}

#[test]
fn the_web_table_answers_reads_probes_and_listings() {
    let t = table();
    assert_eq!(t.len(), 5);
    assert_eq!(t.read(Path::new("game/assets/ui/frames/panel.png")).unwrap(), vec![137, 80, 78, 71]);
    assert_eq!(t.read_to_string(Path::new("game/./assets/ui/theme.json")).unwrap(), "{}");
    assert_eq!(t.read(Path::new("game/assets/none.png")).unwrap_err().kind(), ErrorKind::NotFound);
    assert_eq!(t.read_to_string(Path::new("game/assets/ui/frames/panel.png")).unwrap_err().kind(), ErrorKind::InvalidData);
    assert!(t.is_file(Path::new("game/game.json")) && !t.is_file(Path::new("game/assets")));
    assert!(t.is_dir(Path::new("game/assets")) && t.is_dir(Path::new("game/assets/units/a")) && t.is_dir(Path::new("")));
    assert!(!t.is_dir(Path::new("game/game.json")) && !t.is_dir(Path::new("game/asset")), "a name prefix is no folder");
    assert!(t.exists(Path::new("game/assets/ui")) && !t.exists(Path::new("game/data")));
    let ui = Path::new("game/assets/ui");
    assert_eq!(t.list_dir(ui).unwrap(), vec![ui.join("frames"), ui.join("theme.json")], "sorted, a folder once");
    assert_eq!(t.list_dir(Path::new("game")).unwrap(), vec![Path::new("game").join("assets"), Path::new("game").join("game.json")]);
    assert_eq!(t.list_dir(Path::new("game/data")).unwrap_err().kind(), ErrorKind::NotFound);
}

#[test]
fn a_later_bundle_part_extends_the_table() {
    let mut t = table();
    let mut audio = Table::new();
    audio.insert("game/assets/audio/hit.ogg", vec![1, 2, 3]);
    audio.insert("game/game.json", b"{}".to_vec());
    t.extend(audio);
    assert_eq!(t.len(), 6);
    assert!(t.is_dir(Path::new("game/assets/audio")));
    assert_eq!(t.read_to_string(Path::new("game/game.json")).unwrap(), "{}", "a path in both takes the later part's bytes");
    assert!(!Table::new().is_dir(Path::new("")) && Table::new().is_empty(), "an empty table has no root");
    // native reads never look at a mounted table
    vfs::mount(t);
    assert!(!vfs::is_file("game/assets/audio/hit.ogg"));
}
