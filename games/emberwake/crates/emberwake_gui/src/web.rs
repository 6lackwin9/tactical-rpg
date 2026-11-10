//! The web entry (WEBREL1-G1, `docs/specs/WEBREL1.md` 2.1, 3, 6.1): the library's `wasm-bindgen` start function.
//! It fetches the bundle the build wrote beside the page (`bundle.json`, then each part in its order; `aegis bundle`
//! packs them from `games/emberwake/web_bundle.json`), shows a loading bar on the page while the first part comes,
//! mounts it into the read seam (`vfs::mount`), reads the launch keys from the URL query (`?screen=arena&seed=3`, the
//! desktop's keys, `NOT_ON_WEB` dropped), and starts the shell on the page canvas `aegis` (`shell::start_web`). The
//! later parts (the audio) are fetched and mounted while the game runs; the audio looks its files up again then
//! (`Audio`'s caches follow `vfs::generation`). The folders are the repository's own paths inside the bundle, so the
//! desktop's lookups (`find_game_dir`, `Layers::for_game`) find them from the bundle's root.

use crate::{arena_ruleset, launch, Roots};
use aegis_core::vfs::{self, Table};
use anyhow::{anyhow, bail, Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::JsFuture;

/// The page canvas the shell draws on.
pub const CANVAS: &str = "aegis";
/// The bundle's list of parts, beside the page (`aegis_cli::bundle::INDEX`).
pub const BUNDLE: &str = "bundle.json";
/// The launch keys the web build does not take: the agent link, the headless run and the desktop's folders.
pub const NOT_ON_WEB: [&str; 7] = ["port", "headless", "fps", "dpr", "idle_exit", "data", "saves"];
/// The loading bar's element on the page (made here, removed once the first part is mounted).
const LOADING: &str = "aegis-loading";

fn js(e: JsValue) -> anyhow::Error {
    anyhow!("{}", e.as_string().or_else(|| js_sys::JSON::stringify(&e).ok().and_then(|s| s.as_string())).unwrap_or_else(|| format!("{e:?}")))
}

/// `v` as the JS type `T`, or the reason.
fn cast<T: JsCast>(v: impl Into<JsValue>) -> Result<T> {
    v.into().dyn_into::<T>().map_err(js)
}

fn document() -> Result<web_sys::Document> {
    web_sys::window().and_then(|w| w.document()).context("the page has no document")
}

/// The start function: runs `boot` on the browser's executor; a failure shows on the page and in the console.
#[wasm_bindgen(start)]
pub fn start() {
    std::panic::set_hook(Box::new(|info| web_sys::console::error_1(&format!("emberwake: {info}").into())));
    wasm_bindgen_futures::spawn_local(async {
        if let Err(e) = boot().await {
            let line = format!("The game could not start: {e:#}");
            web_sys::console::error_1(&line.clone().into());
            Loading::show_error(&line);
        }
    });
}

async fn boot() -> Result<()> {
    let bar = Loading::open()?;
    let index: Value = serde_json::from_slice(&fetch(BUNDLE, |_| {}).await?).context("reading bundle.json")?;
    let parts: Vec<(String, f64)> = index["parts"].as_array().context("bundle.json has no parts")?.iter().map(|p| (p["file"].as_str().unwrap_or_default().to_string(), p["bytes"].as_f64().unwrap_or(0.0))).collect();
    let ((first, size), later) = parts.split_first().context("bundle.json lists no part")?;
    let bytes = fetch(first, |got| bar.progress(got as f64, *size)).await?;
    vfs::mount(Table::from_pack(&bytes).map_err(|e| anyhow!("{first}: {e}"))?);
    drop(bytes);
    bar.close();
    let args = query_args();
    let ruleset = arena_ruleset(&args)?;
    let roots = Roots { start: PathBuf::new(), engine: Some(PathBuf::from("engine")) };
    let (cx, stack, options) = launch(&args, roots, ruleset, None)?;
    aegis_ui::shell::start_web(cx, stack, options, CANVAS)?;
    for (file, _) in later {
        match fetch(file, |_| {}).await.and_then(|b| Table::from_pack(&b).map_err(|e| anyhow!("{file}: {e}"))) {
            Ok(table) => vfs::mount(table),
            // A later part (the audio) that does not come leaves the game running without it.
            Err(e) => web_sys::console::warn_1(&format!("emberwake: {file} did not load: {e:#}").into()),
        }
    }
    Ok(())
}

/// The URL query as launch keys (`?screen=arena&seed=3`), URI-decoded; the keys the web build does not take are
/// dropped with a console line.
fn query_args() -> Vec<(String, String)> {
    let search = web_sys::window().and_then(|w| w.location().search().ok()).unwrap_or_default();
    let decode = |s: &str| js_sys::decode_uri_component(&s.replace('+', " ")).ok().and_then(|j| j.as_string()).unwrap_or_else(|| s.to_string());
    let mut args = Vec::new();
    for pair in search.trim_start_matches('?').split('&').filter(|p| !p.is_empty()) {
        let (k, v) = pair.split_once('=').unwrap_or((pair, ""));
        let (k, v) = (decode(k), decode(v));
        if NOT_ON_WEB.contains(&k.as_str()) {
            web_sys::console::warn_1(&format!("emberwake: the web build does not take {k}= (dropped)").into());
        } else {
            args.push((k, v));
        }
    }
    args
}

/// The file at `url` (beside the page), read chunk by chunk; `progress` hears the bytes read so far.
async fn fetch(url: &str, mut progress: impl FnMut(usize)) -> Result<Vec<u8>> {
    let window = web_sys::window().context("no window")?;
    let resp: web_sys::Response = JsFuture::from(window.fetch_with_str(url)).await.map_err(js)?.dyn_into().map_err(js)?;
    if !resp.ok() {
        bail!("{url}: HTTP {}", resp.status());
    }
    let reader: web_sys::ReadableStreamDefaultReader = cast(resp.body().with_context(|| format!("{url}: no body"))?.get_reader())?;
    let mut out = Vec::new();
    loop {
        let chunk = JsFuture::from(reader.read()).await.map_err(js)?;
        if js_sys::Reflect::get(&chunk, &"done".into()).map_err(js)?.as_bool().unwrap_or(true) {
            break;
        }
        let value: js_sys::Uint8Array = js_sys::Reflect::get(&chunk, &"value".into()).map_err(js)?.dyn_into().map_err(js)?;
        let at = out.len();
        out.resize(at + value.length() as usize, 0);
        value.copy_to(&mut out[at..]);
        progress(out.len());
    }
    Ok(out)
}

/// The loading bar: a box over the page (the canvas stays untouched, the renderer takes it later) with a bar and a
/// line "Loading 12.3 / 20.6 MB".
struct Loading {
    fill: web_sys::HtmlElement,
    text: web_sys::HtmlElement,
}

const BOX_STYLE: &str = "position:fixed;left:50%;top:50%;transform:translate(-50%,-50%);width:420px;padding:16px;background:#1b1712;color:#e8dcc0;font:14px sans-serif;border:1px solid #6b5a3a;z-index:10";

impl Loading {
    fn open() -> Result<Loading> {
        let doc = document()?;
        let host: web_sys::HtmlElement = cast(doc.create_element("div").map_err(js)?)?;
        host.set_id(LOADING);
        host.set_attribute("style", BOX_STYLE).map_err(js)?;
        host.set_inner_html("<div data-text>Loading</div><div style=\"margin-top:8px;height:10px;background:#3a3226\"><div data-fill style=\"height:100%;width:0;background:#d9a441\"></div></div>");
        doc.body().context("the page has no body")?.append_child(&host).map_err(js)?;
        let find = |sel: &str| -> Result<web_sys::HtmlElement> { cast(host.query_selector(sel).map_err(js)?.context("the loading bar")?) };
        Ok(Loading { fill: find("[data-fill]")?, text: find("[data-text]")? })
    }

    fn progress(&self, got: f64, total: f64) {
        let mb = |b: f64| b / 1_048_576.0;
        if total > 0.0 {
            let _ = self.fill.style().set_property("width", &format!("{:.1}%", (got / total * 100.0).min(100.0)));
            self.text.set_inner_text(&format!("Loading {:.1} / {:.1} MB", mb(got), mb(total)));
        } else {
            self.text.set_inner_text(&format!("Loading {:.1} MB", mb(got)));
        }
    }

    fn close(&self) {
        if let Ok(Some(el)) = document().map(|d| d.get_element_by_id(LOADING)) {
            el.remove();
        }
    }

    /// A start failure, on the page where the loading bar was (made again when it is gone).
    fn show_error(line: &str) {
        let Ok(doc) = document() else { return };
        let el = match doc.get_element_by_id(LOADING) {
            Some(el) => el,
            None => {
                let Ok(el) = doc.create_element("div") else { return };
                el.set_id(LOADING);
                let _ = el.set_attribute("style", BOX_STYLE);
                if let Some(body) = doc.body() {
                    let _ = body.append_child(&el);
                }
                el
            }
        };
        el.set_text_content(Some(line));
    }
}
