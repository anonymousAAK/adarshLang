//! `adarsh fmt`: consistent formatting that keeps comments and strings exactly.
//!
//! Works on a lossless token stream: re-indents by brace depth (4 spaces),
//! normalises spacing around operators and punctuation, trims trailing
//! whitespace, and keeps at most one blank line. Line breaks the author chose
//! are kept, so short one-line blocks stay on one line.

#[derive(Debug, Clone, PartialEq)]
enum K {
    Word,
    Number,
    Str,
    Comment,
    Punct,
    Newline,
}

#[derive(Debug, Clone)]
struct T {
    k: K,
    text: String,
}

fn scan(src: &str) -> Vec<T> {
    let c: Vec<char> = src.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    let push = |out: &mut Vec<T>, k: K, s: &[char]| out.push(T { k, text: s.iter().collect() });
    while i < c.len() {
        let ch = c[i];
        if ch == '\n' {
            push(&mut out, K::Newline, &c[i..i + 1]);
            i += 1;
        } else if ch.is_whitespace() {
            i += 1;
        } else if ch == '#' && c.get(i + 1) == Some(&'[') {
            let mut j = i + 2;
            while j < c.len() && !(c[j] == ']' && c.get(j + 1) == Some(&'#')) {
                j += 1;
            }
            let end = (j + 2).min(c.len());
            push(&mut out, K::Comment, &c[i..end]);
            i = end;
        } else if ch == '#' {
            let mut j = i;
            while j < c.len() && c[j] != '\n' {
                j += 1;
            }
            let text: String = c[i..j].iter().collect();
            out.push(T { k: K::Comment, text: text.trim_end().to_string() });
            i = j;
        } else if ch == '"' || (ch == 'r' && c.get(i + 1) == Some(&'"')) {
            let start = i;
            let raw = ch == 'r';
            if raw {
                i += 1;
            }
            let triple = c.get(i + 1) == Some(&'"') && c.get(i + 2) == Some(&'"');
            i += if triple { 3 } else { 1 };
            let mut depth = 0;
            while i < c.len() {
                if !raw && c[i] == '\\' {
                    i += 2;
                    continue;
                }
                if !raw && c[i] == '{' {
                    depth += 1;
                } else if !raw && c[i] == '}' && depth > 0 {
                    depth -= 1;
                } else if c[i] == '"' && depth == 0 {
                    if !triple {
                        i += 1;
                        break;
                    }
                    if c.get(i + 1) == Some(&'"') && c.get(i + 2) == Some(&'"') {
                        i += 3;
                        break;
                    }
                } else if c[i] == '"' && depth > 0 {
                    // a string inside {interpolation}
                    i += 1;
                    while i < c.len() && c[i] != '"' {
                        if c[i] == '\\' {
                            i += 1;
                        }
                        i += 1;
                    }
                }
                i += 1;
            }
            let end = i.min(c.len());
            push(&mut out, K::Str, &c[start..end]);
        } else if ch.is_ascii_digit()
            || (ch == '.' && c.get(i + 1).is_some_and(|d| d.is_ascii_digit()) && !prev_is_value(&out))
        {
            let mut j = i;
            while j < c.len()
                && (c[j].is_ascii_alphanumeric()
                    || c[j] == '_'
                    || (c[j] == '.' && c.get(j + 1) != Some(&'.'))
                    || ((c[j] == '+' || c[j] == '-') && matches!(c[j - 1], 'e' | 'E')))
            {
                j += 1;
            }
            push(&mut out, K::Number, &c[i..j]);
            i = j;
        } else if ch.is_alphanumeric() || ch == '_' || ('\u{0900}'..='\u{097F}').contains(&ch) {
            let mut j = i;
            while j < c.len() && (c[j].is_alphanumeric() || c[j] == '_' || ('\u{0900}'..='\u{097F}').contains(&c[j])) {
                j += 1;
            }
            push(&mut out, K::Word, &c[i..j]);
            i = j;
        } else {
            let three: String = c[i..(i + 3).min(c.len())].iter().collect();
            let two: String = c[i..(i + 2).min(c.len())].iter().collect();
            let len = if three == "..." {
                3
            } else if [
                "**", "+=", "-=", "*=", "/=", "%=", "==", "!=", "<=", ">=", "&&", "||", "..", "?.", "??", "//", "=>",
                "->",
            ]
            .contains(&two.as_str())
            {
                2
            } else {
                1
            };
            push(&mut out, K::Punct, &c[i..i + len]);
            i += len;
        }
    }
    out
}

fn ensure(extra: &mut Vec<i32>, case_body: &mut Vec<bool>, d: usize) {
    while extra.len() <= d {
        extra.push(*extra.last().unwrap_or(&0));
        case_body.push(false);
    }
}

fn prev_is_value(out: &[T]) -> bool {
    out.last().is_some_and(|t| matches!(t.k, K::Word | K::Number | K::Str) || t.text == ")" || t.text == "]")
}

const BINARY: &[&str] = &[
    "=", "+=", "-=", "*=", "/=", "%=", "==", "!=", "<", ">", "<=", ">=", "&&", "||", "+", "*", "/", "//", "%", "??",
    "=>", "->",
];
const WORD_OPS: &[&str] = &["aur", "ya", "in", "hai", "as"];

/// Is the token before `i` (skipping nothing) the end of a value? Used to tell
/// binary minus from unary minus.
fn ends_value(t: &T) -> bool {
    match t.k {
        K::Number | K::Str => true,
        K::Word => !matches!(
            t.text.as_str(),
            "wapas"
                | "dikhao"
                | "agar"
                | "jabtak"
                | "aur"
                | "ya"
                | "nahin"
                | "in"
                | "hai"
                | "case"
                | "dete_raho"
                | "chhoddo"
        ),
        K::Punct => t.text == ")" || t.text == "]" || t.text == "}",
        _ => false,
    }
}

/// Format AdarshLang source. Returns the new text.
pub fn format_source(src: &str) -> String {
    let toks = scan(src);
    // which `{` / `}` tokens belong to dict literals (matched by a stack)
    let mut dict_brace = vec![false; toks.len()];
    let mut stack: Vec<(usize, bool)> = Vec::new();
    let mut last_real: Option<&T> = None;
    for (i, t) in toks.iter().enumerate() {
        if t.k == K::Punct && t.text == "{" {
            let d = opens_dict(last_real);
            dict_brace[i] = d;
            stack.push((i, d));
        } else if t.k == K::Punct && t.text == "}" {
            if let Some((_, d)) = stack.pop() {
                dict_brace[i] = d;
            }
        }
        if !matches!(t.k, K::Newline | K::Comment) {
            last_real = Some(t);
        }
    }
    let mut out = String::new();
    let mut depth: i32 = 0;
    let mut paren: i32 = 0;
    let mut line_start = true;
    let mut blank_lines = 0;
    let mut prev: Option<T> = None;
    let mut brackets: Vec<char> = Vec::new();
    // per brace depth: extra indentation inherited from case bodies, and whether
    // we are inside an unbraced `case x:` body at that depth
    let mut extra: Vec<i32> = vec![0];
    let mut case_body: Vec<bool> = vec![false];
    let mut line_is_case = false;
    for (idx, t) in toks.iter().enumerate() {
        if t.k == K::Newline {
            if line_start {
                blank_lines += 1;
            } else {
                // trim trailing spaces
                while out.ends_with(' ') {
                    out.pop();
                }
            }
            out.push('\n');
            line_start = true;
            prev = None;
            continue;
        }
        if line_start {
            // keep at most one empty line
            while blank_lines > 1 && out.ends_with("\n\n\n") {
                out.pop();
                blank_lines -= 1;
            }
            if out.ends_with("\n\n\n") {
                out.pop();
            }
            blank_lines = 0;
            let closing = t.text == "}" || t.text == ")" || t.text == "]";
            let is_case = t.k == K::Word && (t.text == "case" || t.text == "warna_case");
            line_is_case = is_case;
            let d = (depth - if t.text == "}" { 1 } else { 0 }).max(0) as usize;
            ensure(&mut extra, &mut case_body, d);
            // statements under `case x:` sit one level deeper than the case line
            let case_offset = if case_body[d] && !is_case && t.text != "}" { 1 } else { 0 };
            if is_case {
                case_body[d] = true;
            }
            // a closing `}` lines up with the line that opened its block
            let base = if t.text == "}" {
                ensure(&mut extra, &mut case_body, d + 1);
                extra[d + 1]
            } else {
                extra[d] + case_offset
            };
            let level = d as i32 + base + paren.clamp(0, 1) * if closing { 0 } else { 1 };
            out.push_str(&"    ".repeat(level as usize));
            line_start = false;
        } else if let Some(p) = &prev {
            let before = if idx >= 2 { toks.get(idx - 2) } else { None };
            let a_dict_open = p.text == "{" && dict_brace[idx - 1];
            let b_dict_close = t.text == "}" && dict_brace[idx];
            if space_between(p, t, before, brackets.last() == Some(&'['), a_dict_open, b_dict_close) {
                out.push(' ');
            }
        }
        out.push_str(&t.text);
        if t.k == K::Punct {
            match t.text.as_str() {
                "{" => {
                    let d = depth.max(0) as usize;
                    ensure(&mut extra, &mut case_body, d + 1);
                    // a block inside an unbraced case body keeps the case body's extra level
                    extra[d + 1] = extra[d] + if case_body[d] && !line_is_case { 1 } else { 0 };
                    case_body[d + 1] = false;
                    depth += 1;
                    brackets.push('{');
                }
                "}" => {
                    let d = depth.max(0) as usize;
                    ensure(&mut extra, &mut case_body, d);
                    case_body[d] = false;
                    depth -= 1;
                    brackets.pop();
                }
                "(" | "[" => {
                    paren += 1;
                    brackets.push(t.text.chars().next().unwrap());
                }
                ")" | "]" => {
                    paren -= 1;
                    brackets.pop();
                }
                _ => {}
            }
        }
        prev = Some(t.clone());
    }
    let mut out = out.trim_end().to_string();
    // collapse runs of blank lines (from the start of the file too)
    while out.contains("\n\n\n") {
        out = out.replace("\n\n\n", "\n\n");
    }
    out.trim_start_matches('\n').to_string() + "\n"
}

const SPACED_KEYWORDS: &[&str] = &[
    "agar",
    "jabtak",
    "ginnati",
    "ke_liye",
    "chuno",
    "case",
    "aur",
    "ya",
    "nahin",
    "in",
    "hai",
    "wapas",
    "dete_raho",
    "as",
    "badlo",
    "pakka",
    "kaam",
    "अगर",
    "जबतक",
    "गिनती",
    "के_लिए",
    "चुनो",
    "और",
    "या",
    "नहीं",
    "वापस",
    "है",
    "बदलो",
    "पक्का",
];

/// Tokens after which `{` starts a dict literal rather than a code block.
fn opens_dict(prev: Option<&T>) -> bool {
    match prev {
        None => false,
        Some(p) => match p.k {
            K::Punct => matches!(p.text.as_str(), "=" | "(" | "," | ":" | "[" | "=>" | "??" | "+" | "==" | "!=" | "{"),
            K::Word => matches!(p.text.as_str(), "wapas" | "in" | "dete_raho" | "aur" | "ya"),
            _ => false,
        },
    }
}

/// Should there be a space between tokens `a` and `b`? `in_square` is true inside `[ ]`;
/// `a_dict_open` / `b_dict_close` mark the braces of a dict literal.
fn space_between(a: &T, b: &T, before_a: Option<&T>, in_square: bool, a_dict_open: bool, b_dict_close: bool) -> bool {
    let (x, y) = (a.text.as_str(), b.text.as_str());
    if b.k == K::Comment {
        return true;
    }
    if a_dict_open || b_dict_close || (x == "{" && y == "}") {
        return false; // {"a": 1} and {}
    }
    // `agar (c) "a" warna "b"`: a value right after `)` / `]`
    if matches!(x, ")" | "]") && b.k != K::Punct {
        return true;
    }
    // `dhacha Student : Vyakti` (inheritance) keeps a space before the colon
    if y == ":" && before_a.is_some_and(|t| t.text == "dhacha") {
        return true;
    }
    // `} chhoddo (e)` is a catch clause, `chhoddo("x")` a throw
    if x == "chhoddo" && y == "(" {
        return before_a.is_some_and(|t| t.text == "}");
    }
    if matches!(y, "," | ";" | ")" | "]" | "." | "?." | ".." | ":")
        || matches!(x, "(" | "[" | "." | "?." | ".." | "...")
    {
        return false;
    }
    if x == ":" {
        return !in_square; // xs[1:3] stays tight, naam: value gets a space
    }
    if x == "!" || (x == "-" && !before_a.is_some_and(ends_value)) {
        return false; // unary
    }
    if y == "(" || y == "[" {
        return match a.k {
            K::Word => SPACED_KEYWORDS.contains(&x),
            K::Punct => BINARY.contains(&x) || matches!(x, "," | "{" | ";" | "-" | "..."),
            _ => false,
        };
    }
    if matches!(x, "{" | "}" | "," | ";") || matches!(y, "{" | "}") {
        return true;
    }
    if BINARY.contains(&x) || BINARY.contains(&y) || matches!(x, "-" | "**") || matches!(y, "-" | "**") {
        return true;
    }
    if WORD_OPS.contains(&x) || WORD_OPS.contains(&y) {
        return true;
    }
    if y == "..." {
        return false;
    }
    // two words / numbers / strings next to each other
    a.k != K::Punct && b.k != K::Punct
}

#[cfg(test)]
mod tests {
    use super::format_source;

    #[test]
    fn formats_basic_program() {
        let src =
            "badlo x=1+2;\nagar(x>2){dikhao( \"bada\" );}warna{\ndikhao(-x);\n}\n\n\n\nkaam f(a,b=2){wapas a*b;}\n";
        let want = "badlo x = 1 + 2;\nagar (x > 2) { dikhao(\"bada\"); } warna {\n    dikhao(-x);\n}\n\nkaam f(a, b = 2) { wapas a * b; }\n";
        assert_eq!(format_source(src), want);
    }

    #[test]
    fn keeps_strings_and_comments() {
        let src = "dikhao(\"a  {x+1}  b\");   # comment  here\n#[ block\n  comment ]#\n";
        assert_eq!(format_source(src), "dikhao(\"a  {x+1}  b\"); # comment  here\n#[ block\n  comment ]#\n");
    }

    #[test]
    fn indents_case_bodies() {
        let src = "chuno (x) {\ncase 1:\ndikhao(1);\nagar (y) {\ndikhao(2);\n}\ncase 2: {\ndikhao(3);\n}\nwarna_case:\ndikhao(4);\n}\n";
        let want = "chuno (x) {\n    case 1:\n        dikhao(1);\n        agar (y) {\n            dikhao(2);\n        }\n    case 2: {\n        dikhao(3);\n    }\n    warna_case:\n        dikhao(4);\n}\n";
        assert_eq!(format_source(src), want);
    }

    #[test]
    fn is_idempotent() {
        let src = "ke_liye(badlo i,x in xs){\nagar(i%2==0 aur x!=khali){ dikhao(x?.naam??\"-\"); }\n}\n";
        let once = format_source(src);
        assert_eq!(format_source(&once), once);
    }
}
