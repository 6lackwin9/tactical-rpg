//! `Formula::eval_marked` (PR4-2): the input callback learns whether a name shares a product with a marked name (one
//! operand of a `*` or `/` holds it, the other a marked name, through groups, signs, sums and calls, or an enclosing
//! product does); a sum alone never ties two names; the arithmetic equals `eval`.

use aegis_core::formula::Formula;

/// The names `f` asks for under a product with a name starting `m`, in ask order (a name asked twice is listed twice).
fn tied(src: &str) -> Vec<String> {
    let f = Formula::parse(src).expect("parses");
    let mut out = Vec::new();
    f.eval_marked(&|n| n.starts_with('m'), &mut |n, under| {
        if under {
            out.push(n.to_string());
        }
        1.0
    });
    out
}

#[test]
fn a_name_is_tied_only_through_a_product_with_a_marked_name() {
    assert_eq!(tied("k * m1"), vec!["k"]);
    assert_eq!(tied("m1 / k"), vec!["k"]);
    assert_eq!(tied("k + m1"), Vec::<String>::new(), "a sum ties nothing");
    assert_eq!(tied("60 * k + m1 * 8"), Vec::<String>::new(), "each product holds one side only");
    assert_eq!(tied("(60 + m1 * 8) * k"), vec!["k"], "the group holds a marked name");
    assert_eq!(tied("max(m1, 2) * -k"), vec!["k"], "through a call and a sign");
    assert_eq!(tied("(a * k) * (b + m2)"), vec!["a", "k"], "an enclosing product ties every name of its other side, not the marked side's");
}

#[test]
fn eval_marked_computes_what_eval_computes() {
    let f = Formula::parse("max(m1 * 2, k) / (3 - m2) + floor(k * 1.5)").expect("parses");
    let value = |n: &str| match n {
        "m1" => 4.0,
        "m2" => 1.0,
        _ => 7.0,
    };
    let plain = f.eval(&mut |n| value(n));
    let marked = f.eval_marked(&|n| n.starts_with('m'), &mut |n, _| value(n));
    assert_eq!(plain, marked);
}
