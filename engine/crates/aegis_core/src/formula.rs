//! Formula evaluator (DERIVE1-E, docs/specs/DERIVE1.md section 3): small arithmetic formulas written in data, parsed
//! once to a tree and evaluated against the caller's named inputs. Engine-only (no Godot original): it knows names and
//! numbers, never a stat or a game; the caller supplies the input names and validates them (`Formula::names`).
//!
//! Grammar (whitespace ignored): `expr = term (('+' | '-') term)*`, `term = unary (('*' | '/') unary)*`,
//! `unary = '-' unary | primary`, `primary = number | name | call | '(' expr ')'`. Numbers are digits with an optional
//! `.` and 1-4 decimals, no exponent, at most 1000000; names are `[A-Za-z_][A-Za-z0-9_]*` and not a function name;
//! calls are `min` / `max` (2-8 arguments), `clamp(x, lo, hi)`, `floor(x)`, `ceil(x)`, `round(x)`.
//!
//! Numbers are f64 inside (DQ2); `round` is Godot's `roundi` (half away from zero, `json::roundi`); `x / 0` is 0; every
//! step's result that is not finite becomes 0, so no NaN or infinity leaves `eval`. Callers round the result to an
//! integer themselves. Evaluation is left to right with operands in source order, so it is deterministic.

use std::fmt;

/// Longest formula source, in characters.
pub const MAX_LEN: usize = 256;
/// Deepest parenthesis nesting (groups and calls together).
pub const MAX_DEPTH: usize = 16;
/// Most tokens (numbers, names, operators, parentheses, commas) in one formula.
pub const MAX_TOKENS: usize = 64;
/// Largest numeric literal.
pub const MAX_NUMBER: f64 = 1_000_000.0;
/// Most decimals of a numeric literal.
pub const MAX_DECIMALS: usize = 4;
/// The function names; none of them is a valid input name.
pub const FUNCTIONS: [&str; 6] = ["min", "max", "clamp", "floor", "ceil", "round"];

/// A parse error: the 1-based character column it points at and what is wrong. Displayed as `column 9: <message>`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormulaError {
    pub column: usize,
    pub message: String,
}

impl fmt::Display for FormulaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "column {}: {}", self.column, self.message)
    }
}

impl std::error::Error for FormulaError {}

/// True when `s` follows the name rule (`[A-Za-z_][A-Za-z0-9_]*`) and is not a function name.
pub fn is_name(s: &str) -> bool {
    let mut chars = s.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() || c == '_' => {}
        _ => return false,
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '_') && !is_function(s)
}

/// True when `s` is one of `FUNCTIONS`.
pub fn is_function(s: &str) -> bool {
    FUNCTIONS.contains(&s)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Op {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Func {
    Min,
    Max,
    Clamp,
    Floor,
    Ceil,
    Round,
}

impl Func {
    fn of(name: &str) -> Option<Func> {
        Some(match name {
            "min" => Func::Min,
            "max" => Func::Max,
            "clamp" => Func::Clamp,
            "floor" => Func::Floor,
            "ceil" => Func::Ceil,
            "round" => Func::Round,
            _ => return None,
        })
    }

    /// Allowed argument counts (inclusive).
    fn arity(self) -> (usize, usize) {
        match self {
            Func::Min | Func::Max => (2, 8),
            Func::Clamp => (3, 3),
            Func::Floor | Func::Ceil | Func::Round => (1, 1),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
enum Node {
    Num(f64),
    /// Index into `Formula::names`.
    Name(usize),
    Neg(Box<Node>),
    Bin(Op, Box<Node>, Box<Node>),
    Call(Func, Vec<Node>),
}

/// A parsed formula: immutable, cheap to evaluate many times.
#[derive(Clone, Debug, PartialEq)]
pub struct Formula {
    source: String,
    names: Vec<String>,
    root: Node,
}

impl Formula {
    /// Parses `src` (rules in the module comment); the error names the column of the first problem.
    pub fn parse(src: &str) -> Result<Formula, FormulaError> {
        let len = src.chars().count();
        if len > MAX_LEN {
            return Err(err(MAX_LEN + 1, format!("formula longer than {MAX_LEN} characters")));
        }
        let tokens = tokenize(src)?;
        if tokens.is_empty() {
            return Err(err(1, "empty formula"));
        }
        if tokens.len() > MAX_TOKENS {
            return Err(err(tokens[MAX_TOKENS].column, format!("more than {MAX_TOKENS} tokens")));
        }
        let mut p = Parser { tokens: &tokens, pos: 0, depth: 0, end: len + 1, names: Vec::new() };
        let root = p.expr()?;
        if let Some(t) = p.peek() {
            return Err(match t.kind {
                Tok::RParen => err(t.column, "unbalanced parenthesis: ')' without '('"),
                _ => err(t.column, format!("expected an operator or the end, found {}", t.kind.describe())),
            });
        }
        Ok(Formula { source: src.to_string(), names: p.names, root })
    }

    /// Every input name the formula uses, in order of first appearance, without repeats.
    pub fn names(&self) -> &[String] {
        &self.names
    }

    /// The source text as written.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Evaluates the formula, asking `input` for each name as it is reached (left to right, operands in source order;
    /// a name used twice is asked twice). A non-finite input counts as 0. The result is always finite.
    pub fn eval(&self, input: &mut dyn FnMut(&str) -> f64) -> f64 {
        self.node(&self.root, input)
    }

    fn node(&self, n: &Node, input: &mut dyn FnMut(&str) -> f64) -> f64 {
        let v = match n {
            Node::Num(v) => *v,
            Node::Name(i) => input(&self.names[*i]),
            Node::Neg(x) => -self.node(x, input),
            Node::Bin(op, l, r) => {
                let a = self.node(l, input);
                let b = self.node(r, input);
                match op {
                    Op::Add => a + b,
                    Op::Sub => a - b,
                    Op::Mul => a * b,
                    Op::Div => {
                        if b == 0.0 {
                            0.0
                        } else {
                            a / b
                        }
                    }
                }
            }
            Node::Call(f, args) => {
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.node(a, input));
                }
                match f {
                    Func::Min => vals.iter().copied().fold(vals[0], f64::min),
                    Func::Max => vals.iter().copied().fold(vals[0], f64::max),
                    // Godot's clampf: below lo gives lo, else above hi gives hi (no panic when lo > hi).
                    Func::Clamp => {
                        if vals[0] < vals[1] {
                            vals[1]
                        } else if vals[0] > vals[2] {
                            vals[2]
                        } else {
                            vals[0]
                        }
                    }
                    Func::Floor => vals[0].floor(),
                    Func::Ceil => vals[0].ceil(),
                    // f64::round is half away from zero, the same rule as json::roundi.
                    Func::Round => vals[0].round(),
                }
            }
        };
        finite(v)
    }
}

/// A non-finite value becomes 0; negative zero becomes 0.
fn finite(v: f64) -> f64 {
    if v.is_finite() && v != 0.0 {
        v
    } else {
        0.0
    }
}

fn err(column: usize, message: impl Into<String>) -> FormulaError {
    FormulaError { column, message: message.into() }
}

#[derive(Clone, Debug, PartialEq)]
enum Tok {
    Num(f64),
    Name(String),
    Op(char),
    LParen,
    RParen,
    Comma,
}

impl Tok {
    fn describe(&self) -> String {
        match self {
            Tok::Num(_) => "a number".into(),
            Tok::Name(n) => format!("'{n}'"),
            Tok::Op(c) => format!("'{c}'"),
            Tok::LParen => "'('".into(),
            Tok::RParen => "')'".into(),
            Tok::Comma => "','".into(),
        }
    }
}

struct Token {
    kind: Tok,
    column: usize,
}

fn tokenize(src: &str) -> Result<Vec<Token>, FormulaError> {
    let chars: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let column = i + 1;
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let kind = if c.is_ascii_digit() || c == '.' {
            let start = i;
            while i < chars.len() && chars[i].is_ascii_digit() {
                i += 1;
            }
            if start == i {
                return Err(err(column, "number format: a number starts with a digit"));
            }
            if i < chars.len() && chars[i] == '.' {
                i += 1;
                let d = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                if i == d || i - d > MAX_DECIMALS {
                    return Err(err(column, format!("number format: 1 to {MAX_DECIMALS} decimals after '.'")));
                }
            }
            if i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_' || chars[i] == '.') {
                return Err(err(column, "number format: digits with an optional '.' and decimals, no exponent"));
            }
            let text: String = chars[start..i].iter().collect();
            let v: f64 = text.parse().map_err(|_| err(column, "number format"))?;
            if v > MAX_NUMBER {
                return Err(err(column, "number above 1000000"));
            }
            Tok::Num(v)
        } else if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            Tok::Name(chars[start..i].iter().collect())
        } else {
            i += 1;
            match c {
                '+' | '-' | '*' | '/' => Tok::Op(c),
                '(' => Tok::LParen,
                ')' => Tok::RParen,
                ',' => Tok::Comma,
                _ => return Err(err(column, format!("unknown character '{c}'"))),
            }
        };
        out.push(Token { kind, column });
    }
    Ok(out)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    depth: usize,
    /// Column just past the source (errors at the end of input point here).
    end: usize,
    names: Vec<String>,
}

impl<'a> Parser<'a> {
    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn peek_op(&self) -> Option<char> {
        match self.peek() {
            Some(Token { kind: Tok::Op(c), .. }) => Some(*c),
            _ => None,
        }
    }

    fn column(&self) -> usize {
        self.peek().map_or(self.end, |t| t.column)
    }

    fn expr(&mut self) -> Result<Node, FormulaError> {
        let mut left = self.term()?;
        while let Some(c @ ('+' | '-')) = self.peek_op() {
            self.pos += 1;
            let right = self.term()?;
            left = Node::Bin(if c == '+' { Op::Add } else { Op::Sub }, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn term(&mut self) -> Result<Node, FormulaError> {
        let mut left = self.unary()?;
        while let Some(c @ ('*' | '/')) = self.peek_op() {
            self.pos += 1;
            let column = self.column();
            let right = self.unary()?;
            if c == '/' && literal_zero(&right) {
                return Err(err(column, "division by a literal zero"));
            }
            left = Node::Bin(if c == '*' { Op::Mul } else { Op::Div }, Box::new(left), Box::new(right));
        }
        Ok(left)
    }

    fn unary(&mut self) -> Result<Node, FormulaError> {
        if self.peek_op() == Some('-') {
            self.pos += 1;
            return Ok(Node::Neg(Box::new(self.unary()?)));
        }
        self.primary()
    }

    /// Enters a parenthesis at `column`, checking the nesting limit.
    fn open(&mut self, column: usize) -> Result<(), FormulaError> {
        self.depth += 1;
        if self.depth > MAX_DEPTH {
            return Err(err(column, format!("nesting deeper than {MAX_DEPTH}")));
        }
        Ok(())
    }

    fn primary(&mut self) -> Result<Node, FormulaError> {
        let Some(t) = self.peek() else {
            return Err(err(self.end, "expected a number, a name or '('"));
        };
        let column = t.column;
        match &t.kind {
            Tok::Num(v) => {
                let v = *v;
                self.pos += 1;
                Ok(Node::Num(v))
            }
            Tok::LParen => {
                self.pos += 1;
                self.open(column)?;
                let inner = self.expr()?;
                match self.peek() {
                    Some(Token { kind: Tok::RParen, .. }) => self.pos += 1,
                    Some(t) => return Err(err(t.column, format!("expected ')', found {}", t.kind.describe()))),
                    None => return Err(err(column, "unbalanced parenthesis: '(' is not closed")),
                }
                self.depth -= 1;
                Ok(inner)
            }
            Tok::Name(name) => {
                let name = name.clone();
                self.pos += 1;
                let call = matches!(self.peek(), Some(Token { kind: Tok::LParen, .. }));
                match (Func::of(&name), call) {
                    (Some(f), true) => self.call(f, &name, column),
                    (None, true) => Err(err(column, format!("unknown function '{name}'"))),
                    (Some(_), false) => Err(err(column, format!("'{name}' is a function: call it as {name}(...)"))),
                    (None, false) => {
                        let i = match self.names.iter().position(|n| *n == name) {
                            Some(i) => i,
                            None => {
                                self.names.push(name);
                                self.names.len() - 1
                            }
                        };
                        Ok(Node::Name(i))
                    }
                }
            }
            other => Err(err(column, format!("expected a number, a name or '(', found {}", other.describe()))),
        }
    }

    /// The argument list of `name(`; the current token is the `(`.
    fn call(&mut self, f: Func, name: &str, column: usize) -> Result<Node, FormulaError> {
        let paren = self.column();
        self.pos += 1;
        self.open(paren)?;
        let mut args = Vec::new();
        if matches!(self.peek(), Some(Token { kind: Tok::RParen, .. })) {
            self.pos += 1;
        } else {
            loop {
                args.push(self.expr()?);
                match self.peek() {
                    Some(Token { kind: Tok::Comma, .. }) => self.pos += 1,
                    Some(Token { kind: Tok::RParen, .. }) => {
                        self.pos += 1;
                        break;
                    }
                    Some(t) => return Err(err(t.column, format!("expected ',' or ')', found {}", t.kind.describe()))),
                    None => return Err(err(paren, "unbalanced parenthesis: '(' is not closed")),
                }
            }
        }
        self.depth -= 1;
        let (lo, hi) = f.arity();
        if args.len() < lo || args.len() > hi {
            let want = if lo == hi { format!("{lo}") } else { format!("{lo} to {hi}") };
            let s = if hi == 1 { "" } else { "s" };
            return Err(err(column, format!("{name} takes {want} argument{s}, found {}", args.len())));
        }
        Ok(Node::Call(f, args))
    }
}

/// A literal zero divisor: `0`, `0.0`, `-0`, `(0)` (parentheses leave no node).
fn literal_zero(n: &Node) -> bool {
    match n {
        Node::Num(v) => *v == 0.0,
        Node::Neg(x) => literal_zero(x),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json::roundi;
    use crate::rng::GodotRng;

    /// Evaluates `src` with inputs from `vals` (name, value); a missing name panics.
    fn run(src: &str, vals: &[(&str, f64)]) -> f64 {
        let f = Formula::parse(src).unwrap_or_else(|e| panic!("{src}: {e}"));
        f.eval(&mut |n| vals.iter().find(|(k, _)| *k == n).unwrap_or_else(|| panic!("no input {n}")).1)
    }

    fn error(src: &str) -> FormulaError {
        Formula::parse(src).expect_err(src)
    }

    /// 1-based column of the first `pat` in `src`.
    fn col(src: &str, pat: &str) -> usize {
        src[..src.find(pat).expect(pat)].chars().count() + 1
    }

    /// Random (a, b, c) triples; b is never zero so Rust's own division matches.
    fn triples() -> Vec<(f64, f64, f64)> {
        let mut rng = GodotRng::new(7);
        (0..200).map(|_| (rng.randf_range(-50.0, 50.0), rng.randf_range(0.5, 40.0), rng.randf_range(-50.0, 50.0))).collect()
    }

    #[test]
    fn precedence_and_associativity_match_rust() {
        for (a, b, c) in triples() {
            let v = [("a", a), ("b", b), ("c", c)];
            assert_eq!(run("a - b - c", &v), (a - b) - c);
            assert_eq!(run("a + b * c", &v), a + (b * c));
            assert_eq!(run("a / b * c", &v), (a / b) * c);
            assert_eq!(run("a * b + c / b", &v), (a * b) + (c / b));
            assert_eq!(run("(a + b) * c", &v), (a + b) * c);
            assert_eq!(run("a - (b - c)", &v), a - (b - c));
            assert_eq!(run("a / b / b", &v), (a / b) / b);
        }
    }

    #[test]
    fn unary_minus() {
        for (a, b, c) in triples() {
            let v = [("a", a), ("b", b), ("c", c)];
            assert_eq!(run("-a", &v), -a);
            assert_eq!(run("-a * b", &v), (-a) * b);
            assert_eq!(run("a - -b", &v), a - (-b));
            assert_eq!(run("- -a", &v), a);
            assert_eq!(run("-(a + c)", &v), -(a + c));
        }
    }

    #[test]
    fn function_identities() {
        for (a, b, c) in triples() {
            let v = [("a", a), ("b", b), ("c", c)];
            let lo = a.min(c);
            let hi = a.max(c);
            let v2 = [("x", b * 3.0 - 60.0), ("lo", lo), ("hi", hi)];
            let mn = run("min(a, b)", &v);
            assert!(mn <= a && mn <= b && (mn == a || mn == b));
            let mx = run("max(a, b, c)", &v);
            assert!(mx >= a && mx >= b && mx >= c && (mx == a || mx == b || mx == c));
            let cl = run("clamp(x, lo, hi)", &v2);
            assert!(lo <= cl && cl <= hi);
            let x = v2[0].1;
            if lo <= x && x <= hi {
                assert_eq!(cl, x);
            }
            let fl = run("floor(a)", &v);
            assert!(fl <= a && a - fl < 1.0 && fl.fract() == 0.0);
            let ce = run("ceil(a)", &v);
            assert!(ce >= a && ce - a < 1.0 && ce.fract() == 0.0);
            assert_eq!(run("round(a)", &v), roundi(a) as f64);
        }
        // Halves round away from zero, like Godot's roundi.
        for h in [0.5, 1.5, 2.5, -0.5, -1.5, -2.5] {
            assert_eq!(run("round(h)", &[("h", h)]), roundi(h) as f64);
            assert_eq!(run("round(h)", &[("h", h)]).abs(), h.abs() + 0.5);
        }
        // clamp with lo > hi follows Godot's clampf: below lo gives lo, otherwise above hi gives hi.
        let (lo, hi) = (10.0, 0.0);
        for x in [-5.0, 0.0, 5.0, 10.0, 20.0] {
            let want = if x < lo { lo } else if x > hi { hi } else { x };
            assert_eq!(run("clamp(x, lo, hi)", &[("x", x), ("lo", lo), ("hi", hi)]), want, "clamp({x}, {lo}, {hi})");
        }
        assert_eq!(run("clamp(5, 10, 0)", &[]), 10.0);
        assert_eq!(run("clamp(-5, 10, 0)", &[]), 10.0);
        assert_eq!(run("clamp(20, 10, 0)", &[]), 0.0);
        // Eight arguments are allowed for min and max.
        assert_eq!(run("min(8, 7, 6, 5, 4, 3, 2, 1)", &[]), 1.0);
        assert_eq!(run("max(1, 2, 3, 4, 5, 6, 7, 8)", &[]), 8.0);
    }

    #[test]
    fn division_by_zero_is_zero_and_results_are_finite() {
        assert_eq!(run("a / z", &[("a", 5.0), ("z", 0.0)]), 0.0);
        assert_eq!(run("a / (b - b)", &[("a", 5.0), ("b", 3.0)]), 0.0);
        // Non-finite inputs count as 0; overflow becomes 0.
        for bad in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            assert_eq!(run("x", &[("x", bad)]), 0.0);
            assert!(run("x * 2 + 1", &[("x", bad)]).is_finite());
        }
        assert_eq!(run("x * x", &[("x", f64::MAX)]), 0.0);
        // Negative zero leaves as 0.
        assert!(run("-x", &[("x", 0.0)]).is_sign_positive());
    }

    #[test]
    fn numbers() {
        assert_eq!(run("1000000", &[]), MAX_NUMBER);
        assert_eq!(run("2.5", &[]), 2.5);
        assert_eq!(run("0.1234", &[]), 0.1234);
        assert_eq!(run("0.25 * 4", &[]), 1.0);
    }

    #[test]
    fn errors_carry_their_column() {
        let cases: Vec<(&str, usize, &str)> = vec![
            ("", 1, "empty formula"),
            ("   ", 1, "empty formula"),
            ("a + b # c", col("a + b # c", "#"), "unknown character '#'"),
            ("a + .5", col("a + .5", ".5"), "number format"),
            ("a + 1.", col("a + 1.", "1."), "number format"),
            ("2 * 1.23456", col("2 * 1.23456", "1.2"), "number format"),
            ("1e5", 1, "number format"),
            ("1.2.3", 1, "number format"),
            ("2 * 1000001", col("2 * 1000001", "1000001"), "number above 1000000"),
            ("(a + b", 1, "unbalanced parenthesis"),
            ("a + b)", col("a + b)", ")"), "unbalanced parenthesis"),
            ("min(a, b", col("min(a, b", "("), "unbalanced parenthesis"),
            ("min(a)", 1, "min takes 2 to 8 arguments, found 1"),
            ("max(1, 2, 3, 4, 5, 6, 7, 8, 9)", 1, "max takes 2 to 8 arguments, found 9"),
            ("a + clamp(a, b)", col("a + clamp(a, b)", "clamp"), "clamp takes 3 arguments, found 2"),
            ("floor()", 1, "floor takes 1 argument, found 0"),
            ("round(a, b)", 1, "round takes 1 argument, found 2"),
            ("2 * pow(a, 2)", col("2 * pow(a, 2)", "pow"), "unknown function 'pow'"),
            ("a + min", col("a + min", "min"), "'min' is a function"),
            ("a / 0", col("a / 0", "0"), "division by a literal zero"),
            ("a / -0.0", col("a / -0.0", "-"), "division by a literal zero"),
            ("a / (0)", col("a / (0)", "("), "division by a literal zero"),
            ("a b", col("a b", "b"), "expected an operator or the end, found 'b'"),
            ("2 (a)", col("2 (a)", "("), "expected an operator or the end"),
            ("a + * b", col("a + * b", "*"), "expected a number, a name or '('"),
            ("+a", 1, "expected a number, a name or '('"),
            ("a *", 4, "expected a number, a name or '('"),
            ("min(a b)", col("min(a b)", "b"), "expected ',' or ')'"),
            ("(a b)", col("(a b)", "b"), "expected ')'"),
        ];
        for (src, column, msg) in cases {
            let e = error(src);
            assert_eq!(e.column, column, "{src}: {e}");
            assert!(e.message.starts_with(msg), "{src}: {e}");
        }
        // The display form the spec gives.
        assert_eq!(error("abcd * (").to_string(), "column 9: expected a number, a name or '('");
    }

    #[test]
    fn size_limits() {
        // Length: MAX_LEN characters parse, one more fails at the first column past the limit.
        let fits = format!("a{}", " ".repeat(MAX_LEN - 1));
        assert!(Formula::parse(&fits).is_ok());
        let long = format!("a{}", " ".repeat(MAX_LEN));
        let e = error(&long);
        assert_eq!(e.column, MAX_LEN + 1);
        assert!(e.message.contains("longer"));
        // Nesting: MAX_DEPTH parentheses parse, one more fails at the extra '('.
        let nest = |d: usize| format!("{}a{}", "(".repeat(d), ")".repeat(d));
        assert!(Formula::parse(&nest(MAX_DEPTH)).is_ok());
        let e = error(&nest(MAX_DEPTH + 1));
        assert_eq!(e.column, MAX_DEPTH + 1);
        assert!(e.message.contains("nesting"));
        // Calls count as nesting too.
        let calls = |d: usize| format!("{}a{}", "floor(".repeat(d), ")".repeat(d));
        assert!(Formula::parse(&calls(MAX_DEPTH)).is_ok());
        assert_eq!(error(&calls(MAX_DEPTH + 1)).column, MAX_DEPTH * "floor(".len() + "floor(".len());
        // Tokens: `a+a+...` with MAX_TOKENS tokens parses; the token past the limit is the error's column.
        let chain = |n: usize| vec!["a"; n].join("+");
        // chain(n) has n names and n - 1 pluses, one character each: 2n - 1 tokens.
        let exact = format!("-{}", chain(MAX_TOKENS / 2)); // a leading unary minus makes MAX_TOKENS
        assert!(Formula::parse(&exact).is_ok());
        let over = chain(MAX_TOKENS / 2 + 1); // MAX_TOKENS + 1
        let e = error(&over);
        assert_eq!(e.column, MAX_TOKENS + 1);
        assert!(e.message.contains("tokens"));
    }

    #[test]
    fn names_in_first_appearance_order_without_repeats() {
        let f = Formula::parse("k + A * per_a + B * per_b + min(A, _x1) - k").unwrap();
        assert_eq!(f.names(), ["k", "A", "per_a", "B", "per_b", "_x1"]);
        assert!(Formula::parse("2 * (3 + 4)").unwrap().names().is_empty());
        // Names are case-sensitive.
        assert_eq!(Formula::parse("a + A").unwrap().names(), ["a", "A"]);
    }

    #[test]
    fn inputs_are_asked_in_source_order() {
        let f = Formula::parse("c - max(b, a) * c / d").unwrap();
        let mut asked = Vec::new();
        f.eval(&mut |n| {
            asked.push(n.to_string());
            1.0
        });
        assert_eq!(asked, ["c", "b", "a", "c", "d"]);
    }

    #[test]
    fn parse_is_deterministic_and_keeps_the_source() {
        let src = "  clamp(a * 2 + b - 3.5, 0, 100) / max(n, 1) ";
        let a = Formula::parse(src).unwrap();
        let b = Formula::parse(src).unwrap();
        assert_eq!(a, b);
        assert_eq!(a.source(), src);
        let c = a.clone();
        for (x, y, z) in triples() {
            let mut input = |n: &str| match n {
                "a" => x,
                "b" => y,
                _ => z,
            };
            let r = a.eval(&mut input);
            assert_eq!(r.to_bits(), b.eval(&mut input).to_bits());
            assert_eq!(r.to_bits(), c.eval(&mut input).to_bits());
            assert!(r.is_finite());
        }
    }

    #[test]
    fn name_rule() {
        for ok in ["a", "_", "ABC", "per_a", "x1", "_2b"] {
            assert!(is_name(ok), "{ok}");
        }
        for bad in ["", "1a", "a-b", "a b", "min", "round", "é"] {
            assert!(!is_name(bad), "{bad}");
        }
        for f in FUNCTIONS {
            assert!(is_function(f));
        }
    }
}
