//! ROBUST1 (ENGINE-10): string tables, for a game in more than one language. `strings/<language>.json` over the asset
//! layers (engine, kit, game, bottom-up; each layer sets only the keys it names): a flat object of `key: text`, where
//! `{name}` in a text is filled by `fmt`. The language is the settings key `language` (default `en`); a key missing in
//! it falls back to `en`, then to the key itself, so a missing string shows as its key and never as nothing.
//! `tr(key)` reads the tables in force on this thread (the shell loads them at start, `load_into`).
//! `check_layers` is the gate's check: every language holds the keys `en` holds, with the same `{name}`s, and no more.

use aegis_core::assets::Layers;
use aegis_core::vfs;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

pub const DIR: &str = "strings";
pub const FALLBACK: &str = "en";
/// The settings key naming the player's language.
pub const SETTINGS_KEY: &str = "language";

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Strings {
    pub language: String,
    texts: BTreeMap<String, String>,
    fallback: BTreeMap<String, String>,
}

/// The table of `language` over the layers, merged bottom-up, and the problems found reading it.
pub fn table(layers: &Layers, language: &str) -> (BTreeMap<String, String>, Vec<String>) {
    let mut out = BTreeMap::new();
    let mut problems = Vec::new();
    for root in layers.roots.iter().rev() {
        let path = root.join(DIR).join(format!("{language}.json"));
        if !vfs::is_file(&path) {
            continue;
        }
        match vfs::read_to_string(&path).ok().and_then(|t| serde_json::from_str::<Value>(&t).ok()) {
            Some(Value::Object(o)) => {
                for (k, v) in o {
                    match v {
                        Value::String(s) => {
                            out.insert(k, s);
                        }
                        _ => problems.push(format!("{}: '{k}' is not a text", path.display())),
                    }
                }
            }
            _ => problems.push(format!("{}: not a JSON object of key: text", path.display())),
        }
    }
    (out, problems)
}

/// The languages any layer has a table for, sorted.
pub fn languages(layers: &Layers) -> Vec<String> {
    let mut out: Vec<String> = layers.list(DIR).into_iter().filter_map(|f| f.strip_suffix(".json").map(str::to_string)).collect();
    out.sort();
    out.dedup();
    out
}

impl Strings {
    /// The tables of `language` (and the fallback) over `layers`.
    pub fn load(layers: &Layers, language: &str) -> (Strings, Vec<String>) {
        let (fallback, mut problems) = table(layers, FALLBACK);
        let texts = if language == FALLBACK {
            fallback.clone()
        } else {
            let (t, p) = table(layers, language);
            problems.extend(p);
            if t.is_empty() {
                problems.push(format!("no string table for the language '{language}' ({DIR}/{language}.json): {FALLBACK} is used"));
            }
            t
        };
        (Strings { language: language.to_string(), texts, fallback }, problems)
    }

    /// The text of `key`: in the language, else the fallback, else the key itself.
    pub fn get<'a>(&'a self, key: &'a str) -> &'a str {
        self.texts.get(key).or_else(|| self.fallback.get(key)).map(String::as_str).unwrap_or(key)
    }

    /// The text of `key` with each `{name}` filled from `args`.
    pub fn fmt(&self, key: &str, args: &[(&str, &str)]) -> String {
        let mut s = self.get(key).to_string();
        for (n, v) in args {
            s = s.replace(&format!("{{{n}}}"), v);
        }
        s
    }
}

/// The `{name}`s in a text, sorted.
pub fn placeholders(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = text;
    while let Some(i) = rest.find('{') {
        rest = &rest[i + 1..];
        match rest.find('}') {
            Some(j) if rest[..j].chars().all(|c| c.is_ascii_alphanumeric() || c == '_') && j > 0 => {
                out.push(rest[..j].to_string());
                rest = &rest[j + 1..];
            }
            _ => {}
        }
    }
    out.sort();
    out.dedup();
    out
}

/// The gate's check of the tables over `layers` (see the module).
pub fn check_layers(layers: &Layers) -> Vec<String> {
    let langs = languages(layers);
    if langs.is_empty() {
        return Vec::new();
    }
    let (base, mut problems) = table(layers, FALLBACK);
    if base.is_empty() {
        problems.push(format!("string tables {langs:?} but no {DIR}/{FALLBACK}.json, the fallback"));
        return problems;
    }
    for lang in langs.iter().filter(|l| *l != FALLBACK) {
        let (t, p) = table(layers, lang);
        problems.extend(p);
        for (k, text) in &base {
            match t.get(k) {
                None => problems.push(format!("{DIR}/{lang}.json: no '{k}' ({FALLBACK} has it)")),
                Some(other) if placeholders(other) != placeholders(text) => problems.push(format!("{DIR}/{lang}.json: '{k}' fills {:?}, {FALLBACK} fills {:?}", placeholders(other), placeholders(text))),
                Some(_) => {}
            }
        }
        for k in t.keys().filter(|k| !base.contains_key(*k)) {
            problems.push(format!("{DIR}/{lang}.json: '{k}' is not in {FALLBACK}"));
        }
    }
    problems
}

thread_local! {
    static CURRENT: RefCell<Rc<Strings>> = RefCell::new(Rc::new(Strings::default()));
}

/// Puts `s` in force on this thread.
pub fn install(s: Strings) {
    CURRENT.with(|c| *c.borrow_mut() = Rc::new(s));
}

/// The tables in force.
pub fn current() -> Rc<Strings> {
    CURRENT.with(|c| c.borrow().clone())
}

/// The text of `key` in the tables in force (the key itself when no table has it).
pub fn tr(key: &str) -> String {
    current().get(key).to_string()
}

/// `tr` with `{name}`s filled.
pub fn tr_with(key: &str, args: &[(&str, &str)]) -> String {
    current().fmt(key, args)
}

/// Loads the tables of the settings' language over the context's asset layers and puts them in force; problems go to
/// the notice (the shell does this at start).
pub fn load_into<G>(cx: &mut crate::screen::Cx<G>) {
    let lang = cx.settings.v.get(SETTINGS_KEY).and_then(Value::as_str).unwrap_or(FALLBACK).to_string();
    let (s, problems) = Strings::load(cx.art.layers(), &lang);
    cx.notice.extend(problems);
    install(s);
}
