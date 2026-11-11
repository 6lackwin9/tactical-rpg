//! PR5-26 (FB-193 (a)): the seed count of the party and sweep modes. `seeds=N` holds there as in every mode (it was
//! reset to the party default before); the positional count holds too; with neither the party default
//! (`args::PARTY_SEEDS`) applies. Scratch data built here (no shipped data).

use rpgkit_sim::args::{SimArgs, PARTY_SEEDS};
use rpgkit_world::Data;
use serde_json::json;

fn data() -> Data {
    Data::from_values(&json!({}), &json!({}), &json!({}))
}

fn parse(d: &Data, line: &str) -> Result<SimArgs, String> {
    SimArgs::parse(d, &line.split_whitespace().map(String::from).collect::<Vec<_>>())
}

#[test]
fn seeds_holds_in_party_and_sweep() {
    let d = data();
    let n = PARTY_SEEDS * 4;
    for mode in ["party", "sweep"] {
        assert_eq!(parse(&d, &format!("{mode} 5 seeds={n}")).unwrap().seed_count, n, "{mode} seeds=");
        assert_eq!(parse(&d, &format!("{mode} seeds={n} 5")).unwrap().seed_count, n, "{mode} seeds= first");
        assert_eq!(parse(&d, &format!("{mode} 5 {n}")).unwrap().seed_count, n, "{mode} positional");
        assert_eq!(parse(&d, &format!("{mode} 5")).unwrap().seed_count, PARTY_SEEDS, "{mode} default");
    }
}
