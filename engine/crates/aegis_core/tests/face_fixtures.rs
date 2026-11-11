//! PORTRAITFX1-T (T1, T2): the anchor tool `tools/portraits/face.py` mirrors `parse_face` and `face_problems`; its
//! fixture layer `tools/portraits/fixtures/face/` and the expected check lines committed beside it
//! (`expected_check.txt`) are asserted here against the engine's rules and by the tool's own test (`test_face.py`),
//! so the two stay in step. The sidecar `set` wrote (`portraits/heroes/good.face.json`) must parse clean.

use aegis_core::assets::{face_problems, parse_face};
use std::path::{Path, PathBuf};

fn fixtures() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tools/portraits/fixtures/face")
}

/// A PNG's width from its IHDR chunk (bytes 16-19, big-endian).
fn png_width(path: &Path) -> i32 {
    let b = std::fs::read(path).unwrap();
    assert_eq!(&b[1..4], b"PNG", "{} is a PNG", path.display());
    i32::from_be_bytes([b[16], b[17], b[18], b[19]])
}

fn sidecars(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).unwrap() {
        let p = e.unwrap().path();
        if p.is_dir() {
            sidecars(&p, out);
        } else if p.to_string_lossy().ends_with(".face.json") {
            out.push(p);
        }
    }
}

/// The A11 lines of one layer root as the tool's `check` prints them: `path: problem`, paths relative to the root,
/// files sorted; a sidecar without its PNG says so and is checked with no width.
fn check_lines(root: &Path) -> Vec<String> {
    let mut files = Vec::new();
    sidecars(&root.join("portraits"), &mut files);
    let rel = |p: &Path| p.strip_prefix(root).unwrap().to_string_lossy().replace('\\', "/");
    files.sort_by_key(|p| rel(p));
    let mut lines = Vec::new();
    for f in files {
        let r = rel(&f);
        let name = f.file_name().unwrap().to_string_lossy().to_string();
        let png = f.with_file_name(format!("{}.png", name.strip_suffix(".face.json").unwrap()));
        let mut width = 0;
        if png.exists() {
            width = png_width(&png);
        } else {
            lines.push(format!("{r}: no portrait {} beside the sidecar.", png.file_name().unwrap().to_string_lossy()));
        }
        let raw: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(&f).unwrap()).unwrap();
        let problems = match parse_face(&raw) {
            Ok(face) => face_problems(&face, width),
            Err(p) => p,
        };
        lines.extend(problems.into_iter().map(|m| format!("{r}: {m}")));
    }
    lines
}

#[test]
fn t2_the_fixture_check_lines_are_the_engine_rules() {
    let root = fixtures();
    let want: Vec<String> =
        std::fs::read_to_string(root.join("expected_check.txt")).unwrap().lines().map(str::to_string).filter(|l| !l.is_empty()).collect();
    assert!(!want.is_empty(), "the fixture set holds broken sidecars");
    assert_eq!(check_lines(&root), want);
}

#[test]
fn t1_the_sidecar_the_tool_wrote_parses_clean() {
    let root = fixtures();
    let raw: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root.join("portraits/heroes/good.face.json")).unwrap()).unwrap();
    let face = parse_face(&raw).expect("face.py set writes what parse_face reads");
    let width = png_width(&root.join("portraits/heroes/good.png"));
    assert_eq!(face.size, width, "set takes size from the PNG's width");
    assert!(face_problems(&face, width).is_empty(), "{:?}", face_problems(&face, width));
    assert!(!face.eyes.is_empty() && !face.glow.is_empty() && !face.lines.is_empty() && !face.idle.is_empty(), "the fixture uses every anchor kind");
}
