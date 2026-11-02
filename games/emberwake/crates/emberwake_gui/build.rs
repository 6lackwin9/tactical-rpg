//! ICON1: embeds `res/emberwake.ico` in the Windows `.exe` with the toolchain's own resource compiler, so no new crate:
//! `windres` (the GNU and llvm-mingw toolchains) or `rc.exe` (MSVC, when on PATH). Any other target, or no resource
//! compiler, still builds: the `.exe` then simply has no icon (the reason goes to this script's stderr, not a warning).

use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=res/emberwake.rc");
    println!("cargo:rerun-if-changed=res/emberwake.ico");
    let var = |name: &str| std::env::var(name).unwrap_or_default();
    // A resource compiler builds for its own architecture; both here assume x86_64 (the shipped target).
    if var("CARGO_CFG_TARGET_OS") != "windows" || var("CARGO_CFG_TARGET_ARCH") != "x86_64" {
        return;
    }
    let out = Path::new(&var("OUT_DIR")).to_path_buf();
    let built = if var("CARGO_CFG_TARGET_ENV") == "msvc" {
        let res = out.join("emberwake.res");
        Command::new("rc.exe").arg("/nologo").arg("/fo").arg(&res).arg("emberwake.rc").current_dir("res").status().map(|s| s.success().then_some(res))
    } else {
        let obj = out.join("emberwake_icon.o");
        Command::new("windres").args(["-O", "coff", "-i", "emberwake.rc", "-o"]).arg(&obj).current_dir("res").status().map(|s| s.success().then_some(obj))
    };
    match built {
        Ok(Some(file)) => println!("cargo:rustc-link-arg-bins={}", file.display()),
        Ok(None) => eprintln!("the resource compiler failed: the .exe has no icon"),
        Err(e) => eprintln!("no resource compiler ({e}): the .exe has no icon"),
    }
}
