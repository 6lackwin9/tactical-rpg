//! GOLDEN-BISECT1 (ENGINE-14): the switch list's parsing and that a switch no list names is off.

use aegis_core::switches;

#[test]
fn the_list_is_comma_separated_and_trimmed() {
    let s = switches::parse(" regrant, retarget ,,x ");
    assert_eq!(s.into_iter().collect::<Vec<_>>(), ["regrant", "retarget", "x"]);
    assert!(switches::parse("").is_empty());
}

#[test]
fn a_switch_no_list_names_is_off() {
    // the gate runs with no AEGIS_SWITCHES; a name nobody sets is off either way
    assert!(!switches::on("golden_bisect_test_never_set"));
}
