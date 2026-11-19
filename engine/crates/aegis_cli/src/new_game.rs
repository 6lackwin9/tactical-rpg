//! NEW-GAME1 (ENGINE-6): `aegis new game <id>`: a new game folder on the engine alone, from the templates in
//! `templates/new_game/` (compiled in). The game runs, has a menu and a play screen, a command host, a test file and a
//! scenario, and joins the workspace by the `games/*/crates/*` member glob. Placeholders: `__ID__` (the id),
//! `__TITLE__` (the title), `__TYPE__` (the id in upper camel case, the context type).

use anyhow::{bail, Context, Result};
use std::path::{Path, PathBuf};

/// (template path, output path with placeholders, text).
const FILES: &[(&str, &str, &str)] = &[
    ("game.json", "game.json", include_str!("../templates/new_game/game.json")),
    ("README.md", "README.md", include_str!("../templates/new_game/README.md")),
    ("gitignore", ".gitignore", include_str!("../templates/new_game/gitignore")),
    ("data/README.md", "data/README.md", include_str!("../templates/new_game/data/README.md")),
    ("assets/keep", "assets/.gitkeep", include_str!("../templates/new_game/assets/keep")),
    ("assets/strings/en.json", "assets/strings/en.json", include_str!("../templates/new_game/assets/strings/en.json")),
    ("scenarios/start.json", "scenarios/__ID___start.json", include_str!("../templates/new_game/scenarios/start.json")),
    ("crates/game/Cargo.toml.tmpl", "crates/__ID___game/Cargo.toml", include_str!("../templates/new_game/crates/game/Cargo.toml.tmpl")),
    ("crates/game/src/lib.rs", "crates/__ID___game/src/lib.rs", include_str!("../templates/new_game/crates/game/src/lib.rs")),
    ("crates/game/src/main.rs", "crates/__ID___game/src/main.rs", include_str!("../templates/new_game/crates/game/src/main.rs")),
    ("crates/game/src/screens.rs", "crates/__ID___game/src/screens.rs", include_str!("../templates/new_game/crates/game/src/screens.rs")),
    ("crates/game/src/bin/cli.rs", "crates/__ID___game/src/bin/__ID___cli.rs", include_str!("../templates/new_game/crates/game/src/bin/cli.rs")),
    ("crates/game/tests/game.rs", "crates/__ID___game/tests/__ID__.rs", include_str!("../templates/new_game/crates/game/tests/game.rs")),
];

/// Why `id` cannot name a game, if it cannot: 2 to 32 of `a-z`, `0-9` and `_`, starting with a letter, not ending in
/// `_`, and not a name the workspace already uses for a crate suffix.
pub fn id_problem(id: &str) -> Option<String> {
    let ok_chars = id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !(2..=32).contains(&id.len()) || !ok_chars || !id.starts_with(|c: char| c.is_ascii_lowercase()) || id.ends_with('_') || id.contains("__") {
        return Some(format!("'{id}': a game id is 2 to 32 of a-z, 0-9 and _, starting with a letter (e.g. star_forge)"));
    }
    if ["aegis", "rpgkit", "engine", "kit", "kits", "games", "test", "core"].contains(&id) {
        return Some(format!("'{id}' is a name the engine or the workspace uses"));
    }
    None
}

/// `star_forge` -> `StarForge`.
pub fn type_name(id: &str) -> String {
    id.split('_').filter(|p| !p.is_empty()).map(|p| p[..1].to_ascii_uppercase() + &p[1..]).collect()
}

/// `star_forge` -> `Star Forge`.
pub fn default_title(id: &str) -> String {
    id.split('_').filter(|p| !p.is_empty()).map(|p| p[..1].to_ascii_uppercase() + &p[1..]).collect::<Vec<_>>().join(" ")
}

fn fill(text: &str, id: &str, title: &str) -> String {
    text.replace("__ID__", id).replace("__TITLE__", title).replace("__TYPE__", &type_name(id))
}

/// The files of game `id` titled `title`: (path under the game folder, text).
pub fn files(id: &str, title: &str) -> Vec<(PathBuf, String)> {
    FILES.iter().map(|(_, out, text)| (PathBuf::from(fill(out, id, title)), fill(text, id, title))).collect()
}

/// Writes game `id` into `<root>/games/<id>/`, refusing a bad id, a title with a quote or backslash, a root that is not
/// the workspace, or a folder that exists. Returns the files written.
pub fn create(root: &Path, id: &str, title: &str) -> Result<Vec<PathBuf>> {
    if let Some(p) = id_problem(id) {
        bail!(p);
    }
    if title.trim().is_empty() || title.contains(['"', '\\', '\n']) {
        bail!("the title is one line with no quote or backslash");
    }
    if !root.join("Cargo.toml").is_file() || !root.join("engine/crates").is_dir() {
        bail!("{} is not the workspace root (no Cargo.toml and engine/crates/); pass root=<dir>", root.display());
    }
    let dir = root.join("games").join(id);
    if dir.exists() {
        bail!("{} exists already", dir.display());
    }
    let mut written = Vec::new();
    for (rel, text) in files(id, title) {
        let path = dir.join(&rel);
        std::fs::create_dir_all(path.parent().unwrap_or(&dir)).with_context(|| format!("creating {}", path.display()))?;
        std::fs::write(&path, text).with_context(|| format!("writing {}", path.display()))?;
        written.push(path);
    }
    Ok(written)
}

/// The workspace root above `start`: the first folder with a Cargo.toml and engine/crates/.
pub fn find_root(start: &Path) -> Option<PathBuf> {
    start.ancestors().find(|d| d.join("Cargo.toml").is_file() && d.join("engine/crates").is_dir()).map(Path::to_path_buf)
}

/// `new game <id> [title=] [root=]`: prints what it wrote and what to run next.
pub fn command(args: &crate::Args) -> Result<()> {
    let Some(id) = args.pos(2) else {
        bail!("usage: aegis new game <id> [title=<title>] [root=<workspace>]");
    };
    let root = match args.keys.get("root") {
        Some(r) => PathBuf::from(r),
        None => find_root(&std::env::current_dir()?).context("no workspace root (Cargo.toml and engine/crates/) above the working folder; pass root=<dir>")?,
    };
    let title = args.keys.get("title").cloned().unwrap_or_else(|| default_title(id));
    let written = create(&root, id, &title)?;
    println!("§ new game '{id}' ({title}): {} files in {}", written.len(), root.join("games").join(id).display());
    println!("  run it:        cargo run -p {id}_game --bin {id}");
    println!("  test it:       cargo test -p {id}_game");
    println!("  its scenario:  cargo build -p {id}_game && target/debug/{id}_cli scenario run all");
    println!("  the engine's API guide: engine/docs/API.md");
    Ok(())
}
