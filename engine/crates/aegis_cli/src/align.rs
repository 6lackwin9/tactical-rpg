//! `aegis align-map`: game-agnostic diorama-to-map reference validation and closed-loop alignment tool.
//!
//! Subcommands:
//!   aegis align-map refspec   map=<id> [draft=<art_draft.json>] [out=<dir>]
//!   aegis align-map check-ref ref=<img.png|jpg> spec=<refspec.json> [map=<id>] [out=<dir>]
//!   aegis align-map score     ref=<img.png|jpg> render=<map.png> map=<id> [pass=1|2|all] [out=<dir>]
//!   aegis align-map run       ref=<img.png|jpg> map=<id> [out=<dir>] [apply_fx=0|1] [view=square|tilted] [exe=<game exe>]
//!
//! Pure engine tool: reads standard `maps.json` (`map_templates`), optional world `locations.json`, and
//! `catalog/grounds.json`, and invokes the game binary's `render_map=<id>` offline hook for closed-loop Pass 2
//! convergence. Names no game or lore word.

use crate::{Args, Game};
use aegis_content::manifest::Manifest;
use anyhow::{bail, Context, Result};
use indexmap::IndexMap;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::process::Command;

// ---- CLI entry point -----------------------------------------------------------------------------------------------

const ALIGN_USAGE: &str = "usage:
  aegis align-map refspec   map=<id> [draft=<draft.json>] [out=<dir>]
  aegis align-map check-ref ref=<img.png|jpg> spec=<refspec.json> [map=<id>] [out=<dir>]
  aegis align-map score     ref=<img.png|jpg> render=<map.png> map=<id> [pass=1|2|all] [out=<dir>]
  aegis align-map run       ref=<img.png|jpg> map=<id> [out=<dir>] [apply_fx=0|1] [view=square|tilted] [exe=<game exe>]";

pub fn command<G: Game>(game: &G, args: &Args) -> Result<()> {
    match args.pos(1) {
        Some("refspec") => cmd_refspec(game, args),
        Some("check-ref") => cmd_check_ref(game, args),
        Some("score") => cmd_score(game, args),
        Some("run") => cmd_run(game, args),
        _ => {
            println!("{ALIGN_USAGE}");
            Ok(())
        }
    }
}

fn cmd_refspec<G: Game>(game: &G, args: &Args) -> Result<()> {
    let map_id = args.keys.get("map").context("refspec requires map=<id>")?;
    let data_dir = game.data_dir(args)?;
    let draft = args.keys.get("draft").map(PathBuf::from);
    let out_dir = PathBuf::from(args.key("out", &format!("out/align/{map_id}")));
    std::fs::create_dir_all(&out_dir)?;

    let model = MapModel::load(&data_dir, map_id, draft.as_deref())?;
    let spec = build_refspec(&model);
    let prompt = render_prompt(&model, &spec);
    let blockout = render_blockout(&model, 32);

    let spec_path = out_dir.join(format!("{map_id}.refspec.json"));
    let prompt_path = out_dir.join(format!("{map_id}.prompt.txt"));
    let blockout_path = out_dir.join(format!("{map_id}.blockout.png"));
    std::fs::write(&spec_path, serde_json::to_string_pretty(&spec)? + "\n")?;
    std::fs::write(&prompt_path, prompt)?;
    save_png(&blockout, &blockout_path)?;

    let hard_n = spec["required_clauses"].as_array().map_or(0, |a| a.iter().filter(|c| c["severity"] == "HARD").count());
    let exit_n = spec["exits"].as_array().map_or(0, Vec::len);
    println!(
        "refspec ok: {map_id} ({}x{}, {} zones, {exit_n} exits, {hard_n} HARD clauses)\n  spec:     {}\n  prompt:   {}\n  blockout: {}",
        model.cols,
        model.rows,
        model.zone_defs.len(),
        spec_path.display(),
        prompt_path.display(),
        blockout_path.display()
    );
    Ok(())
}

fn cmd_check_ref<G: Game>(game: &G, args: &Args) -> Result<()> {
    let ref_path = PathBuf::from(args.keys.get("ref").context("check-ref requires ref=<img.png|jpg>")?);
    let spec_path = PathBuf::from(args.keys.get("spec").context("check-ref requires spec=<refspec.json>")?);
    let spec: Value = serde_json::from_str(&std::fs::read_to_string(&spec_path).with_context(|| format!("reading {}", spec_path.display()))?)?;
    let map_id = args
        .keys
        .get("map")
        .cloned()
        .or_else(|| spec["map_id"].as_str().map(String::from))
        .context("check-ref: map_id missing from spec and args")?;
    let out_dir = PathBuf::from(args.key("out", &format!("out/align/{map_id}/ref_check")));
    std::fs::create_dir_all(&out_dir)?;

    let data_dir = game.data_dir(args)?;
    let model = MapModel::load(&data_dir, &map_id, None)?;
    let ref_img = load_image(&ref_path)?;
    let audit = check_reference(&ref_img, &model, &spec, &out_dir)?;

    let audit_path = out_dir.join("ref_audit.json");
    std::fs::write(&audit_path, serde_json::to_string_pretty(&audit)? + "\n")?;
    print_ref_audit_summary(&audit, &audit_path);
    Ok(())
}

fn cmd_score<G: Game>(game: &G, args: &Args) -> Result<()> {
    let ref_path = PathBuf::from(args.keys.get("ref").context("score requires ref=<img.png|jpg>")?);
    let render_path = PathBuf::from(args.keys.get("render").context("score requires render=<map.png>")?);
    let map_id = args.keys.get("map").context("score requires map=<id>")?;
    let pass_mode = args.key("pass", "all");
    let out_dir = PathBuf::from(args.key("out", &format!("out/align/{map_id}/score")));
    std::fs::create_dir_all(&out_dir)?;

    let data_dir = game.data_dir(args)?;
    let model = MapModel::load(&data_dir, map_id, None)?;
    let ref_img = load_image(&ref_path)?;
    let render_img = load_image(&render_path)?;
    let report = score_alignment(&ref_img, &render_img, &model, &pass_mode, &out_dir)?;

    let report_path = out_dir.join("report.json");
    std::fs::write(&report_path, serde_json::to_string_pretty(&report.json)? + "\n")?;
    if let Some(fx) = &report.map_fx {
        let fx_path = out_dir.join(format!("{map_id}.fx.json"));
        std::fs::write(&fx_path, serde_json::to_string_pretty(fx)? + "\n")?;
        println!("  solved MapFx: {}", fx_path.display());
    }
    print_score_summary(&report.json, &report_path);
    Ok(())
}

fn cmd_run<G: Game>(game: &G, args: &Args) -> Result<()> {
    let ref_path = PathBuf::from(args.keys.get("ref").context("run requires ref=<img.png|jpg>")?);
    let map_id = args.keys.get("map").context("run requires map=<id>")?;
    let view = args.key("view", "square");
    let apply_fx = args.key("apply_fx", "0") == "1";
    let out_dir = PathBuf::from(args.key("out", &format!("out/align/{map_id}/run")));
    std::fs::create_dir_all(&out_dir)?;

    let data_dir = game.data_dir(args)?;
    let manifest_path = game.manifest_path(args)?;
    let game_dir = manifest_path.parent().context("manifest has no parent folder")?.to_path_buf();
    let manifest = Manifest::read(&manifest_path).map_err(anyhow::Error::msg)?;
    let exe = match args.keys.get("exe") {
        Some(p) => PathBuf::from(p),
        None => std::env::current_exe()?.with_file_name(format!("{}{}", manifest.id, std::env::consts::EXE_SUFFIX)),
    };
    if !exe.is_file() {
        bail!("align-map run: game executable not found at {} (pass exe=<path> or build it first)", exe.display());
    }

    let model = MapModel::load(&data_dir, map_id, None)?;
    let ref_img = load_image(&ref_path)?;

    // Step 1: Baseline render (no MapFx)
    let before_png = out_dir.join("render_before.png");
    invoke_render_map(&exe, &data_dir, map_id, &view, Some("none"), &before_png)?;
    let before_img = load_image(&before_png)?;
    let before_dir = out_dir.join("before");
    std::fs::create_dir_all(&before_dir)?;
    let before_rep = score_alignment(&ref_img, &before_img, &model, "all", &before_dir)?;
    std::fs::write(before_dir.join("report.json"), serde_json::to_string_pretty(&before_rep.json)? + "\n")?;

    let fx_val = before_rep.map_fx.clone().unwrap_or_else(|| json!({}));
    let fx_path = out_dir.join(format!("{map_id}.fx.json"));
    std::fs::write(&fx_path, serde_json::to_string_pretty(&fx_val)? + "\n")?;

    // Step 2: Re-render with Pass 2 solved MapFx
    let after_png = out_dir.join("render_pass2.png");
    let fx_str = fx_path.to_string_lossy().to_string();
    invoke_render_map(&exe, &data_dir, map_id, &view, Some(&fx_str), &after_png)?;
    let after_img = load_image(&after_png)?;
    let after_dir = out_dir.join("after");
    std::fs::create_dir_all(&after_dir)?;
    let after_rep = score_alignment(&ref_img, &after_img, &model, "all", &after_dir)?;
    std::fs::write(after_dir.join("report.json"), serde_json::to_string_pretty(&after_rep.json)? + "\n")?;

    // Step 3: 3-panel visual comparison (Reference | Baseline | Pass 2)
    let reg = register_reference(&ref_img, &model);
    let ref_warped = warp_reference_to_render(&ref_img, &reg, after_img.w, after_img.h);
    let triptych = hconcat(&[&ref_warped, &before_img, &after_img], 6);
    let triptych_path = out_dir.join("before_after_ref.png");
    save_png(&triptych, &triptych_path)?;

    let bl = before_rep.json["pass2"]["mean_zone_delta_l"].as_f64().unwrap_or(0.0);
    let al = after_rep.json["pass2"]["mean_zone_delta_l"].as_f64().unwrap_or(0.0);
    let br = before_rep.json["pass2"]["mean_zone_delta_rgb"].as_f64().unwrap_or(0.0);
    let ar = after_rep.json["pass2"]["mean_zone_delta_rgb"].as_f64().unwrap_or(0.0);
    let p1_g = after_rep.json["pass1"]["ground_agreement_pct"].as_f64().unwrap_or(0.0);
    let p1_iou = after_rep.json["pass1"]["mean_prop_iou"].as_f64().unwrap_or(0.0);

    let summary = json!({
        "map_id": map_id,
        "view": view,
        "before": {
            "mean_zone_delta_l": round2(bl),
            "mean_zone_delta_rgb": round2(br),
        },
        "after_pass2": {
            "mean_zone_delta_l": round2(al),
            "mean_zone_delta_rgb": round2(ar),
            "delta_l_reduction_pct": if bl > 1e-3 { round2((bl - al) / bl * 100.0) } else { 0.0 },
            "delta_rgb_reduction_pct": if br > 1e-3 { round2((br - ar) / br * 100.0) } else { 0.0 },
        },
        "pass1": {
            "ground_agreement_pct": round2(p1_g),
            "mean_prop_iou": round3(p1_iou),
        },
        "outputs": {
            "fx": fx_path.display().to_string(),
            "triptych": triptych_path.display().to_string(),
            "pass3_crops": after_dir.join("pass3_top_crops.png").display().to_string(),
        }
    });
    std::fs::write(out_dir.join("run_summary.json"), serde_json::to_string_pretty(&summary)? + "\n")?;

    println!("align-map run complete ({map_id}, {view}):");
    println!("  Pass 1: ground_agreement={p1_g:.1}%, mean_prop_iou={p1_iou:.3}");
    println!("  Pass 2: mean_zone_delta_L   {bl:.2} -> {al:.2} ({:+.1}%)", if bl > 1e-3 { (al - bl) / bl * 100.0 } else { 0.0 });
    println!("  Pass 2: mean_zone_delta_RGB {br:.2} -> {ar:.2} ({:+.1}%)", if br > 1e-3 { (ar - br) / br * 100.0 } else { 0.0 });
    println!("  triptych:    {}", triptych_path.display());
    println!("  pass3 crops: {}", after_dir.join("pass3_top_crops.png").display());
    println!("  solved fx:   {}", fx_path.display());

    if apply_fx {
        let dest = game_dir.join("assets").join("map_fx").join(format!("{map_id}.fx.json"));
        if let Some(p) = dest.parent() {
            std::fs::create_dir_all(p)?;
        }
        std::fs::copy(&fx_path, &dest)?;
        println!("  applied MapFx -> {}", dest.display());
    }
    Ok(())
}

fn invoke_render_map(exe: &Path, data_dir: &Path, map_id: &str, view: &str, fx: Option<&str>, out_png: &Path) -> Result<()> {
    let mut cmd = Command::new(exe);
    cmd.arg(format!("render_map={map_id}"))
        .arg(format!("out={}", out_png.display()))
        .arg(format!("data={}", data_dir.display()))
        .arg(format!("view={view}"));
    if let Some(f) = fx {
        cmd.arg(format!("fx={f}"));
    }
    let out = cmd.output().with_context(|| format!("spawning {}", exe.display()))?;
    if !out.status.success() {
        let stderr = String::from_utf8_lossy(&out.stderr);
        let stdout = String::from_utf8_lossy(&out.stdout);
        bail!("render_map failed ({}):\n{stdout}\n{stderr}", out.status);
    }
    Ok(())
}

// ---- Minimal RGBA image I/O & drawing (PNG + JPEG, zero unsafe) ----------------------------------------------------

#[derive(Debug, Clone)]
pub struct RgbaImg {
    pub w: usize,
    pub h: usize,
    pub px: Vec<u8>,
}

impl RgbaImg {
    pub fn new(w: usize, h: usize, fill: [u8; 4]) -> Self {
        let mut px = Vec::with_capacity(w * h * 4);
        for _ in 0..(w * h) {
            px.extend_from_slice(&fill);
        }
        Self { w, h, px }
    }

    #[inline]
    pub fn get(&self, x: usize, y: usize) -> [u8; 4] {
        let i = (y * self.w + x) * 4;
        [self.px[i], self.px[i + 1], self.px[i + 2], self.px[i + 3]]
    }

    #[inline]
    pub fn set(&mut self, x: usize, y: usize, c: [u8; 4]) {
        if x < self.w && y < self.h {
            let i = (y * self.w + x) * 4;
            self.px[i..i + 4].copy_from_slice(&c);
        }
    }

    #[inline]
    pub fn blend(&mut self, x: usize, y: usize, c: [u8; 4]) {
        if x >= self.w || y >= self.h {
            return;
        }
        let a = c[3] as f32 / 255.0;
        let inv = 1.0 - a;
        let i = (y * self.w + x) * 4;
        for ch in 0..3 {
            let v = self.px[i + ch] as f32 * inv + c[ch] as f32 * a;
            self.px[i + ch] = v.round().clamp(0.0, 255.0) as u8;
        }
        self.px[i + 3] = 255;
    }

    pub fn fill_rect(&mut self, x0: usize, y0: usize, w: usize, h: usize, c: [u8; 4]) {
        let x1 = (x0 + w).min(self.w);
        let y1 = (y0 + h).min(self.h);
        for y in y0.min(self.h)..y1 {
            for x in x0.min(self.w)..x1 {
                if c[3] == 255 {
                    self.set(x, y, c);
                } else {
                    self.blend(x, y, c);
                }
            }
        }
    }

    pub fn stroke_rect(&mut self, x0: usize, y0: usize, w: usize, h: usize, thickness: usize, c: [u8; 4]) {
        if w == 0 || h == 0 {
            return;
        }
        let t = thickness.max(1);
        self.fill_rect(x0, y0, w, t.min(h), c);
        self.fill_rect(x0, y0 + h.saturating_sub(t), w, t.min(h), c);
        self.fill_rect(x0, y0, t.min(w), h, c);
        self.fill_rect(x0 + w.saturating_sub(t), y0, t.min(w), h, c);
    }

    pub fn crop(&self, x0: usize, y0: usize, w: usize, h: usize) -> Self {
        let mut out = Self::new(w, h, [0, 0, 0, 255]);
        for y in 0..h {
            let sy = y0 + y;
            if sy >= self.h {
                continue;
            }
            for x in 0..w {
                let sx = x0 + x;
                if sx < self.w {
                    out.set(x, y, self.get(sx, sy));
                }
            }
        }
        out
    }

    pub fn resize_bilinear(&self, nw: usize, nh: usize) -> Self {
        let nw = nw.max(1);
        let nh = nh.max(1);
        let mut out = Self::new(nw, nh, [0, 0, 0, 255]);
        let sx = self.w as f32 / nw as f32;
        let sy = self.h as f32 / nh as f32;
        for y in 0..nh {
            let fy = ((y as f32 + 0.5) * sy - 0.5).clamp(0.0, (self.h.saturating_sub(1)) as f32);
            let y0 = fy.floor() as usize;
            let y1 = (y0 + 1).min(self.h - 1);
            let ty = fy - y0 as f32;
            for x in 0..nw {
                let fx = ((x as f32 + 0.5) * sx - 0.5).clamp(0.0, (self.w.saturating_sub(1)) as f32);
                let x0 = fx.floor() as usize;
                let x1 = (x0 + 1).min(self.w - 1);
                let tx = fx - x0 as f32;
                let p00 = self.get(x0, y0);
                let p10 = self.get(x1, y0);
                let p01 = self.get(x0, y1);
                let p11 = self.get(x1, y1);
                let mut c = [255u8; 4];
                for ch in 0..3 {
                    let top = p00[ch] as f32 * (1.0 - tx) + p10[ch] as f32 * tx;
                    let bot = p01[ch] as f32 * (1.0 - tx) + p11[ch] as f32 * tx;
                    c[ch] = (top * (1.0 - ty) + bot * ty).round().clamp(0.0, 255.0) as u8;
                }
                out.set(x, y, c);
            }
        }
        out
    }
}

pub fn load_image(path: &Path) -> Result<RgbaImg> {
    let bytes = std::fs::read(path).with_context(|| format!("reading image {}", path.display()))?;
    if bytes.len() >= 8 && &bytes[0..8] == b"\x89PNG\r\n\x1a\n" {
        let mut dec = png::Decoder::new(std::io::Cursor::new(&bytes));
        dec.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = dec.read_info().with_context(|| format!("decoding PNG {}", path.display()))?;
        let mut buf = vec![0u8; reader.output_buffer_size()];
        let info = reader.next_frame(&mut buf)?;
        let src = &buf[..info.buffer_size()];
        let px: Vec<u8> = match info.color_type {
            png::ColorType::Rgba => src.to_vec(),
            png::ColorType::Rgb => src.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect(),
            png::ColorType::GrayscaleAlpha => src.chunks_exact(2).flat_map(|c| [c[0], c[0], c[0], c[1]]).collect(),
            png::ColorType::Grayscale => src.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
            png::ColorType::Indexed => bail!("unsupported indexed PNG after expansion: {}", path.display()),
        };
        return Ok(RgbaImg { w: info.width as usize, h: info.height as usize, px });
    }
    if bytes.len() >= 2 && bytes[0] == 0xFF && bytes[1] == 0xD8 {
        let mut dec = zune_jpeg::JpegDecoder::new(&bytes);
        let raw = dec.decode().map_err(|e| anyhow::anyhow!("decoding JPEG {}: {e:?}", path.display()))?;
        let info = dec.info().context("JPEG missing header info")?;
        let (w, h) = (info.width as usize, info.height as usize);
        let px: Vec<u8> = if raw.len() == w * h * 3 {
            raw.chunks_exact(3).flat_map(|c| [c[0], c[1], c[2], 255]).collect()
        } else if raw.len() == w * h {
            raw.iter().flat_map(|g| [*g, *g, *g, 255]).collect()
        } else if raw.len() == w * h * 4 {
            raw
        } else {
            bail!("unexpected decoded JPEG byte length {} for {}x{}", raw.len(), w, h);
        };
        return Ok(RgbaImg { w, h, px });
    }
    bail!("unsupported image format (expected PNG or JPEG): {}", path.display())
}

pub fn save_png(img: &RgbaImg, path: &Path) -> Result<()> {
    if let Some(p) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(p)?;
    }
    let file = std::fs::File::create(path).with_context(|| format!("creating {}", path.display()))?;
    let mut enc = png::Encoder::new(std::io::BufWriter::new(file), img.w as u32, img.h as u32);
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    let mut writer = enc.write_header()?;
    writer.write_image_data(&img.px)?;
    Ok(())
}

fn hconcat(imgs: &[&RgbaImg], gap: usize) -> RgbaImg {
    if imgs.is_empty() {
        return RgbaImg::new(1, 1, [0, 0, 0, 255]);
    }
    let h = imgs.iter().map(|i| i.h).max().unwrap_or(1);
    let total_w: usize = imgs.iter().map(|i| i.w).sum::<usize>() + gap * imgs.len().saturating_sub(1);
    let mut out = RgbaImg::new(total_w, h, [16, 16, 20, 255]);
    let mut ox = 0;
    for img in imgs {
        for y in 0..img.h {
            for x in 0..img.w {
                out.set(ox + x, y, img.get(x, y));
            }
        }
        ox += img.w + gap;
    }
    out
}

fn vconcat(imgs: &[RgbaImg], gap: usize) -> RgbaImg {
    if imgs.is_empty() {
        return RgbaImg::new(1, 1, [0, 0, 0, 255]);
    }
    let w = imgs.iter().map(|i| i.w).max().unwrap_or(1);
    let total_h: usize = imgs.iter().map(|i| i.h).sum::<usize>() + gap * imgs.len().saturating_sub(1);
    let mut out = RgbaImg::new(w, total_h, [16, 16, 20, 255]);
    let mut oy = 0;
    for img in imgs {
        for y in 0..img.h {
            for x in 0..img.w {
                out.set(x, oy + y, img.get(x, y));
            }
        }
        oy += img.h + gap;
    }
    out
}

#[inline]
fn luma709(r: f32, g: f32, b: f32) -> f32 {
    0.2126 * r + 0.7152 * g + 0.0722 * b
}

// ---- Map Model (loaded from maps.json + locations.json + grounds.json) ---------------------------------------------

#[derive(Debug, Clone)]
pub struct ZoneInfo {
    pub id: String,
    pub x0: usize,
    pub y0: usize,
    pub x1: usize,
    pub y1: usize,
    pub floor_cells: usize,
    pub dominant_ground: String,
    pub wall_set: String,
    pub indoor: bool,
    pub style_note: String,
}

#[derive(Debug, Clone)]
pub struct ExitInfo {
    pub id: String,
    pub label: String,
    pub x: usize,
    pub y: usize,
    pub wall_side: String,
}

#[derive(Debug, Clone)]
pub struct PropInfo {
    pub id: String,
    pub art: String,
    pub x: usize,
    pub y: usize,
    pub w: usize,
    pub h: usize,
    pub layer: String,
    pub flip: bool,
    pub zone_id: String,
    pub dynamic: bool,
}

#[derive(Debug, Clone)]
pub struct MapModel {
    pub id: String,
    pub name: String,
    pub cols: usize,
    pub rows: usize,
    pub tiles: Vec<char>,
    pub zones: Vec<String>,
    pub zone_defs: IndexMap<String, ZoneInfo>,
    pub grounds: Vec<String>,
    pub ground_colors: IndexMap<String, [u8; 3]>,
    pub anchors: Vec<(String, usize, usize)>,
    pub exits: Vec<ExitInfo>,
    pub npcs: Vec<String>,
    pub props: Vec<PropInfo>,
}

impl MapModel {
    pub fn load(data_dir: &Path, map_id: &str, draft_path: Option<&Path>) -> Result<Self> {
        let maps_path = data_dir.join("maps").join("maps.json");
        let maps_val: Value = serde_json::from_str(&std::fs::read_to_string(&maps_path).with_context(|| format!("reading {}", maps_path.display()))?)?;
        let entry = maps_val["entries"]
            .as_array()
            .and_then(|arr| arr.iter().find(|e| e["id"].as_str() == Some(map_id)))
            .with_context(|| format!("map '{map_id}' not found in {}", maps_path.display()))?;

        // Load catalog grounds.json if present
        let mut ground_colors = IndexMap::new();
        let grounds_path = data_dir.join("catalog").join("grounds.json");
        if grounds_path.is_file() {
            if let Ok(text) = std::fs::read_to_string(&grounds_path) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if let Some(arr) = v["entries"].as_array() {
                        for g in arr {
                            if let (Some(id), Some(hex)) = (g["id"].as_str(), g["colour"].as_str()) {
                                if let Some(rgb) = parse_hex_rgb(hex) {
                                    ground_colors.insert(id.to_string(), rgb);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Load optional draft_room_styles from art draft JSON
        let mut draft_styles: BTreeMap<String, String> = BTreeMap::new();
        if let Some(dp) = draft_path {
            if let Ok(text) = std::fs::read_to_string(dp) {
                if let Ok(v) = serde_json::from_str::<Value>(&text) {
                    if let Some(obj) = v["draft_room_styles"].as_object() {
                        for (k, val) in obj {
                            if let Some(s) = val.as_str() {
                                draft_styles.insert(k.clone(), s.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Scan worlds/*/locations.json for a location pointing to this map
        let mut location_entry: Option<Value> = None;
        let worlds_dir = data_dir.join("worlds");
        if worlds_dir.is_dir() {
            if let Ok(rd) = std::fs::read_dir(&worlds_dir) {
                let mut dirs: Vec<PathBuf> = rd.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
                dirs.sort();
                'scan: for wd in dirs {
                    let lp = wd.join("locations.json");
                    if !lp.is_file() {
                        continue;
                    }
                    if let Ok(text) = std::fs::read_to_string(&lp) {
                        if let Ok(v) = serde_json::from_str::<Value>(&text) {
                            if let Some(arr) = v["entries"].as_array() {
                                for loc in arr {
                                    if loc["map"].as_str() == Some(map_id) {
                                        location_entry = Some(loc.clone());
                                        break 'scan;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        Self::from_json(entry, location_entry.as_ref(), ground_colors, &draft_styles)
    }

    pub fn from_json(
        entry: &Value,
        location: Option<&Value>,
        ground_colors: IndexMap<String, [u8; 3]>,
        draft_styles: &BTreeMap<String, String>,
    ) -> Result<Self> {
        let id = entry["id"].as_str().unwrap_or("unknown").to_string();
        let name = entry["name"].as_str().unwrap_or(&id).to_string();
        let row_strs: Vec<&str> = entry["rows"].as_array().context("map missing 'rows'")?.iter().filter_map(Value::as_str).collect();
        let rows = row_strs.len();
        let cols = row_strs.first().map_or(0, |r| r.chars().count());
        if cols == 0 || rows == 0 {
            bail!("map '{id}' has empty grid");
        }
        let mut tiles = Vec::with_capacity(cols * rows);
        for r in &row_strs {
            let mut row_chars: Vec<char> = r.chars().collect();
            row_chars.resize(cols, '#');
            tiles.extend(row_chars);
        }

        // Zones
        let mut zone_char_map: BTreeMap<char, (String, String, bool)> = BTreeMap::new();
        if let Some(zl) = entry["zone_legend"].as_object() {
            for (k, v) in zl {
                if let Some(ch) = k.chars().next() {
                    let zid = v["id"].as_str().unwrap_or(k).to_string();
                    let ws = v["wall_set"].as_str().unwrap_or("").to_string();
                    let indoor = v["indoor"].as_bool().unwrap_or(false);
                    zone_char_map.insert(ch, (zid, ws, indoor));
                }
            }
        }
        let mut zones = vec![String::new(); cols * rows];
        if let Some(zrows) = entry["zones"].as_array() {
            for (y, zr) in zrows.iter().enumerate().take(rows) {
                if let Some(s) = zr.as_str() {
                    for (x, ch) in s.chars().enumerate().take(cols) {
                        if let Some((zid, _, _)) = zone_char_map.get(&ch) {
                            zones[y * cols + x] = zid.clone();
                        }
                    }
                }
            }
        }

        // Grounds
        let mut g_char_map: BTreeMap<char, String> = BTreeMap::new();
        if let Some(gl) = entry["ground_legend"].as_object() {
            for (k, v) in gl {
                if let (Some(ch), Some(gid)) = (k.chars().next(), v.as_str()) {
                    g_char_map.insert(ch, gid.to_string());
                }
            }
        }
        let mut grounds = vec![String::new(); cols * rows];
        if let Some(grows) = entry["ground"].as_array() {
            for (y, gr) in grows.iter().enumerate().take(rows) {
                if let Some(s) = gr.as_str() {
                    for (x, ch) in s.chars().enumerate().take(cols) {
                        if let Some(gid) = g_char_map.get(&ch) {
                            grounds[y * cols + x] = gid.clone();
                        }
                    }
                }
            }
        }

        // Aggregate per-zone info
        let mut zone_defs: IndexMap<String, ZoneInfo> = IndexMap::new();
        let mut zone_ground_counts: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();
        for (_, (zid, ws, indoor)) in &zone_char_map {
            zone_defs.entry(zid.clone()).or_insert_with(|| ZoneInfo {
                id: zid.clone(),
                x0: cols,
                y0: rows,
                x1: 0,
                y1: 0,
                floor_cells: 0,
                dominant_ground: String::new(),
                wall_set: ws.clone(),
                indoor: *indoor,
                style_note: draft_styles.get(zid).cloned().unwrap_or_default(),
            });
        }
        for y in 0..rows {
            for x in 0..cols {
                let idx = y * cols + x;
                let zid = &zones[idx];
                if zid.is_empty() {
                    continue;
                }
                if let Some(zi) = zone_defs.get_mut(zid) {
                    zi.x0 = zi.x0.min(x);
                    zi.y0 = zi.y0.min(y);
                    zi.x1 = zi.x1.max(x);
                    zi.y1 = zi.y1.max(y);
                    if tiles[idx] != '#' {
                        zi.floor_cells += 1;
                        let g = &grounds[idx];
                        if !g.is_empty() {
                            *zone_ground_counts.entry(zid.clone()).or_default().entry(g.clone()).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        for (zid, zi) in &mut zone_defs {
            if let Some(counts) = zone_ground_counts.get(zid) {
                if let Some((dom, _)) = counts.iter().max_by_key(|(_, c)| *c) {
                    zi.dominant_ground = dom.clone();
                }
            }
        }

        // Anchors
        let mut anchors = Vec::new();
        if let Some(arr) = entry["anchors"].as_array() {
            for a in arr {
                if let Some(aid) = a["id"].as_str() {
                    let x = a["x"].as_u64().unwrap_or(0) as usize;
                    let y = a["y"].as_u64().unwrap_or(0) as usize;
                    anchors.push((aid.to_string(), x.min(cols - 1), y.min(rows - 1)));
                }
            }
        }

        // Exits & NPCs from location markers (or anchor fallback)
        let mut exits = Vec::new();
        let mut npcs = Vec::new();
        if let Some(loc) = location {
            if let Some(markers) = loc["markers"].as_array() {
                for m in markers {
                    let kind = m["kind"].as_str().unwrap_or("");
                    let mid = m["id"].as_str().unwrap_or("").to_string();
                    let anchor_id = m["anchor"].as_str().unwrap_or(&mid);
                    if kind == "exit" {
                        if let Some((_, x, y)) = anchors.iter().find(|(a, _, _)| a == anchor_id) {
                            exits.push(ExitInfo {
                                id: mid,
                                label: m["label"].as_str().unwrap_or("").to_string(),
                                x: *x,
                                y: *y,
                                wall_side: classify_wall_side(*x, *y, cols, rows),
                            });
                        }
                    } else if kind == "npc" {
                        let ch_id = m["character"].as_str().unwrap_or(&mid).to_string();
                        npcs.push(ch_id);
                    }
                }
            }
        } else {
            for (aid, x, y) in &anchors {
                if aid.starts_with("gate") || aid.starts_with("road") || aid.starts_with("exit") || aid.starts_with("streets") || aid.starts_with("sluice") {
                    exits.push(ExitInfo {
                        id: aid.clone(),
                        label: aid.clone(),
                        x: *x,
                        y: *y,
                        wall_side: classify_wall_side(*x, *y, cols, rows),
                    });
                }
            }
        }

        // Props
        let mut props = Vec::new();
        if let Some(arr) = entry["props"].as_array() {
            for p in arr {
                let pid = p["id"].as_str().unwrap_or("").to_string();
                let art = p["art"].as_str().unwrap_or("").to_string();
                let x = p["x"].as_u64().unwrap_or(0) as usize;
                let y = p["y"].as_u64().unwrap_or(0) as usize;
                let w = p["w"].as_u64().unwrap_or(1).max(1) as usize;
                let h = p["h"].as_u64().unwrap_or(1).max(1) as usize;
                let layer = p["layer"].as_str().unwrap_or("sort").to_string();
                let flip = p["flip"].as_bool().unwrap_or(false);
                let dynamic = p.get("until_bunk").is_some()
                    || p.get("requires_facility").is_some()
                    || p.get("until_facility").is_some()
                    || p.get("requires_flags").is_some();
                let zone_id = prop_zone_id(&zones, cols, rows, x, y, w, h);
                props.push(PropInfo { id: pid, art, x, y, w, h, layer, flip, zone_id, dynamic });
            }
        }

        Ok(Self { id, name, cols, rows, tiles, zones, zone_defs, grounds, ground_colors, anchors, exits, npcs, props })
    }

    #[inline]
    pub fn tile(&self, x: usize, y: usize) -> char {
        self.tiles[y * self.cols + x]
    }

    #[inline]
    pub fn zone(&self, x: usize, y: usize) -> &str {
        &self.zones[y * self.cols + x]
    }

    #[inline]
    pub fn ground(&self, x: usize, y: usize) -> &str {
        &self.grounds[y * self.cols + x]
    }
}

fn classify_wall_side(x: usize, y: usize, cols: usize, rows: usize) -> String {
    if y <= 2 {
        "north".into()
    } else if y + 3 >= rows {
        "south".into()
    } else if x <= 2 {
        "west".into()
    } else if x + 3 >= cols {
        "east".into()
    } else {
        "interior".into()
    }
}

fn prop_zone_id(zones: &[String], cols: usize, rows: usize, x: usize, y: usize, w: usize, h: usize) -> String {
    let mut counts: BTreeMap<&str, usize> = BTreeMap::new();
    for dy in 0..=h {
        let yy = (y + dy).min(rows - 1);
        for dx in 0..w {
            let xx = (x + dx).min(cols - 1);
            let z = zones[yy * cols + xx].as_str();
            if !z.is_empty() {
                *counts.entry(z).or_insert(0) += 1;
            }
        }
    }
    counts.into_iter().max_by_key(|(_, c)| *c).map(|(z, _)| z.to_string()).unwrap_or_default()
}

fn parse_hex_rgb(hex: &str) -> Option<[u8; 3]> {
    let s = hex.strip_prefix('#')?;
    if s.len() != 6 {
        return None;
    }
    Some([
        u8::from_str_radix(&s[0..2], 16).ok()?,
        u8::from_str_radix(&s[2..4], 16).ok()?,
        u8::from_str_radix(&s[4..6], 16).ok()?,
    ])
}

// ---- Stage A: Reference Spec, Prompt & Blockout Guide --------------------------------------------------------------

fn infer_expected_hue(zi: &ZoneInfo) -> &'static str {
    let note = zi.style_note.to_ascii_lowercase();
    if zi.dominant_ground == "water" || note.contains("water") {
        "cool_water"
    } else if zi.wall_set.contains("plaster") || note.contains("whitewash") || note.contains("bright") {
        "bright_plaster"
    } else if zi.dominant_ground == "floorboards" || note.contains("wood") || note.contains("floorboard") {
        "warm_wood"
    } else if zi.dominant_ground == "moss" || note.contains("moss") || note.contains("unlit") {
        "dark_moss"
    } else if note.contains("forge") || note.contains("hearth") || note.contains("ember") {
        "warm_forge"
    } else {
        "neutral_stone"
    }
}

pub fn build_refspec(model: &MapModel) -> Value {
    let cw = model.cols as f64;
    let ch = model.rows as f64;

    let mut zones_json = Vec::new();
    let mut bright_zones = Vec::new();
    let mut mid_zones = Vec::new();
    let mut dark_zones = Vec::new();
    for zi in model.zone_defs.values() {
        let hue = infer_expected_hue(zi);
        match hue {
            "bright_plaster" => bright_zones.push(zi.id.clone()),
            "dark_moss" => dark_zones.push(zi.id.clone()),
            _ if hue != "cool_water" => mid_zones.push(zi.id.clone()),
            _ => {}
        }
        zones_json.push(json!({
            "id": zi.id,
            "bbox_cells": [zi.x0, zi.y0, zi.x1, zi.y1],
            "bbox_norm": [round3(zi.x0 as f64 / cw), round3(zi.y0 as f64 / ch), round3((zi.x1 + 1) as f64 / cw), round3((zi.y1 + 1) as f64 / ch)],
            "dominant_ground": zi.dominant_ground,
            "wall_set": zi.wall_set,
            "indoor": zi.indoor,
            "expected_hue": hue,
            "style_note": zi.style_note,
        }));
    }

    let mut luma_pairs = Vec::new();
    for b in &bright_zones {
        for m in &mid_zones {
            luma_pairs.push(json!([b, m]));
        }
        for d in &dark_zones {
            luma_pairs.push(json!([b, d]));
        }
    }

    let exits_json: Vec<Value> = model
        .exits
        .iter()
        .map(|e| {
            json!({
                "id": e.id,
                "label": e.label,
                "cell": [e.x, e.y],
                "norm": [round3((e.x as f64 + 0.5) / cw), round3((e.y as f64 + 0.5) / ch)],
                "wall_side": e.wall_side,
                "severity": "HARD",
            })
        })
        .collect();

    // Group static props by (zone_id, art) into machine-checkable clauses
    let mut grouped: IndexMap<(String, String), Vec<&PropInfo>> = IndexMap::new();
    let mut dynamic_exclusions = Vec::new();
    for p in &model.props {
        if p.dynamic {
            dynamic_exclusions.push(json!({
                "kind": "dynamic_prop",
                "id": p.id,
                "art": p.art,
                "reason": "state-gated runtime prop; omit from static reference",
            }));
            continue;
        }
        grouped.entry((p.zone_id.clone(), p.art.clone())).or_default().push(p);
    }
    for npc in &model.npcs {
        dynamic_exclusions.push(json!({
            "kind": "npc",
            "id": npc,
            "reason": "runtime character sprite; never bake characters into map diorama",
        }));
    }

    let mut required_clauses = Vec::new();
    for ((zid, art), items) in &grouped {
        let boxes: Vec<Value> = items
            .iter()
            .map(|p| {
                json!({
                    "id": p.id,
                    "cell": [p.x, p.y, p.w, p.h],
                    "norm": [
                        round3(p.x as f64 / cw),
                        round3(p.y as f64 / ch),
                        round3((p.x + p.w) as f64 / cw),
                        round3((p.y + p.h) as f64 / ch)
                    ],
                })
            })
            .collect();
        let is_minor = art.contains("lantern") || art.contains("candle") || art.contains("crate");
        required_clauses.push(json!({
            "id": format!("{}.{}", if zid.is_empty() { "map" } else { zid.as_str() }, art),
            "zone": zid,
            "art": art,
            "count": items.len(),
            "severity": if is_minor { "SOFT" } else { "HARD" },
            "placements": boxes,
        }));
    }

    let forbidden_clauses = json!([
        {"id": "no_isometric_perspective", "severity": "HARD", "rule": "Front-on top-down orthographic view only; horizontal and vertical edges square to grid, zero 3/4 or isometric tilt."},
        {"id": "no_cast_oval_shadows", "severity": "HARD", "rule": "No soft oval drop shadows under props; thin base contact lines only."},
        {"id": "no_baked_characters", "severity": "HARD", "rule": "No NPCs, party members, or creatures baked into the diorama."},
        {"id": "no_ui_or_text", "severity": "HARD", "rule": "No labels, captions, UI bars, watermarks, or overlaid grid text."},
        {"id": "no_unlisted_major_props", "severity": "HARD", "rule": "No hallucinated architectural landmarks or machinery absent from required_clauses."}
    ]);

    json!({
        "schema": "aegis.refspec.v1",
        "map_id": model.id,
        "name": model.name,
        "grid": {
            "cols": model.cols,
            "rows": model.rows,
            "tile_px": 32,
            "aspect_square": round3(cw / ch),
            "aspect_tilted": round3((cw * 32.0) / (ch * 24.0)),
        },
        "zones": zones_json,
        "exits": exits_json,
        "required_clauses": required_clauses,
        "dynamic_exclusions": dynamic_exclusions,
        "forbidden_clauses": forbidden_clauses,
        "style_checks": {
            "min_ortho_hv_ratio": 1.65,
            "max_aspect_drift_pct": 25.0,
            "min_wall_floor_contrast": 8.0,
            "expected_zone_luma_pairs": luma_pairs,
        }
    })
}

fn render_prompt(model: &MapModel, spec: &Value) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Reference Diorama Prompt — {} ({})\n", model.name, model.id));
    out.push_str("# Compiled deterministically from maps.json + locations.json. Do not hand-edit without syncing refspec.\n\n");
    out.push_str("Top-down front-on orthographic 16-bit pixel-art RPG map diorama (south wall faces visible, every wall edge strictly horizontal or vertical, zero isometric tilt, zero characters/NPCs, zero UI/text, zero cast oval shadows).\n");
    out.push_str(&format!("Grid proportion: {} columns x {} rows.\n\n", model.cols, model.rows));

    out.push_str("ROOMS & MATERIALS:\n");
    if let Some(zones) = spec["zones"].as_array() {
        for z in zones {
            let id = z["id"].as_str().unwrap_or("");
            let b = &z["bbox_norm"];
            let g = z["dominant_ground"].as_str().unwrap_or("");
            let ws = z["wall_set"].as_str().unwrap_or("");
            let note = z["style_note"].as_str().unwrap_or("");
            out.push_str(&format!(
                "- Zone '{id}' (norm [{:.2},{:.2}]..[{:.2},{:.2}]): ground={g}, wall_set={ws}{}\n",
                b[0].as_f64().unwrap_or(0.0),
                b[1].as_f64().unwrap_or(0.0),
                b[2].as_f64().unwrap_or(1.0),
                b[3].as_f64().unwrap_or(1.0),
                if note.is_empty() { String::new() } else { format!(" — {note}") }
            ));
        }
    }

    out.push_str("\nREQUIRED EXITS (HARD):\n");
    if let Some(exits) = spec["exits"].as_array() {
        for e in exits {
            out.push_str(&format!(
                "- {} on {} wall (cell [{}, {}]): {}\n",
                e["id"].as_str().unwrap_or(""),
                e["wall_side"].as_str().unwrap_or(""),
                e["cell"][0],
                e["cell"][1],
                e["label"].as_str().unwrap_or("")
            ));
        }
    }

    out.push_str("\nREQUIRED STATIC LANDMARKS & PROPS:\n");
    if let Some(clauses) = spec["required_clauses"].as_array() {
        for c in clauses {
            out.push_str(&format!(
                "- [{}] {} x{} in zone '{}'\n",
                c["severity"].as_str().unwrap_or("HARD"),
                c["art"].as_str().unwrap_or(""),
                c["count"],
                c["zone"].as_str().unwrap_or("map")
            ));
        }
    }

    out.push_str("\nFORBIDDEN:\n");
    if let Some(forb) = spec["forbidden_clauses"].as_array() {
        for f in forb {
            out.push_str(&format!("- {}\n", f["rule"].as_str().unwrap_or("")));
        }
    }
    out
}

pub fn render_blockout(model: &MapModel, tile_px: usize) -> RgbaImg {
    let w = model.cols * tile_px;
    let h = model.rows * tile_px;
    let mut img = RgbaImg::new(w, h, [14, 14, 18, 255]);

    for y in 0..model.rows {
        for x in 0..model.cols {
            let t = model.tile(x, y);
            let c = if t == '#' {
                let z = if y + 1 < model.rows && model.tile(x, y + 1) != '#' { model.zone(x, y + 1) } else { model.zone(x, y) };
                let ws = model.zone_defs.get(z).map(|zi| zi.wall_set.as_str()).unwrap_or("");
                if ws.contains("plaster") {
                    [150, 148, 142, 255]
                } else if ws.contains("stone") {
                    [58, 64, 70, 255]
                } else {
                    [32, 34, 42, 255]
                }
            } else {
                let g = model.ground(x, y);
                let rgb = model.ground_colors.get(g).copied().unwrap_or(match t {
                    '~' => [26, 52, 62],
                    _ => [72, 74, 78],
                });
                // Lift dark catalog colors slightly so the blockout reads clearly as a generation guide
                [(rgb[0] as u16 * 3 / 2).min(230) as u8, (rgb[1] as u16 * 3 / 2).min(230) as u8, (rgb[2] as u16 * 3 / 2).min(230) as u8, 255]
            };
            img.fill_rect(x * tile_px, y * tile_px, tile_px, tile_px, c);
            img.stroke_rect(x * tile_px, y * tile_px, tile_px, tile_px, 1, [0, 0, 0, 40]);
        }
    }
    // Static prop footprints
    for p in &model.props {
        if p.dynamic {
            continue;
        }
        let col = if p.layer == "ground" { [160, 140, 90, 150] } else { [210, 165, 95, 195] };
        img.fill_rect(p.x * tile_px + 2, p.y * tile_px + 2, (p.w * tile_px).saturating_sub(4), (p.h * tile_px).saturating_sub(4), col);
        img.stroke_rect(p.x * tile_px + 1, p.y * tile_px + 1, (p.w * tile_px).saturating_sub(2), (p.h * tile_px).saturating_sub(2), 1, [30, 20, 12, 255]);
    }
    // Exits highlighted in bright cyan
    for e in &model.exits {
        img.stroke_rect(e.x * tile_px + 2, e.y * tile_px + 2, tile_px.saturating_sub(4), tile_px.saturating_sub(4), 3, [80, 240, 255, 255]);
    }
    img
}

// ---- Grid-Quantized Reference Registration -------------------------------------------------------------------------

#[derive(Debug, Clone)]
pub struct GridRegistration {
    pub trim: (usize, usize, usize, usize), // (x0, y0, x1, y1) exclusive
    pub x_stops: Vec<f32>,                  // len = cols + 1
    pub y_stops: Vec<f32>,                  // len = rows + 1
    pub snapped_cols: Vec<usize>,
    pub snapped_rows: Vec<usize>,
}

impl GridRegistration {
    #[inline]
    pub fn cell_rect(&self, x: usize, y: usize) -> (usize, usize, usize, usize) {
        let x0 = self.x_stops[x].round().max(0.0) as usize;
        let x1 = self.x_stops[x + 1].round().max((x0 + 1) as f32) as usize;
        let y0 = self.y_stops[y].round().max(0.0) as usize;
        let y1 = self.y_stops[y + 1].round().max((y0 + 1) as f32) as usize;
        (x0, y0, x1, y1)
    }
}

/// Trims dark outer void borders and snaps major interior wall columns/rows of `model` onto the reference image's
/// dark wall-band projection minima, yielding piecewise-linear cell stops `(x_stops, y_stops)`.
pub fn register_reference(img: &RgbaImg, model: &MapModel) -> GridRegistration {
    // 1. Trim solid-black outer margin (luminance < 10)
    let row_luma = |y: usize| -> f32 {
        let mut s = 0.0f32;
        for x in 0..img.w {
            let p = img.get(x, y);
            s += luma709(p[0] as f32, p[1] as f32, p[2] as f32);
        }
        s / img.w.max(1) as f32
    };
    let col_luma = |x: usize| -> f32 {
        let mut s = 0.0f32;
        for y in 0..img.h {
            let p = img.get(x, y);
            s += luma709(p[0] as f32, p[1] as f32, p[2] as f32);
        }
        s / img.h.max(1) as f32
    };
    let thresh = 10.0f32;
    let mut y0 = 0usize;
    while y0 + 8 < img.h && row_luma(y0) < thresh {
        y0 += 1;
    }
    let mut y1 = img.h;
    while y1 > y0 + 8 && row_luma(y1 - 1) < thresh {
        y1 -= 1;
    }
    let mut x0 = 0usize;
    while x0 + 8 < img.w && col_luma(x0) < thresh {
        x0 += 1;
    }
    let mut x1 = img.w;
    while x1 > x0 + 8 && col_luma(x1 - 1) < thresh {
        x1 -= 1;
    }

    let tw = (x1 - x0) as f32;
    let th = (y1 - y0) as f32;

    // 2. Find major interior wall divider columns & rows (>= 58% `#` in interior)
    let mut wall_cols = Vec::new();
    for x in 2..model.cols.saturating_sub(2) {
        let wall_cnt = (1..model.rows - 1).filter(|&y| model.tile(x, y) == '#').count();
        if wall_cnt * 100 >= (model.rows.saturating_sub(2)) * 58 {
            wall_cols.push(x);
        }
    }
    let mut wall_rows = Vec::new();
    for y in 2..model.rows.saturating_sub(2) {
        let wall_cnt = (1..model.cols - 1).filter(|&x| model.tile(x, y) == '#').count();
        if wall_cnt * 100 >= (model.cols.saturating_sub(2)) * 58 {
            wall_rows.push(y);
        }
    }

    // Group contiguous divider bands into anchor midpoints
    let group_bands = |indices: &[usize]| -> Vec<(usize, usize)> {
        let mut bands = Vec::new();
        let mut i = 0;
        while i < indices.len() {
            let start = indices[i];
            let mut end = start;
            while i + 1 < indices.len() && indices[i + 1] == end + 1 {
                i += 1;
                end = indices[i];
            }
            bands.push((start, end + 1)); // half-open [start, end+1)
            i += 1;
        }
        bands
    };

    // Snap X divider centers to dark vertical projection minima within +-6% of image width
    let col_prof: Vec<f32> = (x0..x1).map(col_luma).collect();
    let row_prof: Vec<f32> = (y0..y1).map(row_luma).collect();
    let x_anchors = snap_axis_anchors(model.cols, x0 as f32, tw, &group_bands(&wall_cols), &col_prof);
    let y_anchors = snap_axis_anchors(model.rows, y0 as f32, th, &group_bands(&wall_rows), &row_prof);

    let x_stops = interpolate_stops(model.cols, &x_anchors);
    let y_stops = interpolate_stops(model.rows, &y_anchors);
    GridRegistration { trim: (x0, y0, x1, y1), x_stops, y_stops, snapped_cols: wall_cols, snapped_rows: wall_rows }
}

fn snap_axis_anchors(cells: usize, origin: f32, span: f32, bands: &[(usize, usize)], prof: &[f32]) -> Vec<(f32, f32)> {
    let mut anchors = vec![(0.0f32, origin)];
    let search = ((prof.len() as f32 * 0.06).round() as isize).max(4);
    for &(b0, b1) in bands {
        let mid_cell = (b0 + b1) as f32 * 0.5;
        let expected_rel = (mid_cell / cells as f32) * prof.len() as f32;
        let band_w = (((b1 - b0) as f32 / cells as f32) * prof.len() as f32 * 0.5).max(2.0) as isize;
        let lo = (expected_rel.round() as isize - search).max(band_w);
        let hi = (expected_rel.round() as isize + search).min(prof.len() as isize - band_w - 1);
        let mut best_pos = expected_rel;
        let mut best_val = f32::MAX;
        for c in lo..=hi {
            let mut sum = 0.0f32;
            let mut cnt = 0;
            for d in -band_w..=band_w {
                let idx = (c + d).clamp(0, prof.len() as isize - 1) as usize;
                sum += prof[idx];
                cnt += 1;
            }
            let mean = sum / cnt.max(1) as f32;
            // Small quadratic prior toward expected position avoids jumping into a dark water basin
            let dist = (c as f32 - expected_rel) / search.max(1) as f32;
            let score = mean + dist * dist * 8.0;
            if score < best_val {
                best_val = score;
                best_pos = c as f32;
            }
        }
        let px = origin + best_pos;
        if px > anchors.last().unwrap().1 + 4.0 && px < origin + span - 4.0 {
            anchors.push((mid_cell, px));
        }
    }
    anchors.push((cells as f32, origin + span));
    anchors
}

fn interpolate_stops(cells: usize, anchors: &[(f32, f32)]) -> Vec<f32> {
    let mut stops = Vec::with_capacity(cells + 1);
    for i in 0..=cells {
        let c = i as f32;
        let mut val = anchors[0].1;
        for w in anchors.windows(2) {
            let (c0, p0) = w[0];
            let (c1, p1) = w[1];
            if c >= c0 - 1e-4 && c <= c1 + 1e-4 {
                let t = if (c1 - c0).abs() > 1e-5 { (c - c0) / (c1 - c0) } else { 0.0 };
                val = p0 + (p1 - p0) * t;
                break;
            }
        }
        stops.push(val);
    }
    stops
}

fn warp_reference_to_render(ref_img: &RgbaImg, reg: &GridRegistration, out_w: usize, out_h: usize) -> RgbaImg {
    let cols = reg.x_stops.len().saturating_sub(1).max(1);
    let rows = reg.y_stops.len().saturating_sub(1).max(1);
    let mut out = RgbaImg::new(out_w, out_h, [0, 0, 0, 255]);
    for y in 0..out_h {
        let gy = (y as f32 + 0.5) / out_h as f32 * rows as f32;
        let cy = (gy.floor() as usize).min(rows - 1);
        let ty = gy - cy as f32;
        let ry = (reg.y_stops[cy] + (reg.y_stops[cy + 1] - reg.y_stops[cy]) * ty).round().clamp(0.0, (ref_img.h - 1) as f32) as usize;
        for x in 0..out_w {
            let gx = (x as f32 + 0.5) / out_w as f32 * cols as f32;
            let cx = (gx.floor() as usize).min(cols - 1);
            let tx = gx - cx as f32;
            let rx = (reg.x_stops[cx] + (reg.x_stops[cx + 1] - reg.x_stops[cx]) * tx).round().clamp(0.0, (ref_img.w - 1) as f32) as usize;
            out.set(x, y, ref_img.get(rx, ry));
        }
    }
    out
}

// ---- Per-cell feature extraction -----------------------------------------------------------------------------------

#[derive(Debug, Clone, Default)]
pub struct CellStats {
    pub rgb: [f32; 3],
    pub luma: f32,
    pub edge: f32,
    pub rgb_std: f32,
}

fn extract_grid_stats(img: &RgbaImg, reg: &GridRegistration, cols: usize, rows: usize) -> Vec<CellStats> {
    let mut out = Vec::with_capacity(cols * rows);
    for y in 0..rows {
        for x in 0..cols {
            let (x0, y0, x1, y1) = reg.cell_rect(x, y);
            let x1 = x1.min(img.w);
            let y1 = y1.min(img.h);
            if x1 <= x0 || y1 <= y0 {
                out.push(CellStats::default());
                continue;
            }
            let (mut sr, mut sg, mut sb, mut sl, mut sl2, mut se) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            let mut n = 0usize;
            for py in y0..y1 {
                for px in x0..x1 {
                    let p = img.get(px, py);
                    let (r, g, b) = (p[0] as f64, p[1] as f64, p[2] as f64);
                    let l = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    sr += r;
                    sg += g;
                    sb += b;
                    sl += l;
                    sl2 += l * l;
                    if px + 1 < x1 && py + 1 < y1 {
                        let pr = img.get(px + 1, py);
                        let pd = img.get(px, py + 1);
                        let lr = 0.2126 * pr[0] as f64 + 0.7152 * pr[1] as f64 + 0.0722 * pr[2] as f64;
                        let ld = 0.2126 * pd[0] as f64 + 0.7152 * pd[1] as f64 + 0.0722 * pd[2] as f64;
                        se += (l - lr).abs() + (l - ld).abs();
                    }
                    n += 1;
                }
            }
            let nf = n.max(1) as f64;
            let mean_l = sl / nf;
            let var_l = (sl2 / nf - mean_l * mean_l).max(0.0);
            out.push(CellStats {
                rgb: [(sr / nf) as f32, (sg / nf) as f32, (sb / nf) as f32],
                luma: mean_l as f32,
                edge: (se / nf) as f32,
                rgb_std: var_l.sqrt() as f32,
            });
        }
    }
    out
}

fn uniform_registration(w: usize, h: usize, cols: usize, rows: usize) -> GridRegistration {
    let x_stops = (0..=cols).map(|i| i as f32 * w as f32 / cols as f32).collect();
    let y_stops = (0..=rows).map(|i| i as f32 * h as f32 / rows as f32).collect();
    GridRegistration { trim: (0, 0, w, h), x_stops, y_stops, snapped_cols: Vec::new(), snapped_rows: Vec::new() }
}

/// Computes horizontal/vertical vs diagonal Sobel gradient energy ratio (high for front-on orthographic pixel art).
pub fn orthographic_hv_ratio(img: &RgbaImg, trim: (usize, usize, usize, usize)) -> f32 {
    let (x0, y0, x1, y1) = trim;
    if x1 <= x0 + 2 || y1 <= y0 + 2 {
        return 0.0;
    }
    let (mut hv, mut diag) = (0.0f64, 0.0f64);
    let sample_l = |x: usize, y: usize| -> f64 {
        let p = img.get(x, y);
        0.2126 * p[0] as f64 + 0.7152 * p[1] as f64 + 0.0722 * p[2] as f64
    };
    for y in (y0 + 1)..(y1 - 1) {
        for x in (x0 + 1)..(x1 - 1) {
            let gx = -sample_l(x - 1, y - 1) + sample_l(x + 1, y - 1)
                - 2.0 * sample_l(x - 1, y) + 2.0 * sample_l(x + 1, y)
                - sample_l(x - 1, y + 1) + sample_l(x + 1, y + 1);
            let gy = -sample_l(x - 1, y - 1) - 2.0 * sample_l(x, y - 1) - sample_l(x + 1, y - 1)
                + sample_l(x - 1, y + 1) + 2.0 * sample_l(x, y + 1) + sample_l(x + 1, y + 1);
            let mag = (gx * gx + gy * gy).sqrt();
            if mag < 24.0 {
                continue;
            }
            let ax = gx.abs();
            let ay = gy.abs();
            // Near H or V (tan(20°) ≈ 0.364) vs near 45° diagonal (0.55..1.8)
            if ax > ay * 2.5 || ay > ax * 2.5 {
                hv += mag;
            } else if ax > ay * 0.55 && ay > ax * 0.55 {
                diag += mag;
            }
        }
    }
    if diag < 1.0 {
        return 9.99;
    }
    (hv / diag) as f32
}

// ---- Stage B: Reference Image Validation Gate (`check-ref`) --------------------------------------------------------

fn draw_grid_fit(ref_img: &RgbaImg, reg: &GridRegistration, model: &MapModel) -> RgbaImg {
    let mut vis = ref_img.clone();
    // Draw subtle cyan grid lines
    for (ci, &x) in reg.x_stops.iter().enumerate() {
        let px = (x.round() as usize).min(vis.w.saturating_sub(1));
        let col = if reg.snapped_cols.contains(&ci) { [255, 220, 40, 220] } else { [0, 210, 240, 90] };
        for y in reg.trim.1..reg.trim.3 {
            vis.blend(px, y, col);
        }
    }
    for (ri, &y) in reg.y_stops.iter().enumerate() {
        let py = (y.round() as usize).min(vis.h.saturating_sub(1));
        let col = if reg.snapped_rows.contains(&ri) { [255, 220, 40, 220] } else { [0, 210, 240, 90] };
        for x in reg.trim.0..reg.trim.2 {
            vis.blend(x, py, col);
        }
    }
    // Highlight exits in bright red
    for e in &model.exits {
        let (x0, y0, x1, y1) = reg.cell_rect(e.x, e.y);
        vis.stroke_rect(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0), 2, [255, 60, 60, 255]);
    }
    vis
}

fn check_reference(ref_img: &RgbaImg, model: &MapModel, spec: &Value, out_dir: &Path) -> Result<Value> {
    let reg = register_reference(ref_img, model);
    let stats = extract_grid_stats(ref_img, &reg, model.cols, model.rows);
    save_png(&draw_grid_fit(ref_img, &reg, model), &out_dir.join("grid_fit.png"))?;

    let mut det_checks = Vec::new();

    // 1. Aspect fit
    let tw = (reg.trim.2 - reg.trim.0) as f64;
    let th = (reg.trim.3 - reg.trim.1) as f64;
    let ref_aspect = tw / th.max(1.0);
    let sq_aspect = model.cols as f64 / model.rows as f64;
    let tilt_aspect = (model.cols as f64 * 32.0) / (model.rows as f64 * 24.0);
    let sq_drift = ((ref_aspect - sq_aspect).abs() / sq_aspect) * 100.0;
    let tilt_drift = ((ref_aspect - tilt_aspect).abs() / tilt_aspect) * 100.0;
    let best_drift = sq_drift.min(tilt_drift);
    let max_drift = spec["style_checks"]["max_aspect_drift_pct"].as_f64().unwrap_or(25.0);
    det_checks.push(json!({
        "id": "aspect_fit",
        "status": if best_drift <= max_drift { "PASS" } else { "FAIL" },
        "measured": round3(ref_aspect),
        "expected_square": round3(sq_aspect),
        "expected_tilted": round3(tilt_aspect),
        "best_drift_pct": round2(best_drift),
    }));

    // 2. Orthographic H/V edge ratio
    let hv = orthographic_hv_ratio(ref_img, reg.trim) as f64;
    let min_hv = spec["style_checks"]["min_ortho_hv_ratio"].as_f64().unwrap_or(1.65);
    det_checks.push(json!({
        "id": "orthographic_hv_ratio",
        "status": if hv >= min_hv { "PASS" } else { "FAIL" },
        "measured": round2(hv),
        "threshold": min_hv,
    }));

    // 3. Wall vs floor contrast
    let (mut w_sum, mut w_n, mut f_sum, mut f_n) = (0.0f64, 0usize, 0.0f64, 0usize);
    for y in 0..model.rows {
        for x in 0..model.cols {
            let l = stats[y * model.cols + x].luma as f64;
            if model.tile(x, y) == '#' {
                w_sum += l;
                w_n += 1;
            } else if model.tile(x, y) == '.' {
                f_sum += l;
                f_n += 1;
            }
        }
    }
    let wall_l = w_sum / w_n.max(1) as f64;
    let floor_l = f_sum / f_n.max(1) as f64;
    let wf_contrast = floor_l - wall_l;
    let min_wf = spec["style_checks"]["min_wall_floor_contrast"].as_f64().unwrap_or(8.0);
    det_checks.push(json!({
        "id": "wall_floor_contrast",
        "status": if wf_contrast >= min_wf { "PASS" } else { "WARN" },
        "wall_mean_l": round2(wall_l),
        "floor_mean_l": round2(floor_l),
        "contrast": round2(wf_contrast),
    }));

    // 4. Per-zone material/hue & luminance ordering
    let mut zone_metrics: BTreeMap<String, ([f32; 3], f32)> = BTreeMap::new();
    let zones_dir = out_dir.join("zones");
    std::fs::create_dir_all(&zones_dir)?;
    for zi in model.zone_defs.values() {
        let mut floor_cells: Vec<&CellStats> = Vec::new();
        for y in zi.y0..=zi.y1 {
            for x in zi.x0..=zi.x1 {
                if model.zone(x, y) == zi.id && model.tile(x, y) != '#' {
                    floor_cells.push(&stats[y * model.cols + x]);
                }
            }
        }
        // Take the quietest 65% by edge energy to isolate floor from props
        floor_cells.sort_by(|a, b| a.edge.total_cmp(&b.edge));
        let keep = ((floor_cells.len() as f32 * 0.65).ceil() as usize).max(1).min(floor_cells.len());
        let (mut r, mut g, mut b, mut l) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for c in &floor_cells[..keep] {
            r += c.rgb[0];
            g += c.rgb[1];
            b += c.rgb[2];
            l += c.luma;
        }
        let kf = keep.max(1) as f32;
        let mean_rgb = [r / kf, g / kf, b / kf];
        let mean_l = l / kf;
        zone_metrics.insert(zi.id.clone(), (mean_rgb, mean_l));

        let hue = infer_expected_hue(zi);
        let hue_ok = match hue {
            "cool_water" => mean_rgb[2] + 3.0 >= mean_rgb[0],
            "warm_wood" => mean_rgb[0] > mean_rgb[2] + 6.0,
            "dark_moss" => mean_rgb[1] + 2.0 >= mean_rgb[0] && mean_l < 110.0,
            "bright_plaster" => mean_l > 65.0,
            _ => true,
        };
        det_checks.push(json!({
            "id": format!("zone_material.{}", zi.id),
            "status": if hue_ok { "PASS" } else { "WARN" },
            "expected_hue": hue,
            "floor_mean_rgb": [round1(mean_rgb[0] as f64), round1(mean_rgb[1] as f64), round1(mean_rgb[2] as f64)],
            "floor_mean_l": round2(mean_l as f64),
        }));

        // Export padded zone crop for Layer 2 vision audit
        let cx0 = reg.x_stops[zi.x0.saturating_sub(1)].floor().max(0.0) as usize;
        let cy0 = reg.y_stops[zi.y0.saturating_sub(1)].floor().max(0.0) as usize;
        let cx1 = (reg.x_stops[(zi.x1 + 2).min(model.cols)].ceil() as usize).min(ref_img.w);
        let cy1 = (reg.y_stops[(zi.y1 + 2).min(model.rows)].ceil() as usize).min(ref_img.h);
        if cx1 > cx0 && cy1 > cy0 {
            let crop = ref_img.crop(cx0, cy0, cx1 - cx0, cy1 - cy0);
            save_png(&crop, &zones_dir.join(format!("{}.png", zi.id)))?;
        }
    }

    if let Some(pairs) = spec["style_checks"]["expected_zone_luma_pairs"].as_array() {
        for pair in pairs {
            if let (Some(hi), Some(lo)) = (pair[0].as_str(), pair[1].as_str()) {
                if let (Some((_, lh)), Some((_, ll))) = (zone_metrics.get(hi), zone_metrics.get(lo)) {
                    det_checks.push(json!({
                        "id": format!("zone_luma_order.{hi}_gt_{lo}"),
                        "status": if lh > ll { "PASS" } else { "FAIL" },
                        "hi_l": round2(*lh as f64),
                        "lo_l": round2(*ll as f64),
                    }));
                }
            }
        }
    }

    // 5. Foreground presence hints per required clause + exit checks
    let fg_ref = compute_foreground_mask(&stats, model);
    let mut clause_audit = Vec::new();
    for e in &model.exits {
        let (x0, y0, x1, y1) = reg.cell_rect(e.x, e.y);
        let st = &stats[e.y * model.cols + e.x];
        // Check if exit cell shows structural detail / contrast rather than flat void
        let has_detail = st.edge > 8.0 || st.luma > (wall_l as f32 + 12.0);
        clause_audit.push(json!({
            "id": format!("exit.{}", e.id),
            "kind": "exit",
            "severity": "HARD",
            "wall_side": e.wall_side,
            "label": e.label,
            "cell": [e.x, e.y],
            "pixel_box": [x0, y0, x1, y1],
            "deterministic_hint": if has_detail { "FEATURE_DETECTED" } else { "SUSPICIOUS_FLAT_OR_MISSING" },
            "status": "PENDING_VISION",
        }));
    }
    if let Some(clauses) = spec["required_clauses"].as_array() {
        for c in clauses {
            let zid = c["zone"].as_str().unwrap_or("");
            let mut present = 0usize;
            let mut total = 0usize;
            if let Some(pls) = c["placements"].as_array() {
                for p in pls {
                    let px = p["cell"][0].as_u64().unwrap_or(0) as usize;
                    let py = p["cell"][1].as_u64().unwrap_or(0) as usize;
                    let pw = p["cell"][2].as_u64().unwrap_or(1) as usize;
                    let ph = p["cell"][3].as_u64().unwrap_or(1) as usize;
                    let mut hit = false;
                    for dy in 0..ph {
                        for dx in 0..pw {
                            let xx = (px + dx).min(model.cols - 1);
                            let yy = (py + dy).min(model.rows - 1);
                            if fg_ref[yy * model.cols + xx] > 0.35 || stats[yy * model.cols + xx].edge > 14.0 {
                                hit = true;
                            }
                        }
                    }
                    if hit {
                        present += 1;
                    }
                    total += 1;
                }
            }
            clause_audit.push(json!({
                "id": c["id"],
                "kind": "prop",
                "severity": c["severity"],
                "zone": zid,
                "art": c["art"],
                "expected_count": c["count"],
                "zone_crop": if zid.is_empty() { Value::Null } else { json!(format!("zones/{zid}.png")) },
                "deterministic_fg_hits": format!("{present}/{total}"),
                "deterministic_hint": if present == total { "SILHOUETTE_PRESENT" } else if present > 0 { "PARTIAL_PRESENCE" } else { "SUSPICIOUS_EMPTY" },
                "status": "PENDING_VISION",
            }));
        }
    }

    let fail_n = det_checks.iter().filter(|c| c["status"] == "FAIL").count();
    let warn_n = det_checks.iter().filter(|c| c["status"] == "WARN").count();
    let pass_n = det_checks.iter().filter(|c| c["status"] == "PASS").count();
    let suspicious_n = clause_audit.iter().filter(|c| c["deterministic_hint"].as_str().unwrap_or("").starts_with("SUSPICIOUS")).count();

    Ok(json!({
        "map_id": model.id,
        "deterministic_summary": {
            "pass": pass_n,
            "warn": warn_n,
            "fail": fail_n,
            "suspicious_clauses": suspicious_n,
            "gate_ready_for_vision": fail_n == 0,
        },
        "registration": {
            "trim": [reg.trim.0, reg.trim.1, reg.trim.2, reg.trim.3],
            "snapped_wall_cols": reg.snapped_cols,
            "snapped_wall_rows": reg.snapped_rows,
            "grid_fit_image": out_dir.join("grid_fit.png").display().to_string(),
        },
        "deterministic_checks": det_checks,
        "clauses": clause_audit,
        "forbidden_clauses": spec["forbidden_clauses"],
    }))
}

fn print_ref_audit_summary(audit: &Value, path: &Path) {
    let s = &audit["deterministic_summary"];
    println!(
        "check-ref ({}) -> PASS={} WARN={} FAIL={} suspicious_clauses={}",
        audit["map_id"].as_str().unwrap_or(""),
        s["pass"],
        s["warn"],
        s["fail"],
        s["suspicious_clauses"]
    );
    if let Some(arr) = audit["deterministic_checks"].as_array() {
        for c in arr {
            if c["status"] != "PASS" {
                println!("  [{}] {}: {}", c["status"].as_str().unwrap_or(""), c["id"].as_str().unwrap_or(""), c);
            }
        }
    }
    println!("  audit pack: {}", path.display());
}

// ---- Pass 1 + Pass 2 + Pass 3 Alignment Scorer ---------------------------------------------------------------------

pub struct AlignmentReport {
    pub json: Value,
    pub map_fx: Option<Value>,
}

/// Computes per-cell foreground/silhouette strength in `[0.0, 1.0]` by combining edge energy and RGB deviation from the
/// cell's own zone median floor color.
fn compute_foreground_mask(stats: &[CellStats], model: &MapModel) -> Vec<f32> {
    let mut zone_floor: BTreeMap<&str, ([f32; 3], f32)> = BTreeMap::new();
    for zi in model.zone_defs.values() {
        let mut cells: Vec<&CellStats> = Vec::new();
        for y in zi.y0..=zi.y1 {
            for x in zi.x0..=zi.x1 {
                if model.zone(x, y) == zi.id && model.tile(x, y) != '#' {
                    cells.push(&stats[y * model.cols + x]);
                }
            }
        }
        if cells.is_empty() {
            continue;
        }
        cells.sort_by(|a, b| a.edge.total_cmp(&b.edge));
        let keep = ((cells.len() as f32 * 0.55).ceil() as usize).max(1).min(cells.len());
        let (mut r, mut g, mut b, mut e) = (0.0f32, 0.0f32, 0.0f32, 0.0f32);
        for c in &cells[..keep] {
            r += c.rgb[0];
            g += c.rgb[1];
            b += c.rgb[2];
            e += c.edge;
        }
        let kf = keep as f32;
        zone_floor.insert(zi.id.as_str(), ([r / kf, g / kf, b / kf], e / kf));
    }

    let mut mask = vec![0.0f32; model.cols * model.rows];
    for y in 0..model.rows {
        for x in 0..model.cols {
            let idx = y * model.cols + x;
            if model.tile(x, y) == '#' {
                continue;
            }
            let st = &stats[idx];
            let (base_rgb, base_edge) = zone_floor.get(model.zone(x, y)).copied().unwrap_or(([48.0, 48.0, 48.0], 8.0));
            let dr = st.rgb[0] - base_rgb[0];
            let dg = st.rgb[1] - base_rgb[1];
            let db = st.rgb[2] - base_rgb[2];
            let color_dev = (dr * dr + dg * dg + db * db).sqrt();
            let edge_excess = (st.edge - base_edge).max(0.0);
            let score = ((color_dev / 42.0) * 0.55 + (edge_excess / 18.0) * 0.45).clamp(0.0, 1.0);
            mask[idx] = score;
        }
    }
    mask
}

/// Checks whether blocking cells in `blocked_extra` preserves BFS walkability between all walkable anchors.
fn anchors_stay_connected(model: &MapModel, blocked_extra: &[(usize, usize)]) -> bool {
    let mut blocked = vec![false; model.cols * model.rows];
    for y in 0..model.rows {
        for x in 0..model.cols {
            if model.tile(x, y) != '.' {
                blocked[y * model.cols + x] = true;
            }
        }
    }
    for &(bx, by) in blocked_extra {
        if bx < model.cols && by < model.rows {
            // Never step onto an anchor tile
            if model.anchors.iter().any(|(_, ax, ay)| *ax == bx && *ay == by) {
                return false;
            }
            blocked[by * model.cols + bx] = true;
        }
    }
    let open_anchors: Vec<(usize, usize)> = model
        .anchors
        .iter()
        .filter(|(_, ax, ay)| model.tile(*ax, *ay) == '.')
        .map(|(_, ax, ay)| (*ax, *ay))
        .collect();
    let Some(&start) = open_anchors.first() else {
        return true;
    };
    let mut vis = vec![false; model.cols * model.rows];
    let mut q = VecDeque::new();
    vis[start.1 * model.cols + start.0] = true;
    q.push_back(start);
    while let Some((x, y)) = q.pop_front() {
        for (nx, ny) in [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ] {
            if nx < model.cols && ny < model.rows {
                let ni = ny * model.cols + nx;
                if !blocked[ni] && !vis[ni] {
                    vis[ni] = true;
                    q.push_back((nx, ny));
                }
            }
        }
    }
    open_anchors.iter().all(|&(ax, ay)| vis[ay * model.cols + ax])
}

pub fn score_alignment(ref_img: &RgbaImg, render_img: &RgbaImg, model: &MapModel, pass_mode: &str, out_dir: &Path) -> Result<AlignmentReport> {
    let reg_ref = register_reference(ref_img, model);
    let reg_ren = uniform_registration(render_img.w, render_img.h, model.cols, model.rows);

    let ref_stats = extract_grid_stats(ref_img, &reg_ref, model.cols, model.rows);
    let ren_stats = extract_grid_stats(render_img, &reg_ren, model.cols, model.rows);

    save_png(&draw_grid_fit(ref_img, &reg_ref, model), &out_dir.join("grid_fit.png"))?;
    let ref_warped = warp_reference_to_render(ref_img, &reg_ref, render_img.w, render_img.h);
    save_png(&hconcat(&[&ref_warped, render_img], 6), &out_dir.join("comparison.png"))?;

    let run_p1 = pass_mode == "1" || pass_mode == "all";
    let run_p2 = pass_mode == "2" || pass_mode == "all";

    // ---- PASS 1: Grounds & Prop Silhouettes ----
    let fg_ref = compute_foreground_mask(&ref_stats, model);
    let fg_ren = compute_foreground_mask(&ren_stats, model);

    // Build ground color prototypes from rendered low-edge floor cells (or catalog fallback), scaled to reference brightness
    let mut g_samples: BTreeMap<&str, Vec<[f32; 3]>> = BTreeMap::new();
    for y in 0..model.rows {
        for x in 0..model.cols {
            let idx = y * model.cols + x;
            if model.tile(x, y) != '#' && fg_ren[idx] < 0.30 {
                let g = model.ground(x, y);
                if !g.is_empty() {
                    g_samples.entry(g).or_default().push(ref_stats[idx].rgb);
                }
            }
        }
    }
    let mut g_proto: BTreeMap<&str, [f32; 3]> = BTreeMap::new();
    for (g, list) in &g_samples {
        let (mut r, mut gr, mut b) = (0.0f32, 0.0f32, 0.0f32);
        for c in list {
            r += c[0];
            gr += c[1];
            b += c[2];
        }
        let n = list.len().max(1) as f32;
        g_proto.insert(g, [r / n, gr / n, b / n]);
    }

    let (mut ground_checked, mut ground_matched) = (0usize, 0usize);
    let mut ground_mismatches = Vec::new();
    for y in 0..model.rows {
        for x in 0..model.cols {
            let idx = y * model.cols + x;
            if model.tile(x, y) == '#' || fg_ref[idx] >= 0.42 || fg_ren[idx] >= 0.42 {
                continue;
            }
            let cur = model.ground(x, y);
            if cur.is_empty() || g_proto.len() < 2 {
                continue;
            }
            let c = ref_stats[idx].rgb;
            let dist = |p: [f32; 3]| {
                let (dr, dg, db) = (c[0] - p[0], c[1] - p[1], c[2] - p[2]);
                (dr * dr + dg * dg + db * db).sqrt()
            };
            let cur_d = g_proto.get(cur).map_or(999.0, |p| dist(*p));
            if let Some((best_g, best_p)) = g_proto.iter().min_by(|a, b| dist(*a.1).total_cmp(&dist(*b.1))) {
                let best_d = dist(*best_p);
                ground_checked += 1;
                if *best_g == cur || cur_d <= best_d + 10.0 {
                    ground_matched += 1;
                } else {
                    ground_mismatches.push(json!({
                        "cell": [x, y],
                        "zone": model.zone(x, y),
                        "current": cur,
                        "suggested": best_g,
                        "delta": round1((cur_d - best_d) as f64),
                    }));
                }
            }
        }
    }
    ground_mismatches.sort_by(|a, b| b["delta"].as_f64().unwrap_or(0.0).total_cmp(&a["delta"].as_f64().unwrap_or(0.0)));
    ground_mismatches.truncate(24);
    let ground_agreement_pct = if ground_checked > 0 { ground_matched as f64 * 100.0 / ground_checked as f64 } else { 100.0 };

    // Per-prop silhouette IoU & bounded anchor-safe Tier 1 shift search
    let mut prop_reports = Vec::new();
    let mut iou_sum = 0.0f64;
    let mut iou_cnt = 0usize;
    let mut orphan_props = Vec::new();

    for p in &model.props {
        if p.dynamic {
            continue;
        }
        let eval_iou = |ox: isize, oy: isize| -> f32 {
            let (mut inter, mut uni) = (0.0f32, 0.0f32);
            for dy in 0..p.h as isize {
                for dx in 0..p.w as isize {
                    let x = ox + dx;
                    let y = oy + dy;
                    if x < 0 || y < 0 || x >= model.cols as isize || y >= model.rows as isize {
                        return 0.0;
                    }
                    let idx = y as usize * model.cols + x as usize;
                    let a = fg_ref[idx].max(if ref_stats[idx].edge > 14.0 { 0.55 } else { 0.0 });
                    let b = fg_ren[idx].max(if ren_stats[idx].edge > 14.0 { 0.55 } else { 0.0 });
                    inter += a.min(b);
                    uni += a.max(b);
                }
            }
            if uni > 1e-4 { inter / uni } else { 0.0 }
        };
        let cur_iou = eval_iou(p.x as isize, p.y as isize);
        let mut best_shift = (0isize, 0isize, cur_iou);
        if cur_iou < 0.55 {
            for dy in -2isize..=2 {
                for dx in -2isize..=2 {
                    if dx == 0 && dy == 0 {
                        continue;
                    }
                    let nx = p.x as isize + dx;
                    let ny = p.y as isize + dy;
                    if nx < 0 || ny < 0 || nx + p.w as isize > model.cols as isize || ny + p.h as isize > model.rows as isize {
                        continue;
                    }
                    // Keep shifts inside the same zone and anchor/walkability safe
                    let candidate_cells: Vec<(usize, usize)> = (0..p.h)
                        .flat_map(|ry| (0..p.w).map(move |rx| (nx as usize + rx, ny as usize + ry)))
                        .collect();
                    let same_zone = p.zone_id.is_empty()
                        || candidate_cells.iter().any(|&(cx, cy)| model.zone(cx, cy) == p.zone_id);
                    if !same_zone || !anchors_stay_connected(model, &candidate_cells) {
                        continue;
                    }
                    let cand = eval_iou(nx, ny);
                    if cand > best_shift.2 + 0.12 {
                        best_shift = (dx, dy, cand);
                    }
                }
            }
        }
        iou_sum += cur_iou as f64;
        iou_cnt += 1;
        if cur_iou < 0.25 {
            orphan_props.push(p.id.clone());
        }
        let mut entry = json!({
            "id": p.id,
            "art": p.art,
            "zone": p.zone_id,
            "cell": [p.x, p.y, p.w, p.h],
            "iou": round3(cur_iou as f64),
        });
        if best_shift.0 != 0 || best_shift.1 != 0 {
            entry["tier1_suggested_shift"] = json!({"dx": best_shift.0, "dy": best_shift.1, "projected_iou": round3(best_shift.2 as f64)});
        }
        prop_reports.push(entry);
    }

    // Detect unexplained reference foreground blobs (cells with strong ref fg, weak render fg, no prop footprint)
    let mut prop_occupied = vec![false; model.cols * model.rows];
    for p in &model.props {
        for dy in 0..p.h {
            for dx in 0..p.w {
                let xx = (p.x + dx).min(model.cols - 1);
                let yy = (p.y + dy).min(model.rows - 1);
                prop_occupied[yy * model.cols + xx] = true;
            }
        }
    }
    let mut unexplained_cells = Vec::new();
    for y in 0..model.rows {
        for x in 0..model.cols {
            let idx = y * model.cols + x;
            if model.tile(x, y) == '.' && !prop_occupied[idx] && fg_ref[idx] > 0.55 && fg_ren[idx] < 0.25 {
                unexplained_cells.push(json!({"cell": [x, y], "zone": model.zone(x, y), "fg_ref": round2(fg_ref[idx] as f64)}));
            }
        }
    }
    unexplained_cells.truncate(24);

    // Write Pass 1 silhouette debug overlay
    if run_p1 {
        let mut sil = render_img.clone();
        for y in 0..model.rows {
            for x in 0..model.cols {
                let idx = y * model.cols + x;
                if model.tile(x, y) == '#' {
                    continue;
                }
                let (x0, y0, x1, y1) = reg_ren.cell_rect(x, y);
                let (w, h) = (x1.saturating_sub(x0), y1.saturating_sub(y0));
                if fg_ref[idx] > 0.45 && fg_ren[idx] > 0.40 {
                    sil.fill_rect(x0, y0, w, h, [40, 220, 90, 85]); // matched
                } else if fg_ren[idx] > 0.45 && fg_ref[idx] < 0.25 {
                    sil.fill_rect(x0, y0, w, h, [235, 60, 60, 105]); // orphan render
                } else if fg_ref[idx] > 0.55 && fg_ren[idx] < 0.25 && !prop_occupied[idx] {
                    sil.fill_rect(x0, y0, w, h, [250, 210, 40, 110]); // unexplained ref blob
                }
            }
        }
        save_png(&sil, &out_dir.join("pass1_silhouettes.png"))?;
    }

    // Detect localized warm/cool light pools around light-emitting props in reference first so zone grades can
    // compensate for `ambience_dark` when light pools are enabled.
    let mut fx_lights = Map::new();
    for p in &model.props {
        if !is_light_prop(&p.art) {
            continue;
        }
        if let Some((rad, col, strength)) = estimate_prop_light(&ref_stats, model, p) {
            fx_lights.insert(
                p.id.clone(),
                json!({
                    "radius": round1(rad as f64),
                    "color": col,
                    "strength": round3(strength as f64),
                }),
            );
        }
    }
    let amb_dark = if !fx_lights.is_empty() { 0.14f32 } else { 0.0f32 };
    let amb_comp = 1.0 / (1.0 - amb_dark);

    // ---- PASS 2: Per-Zone Value, Palette & Atmosphere Solver ----
    let mut zone_reports = Vec::new();
    let mut fx_zones = Map::new();
    let (mut dl_sum, mut drgb_sum, mut z_cnt) = (0.0f64, 0.0f64, 0usize);

    for zi in model.zone_defs.values() {
        let (ref_floor, ref_wall) = zone_floor_wall_means(&ref_stats, &fg_ref, model, zi);
        let (ren_floor, ren_wall) = zone_floor_wall_means(&ren_stats, &fg_ren, model, zi);

        let ref_fl = luma709(ref_floor[0], ref_floor[1], ref_floor[2]);
        let ren_fl = luma709(ren_floor[0], ren_floor[1], ren_floor[2]);
        let dl = (ref_fl - ren_fl).abs() as f64;
        let drgb = ((ref_floor[0] - ren_floor[0]).powi(2) + (ref_floor[1] - ren_floor[1]).powi(2) + (ref_floor[2] - ren_floor[2]).powi(2)).sqrt() as f64;
        dl_sum += dl;
        drgb_sum += drgb;
        z_cnt += 1;

        // Solve bounded multiplicative grade keeping ground readable under sprites (target L <= 165)
        // and clamping inter-channel skew so neutral stone/wood never shifts into unnatural neon hues.
        let target_scale = if ref_fl > 165.0 { 165.0 / ref_fl } else { 1.0 };
        let solve_mul = |r_rgb: [f32; 3], e_rgb: [f32; 3]| -> [f32; 3] {
            let mut m = [1.0f32; 3];
            for c in 0..3 {
                let target = r_rgb[c] * target_scale * amb_comp;
                let denom = e_rgb[c].max(12.0);
                let raw = (target / denom).clamp(0.40, 2.35);
                m[c] = (1.0 + (raw - 1.0) * 0.92).clamp(0.40, 2.35);
            }
            let avg = (m[0] + m[1] + m[2]) / 3.0;
            let lo = (avg * 0.93).clamp(0.40, 2.35);
            let hi = (avg * 1.08).clamp(0.40, 2.35);
            for ch in &mut m {
                *ch = ch.clamp(lo, hi);
            }
            m
        };
        let floor_mul = solve_mul(ref_floor, ren_floor);
        let wall_mul = solve_mul(ref_wall, ren_wall);

        // Residual achromatic luminance lift only when floor_mul saturates at its clamp (never fights chroma clamp)
        let pred_l = luma709(ren_floor[0] * floor_mul[0], ren_floor[1] * floor_mul[1], ren_floor[2] * floor_mul[2]) * (1.0 - amb_dark);
        let l_err = ref_fl * target_scale - pred_l;
        let lift = if l_err.abs() > 6.0 { (l_err * 0.55).clamp(-18.0, 20.0) } else { 0.0 };
        let floor_add = [lift, lift, lift];

        let mut zo = Map::new();
        zo.insert("floor_mul".into(), json!([round3(floor_mul[0] as f64), round3(floor_mul[1] as f64), round3(floor_mul[2] as f64)]));
        zo.insert("wall_mul".into(), json!([round3(wall_mul[0] as f64), round3(wall_mul[1] as f64), round3(wall_mul[2] as f64)]));
        if floor_add.iter().any(|v| v.abs() > 0.5) {
            zo.insert("floor_add".into(), json!([round2(floor_add[0] as f64), round2(floor_add[1] as f64), round2(floor_add[2] as f64)]));
        }
        fx_zones.insert(zi.id.clone(), Value::Object(zo));

        zone_reports.push(json!({
            "zone": zi.id,
            "ref_floor_rgb": [round1(ref_floor[0] as f64), round1(ref_floor[1] as f64), round1(ref_floor[2] as f64)],
            "render_floor_rgb": [round1(ren_floor[0] as f64), round1(ren_floor[1] as f64), round1(ren_floor[2] as f64)],
            "delta_l": round2(dl),
            "delta_rgb": round2(drgb),
            "solved_floor_mul": [round3(floor_mul[0] as f64), round3(floor_mul[1] as f64), round3(floor_mul[2] as f64)],
            "solved_wall_mul": [round3(wall_mul[0] as f64), round3(wall_mul[1] as f64), round3(wall_mul[2] as f64)],
        }));
    }

    let map_fx = if run_p2 {
        let mut root = Map::new();
        root.insert("dim_ambience".into(), json!(!fx_lights.is_empty()));
        if !fx_lights.is_empty() {
            root.insert("ambience_dark".into(), json!(round3(amb_dark as f64)));
            root.insert("ambience_rgb".into(), json!([6, 8, 14]));
        }
        root.insert("zones".into(), Value::Object(fx_zones));
        if !fx_lights.is_empty() {
            root.insert("lights".into(), Value::Object(fx_lights));
        }
        Some(Value::Object(root))
    } else {
        None
    };

    // Write Pass 2 luminance delta heatmap
    if run_p2 {
        let mut heat = RgbaImg::new(render_img.w, render_img.h, [0, 0, 0, 255]);
        for y in 0..model.rows {
            for x in 0..model.cols {
                let idx = y * model.cols + x;
                let diff = ren_stats[idx].luma - ref_stats[idx].luma; // + = render too bright (red), - = too dark (blue)
                let mag = (diff.abs() * 3.0).clamp(0.0, 255.0) as u8;
                let col = if diff > 2.0 {
                    [mag, 32, 32, 255]
                } else if diff < -2.0 {
                    [32, 64, mag, 255]
                } else {
                    [36, 96, 48, 255]
                };
                let (x0, y0, x1, y1) = reg_ren.cell_rect(x, y);
                heat.fill_rect(x0, y0, x1.saturating_sub(x0), y1.saturating_sub(y0), col);
            }
        }
        save_png(&heat, &out_dir.join("pass2_luma_diff.png"))?;
    }

    // ---- PASS 3: Targeted Building-Block Flagging (Top 3 Worst-Diverging Local Crops) ----
    let mut residuals: Vec<(&PropInfo, f32, f32)> = Vec::new();
    for p in &model.props {
        if p.dynamic {
            continue;
        }
        let (mut l_err, mut e_err, mut n) = (0.0f32, 0.0f32, 0usize);
        for dy in 0..p.h {
            for dx in 0..p.w {
                let xx = (p.x + dx).min(model.cols - 1);
                let yy = (p.y + dy).min(model.rows - 1);
                let idx = yy * model.cols + xx;
                l_err += (ref_stats[idx].luma - ren_stats[idx].luma).abs();
                e_err += (ref_stats[idx].edge - ren_stats[idx].edge).abs();
                n += 1;
            }
        }
        let nf = n.max(1) as f32;
        let combined = (l_err / nf) * 0.6 + (e_err / nf) * 1.4;
        residuals.push((p, combined, l_err / nf));
    }
    residuals.sort_by(|a, b| b.1.total_cmp(&a.1));

    let mut flagged = Vec::new();
    let mut crop_rows = Vec::new();
    for &(p, score, l_err) in residuals.iter().take(3) {
        let px0 = p.x.saturating_sub(1);
        let py0 = p.y.saturating_sub(1);
        let px1 = (p.x + p.w + 1).min(model.cols);
        let py1 = (p.y + p.h + 1).min(model.rows);

        let rx0 = reg_ref.x_stops[px0].floor().max(0.0) as usize;
        let ry0 = reg_ref.y_stops[py0].floor().max(0.0) as usize;
        let rx1 = (reg_ref.x_stops[px1].ceil() as usize).min(ref_img.w);
        let ry1 = (reg_ref.y_stops[py1].ceil() as usize).min(ref_img.h);

        let ex0 = reg_ren.x_stops[px0].floor().max(0.0) as usize;
        let ey0 = reg_ren.y_stops[py0].floor().max(0.0) as usize;
        let ex1 = (reg_ren.x_stops[px1].ceil() as usize).min(render_img.w);
        let ey1 = (reg_ren.y_stops[py1].ceil() as usize).min(render_img.h);

        let rcrop = ref_img.crop(rx0, ry0, rx1.saturating_sub(rx0).max(1), ry1.saturating_sub(ry0).max(1)).resize_bilinear(160, 128);
        let ecrop = render_img.crop(ex0, ey0, ex1.saturating_sub(ex0).max(1), ey1.saturating_sub(ey0).max(1)).resize_bilinear(160, 128);
        let mut dcrop = RgbaImg::new(160, 128, [0, 0, 0, 255]);
        for y in 0..128 {
            for x in 0..160 {
                let a = rcrop.get(x, y);
                let b = ecrop.get(x, y);
                let diff = ((a[0] as i32 - b[0] as i32).abs() + (a[1] as i32 - b[1] as i32).abs() + (a[2] as i32 - b[2] as i32).abs()) / 3;
                let v = (diff * 2).min(255) as u8;
                dcrop.set(x, y, [v, v / 3, 255 - v, 255]);
            }
        }
        crop_rows.push(hconcat(&[&rcrop, &ecrop, &dcrop], 4));

        let is_orphan = orphan_props.contains(&p.id);
        let tier = if is_orphan {
            "tier1_placement_or_layout"
        } else if l_err > 28.0 {
            "tier2_grade_or_light"
        } else {
            "tier4_asset_block_review"
        };
        flagged.push(json!({
            "id": p.id,
            "art": p.art,
            "zone": p.zone_id,
            "cell": [p.x, p.y, p.w, p.h],
            "residual_score": round2(score as f64),
            "mean_luma_err": round2(l_err as f64),
            "reuse_first_tier": tier,
        }));
    }
    if !crop_rows.is_empty() {
        save_png(&vconcat(&crop_rows, 6), &out_dir.join("pass3_top_crops.png"))?;
    }

    let mean_zone_dl = dl_sum / z_cnt.max(1) as f64;
    let mean_zone_drgb = drgb_sum / z_cnt.max(1) as f64;
    let mean_prop_iou = iou_sum / iou_cnt.max(1) as f64;

    let report_json = json!({
        "map_id": model.id,
        "grid": [model.cols, model.rows],
        "pass1": {
            "ground_agreement_pct": round2(ground_agreement_pct),
            "ground_cells_checked": ground_checked,
            "top_ground_mismatches": ground_mismatches,
            "mean_prop_iou": round3(mean_prop_iou),
            "orphan_rendered_props": orphan_props,
            "unexplained_ref_blobs": unexplained_cells,
            "props": prop_reports,
        },
        "pass2": {
            "mean_zone_delta_l": round2(mean_zone_dl),
            "mean_zone_delta_rgb": round2(mean_zone_drgb),
            "zones": zone_reports,
        },
        "pass3": {
            "flagged_top_blocks": flagged,
            "contact_sheet": out_dir.join("pass3_top_crops.png").display().to_string(),
        }
    });

    Ok(AlignmentReport { json: report_json, map_fx })
}

fn zone_floor_wall_means(stats: &[CellStats], fg: &[f32], model: &MapModel, zi: &ZoneInfo) -> ([f32; 3], [f32; 3]) {
    let mut land_cells: Vec<&CellStats> = Vec::new();
    let mut any_open: Vec<&CellStats> = Vec::new();
    let mut w_cells: Vec<&CellStats> = Vec::new();
    for y in zi.y0..=zi.y1 {
        for x in zi.x0..=zi.x1 {
            let idx = y * model.cols + x;
            if model.zone(x, y) != zi.id {
                continue;
            }
            let ch = model.tile(x, y);
            if ch != '#' && fg[idx] < 0.40 {
                any_open.push(&stats[idx]);
                if ch == '.' {
                    land_cells.push(&stats[idx]);
                }
            }
            // Bordering north wall face directly above open cells of this zone
            if y > 0 && ch != '#' && model.tile(x, y - 1) == '#' {
                w_cells.push(&stats[(y - 1) * model.cols + x]);
            }
        }
    }
    // Prefer land (`.`) over water (`~`) when a zone contains both, so dark water never skews stone/wood grading.
    let f_cells = if land_cells.len() >= 4 {
        land_cells
    } else if !any_open.is_empty() {
        any_open
    } else {
        let mut fallback = Vec::new();
        for y in zi.y0..=zi.y1 {
            for x in zi.x0..=zi.x1 {
                if model.zone(x, y) == zi.id && model.tile(x, y) != '#' {
                    fallback.push(&stats[y * model.cols + x]);
                }
            }
        }
        fallback
    };
    let median_rgb = |list: &[&CellStats], fallback: [f32; 3]| -> [f32; 3] {
        if list.is_empty() {
            return fallback;
        }
        let mut out = [0.0f32; 3];
        for c in 0..3 {
            let mut vals: Vec<f32> = list.iter().map(|s| s.rgb[c]).collect();
            vals.sort_by(|a, b| a.total_cmp(b));
            out[c] = vals[vals.len() / 2];
        }
        out
    };
    let floor = median_rgb(&f_cells, [48.0, 48.0, 48.0]);
    let wall = median_rgb(&w_cells, floor);
    (floor, wall)
}

fn is_light_prop(art: &str) -> bool {
    art.contains("brazier")
        || art.contains("hearth")
        || art.contains("furnace")
        || art.contains("lantern")
        || art.contains("torch")
        || art.contains("candle")
        || art.contains("lamp")
        || art.contains("stove")
}

fn estimate_prop_light(stats: &[CellStats], model: &MapModel, p: &PropInfo) -> Option<(f32, [u8; 3], f32)> {
    let cx = p.x + p.w / 2;
    let cy = p.y + p.h / 2;
    let (mut near_l, mut near_rgb, mut near_n) = (0.0f32, [0.0f32; 3], 0usize);
    let (mut ring_l, mut ring_n) = (0.0f32, 0usize);
    for dy in -4isize..=4 {
        for dx in -4isize..=4 {
            let x = cx as isize + dx;
            let y = cy as isize + dy;
            if x < 0 || y < 0 || x >= model.cols as isize || y >= model.rows as isize {
                continue;
            }
            let (ux, uy) = (x as usize, y as usize);
            if model.tile(ux, uy) == '#' {
                continue;
            }
            let d2 = dx * dx + dy * dy;
            let st = &stats[uy * model.cols + ux];
            if d2 <= 2 {
                near_l += st.luma;
                for c in 0..3 {
                    near_rgb[c] += st.rgb[c];
                }
                near_n += 1;
            } else if (9..=16).contains(&d2) {
                ring_l += st.luma;
                ring_n += 1;
            }
        }
    }
    if near_n == 0 || ring_n == 0 {
        return None;
    }
    let lift = (near_l / near_n as f32) - (ring_l / ring_n as f32);
    if lift < 10.0 {
        return None;
    }
    let mr = near_rgb[0] / near_n as f32;
    let mb = near_rgb[2] / near_n as f32;
    let warm = mr > mb + 8.0;
    let col = if warm { [255, 175, 85] } else { [220, 228, 245] };
    let radius = (80.0 + lift * 1.4).clamp(72.0, 175.0);
    let strength = (0.14 + (lift / 120.0) * 0.22).clamp(0.14, 0.38);
    Some((radius, col, strength))
}

fn print_score_summary(report: &Value, path: &Path) {
    println!(
        "score ({}): Pass1 ground={:.1}% prop_iou={:.3} orphans={} | Pass2 mean_dL={:.2} mean_dRGB={:.2}",
        report["map_id"].as_str().unwrap_or(""),
        report["pass1"]["ground_agreement_pct"].as_f64().unwrap_or(0.0),
        report["pass1"]["mean_prop_iou"].as_f64().unwrap_or(0.0),
        report["pass1"]["orphan_rendered_props"].as_array().map_or(0, Vec::len),
        report["pass2"]["mean_zone_delta_l"].as_f64().unwrap_or(0.0),
        report["pass2"]["mean_zone_delta_rgb"].as_f64().unwrap_or(0.0),
    );
    if let Some(flagged) = report["pass3"]["flagged_top_blocks"].as_array() {
        for f in flagged {
            println!(
                "  Pass3 flag: {} ({}) zone={} residual={} -> {}",
                f["id"].as_str().unwrap_or(""),
                f["art"].as_str().unwrap_or(""),
                f["zone"].as_str().unwrap_or(""),
                f["residual_score"],
                f["reuse_first_tier"].as_str().unwrap_or("")
            );
        }
    }
    println!("  report: {}", path.display());
}

fn round1(x: f64) -> f64 {
    (x * 10.0).round() / 10.0
}
fn round2(x: f64) -> f64 {
    (x * 100.0).round() / 100.0
}
fn round3(x: f64) -> f64 {
    (x * 1000.0).round() / 1000.0
}

// ---- Tests ---------------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_model() -> MapModel {
        let entry = json!({
            "id": "demo_vault",
            "name": "Demo Vault",
            "rows": [
                "############",
                "#....##....#",
                "#....##....#",
                "#..........#",
                "#....##....#",
                "############"
            ],
            "zones": [
                "------------",
                "-aaaa--bbbb-",
                "-aaaa--bbbb-",
                "-aaaaabbbbb-",
                "-aaaa--bbbb-",
                "------------"
            ],
            "zone_legend": {
                "a": {"id": "bright_room", "kind": "safe", "indoor": true, "wall_set": "interior_plaster"},
                "b": {"id": "dark_room", "kind": "safe", "indoor": true, "wall_set": "interior_stone"}
            },
            "ground": [
                "ffffffffffff",
                "ffffffffffmm",
                "ffffffffffmm",
                "ffffffffffmm",
                "ffffffffffmm",
                "ffffffffffff"
            ],
            "ground_legend": { "f": "flagstone", "m": "moss" },
            "anchors": [
                {"id": "spawn", "x": 2, "y": 2},
                {"id": "gate", "x": 10, "y": 3}
            ],
            "props": [
                {"id": "bench_a", "art": "work_bench", "x": 2, "y": 1, "w": 2, "h": 1, "layer": "sort", "flip": false},
                {"id": "brazier_1", "art": "brazier", "x": 8, "y": 2, "w": 1, "h": 1, "layer": "sort", "flip": false}
            ]
        });
        let mut gcol = IndexMap::new();
        gcol.insert("flagstone".into(), [60, 62, 66]);
        gcol.insert("moss".into(), [34, 46, 32]);
        let mut styles = BTreeMap::new();
        styles.insert("bright_room".into(), "whitewashed bright flagstone".into());
        styles.insert("dark_room".into(), "moss patches, unlit".into());
        MapModel::from_json(&entry, None, gcol, &styles).unwrap()
    }

    #[test]
    fn refspec_and_blockout_capture_zones_exits_and_clauses() {
        let m = sample_model();
        let spec = build_refspec(&m);
        assert_eq!(spec["map_id"], "demo_vault");
        assert_eq!(spec["zones"].as_array().unwrap().len(), 2);
        assert_eq!(spec["exits"].as_array().unwrap().len(), 1);
        assert_eq!(spec["required_clauses"].as_array().unwrap().len(), 2);
        let pairs = spec["style_checks"]["expected_zone_luma_pairs"].as_array().unwrap();
        assert!(pairs.iter().any(|p| p[0] == "bright_room" && p[1] == "dark_room"));

        let block = render_blockout(&m, 16);
        assert_eq!((block.w, block.h), (12 * 16, 6 * 16));
    }

    #[test]
    fn orthographic_ratio_separates_axis_aligned_from_diagonal() {
        let mut ortho = RgbaImg::new(64, 64, [20, 20, 20, 255]);
        ortho.stroke_rect(8, 8, 48, 48, 2, [220, 220, 220, 255]);
        ortho.stroke_rect(20, 20, 24, 24, 2, [200, 200, 200, 255]);
        let hv_ortho = orthographic_hv_ratio(&ortho, (0, 0, 64, 64));

        let mut diag = RgbaImg::new(64, 64, [20, 20, 20, 255]);
        for k in 4..60 {
            diag.fill_rect(k, k, 2, 2, [220, 220, 220, 255]);
            diag.fill_rect(k, 63 - k, 2, 2, [220, 220, 220, 255]);
        }
        let hv_diag = orthographic_hv_ratio(&diag, (0, 0, 64, 64));
        assert!(hv_ortho > 3.0, "ortho ratio should be high: {hv_ortho}");
        assert!(hv_diag < 0.8, "diagonal ratio should be low: {hv_diag}");
    }

    #[test]
    fn pass2_solver_recovers_bright_and_dark_room_grades() {
        let m = sample_model();
        let (w, h) = (m.cols * 16, m.rows * 16);
        // Rendered map: both rooms flat mid-grey (60, 60, 60)
        let mut ren = RgbaImg::new(w, h, [20, 20, 20, 255]);
        // Reference: bright_room is (120, 120, 126), dark_room is (36, 44, 34)
        let mut ref_img = RgbaImg::new(w, h, [18, 18, 18, 255]);
        for y in 0..m.rows {
            for x in 0..m.cols {
                if m.tile(x, y) == '#' {
                    continue;
                }
                ren.fill_rect(x * 16, y * 16, 16, 16, [60, 60, 60, 255]);
                let rc = if m.zone(x, y) == "bright_room" { [120, 120, 126, 255] } else { [36, 44, 34, 255] };
                ref_img.fill_rect(x * 16, y * 16, 16, 16, rc);
            }
        }
        let scratch = aegis_core::scratch::Scratch::new("align_test");
        let rep = score_alignment(&ref_img, &ren, &m, "all", scratch.path()).unwrap();
        let fx = rep.map_fx.expect("solved map_fx");
        let bm = fx["zones"]["bright_room"]["floor_mul"][0].as_f64().unwrap();
        let dm = fx["zones"]["dark_room"]["floor_mul"][0].as_f64().unwrap();
        assert!(bm > 1.6, "bright_room should be lifted: {bm}");
        assert!(dm < 0.75, "dark_room should be dimmed: {dm}");
    }

    #[test]
    fn anchor_walkability_blocks_corridor_choking() {
        let m = sample_model();
        // Blocking the only doorway between spawn (2,2) and gate (10,3) at (5,3) must fail
        assert!(!anchors_stay_connected(&m, &[(5, 3)]));
        // Blocking an off-corridor tile (1,1) keeps connectivity intact
        assert!(anchors_stay_connected(&m, &[(1, 1)]));
    }
}
