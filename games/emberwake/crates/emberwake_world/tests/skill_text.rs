//! FB-107 (W2-4, A1_VALUE acceptance 2): skill text as a ruleset shows it on the shipped catalog. Under a ruleset with
//! stances off (timeline) no skill description prints the stance tokens (`{flow}`, `{disrupt}`) nor leaves their
//! separator behind; under classic (stances on) the stored text is shown. Skills are picked by rule (their template
//! uses a stance token), never by id; the shipped data is only read.

use aegis_core::json::{self, Dict};
use emberwake_content::source::{self, STANCE_TOKENS};
use emberwake_rules::ruleset::Ruleset;
use emberwake_world::Data;
use std::path::Path;

fn data() -> Data {
    let dir = Data::find_data_dir(Path::new(env!("CARGO_MANIFEST_DIR"))).expect("data folder");
    Data::open(&dir).expect("data").0
}

fn uses_stance_token(raw: &Dict) -> bool {
    let template = json::s(raw, "template");
    STANCE_TOKENS.iter().any(|t| template.contains(&format!("{{{t}")))
}

/// The text each stance token renders as with stances on, for this row.
fn stance_texts(raw: &Dict) -> Vec<String> {
    let kind = json::s(raw, "kind");
    STANCE_TOKENS.iter().map(|t| source::render(&format!("{{{t}}}"), &kind, raw, 1, true)).filter(|s| !s.is_empty()).collect()
}

#[test]
fn timeline_skill_text_drops_the_stance_tokens_and_classic_keeps_them() {
    let d = data();
    let classic = Ruleset::of_or_classic(&d.cat, "classic");
    let timeline = Ruleset::of(&d.cat, "timeline").expect("the shipped timeline ruleset");
    assert!(classic.stances() && !timeline.stances(), "classic runs stances, timeline does not");
    let mut checked = 0;
    for sk in &d.cat.skills {
        let raw = &sk.raw;
        let stored = json::s(raw, "description");
        // classic: the stored catalog text, for every skill
        assert_eq!(source::skill_description(raw, classic.stances()), stored, "{}", sk.id);
        let shown = source::skill_description(raw, timeline.stances());
        if !uses_stance_token(raw) {
            assert_eq!(shown, stored, "{}: no stance token, text unchanged", sk.id);
            continue;
        }
        let texts = stance_texts(raw);
        if !texts.iter().any(|t| stored.contains(t.as_str())) {
            continue; // a rank with a literal text
        }
        checked += 1;
        for t in &texts {
            assert!(!shown.contains(t.as_str()), "{}: timeline shows '{t}': {shown}", sk.id);
        }
        assert!(!shown.contains(", .") && !shown.contains(" .") && !shown.contains(",."), "{}: separator left: {shown}", sk.id);
        assert!(texts.iter().any(|t| stored.contains(t.as_str())), "{}: classic shows the stance text: {stored}", sk.id);
    }
    assert!(checked > 0, "the catalog has skills whose text uses a stance token");
}
