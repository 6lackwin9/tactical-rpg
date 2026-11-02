//! PLAYUI1-T: the kit's text editor (`ui::Edit` over a `ui::Layout`): moves by character, word and visual line with
//! the sticky column, selection, typing over it, clipboard keys, paste normalising, undo grouping and the scroll that
//! keeps the caret in view. Pure functions: the layout uses a fixed advance of one unit per character (no font), the
//! undo clock is set by hand (no wall clock), and the clipboard is a closure (never the system's).

use aegis_ui::ui::{reveal_offset, word_end, word_start, Edit, EditKey, Layout, UNDO_MAX, UNDO_PAUSE_MS};

/// `text` laid out with every character one unit wide, wrapped to `w` units.
fn lay(text: &str, w: f32, multiline: bool) -> Layout {
    Layout::new(text, w, multiline, &mut |_| 1.0)
}

fn spans(l: &Layout) -> Vec<(usize, usize)> {
    l.lines.iter().map(|v| (v.start, v.end)).collect()
}

fn key(e: &mut Edit, name: &str, l: &Layout) -> Option<String> {
    let (k, ctrl, shift) = EditKey::parse(name).unwrap_or_else(|| panic!("{name} is an editing key"));
    e.key(k, ctrl, shift, l, 3, || String::from("never asked"))
}

#[test]
fn an_edit_types_deletes_and_moves_by_characters() {
    let mut e = Edit::new("t", "héllo", false);
    assert_eq!(e.caret, 5, "the caret starts at the end");
    e.insert("!");
    e.left(false);
    e.left(false);
    e.backspace();
    assert_eq!((e.text.as_str(), e.caret), ("hélo!", 3), "keeps the multibyte é");
    let one = lay(&e.text, 100.0, false);
    e.home(&one, false);
    e.delete();
    let one = lay(&e.text, 100.0, false);
    e.end(&one, false);
    e.insert("\n x\t");
    assert_eq!(e.text, "élo! x", "typing into a one-line field drops the newline and the tab");
    let mut m = Edit::new("m", "a", true);
    m.insert("\nb");
    assert_eq!((m.text.as_str(), m.caret), ("a\nb", 3));
}

#[test]
fn a_text_area_wraps_at_spaces_breaks_long_words_and_keeps_paragraphs() {
    // "one two" fits 9 units; " three" would not, so it starts line 2; the break's space is not drawn.
    assert_eq!(spans(&lay("one two three four", 9.0, true)), [(0, 7), (8, 13), (14, 18)]);
    // A word wider than the line breaks where it must: the next line starts where the last ends.
    let long = lay("abcdefghijkl xyz", 5.0, true);
    assert_eq!(spans(&long), [(0, 5), (5, 10), (10, 12), (13, 16)], "\"kl xyz\" is 6 wide: xyz wraps");
    assert_eq!(spans(&lay("abcdefghijkl xy", 5.0, true)), [(0, 5), (5, 10), (10, 15)], "\"kl xy\" fits 5");
    assert_eq!(long.last_pos(0), 4, "inside a word the shared position belongs to the next line");
    assert_eq!(long.line_of(5), 1);
    // Newlines start paragraphs; an empty one is an empty line, a trailing newline a last empty line.
    assert_eq!(spans(&lay("ab\n\ncd\n", 10.0, true)), [(0, 2), (3, 3), (4, 6), (7, 7)]);
    assert_eq!(spans(&lay("one two three", 4.0, false)), [(0, 13)], "a one-line field never wraps");
    let l = lay("one two", 10.0, true);
    assert_eq!((l.x_of(4), l.pos_at(4.4, 0.0, 9.0), l.pos_at(99.0, 50.0, 9.0)), (4.0, 4, 7), "a point maps to the nearest position");
}

#[test]
fn up_and_down_move_by_visual_line_and_keep_the_column() {
    let text = "one two three four";
    let l = lay(text, 9.0, true);
    let mut e = Edit::new("t", text, true);
    e.move_to(6, false);
    e.vertical(&l, 1, false);
    assert_eq!(e.caret, 13, "line 2 is shorter: its end, the column x 6 kept");
    e.vertical(&l, 1, false);
    assert_eq!(e.caret, 18, "line 3's end");
    e.vertical(&l, -1, false);
    e.vertical(&l, -1, false);
    assert_eq!(e.caret, 6, "back up on the same column (sticky)");
    e.vertical(&l, -1, false);
    assert_eq!(e.caret, 0, "Up on the first line goes to the start");
    e.vertical(&l, 5, false);
    assert_eq!(e.caret, text.chars().count(), "past the last line: the end");
    // A horizontal move drops the sticky column.
    e.move_to(6, false);
    e.vertical(&l, 1, false);
    e.left(false);
    e.vertical(&l, -1, false);
    assert_eq!(e.caret, 4, "x 4 from the left move, not the old 6");
    // Home and End are the visual line's ends; Ctrl+Home and Ctrl+End the text's.
    e.move_to(10, false);
    assert_eq!(key(&mut e, "home", &l), None);
    assert_eq!(e.caret, 8);
    key(&mut e, "end", &l);
    assert_eq!(e.caret, 13);
    key(&mut e, "ctrl+home", &l);
    assert_eq!(e.caret, 0);
    key(&mut e, "ctrl+shift+end", &l);
    assert_eq!(e.selection(), Some((0, 18)), "Shift with a move selects");
    // Page Down moves a view's worth of lines (3 here).
    let mut p = Edit::new("p", "a\nb\nc\nd\ne", true);
    let pl = lay(&p.text, 9.0, true);
    p.move_to(0, false);
    key(&mut p, "pagedown", &pl);
    assert_eq!(p.caret, 6, "three lines down");
}

#[test]
fn selection_is_replaced_by_typing_and_collapses_without_shift() {
    let mut e = Edit::new("t", "hello world", false);
    let l = lay(&e.text, 100.0, false);
    e.move_to(0, false);
    for _ in 0..5 {
        key(&mut e, "shift+right", &l);
    }
    assert_eq!((e.selection(), e.selected().as_str()), (Some((0, 5)), "hello"));
    e.insert("bye");
    assert_eq!((e.text.as_str(), e.caret, e.selection()), ("bye world", 3, None), "typing replaces the selection");
    e.select_word(6);
    assert_eq!(e.selected(), "world", "a double click takes the word");
    key(&mut e, "left", &l);
    assert_eq!((e.caret, e.selection()), (4, None), "Left collapses to the selection's start");
    key(&mut e, "ctrl+a", &l);
    assert_eq!(e.selected(), "bye world");
    key(&mut e, "backspace", &l);
    assert_eq!(e.text, "", "Backspace deletes the selection");
}

#[test]
fn ctrl_moves_and_deletes_by_word() {
    let text = "Hello, world  foo";
    let ch: Vec<char> = text.chars().collect();
    let rights: Vec<usize> = std::iter::successors(Some(0), |p| (*p < ch.len()).then(|| word_end(&ch, *p))).collect();
    assert_eq!(rights, [0, 5, 7, 14, 17], "a word, then its punctuation, each with the spaces after it");
    let lefts: Vec<usize> = std::iter::successors(Some(17), |p| (*p > 0).then(|| word_start(&ch, *p))).collect();
    assert_eq!(lefts, [17, 14, 7, 5, 0]);
    let mut e = Edit::new("t", text, false);
    let l = lay(text, 100.0, false);
    key(&mut e, "ctrl+backspace", &l);
    assert_eq!(e.text, "Hello, world  ", "Ctrl+Backspace takes the word before the caret");
    e.move_to(0, false);
    key(&mut e, "ctrl+delete", &l);
    assert_eq!(e.text, ", world  ", "Ctrl+Delete takes the word after it");
    key(&mut e, "ctrl+shift+right", &l);
    assert_eq!(e.selected(), ", ");
}

#[test]
fn clipboard_keys_copy_cut_and_paste_normalised_text() {
    let mut e = Edit::new("t", "alpha beta", true);
    let l = lay(&e.text, 100.0, true);
    assert_eq!(key(&mut e, "ctrl+c", &l), None, "nothing selected: nothing to copy");
    e.select_word(0);
    assert_eq!(key(&mut e, "ctrl+c", &l).as_deref(), Some("alpha"));
    assert_eq!(e.text, "alpha beta", "copy leaves the text");
    assert_eq!(key(&mut e, "ctrl+x", &l).as_deref(), Some("alpha"));
    assert_eq!(e.text, " beta");
    // Paste: Windows line ends become one newline, other control characters go; a one-line field joins lines.
    let (k, ctrl, shift) = EditKey::parse("ctrl+v").unwrap();
    e.key(k, ctrl, shift, &l, 3, || "a\r\nb\rc\td".to_string());
    assert_eq!(e.text, "a\nb\ncd beta");
    let mut one = Edit::new("o", "", false);
    one.paste("first\r\nsecond\nthird");
    assert_eq!(one.text, "first second third");
    assert_eq!(EditKey::parse("ctrl+enter"), None, "Ctrl+Enter goes to the screen");
    assert_eq!(EditKey::parse("c"), None, "a plain letter is typed, not a key");
    assert_eq!(EditKey::parse("ctrl+shift+z"), Some((EditKey::Redo, true, true)));
}

#[test]
fn undo_groups_a_typing_run_until_a_pause_a_move_or_a_delete() {
    let mut e = Edit::new("t", "", true);
    for (i, c) in ["a", "b", "c"].iter().enumerate() {
        e.now_ms = i as u64 * (UNDO_PAUSE_MS / 4);
        e.insert(c);
    }
    assert_eq!(e.history(), (1, 0), "one run, one step");
    e.now_ms += UNDO_PAUSE_MS;
    e.insert("d");
    assert_eq!(e.history().0, 2, "a pause starts a new step");
    e.left(false);
    e.insert("x");
    assert_eq!(e.history().0, 3, "a move starts a new step");
    e.backspace();
    e.backspace();
    assert_eq!((e.text.as_str(), e.history().0), ("abd", 4), "a delete ends the typing run; the deletes are one step");
    assert!(e.undo());
    assert_eq!(e.text, "abcxd");
    assert!(e.undo() && e.undo());
    assert_eq!(e.text, "abc");
    assert!(e.redo());
    assert_eq!((e.text.as_str(), e.history()), ("abcd", (2, 2)));
    e.insert("!");
    assert_eq!(e.history().1, 0, "a change after undo drops the redo steps");
    assert!(e.undo() && e.undo() && e.undo());
    assert!(!e.undo(), "nothing left to undo");
    assert_eq!(e.text, "");
    // At most UNDO_MAX steps per edit.
    let mut long = Edit::new("l", "", false);
    for _ in 0..UNDO_MAX + 20 {
        long.paste("x");
    }
    assert_eq!(long.history().0, UNDO_MAX);
}

#[test]
fn the_scroll_keeps_the_caret_line_in_view() {
    // Lines 10 apart, a 50 tall view, 20 lines (content 206).
    let content = 20 * 10 + 6;
    assert_eq!(reveal_offset(0, 0, 10.0, 50, content, 1), 0, "a line in view keeps the offset");
    let down = reveal_offset(0, 10, 10.0, 50, content, 1);
    assert!(down > 0 && (3 + 10 * 10 - down) + 10 + 3 <= 50, "line 10 is brought up to the bottom edge: {down}");
    assert_eq!(reveal_offset(down, 10, 10.0, 50, content, 1), down, "and stays");
    assert_eq!(reveal_offset(80, 2, 10.0, 50, content, 1), 20, "a line above the view scrolls up to it");
    assert_eq!(reveal_offset(0, 19, 10.0, 50, content, 1), content - 50, "never past the end");
}
