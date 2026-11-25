//! Self-contained batch visual contact-sheet generator for `aegis align-map batch-sheet`.
//!
//! Game-agnostic: collects automated alignment/audit run folders (`ref_audit.json`, `run_summary.json`,
//! `report.json`, `layers_report.json`) and generates a single interactive HTML contact sheet plus
//! `batch_review.json` summary so a human reviewer can approve an entire asset batch by exception
//! (e.g., "Approve all except #3 and #7") without micro-approving individual files.

use crate::Args;
use anyhow::{bail, Context, Result};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

/// Preferred visual preview images in priority order for each alignment or sprite run directory.
const PREVIEW_CANDIDATES: &[&str] = &[
    "before_after_ref.png",
    "layer_sheet.png",
    "render_pass2.png",
    "l0_water_land_check.png",
    "l3_l2_ratio_map.png",
    "lighting_ratio_map.png",
    "prop_diff_mask.png",
    "render_before.png",
    "after/pass3_top_crops.png",
    "pass3_top_crops.png",
    "sheet.png",
    "preview.png",
    "sliced.png",
    "keyed.png",
];

/// Summary of one asset/map run directory included in a batch review sheet.
#[derive(Debug, Clone)]
pub struct BatchItem {
    pub index: usize,
    pub id: String,
    pub dir: PathBuf,
    pub status: String, // "PASS" | "WARN" | "FAIL" | "INFO"
    pub stage: Option<String>,
    pub badges: Vec<String>,
    pub issues: Vec<String>,
    pub previews: Vec<(String, PathBuf)>, // (label, file_path)
}

/// CLI entrypoint: `aegis align-map batch-sheet (dirs=a,b,c | root=<dir>) [out=<batch_review.html>] [title=<text>] [embed=0|1]`.
pub fn cmd_batch_sheet(args: &Args) -> Result<()> {
    cmd_batch_sheet_with_defaults(args, "out/align/batch_review.html", "Batch Asset Review Sheet")
}

/// Shared CLI entrypoint with caller-supplied default output path and sheet title.
pub fn cmd_batch_sheet_with_defaults(args: &Args, default_out: &str, default_title: &str) -> Result<()> {
    let dirs = collect_dirs(args)?;
    if dirs.is_empty() {
        bail!("batch-sheet: no run directories found (pass dirs=a,b or root=<parent_dir>)");
    }
    let out_html = PathBuf::from(args.key("out", default_out));
    let title = args.key("title", default_title);
    let embed = args.key("embed", "0") == "1";

    let items = build_batch_items(&dirs);
    let summary = write_batch_sheet(&items, &out_html, &title, embed)?;
    let counts = &summary["counts"];
    println!(
        "batch-sheet: {} item(s) ({} PASS, {} WARN, {} FAIL, {} INFO) -> {}",
        counts["total"],
        counts["pass"],
        counts["warn"],
        counts["fail"],
        counts["info"],
        out_html.display()
    );
    Ok(())
}

fn collect_dirs(args: &Args) -> Result<Vec<PathBuf>> {
    let mut dirs = Vec::new();
    if let Some(raw) = args.keys.get("dirs") {
        for part in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let p = PathBuf::from(part);
            if !p.is_dir() {
                bail!("batch-sheet: directory not found: {}", p.display());
            }
            dirs.push(p);
        }
    }
    if let Some(root_s) = args.keys.get("root") {
        let root = Path::new(root_s);
        if !root.is_dir() {
            bail!("batch-sheet: root directory not found: {}", root.display());
        }
        let mut found: Vec<PathBuf> = std::fs::read_dir(root)
            .with_context(|| format!("reading {}", root.display()))?
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.is_dir() && is_run_dir(p))
            .collect();
        found.sort();
        dirs.extend(found);
    }
    Ok(dirs)
}

pub(crate) fn is_run_dir(dir: &Path) -> bool {
    ["ref_audit.json", "run_summary.json", "report.json", "layers_report.json", "sprite_check.json"]
        .iter()
        .any(|f| dir.join(f).is_file())
        || PREVIEW_CANDIDATES.iter().any(|f| dir.join(f).is_file())
}

fn read_json(path: &Path) -> Option<Value> {
    let text = std::fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

/// Inspects each run directory and extracts normalized status, metrics, issues, and preview images.
pub fn build_batch_items(dirs: &[PathBuf]) -> Vec<BatchItem> {
    dirs.iter()
        .enumerate()
        .map(|(i, dir)| inspect_run_dir(i + 1, dir))
        .collect()
}

fn inspect_run_dir(index: usize, dir: &Path) -> BatchItem {
    let ref_audit = read_json(&dir.join("ref_audit.json"));
    let run_summary = read_json(&dir.join("run_summary.json"));
    let score_report = read_json(&dir.join("report.json"));
    let layers_report = read_json(&dir.join("layers_report.json"));
    let sprite_check = read_json(&dir.join("sprite_check.json"));

    let fallback_id = dir
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or("item")
        .to_string();
    let id = [
        ref_audit.as_ref(),
        run_summary.as_ref(),
        score_report.as_ref(),
        layers_report.as_ref(),
        sprite_check.as_ref(),
    ]
    .into_iter()
    .flatten()
    .find_map(|v| {
        v.get("map_id")
            .or_else(|| v.get("map"))
            .or_else(|| v.get("sprite_id"))
            .or_else(|| v.get("id"))
            .and_then(Value::as_str)
    })
    .filter(|s| !s.is_empty())
    .map(ToString::to_string)
    .unwrap_or(fallback_id);

    let mut rank = 0u8; // 0 = INFO, 1 = PASS, 2 = WARN, 3 = FAIL
    let bump = |rank: &mut u8, st: &str| {
        let r = match st.to_ascii_uppercase().as_str() {
            "FAIL" | "ERROR" => 3,
            "WARN" | "WARNING" => 2,
            "PASS" | "OK" => 1,
            _ => 0,
        };
        if r > *rank {
            *rank = r;
        }
    };

    let mut stage = None;
    let mut badges = Vec::new();
    let mut issues = Vec::new();

    if let Some(ref a) = ref_audit {
        if let Some(st) = a.get("stage").and_then(Value::as_str) {
            stage = Some(st.to_string());
        }
        if let Some(ov) = a.get("overall").and_then(Value::as_str) {
            bump(&mut rank, ov);
        }
        if let Some(c) = a.get("counts").and_then(Value::as_object) {
            let p = c.get("pass").and_then(Value::as_u64).unwrap_or(0);
            let w = c.get("warn").and_then(Value::as_u64).unwrap_or(0);
            let f = c.get("fail").and_then(Value::as_u64).unwrap_or(0);
            badges.push(format!("audit {p}P/{w}W/{f}F"));
            if f > 0 {
                bump(&mut rank, "FAIL");
            } else if w > 0 {
                bump(&mut rank, "WARN");
            } else if p > 0 {
                bump(&mut rank, "PASS");
            }
        }
        if let Some(checks) = a.get("checks").and_then(Value::as_array) {
            for ch in checks {
                let st = ch.get("status").and_then(Value::as_str).unwrap_or("");
                let name = ch.get("name").and_then(Value::as_str).unwrap_or("check");
                let detail = ch.get("detail").and_then(Value::as_str).unwrap_or("");
                if matches!(st, "FAIL" | "WARN") {
                    issues.push(format!("{st} {name}: {detail}"));
                }
            }
        }
    }

    if let Some(ref r) = run_summary {
        let b_dl = r.get("before_mean_dl").and_then(Value::as_f64);
        let a_dl = r.get("after_mean_dl").and_then(Value::as_f64);
        if let (Some(b), Some(a)) = (b_dl, a_dl) {
            badges.push(format!("dL {b:.1} -> {a:.1}"));
            bump(&mut rank, if a <= 6.0 { "PASS" } else if a <= 12.0 { "WARN" } else { "FAIL" });
        }
        let b_rgb = r.get("before_mean_drgb").and_then(Value::as_f64);
        let a_rgb = r.get("after_mean_drgb").and_then(Value::as_f64);
        if let (Some(b), Some(a)) = (b_rgb, a_rgb) {
            badges.push(format!("dRGB {b:.1} -> {a:.1}"));
        }
        if let Some(src) = r.get("lighting_source").and_then(Value::as_str) {
            badges.push(format!("light: {src}"));
        }
    } else if let Some(ref s) = score_report {
        if let Some(dl) = s.pointer("/pass2/mean_dl").or_else(|| s.pointer("/summary/mean_dl")).and_then(Value::as_f64) {
            badges.push(format!("dL {dl:.1}"));
            bump(&mut rank, if dl <= 6.0 { "PASS" } else if dl <= 12.0 { "WARN" } else { "FAIL" });
        }
        if let Some(src) = s.pointer("/pass2/lighting_source").and_then(Value::as_str) {
            badges.push(format!("light: {src}"));
        }
    }

    if let Some(ref l) = layers_report {
        if let Some(obj) = l.get("layers").and_then(Value::as_object) {
            let mut layer_bits = Vec::new();
            for (k, v) in obj {
                let st = v.get("status").and_then(Value::as_str).unwrap_or("INFO");
                bump(&mut rank, st);
                let short = k.split('_').next().unwrap_or(k);
                layer_bits.push(format!("{short}:{st}"));
            }
            if !layer_bits.is_empty() {
                badges.push(layer_bits.join(" "));
            }
        }
    }

    if let Some(ref sc) = sprite_check {
        if stage.is_none() {
            stage = Some("sprite".to_string());
        }
        if let Some(st) = sc.get("status").and_then(Value::as_str) {
            bump(&mut rank, st);
        }
        if let Some(m) = sc.get("metrics").and_then(Value::as_object) {
            if let (Some(ne), Some(tot)) = (
                m.get("frames_non_empty").and_then(Value::as_u64),
                m.get("frames_total").and_then(Value::as_u64),
            ) {
                badges.push(format!("frames {ne}/{tot}"));
            }
            if let Some(drift) = m.get("max_foot_drift").and_then(Value::as_u64) {
                badges.push(format!("foot_drift {drift}px"));
            }
            if let Some(chroma) = m.get("max_chroma_pct").and_then(Value::as_f64) {
                badges.push(format!("chroma {chroma:.2}%"));
            }
        }
        if let Some(arr) = sc.get("issues").and_then(Value::as_array) {
            for iss in arr {
                if let Some(s) = iss.as_str() {
                    issues.push(s.to_string());
                } else if let Some(obj) = iss.as_object() {
                    let sev = obj.get("severity").and_then(Value::as_str).unwrap_or("INFO");
                    let msg = obj.get("message").and_then(Value::as_str).unwrap_or("");
                    if !msg.is_empty() {
                        issues.push(format!("{sev} {msg}"));
                    }
                }
            }
        }
    }

    let mut previews = Vec::new();
    for rel in PREVIEW_CANDIDATES {
        let p = dir.join(rel);
        if p.is_file() {
            let label = Path::new(rel)
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or(rel)
                .to_string();
            previews.push((label, p));
            if previews.len() >= 4 {
                break;
            }
        }
    }
    if previews.is_empty() {
        if let Ok(rd) = std::fs::read_dir(dir) {
            let mut pngs: Vec<PathBuf> = rd
                .filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| p.extension().and_then(|e| e.to_str()).is_some_and(|e| e.eq_ignore_ascii_case("png")))
                .collect();
            pngs.sort();
            for p in pngs.into_iter().take(4) {
                let label = p.file_stem().and_then(|s| s.to_str()).unwrap_or("image").to_string();
                previews.push((label, p));
            }
        }
    }

    let status = match rank {
        3 => "FAIL",
        2 => "WARN",
        1 => "PASS",
        _ => "INFO",
    }
    .to_string();

    BatchItem { index, id, dir: dir.to_path_buf(), status, stage, badges, issues, previews }
}

/// Writes the self-contained interactive HTML contact sheet to `out_html` and a sibling `<stem>.json` summary.
pub fn write_batch_sheet(items: &[BatchItem], out_html: &Path, title: &str, embed: bool) -> Result<Value> {
    if let Some(parent) = out_html.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let out_dir = out_html.parent().unwrap_or_else(|| Path::new("."));

    let (mut pass, mut warn, mut fail, mut info) = (0usize, 0usize, 0usize, 0usize);
    for it in items {
        match it.status.as_str() {
            "PASS" => pass += 1,
            "WARN" => warn += 1,
            "FAIL" => fail += 1,
            _ => info += 1,
        }
    }
    let summary = json!({
        "title": title,
        "counts": { "total": items.len(), "pass": pass, "warn": warn, "fail": fail, "info": info },
        "auto_gate_clean": fail == 0,
        "items": items.iter().map(|it| json!({
            "index": it.index,
            "id": it.id,
            "dir": it.dir.display().to_string(),
            "status": it.status,
            "stage": it.stage,
            "badges": it.badges,
            "issues": it.issues,
            "previews": it.previews.iter().map(|(l, p)| json!({"label": l, "path": p.display().to_string()})).collect::<Vec<_>>(),
        })).collect::<Vec<_>>(),
    });

    let html = render_html(items, title, out_dir, embed, (pass, warn, fail, info))?;
    std::fs::write(out_html, html)?;

    let out_json = out_html.with_extension("json");
    std::fs::write(&out_json, serde_json::to_string_pretty(&summary)? + "\n")?;
    Ok(summary)
}

fn render_html(
    items: &[BatchItem],
    title: &str,
    out_dir: &Path,
    embed: bool,
    (pass, warn, fail, info): (usize, usize, usize, usize),
) -> Result<String> {
    let mut cards = String::new();
    for it in items {
        let st_cls = it.status.to_ascii_lowercase();
        let stage_html = it
            .stage
            .as_deref()
            .map(|s| format!(r#"<span class="chip stage">{}</span>"#, esc(s)))
            .unwrap_or_default();
        let mut badge_html = String::new();
        for b in &it.badges {
            badge_html.push_str(&format!(r#"<span class="chip">{}</span>"#, esc(b)));
        }
        let mut issue_html = String::new();
        if !it.issues.is_empty() {
            issue_html.push_str(r#"<ul class="issues">"#);
            for msg in it.issues.iter().take(5) {
                issue_html.push_str(&format!("<li>{}</li>", esc(msg)));
            }
            issue_html.push_str("</ul>");
        }
        let mut thumbs_html = String::new();
        for (label, path) in &it.previews {
            let src = image_src(path, out_dir, embed)?;
            thumbs_html.push_str(&format!(
                r#"<figure class="thumb"><img src="{src}" alt="{lbl}" onclick="openLight(this.src,'#{idx} {id} — {lbl}')"><figcaption>{lbl}</figcaption></figure>"#,
                src = esc(&src),
                lbl = esc(label),
                idx = it.index,
                id = esc(&it.id),
            ));
        }
        if thumbs_html.is_empty() {
            thumbs_html.push_str(r#"<div class="empty">No preview PNGs in folder</div>"#);
        }
        cards.push_str(&format!(
            r#"<article class="card st-{st_cls}" data-idx="{idx}" data-id="{id}" data-flagged="0">
  <header class="card-head">
    <div class="title-row">
      <span class="idx">#{idx}</span>
      <h2>{id}</h2>
      {stage_html}
      <span class="pill pill-{st_cls}">{status}</span>
    </div>
    <button type="button" class="flag-btn" onclick="toggleFlag(this)">Flag exception</button>
  </header>
  <div class="chips">{badge_html}</div>
  {issue_html}
  <div class="thumbs">{thumbs_html}</div>
  <input class="note" type="text" placeholder="Optional exception note (e.g. doorway cluttered, too dark)..." oninput="updateDecision()">
</article>
"#,
            st_cls = st_cls,
            idx = it.index,
            id = esc(&it.id),
            stage_html = stage_html,
            status = esc(&it.status),
            badge_html = badge_html,
            issue_html = issue_html,
            thumbs_html = thumbs_html,
        ));
    }

    Ok(format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<style>
  :root {{ --bg:#111419; --card:#191e26; --line:#2c3442; --fg:#e6ebf2; --muted:#95a2b8; --pass:#2ea043; --warn:#d29922; --fail:#f85149; --info:#58a6ff; }}
  * {{ box-sizing:border-box; }}
  body {{ margin:0; font-family:system-ui,-apple-system,sans-serif; background:var(--bg); color:var(--fg); padding:20px; }}
  .top {{ display:flex; flex-wrap:wrap; gap:12px; align-items:center; justify-content:space-between; border-bottom:1px solid var(--line); padding-bottom:14px; margin-bottom:18px; }}
  .top h1 {{ margin:0; font-size:20px; }}
  .counts {{ display:flex; gap:8px; flex-wrap:wrap; }}
  .pill {{ font-size:12px; font-weight:700; padding:3px 9px; border-radius:999px; letter-spacing:0.03em; }}
  .pill-pass {{ background:#173822; color:#7ee787; }} .pill-warn {{ background:#3d2f12; color:#f2cc60; }}
  .pill-fail {{ background:#42191d; color:#ff7b72; }} .pill-info {{ background:#142b47; color:#79c0ff; }}
  .bar {{ display:flex; gap:10px; align-items:center; flex-wrap:wrap; background:var(--card); border:1px solid var(--line); border-radius:10px; padding:10px 14px; margin-bottom:18px; }}
  .bar code {{ flex:1; font-size:13px; color:#c9d6ea; word-break:break-word; }}
  .bar button {{ background:#2563eb; color:#fff; border:0; border-radius:7px; padding:7px 12px; font-weight:600; cursor:pointer; }}
  .grid {{ display:grid; grid-template-columns:repeat(auto-fill, minmax(420px, 1fr)); gap:16px; }}
  .card {{ background:var(--card); border:1px solid var(--line); border-radius:12px; padding:14px; display:flex; flex-direction:column; gap:10px; }}
  .card[data-flagged="1"] {{ border-color:var(--fail); box-shadow:0 0 0 1px var(--fail); }}
  .card-head {{ display:flex; justify-content:space-between; align-items:center; gap:8px; }}
  .title-row {{ display:flex; align-items:center; gap:8px; flex-wrap:wrap; }}
  .title-row h2 {{ margin:0; font-size:16px; }}
  .idx {{ font-weight:800; color:var(--info); font-size:15px; }}
  .flag-btn {{ background:#262e3a; color:var(--fg); border:1px solid var(--line); border-radius:7px; padding:5px 10px; cursor:pointer; font-size:12px; }}
  .card[data-flagged="1"] .flag-btn {{ background:#5c1e23; border-color:var(--fail); color:#ffd2ce; }}
  .chips {{ display:flex; flex-wrap:wrap; gap:6px; }}
  .chip {{ font-size:12px; background:#232a35; color:var(--muted); padding:3px 8px; border-radius:6px; }}
  .chip.stage {{ color:#d2a8ff; }}
  .issues {{ margin:0; padding-left:18px; color:#f2cc60; font-size:12px; }}
  .thumbs {{ display:grid; grid-template-columns:repeat(auto-fit, minmax(170px, 1fr)); gap:8px; }}
  .thumb {{ margin:0; background:#0d1015; border:1px solid var(--line); border-radius:8px; overflow:hidden; }}
  .thumb img {{ width:100%; height:150px; object-fit:contain; display:block; cursor:zoom-in; background:#0b0e13; }}
  .thumb figcaption {{ font-size:11px; color:var(--muted); padding:4px 7px; border-top:1px solid #202733; }}
  .empty {{ color:var(--muted); font-size:12px; padding:14px; text-align:center; border:1px dashed var(--line); border-radius:8px; }}
  .note {{ width:100%; background:#0f131a; color:var(--fg); border:1px solid var(--line); border-radius:7px; padding:7px 9px; font-size:12px; }}
  #light {{ position:fixed; inset:0; background:rgba(5,7,11,0.92); display:none; flex-direction:column; align-items:center; justify-content:center; padding:24px; z-index:99; }}
  #light img {{ max-width:95vw; max-height:85vh; object-fit:contain; border:1px solid var(--line); border-radius:8px; }}
  #light p {{ color:var(--fg); margin:10px 0 0; font-size:14px; }}
</style>
</head>
<body>
  <div class="top">
    <h1>{title}</h1>
    <div class="counts">
      <span class="pill pill-info">{total} total</span>
      <span class="pill pill-pass">{pass} PASS</span>
      <span class="pill pill-warn">{warn} WARN</span>
      <span class="pill pill-fail">{fail} FAIL</span>
      <span class="pill pill-info">{info} INFO</span>
    </div>
  </div>
  <div class="bar">
    <code id="decision">Approve all ({total} item(s))</code>
    <button type="button" onclick="copyDecision()">Copy decision</button>
  </div>
  <main class="grid">
{cards}
  </main>
  <div id="light" onclick="this.style.display='none'"><img id="light-img" alt=""><p id="light-cap"></p></div>
<script>
function toggleFlag(btn) {{
  const card = btn.closest('.card');
  const next = card.dataset.flagged === '1' ? '0' : '1';
  card.dataset.flagged = next;
  btn.textContent = next === '1' ? 'Flagged (click to clear)' : 'Flag exception';
  updateDecision();
}}
function updateDecision() {{
  const cards = [...document.querySelectorAll('.card')];
  const flagged = cards.filter(c => c.dataset.flagged === '1').map(c => {{
    const note = c.querySelector('.note').value.trim();
    return '#' + c.dataset.idx + ' ' + c.dataset.id + (note ? ' (' + note + ')' : '');
  }});
  const out = flagged.length === 0
    ? 'Approve all (' + cards.length + ' item(s))'
    : 'Approve all except ' + flagged.join(', ');
  document.getElementById('decision').textContent = out;
}}
function copyDecision() {{
  const txt = document.getElementById('decision').textContent;
  if (navigator.clipboard) navigator.clipboard.writeText(txt);
}}
function openLight(src, cap) {{
  document.getElementById('light-img').src = src;
  document.getElementById('light-cap').textContent = cap;
  document.getElementById('light').style.display = 'flex';
}}
</script>
</body>
</html>
"#,
        title = esc(title),
        total = items.len(),
        pass = pass,
        warn = warn,
        fail = fail,
        info = info,
        cards = cards,
    ))
}

fn image_src(img: &Path, out_dir: &Path, embed: bool) -> Result<String> {
    if embed {
        let bytes = std::fs::read(img).with_context(|| format!("reading {}", img.display()))?;
        return Ok(format!("data:image/png;base64,{}", encode_base64(&bytes)));
    }
    let rel = relative_path(out_dir, img);
    Ok(rel.to_string_lossy().replace('\\', "/"))
}

fn relative_path(base: &Path, target: &Path) -> PathBuf {
    let base_abs = std::fs::canonicalize(base).unwrap_or_else(|_| base.to_path_buf());
    let target_abs = std::fs::canonicalize(target).unwrap_or_else(|_| target.to_path_buf());
    let b: Vec<_> = base_abs.components().collect();
    let t: Vec<_> = target_abs.components().collect();
    let mut common = 0usize;
    while common < b.len() && common < t.len() && b[common] == t[common] {
        common += 1;
    }
    if common == 0 {
        return target.to_path_buf();
    }
    let mut out = PathBuf::new();
    for _ in common..b.len() {
        out.push("..");
    }
    for comp in &t[common..] {
        out.push(comp.as_os_str());
    }
    out
}

fn esc(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn encode_base64(bytes: &[u8]) -> String {
    const TABLE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = *chunk.get(1).unwrap_or(&0) as u32;
        let b2 = *chunk.get(2).unwrap_or(&0) as u32;
        let n = (b0 << 16) | (b1 << 8) | b2;
        out.push(TABLE[((n >> 18) & 63) as usize] as char);
        out.push(TABLE[((n >> 12) & 63) as usize] as char);
        out.push(if chunk.len() > 1 { TABLE[((n >> 6) & 63) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { TABLE[(n & 63) as usize] as char } else { '=' });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn batch_sheet_aggregates_audit_run_and_layers_into_html_and_json() {
        let scratch = aegis_core::scratch::Scratch::new("batch_sheet");
        let root = scratch.path().join("runs");
        let r1 = root.join("01_courtyard");
        let r2 = root.join("02_basin");
        std::fs::create_dir_all(&r1).unwrap();
        std::fs::create_dir_all(&r2).unwrap();

        std::fs::write(
            r1.join("ref_audit.json"),
            serde_json::to_string(&json!({
                "map_id": "demo_courtyard",
                "stage": "cleanplate",
                "overall": "PASS",
                "counts": {"pass": 5, "warn": 0, "fail": 0},
                "checks": [{"name": "l0_water_land_consistency", "status": "PASS", "detail": "97.2%"}]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(
            r1.join("run_summary.json"),
            serde_json::to_string(&json!({
                "map_id": "demo_courtyard",
                "before_mean_dl": 14.0,
                "after_mean_dl": 1.8,
                "before_mean_drgb": 28.0,
                "after_mean_drgb": 6.5,
                "lighting_source": "l3_l2_direct"
            }))
            .unwrap(),
        )
        .unwrap();
        // Minimal PNG signature bytes to test preview discovery & base64 embed
        std::fs::write(r1.join("before_after_ref.png"), b"\x89PNG\r\n\x1a\n").unwrap();

        std::fs::write(
            r2.join("ref_audit.json"),
            serde_json::to_string(&json!({
                "map_id": "demo_basin",
                "stage": "propped",
                "overall": "WARN",
                "counts": {"pass": 4, "warn": 1, "fail": 0},
                "checks": [{"name": "stage_drift_non_prop", "status": "WARN", "detail": "drift 8.2"}]
            }))
            .unwrap(),
        )
        .unwrap();
        std::fs::write(r2.join("layer_sheet.png"), b"\x89PNG\r\n\x1a\n").unwrap();

        let items = build_batch_items(&[r1.clone(), r2.clone()]);
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].id, "demo_courtyard");
        assert_eq!(items[0].status, "PASS");
        assert_eq!(items[1].id, "demo_basin");
        assert_eq!(items[1].status, "WARN");

        let out_html = scratch.path().join("review/batch_review.html");
        let summary = write_batch_sheet(&items, &out_html, "Test Batch", true).unwrap();
        assert_eq!(summary["counts"]["total"], 2);
        assert_eq!(summary["counts"]["pass"], 1);
        assert_eq!(summary["counts"]["warn"], 1);
        assert_eq!(summary["auto_gate_clean"], true);

        let html = std::fs::read_to_string(&out_html).unwrap();
        assert!(html.contains("demo_courtyard"));
        assert!(html.contains("demo_basin"));
        assert!(html.contains("data:image/png;base64,"));
        assert!(html.contains("Approve all except"));
        assert!(out_html.with_extension("json").is_file());
    }

    #[test]
    fn base64_and_html_escape_are_exact() {
        assert_eq!(encode_base64(b"hi"), "aGk=");
        assert_eq!(encode_base64(b"abc"), "YWJj");
        assert_eq!(esc("<a&\"'b>"), "&lt;a&amp;&quot;&#39;b&gt;");
    }
}
