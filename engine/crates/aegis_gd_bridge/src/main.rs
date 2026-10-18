pub mod converter;

use aegis_md_parser::MdParser;
use converter::GdConverter;
use std::fs;
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    println!("=== Aegis GD -> Markdown Migration Bridge ===");

    let project_root = Path::new(".");
    let scripts_dir = project_root.join("scripts");
    let scenes_dir = project_root.join("scenes");
    let output_dir = project_root.join("engine").join("game_specs");

    let converter = GdConverter::new();
    let parser = MdParser::new();

    let mut gd_count = 0;
    let mut tscn_count = 0;
    let mut parsed_ok = 0;

    // 1. Process all .gd files
    if scripts_dir.exists() {
        let mut gd_files = Vec::new();
        collect_files(&scripts_dir, "gd", &mut gd_files)?;

        for file in &gd_files {
            let rel_path = file.strip_prefix(&scripts_dir)?;
            let stem = file.file_stem().unwrap().to_string_lossy();
            let content = fs::read_to_string(file)?;

            let md = converter.gd_to_markdown(&content, &stem);

            // Verify with aegis_md_parser
            if parser.parse(&md).is_ok() {
                parsed_ok += 1;
            }

            // Write output .md
            let target_path = output_dir
                .join("scripts")
                .join(rel_path)
                .with_extension("md");

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_path, md)?;
            gd_count += 1;
        }
    }

    // 2. Process all .tscn files
    if scenes_dir.exists() {
        let mut tscn_files = Vec::new();
        collect_files(&scenes_dir, "tscn", &mut tscn_files)?;

        for file in &tscn_files {
            let rel_path = file.strip_prefix(&scenes_dir)?;
            let stem = file.file_stem().unwrap().to_string_lossy();
            let content = fs::read_to_string(file)?;

            let md = converter.tscn_to_markdown(&content, &stem);

            let target_path = output_dir
                .join("scenes")
                .join(rel_path)
                .with_extension("md");

            if let Some(parent) = target_path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(target_path, md)?;
            tscn_count += 1;
        }
    }

    println!("Conversion Complete!");
    println!("  - .gd scripts converted:   {}", gd_count);
    println!("  - .tscn scenes converted:  {}", tscn_count);
    println!("  - Cleanly verified as AST: {} / {}", parsed_ok, gd_count);
    println!("  - Target directory:        {}", output_dir.display());

    Ok(())
}

fn collect_files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) -> std::io::Result<()> {
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                collect_files(&path, ext, out)?;
            } else if path.extension().and_then(|s| s.to_str()) == Some(ext) {
                out.push(path);
            }
        }
    }
    Ok(())
}
