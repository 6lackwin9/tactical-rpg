//! INPUT2: the screen keys' names (`aegis_ui::screen::Key`): Shift with a digit is its own key ("shift+1"), read by
//! the digit key's place whatever the layout types for it, and round-trips through `parse` and `name` as every key.

use aegis_ui::screen::Key;

#[test]
fn shift_digits_are_keys_of_their_own() {
    for d in 0..=9u8 {
        let k = Key::ShiftDigit(d);
        assert_eq!(k.name(), format!("shift+{d}"));
        assert_eq!(Key::parse(&k.name()), Some(k));
        assert_eq!(Key::parse(&format!("Shift+{d}")), Some(k), "case does not matter");
    }
    assert_eq!(Key::parse("1"), Some(Key::Char('1')), "a plain digit stays a character");
    assert_eq!(Key::parse("shift+a"), None, "only digits");
    assert_eq!(Key::parse("shift+tab"), Some(Key::BackTab));
    for k in [Key::Up, Key::Enter, Key::BackTab, Key::CtrlEnter, Key::Char('t')] {
        assert_eq!(Key::parse(&k.name()), Some(k), "{k:?}");
    }
}
