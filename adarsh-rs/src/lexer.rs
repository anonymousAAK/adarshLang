//! Lexical analysis.

use crate::error::{AdarshError, Pos};
use num_bigint::BigInt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tok {
    Eof,
    Ident,
    Number,
    Str,
    // keywords
    Badlo,
    Kaam,
    Wapas,
    Agar,
    Warna,
    Jabtak,
    Dikhao,
    Bas,
    AageBadho,
    SahiHaiBe,
    Jhuth,
    Ginnati,
    KeLiye,
    In,
    Chuno,
    Case,
    WarnaCase,
    Pakdo,
    Chhoddo,
    Lao,
    Dhacha,
    Baaki,
    Khali,
    Pakka,
    Karo,
    Hai,
    Vikalp,
    DeteRaho,
    Aakhir,
    As,
    // operators
    Plus,
    Minus,
    Mul,
    Div,
    Mod,
    Power,
    Assign,
    PlusAssign,
    MinusAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    Aur,
    Ya,
    Nahin,
    Semi,
    Comma,
    Colon,
    Dot,
    LParen,
    RParen,
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    DotDot,
    Ellipsis,
    QDot,
    QQ,
    IntDiv,
    Arrow,
}

impl Tok {
    /// How the token is described in error messages.
    pub fn describe(self) -> &'static str {
        use Tok::*;
        match self {
            Eof => "file ka ant",
            Ident => "naam",
            Number => "number",
            Str => "string",
            Badlo => "badlo",
            Kaam => "kaam",
            Wapas => "wapas",
            Agar => "agar",
            Warna => "warna",
            Jabtak => "jabtak",
            Dikhao => "dikhao",
            Bas => "bas",
            AageBadho => "aage_badho",
            SahiHaiBe => "sahi_hai_be",
            Jhuth => "jhuth",
            Ginnati => "ginnati",
            KeLiye => "ke_liye",
            In => "in",
            Chuno => "chuno",
            Case => "case",
            WarnaCase => "warna_case",
            Pakdo => "pakdo",
            Chhoddo => "chhoddo",
            Lao => "lao",
            Dhacha => "dhacha",
            Baaki => "baaki",
            Khali => "khali",
            Pakka => "pakka",
            Karo => "karo",
            Hai => "hai",
            Vikalp => "vikalp",
            DeteRaho => "dete_raho",
            Aakhir => "aakhir",
            As => "as",
            Plus => "+",
            Minus => "-",
            Mul => "*",
            Div => "/",
            Mod => "%",
            Power => "**",
            Assign => "=",
            PlusAssign => "+=",
            MinusAssign => "-=",
            MulAssign => "*=",
            DivAssign => "/=",
            ModAssign => "%=",
            Eq => "==",
            Neq => "!=",
            Lt => "<",
            Gt => ">",
            Lte => "<=",
            Gte => ">=",
            Aur => "aur",
            Ya => "ya",
            Nahin => "nahin",
            Semi => ";",
            Comma => ",",
            Colon => ":",
            Dot => ".",
            LParen => "(",
            RParen => ")",
            LBrace => "{",
            RBrace => "}",
            LBracket => "[",
            RBracket => "]",
            DotDot => "..",
            Ellipsis => "...",
            QDot => "?.",
            QQ => "??",
            IntDiv => "//",
            Arrow => "=>",
        }
    }

    /// Tokens after which a `.` is attribute access rather than the start of `.5`.
    fn ends_value(self) -> bool {
        matches!(
            self,
            Tok::Ident
                | Tok::Number
                | Tok::Str
                | Tok::RParen
                | Tok::RBracket
                | Tok::RBrace
                | Tok::Khali
                | Tok::SahiHaiBe
                | Tok::Jhuth
        )
    }
}

fn keyword(s: &str) -> Option<Tok> {
    use Tok::*;
    Some(match s {
        "badlo" => Badlo,
        "kaam" => Kaam,
        "wapas" => Wapas,
        "agar" => Agar,
        "warna" => Warna,
        "jabtak" => Jabtak,
        "dikhao" => Dikhao,
        "bas" => Bas,
        "aage_badho" => AageBadho,
        "sahi_hai_be" => SahiHaiBe,
        "jhuth" => Jhuth,
        "ginnati" => Ginnati,
        "ke_liye" => KeLiye,
        "in" => In,
        "chuno" => Chuno,
        "case" => Case,
        "warna_case" => WarnaCase,
        "pakdo" => Pakdo,
        "chhoddo" => Chhoddo,
        "lao" => Lao,
        "dhacha" => Dhacha,
        "baaki" => Baaki,
        "khali" => Khali,
        "aur" => Aur,
        "ya" => Ya,
        "nahin" => Nahin,
        "pakka" => Pakka,
        "karo" => Karo,
        "hai" => Hai,
        "vikalp" => Vikalp,
        "dete_raho" => DeteRaho,
        "aakhir" => Aakhir,
        "as" => As,
        _ => return devanagari_keyword(s),
    })
}

/// Devanagari spellings of the keywords: `दिखाओ("नमस्ते");` works like `dikhao("नमस्ते");`.
const DEVANAGARI_KEYWORDS: &[(&str, Tok)] = &[
    ("बदलो", Tok::Badlo),
    ("काम", Tok::Kaam),
    ("वापस", Tok::Wapas),
    ("अगर", Tok::Agar),
    ("वरना", Tok::Warna),
    ("जबतक", Tok::Jabtak),
    ("दिखाओ", Tok::Dikhao),
    ("बस", Tok::Bas),
    ("आगे_बढ़ो", Tok::AageBadho),
    ("सही", Tok::SahiHaiBe),
    ("झूठ", Tok::Jhuth),
    ("गिनती", Tok::Ginnati),
    ("के_लिए", Tok::KeLiye),
    ("चुनो", Tok::Chuno),
    ("पकड़ो", Tok::Pakdo),
    ("छोड़ो", Tok::Chhoddo),
    ("लाओ", Tok::Lao),
    ("ढांचा", Tok::Dhacha),
    ("बाकी", Tok::Baaki),
    ("खाली", Tok::Khali),
    ("और", Tok::Aur),
    ("या", Tok::Ya),
    ("नहीं", Tok::Nahin),
    ("पक्का", Tok::Pakka),
    ("करो", Tok::Karo),
    ("है", Tok::Hai),
    ("आखिर", Tok::Aakhir),
];

/// Decompose nukta letters so `ड़` typed either way matches.
fn normalize_devanagari(s: &str) -> String {
    s.replace('\u{095C}', "\u{0921}\u{093C}")
        .replace('\u{095D}', "\u{0922}\u{093C}")
        .replace('\u{0958}', "\u{0915}\u{093C}")
}

fn devanagari_keyword(s: &str) -> Option<Tok> {
    if s.is_ascii() {
        return None;
    }
    let n = normalize_devanagari(s);
    DEVANAGARI_KEYWORDS.iter().find(|(k, _)| normalize_devanagari(k) == n).map(|(_, t)| *t)
}

fn is_devanagari(c: char) -> bool {
    ('\u{0900}'..='\u{097F}').contains(&c)
}

fn single_op(c: char) -> Option<Tok> {
    use Tok::*;
    Some(match c {
        '+' => Plus,
        '-' => Minus,
        '*' => Mul,
        '/' => Div,
        '%' => Mod,
        '=' => Assign,
        ';' => Semi,
        ',' => Comma,
        ':' => Colon,
        '.' => Dot,
        '(' => LParen,
        ')' => RParen,
        '{' => LBrace,
        '}' => RBrace,
        '[' => LBracket,
        ']' => RBracket,
        '!' => Nahin,
        '<' => Lt,
        '>' => Gt,
        _ => return None,
    })
}

fn multi_op(a: char, b: char) -> Option<Tok> {
    use Tok::*;
    Some(match (a, b) {
        ('*', '*') => Power,
        ('+', '=') => PlusAssign,
        ('-', '=') => MinusAssign,
        ('*', '=') => MulAssign,
        ('/', '=') => DivAssign,
        ('%', '=') => ModAssign,
        ('=', '=') => Eq,
        ('!', '=') => Neq,
        ('<', '=') => Lte,
        ('>', '=') => Gte,
        ('&', '&') => Aur,
        ('|', '|') => Ya,
        ('.', '.') => DotDot,
        ('?', '.') => QDot,
        ('?', '?') => QQ,
        ('/', '/') => IntDiv,
        ('=', '>') => Arrow,
        _ => return None,
    })
}

/// A piece of a string literal: plain text or the source of an `{expression}`.
#[derive(Debug, Clone)]
pub enum RawPart {
    Lit(String),
    Code(String, Pos),
}

#[derive(Debug, Clone)]
pub enum TokValue {
    None,
    Int(i64),
    BigInt(BigInt),
    Float(f64),
    Text(String),
    Interp(Vec<RawPart>),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: Tok,
    pub value: TokValue,
    pub pos: Pos,
    /// Position just after the token's last character.
    pub end: Pos,
    /// `## doc comment` lines directly above this token.
    pub doc: Option<String>,
}

impl Token {
    pub fn text(&self) -> String {
        match &self.value {
            TokValue::Text(s) => s.clone(),
            _ => String::new(),
        }
    }
}

pub struct Lexer {
    chars: Vec<char>,
    i: usize,
    line: u32,
    col: u32,
    file: u32,
    last: Tok,
    pending_doc: Option<String>,
}

impl Lexer {
    pub fn new(text: &str, file: u32) -> Self {
        Self::at(text, Pos { line: 1, col: 1, file })
    }

    /// Lex `text` as if it started at `start` (used for `{expr}` inside strings).
    pub fn at(text: &str, start: Pos) -> Self {
        Lexer {
            chars: text.chars().collect(),
            i: 0,
            line: start.line,
            col: start.col,
            file: start.file,
            last: Tok::Eof,
            pending_doc: None,
        }
    }

    fn cur(&self) -> Option<char> {
        self.chars.get(self.i).copied()
    }
    fn peek(&self, n: usize) -> Option<char> {
        self.chars.get(self.i + n).copied()
    }
    fn pos(&self) -> Pos {
        Pos { line: self.line, col: self.col, file: self.file }
    }
    fn advance(&mut self) {
        if self.cur() == Some('\n') {
            self.line += 1;
            self.col = 1;
        } else {
            self.col += 1;
        }
        self.i += 1;
    }

    fn number(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        let mut s = String::new();
        let mut is_float = false;
        let digits = |lx: &mut Lexer, s: &mut String| {
            while let Some(c) = lx.cur() {
                if c.is_ascii_digit() {
                    s.push(c);
                } else if c == '_'
                    && lx.peek(1).is_some_and(|n| n.is_ascii_digit())
                    && s.ends_with(|p: char| p.is_ascii_digit())
                {
                    // 1_000_000
                } else {
                    break;
                }
                lx.advance();
            }
        };
        digits(self, &mut s);
        if self.cur() == Some('.') {
            let next = self.peek(1);
            // `1.5`, `.5`, and a bare trailing `1.` are floats; `1.foo` is not.
            if next.is_some_and(|c| c.is_ascii_digit())
                || !next.is_some_and(|c| c.is_alphabetic() || c == '_' || c == '.')
            {
                is_float = true;
                s.push('.');
                self.advance();
                digits(self, &mut s);
            }
        }
        if matches!(self.cur(), Some('e' | 'E')) {
            let (a, b) = (self.peek(1), self.peek(2));
            let ok = a.is_some_and(|c| c.is_ascii_digit())
                || (matches!(a, Some('+' | '-')) && b.is_some_and(|c| c.is_ascii_digit()));
            if ok {
                is_float = true;
                s.push('e');
                self.advance();
                if let Some(sign @ ('+' | '-')) = self.cur() {
                    s.push(sign);
                    self.advance();
                }
                digits(self, &mut s);
            }
        }
        let value = if is_float {
            let text = if s.starts_with('.') { format!("0{s}") } else { s.clone() };
            TokValue::Float(
                text.parse::<f64>().map_err(|_| AdarshError::syntax(format!("'{s}' sahi number nahi hai."), pos))?,
            )
        } else {
            match s.parse::<i64>() {
                Ok(i) => TokValue::Int(i),
                Err(_) => TokValue::BigInt(s.parse::<BigInt>().unwrap_or_default()),
            }
        };
        Ok(Token { ty: Tok::Number, value, pos, end: Pos::default(), doc: None })
    }

    fn string(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        let triple = self.peek(1) == Some('"') && self.peek(2) == Some('"');
        self.advance(); // opening quote
        if triple {
            self.advance();
            self.advance();
            // a newline right after the opening quotes is not part of the text
            if self.cur() == Some('\n') {
                self.advance();
            }
        }
        let mut parts: Vec<RawPart> = Vec::new();
        let mut lit = String::new();
        loop {
            let Some(c) = self.cur() else {
                let what = if triple { "\"\"\"" } else { "\"" };
                return Err(AdarshError::syntax(format!("String band nahi hui: closing {what} missing hai."), pos));
            };
            match c {
                '"' if triple && !(self.peek(1) == Some('"') && self.peek(2) == Some('"')) => {
                    lit.push('"');
                    self.advance();
                }
                '"' => {
                    self.advance();
                    if triple {
                        self.advance();
                        self.advance();
                    }
                    break;
                }
                '\\' => {
                    self.advance();
                    let esc = self.cur();
                    match esc {
                        Some('n') => lit.push('\n'),
                        Some('t') => lit.push('\t'),
                        Some('r') => lit.push('\r'),
                        Some('0') => lit.push('\0'),
                        Some('\\') => lit.push('\\'),
                        Some('"') => lit.push('"'),
                        Some('{') => lit.push('{'),
                        Some('}') => lit.push('}'),
                        Some(other) => {
                            lit.push('\\');
                            lit.push(other);
                        }
                        None => continue,
                    }
                    self.advance();
                }
                '{' => {
                    let open = self.pos();
                    self.advance();
                    let code_pos = self.pos();
                    let mut code = String::new();
                    let mut depth = 1;
                    loop {
                        let Some(c) = self.cur() else {
                            return Err(AdarshError::syntax(
                                "String ke andar '{' band nahi hua: '}' missing hai. Agar '{' text ka hissa hai to \\{ likho, ya raw string r\"...\" use karo (usmein {} interpolation nahi hota).",
                                open,
                            ));
                        };
                        if c == '"' {
                            // nested string inside the expression
                            code.push(c);
                            self.advance();
                            while let Some(d) = self.cur() {
                                code.push(d);
                                self.advance();
                                if d == '\\' {
                                    if let Some(e) = self.cur() {
                                        code.push(e);
                                        self.advance();
                                    }
                                } else if d == '"' {
                                    break;
                                }
                            }
                            continue;
                        }
                        if c == '{' {
                            depth += 1;
                        } else if c == '}' {
                            depth -= 1;
                            if depth == 0 {
                                self.advance();
                                break;
                            }
                        }
                        code.push(c);
                        self.advance();
                    }
                    if code.trim().is_empty() {
                        // `{}` holds no expression, so keep it as literal text
                        lit.push('{');
                        lit.push_str(&code);
                        lit.push('}');
                        continue;
                    }
                    if !lit.is_empty() {
                        parts.push(RawPart::Lit(std::mem::take(&mut lit)));
                    }
                    parts.push(RawPart::Code(code, code_pos));
                }
                _ => {
                    lit.push(c);
                    self.advance();
                }
            }
        }
        if parts.is_empty() {
            return Ok(Token { ty: Tok::Str, value: TokValue::Text(lit), pos, end: Pos::default(), doc: None });
        }
        if !lit.is_empty() {
            parts.push(RawPart::Lit(lit));
        }
        Ok(Token { ty: Tok::Str, value: TokValue::Interp(parts), pos, end: Pos::default(), doc: None })
    }

    /// `r"..."` / `r"""..."""`: no escapes, no interpolation (good for JSON, regex, paths).
    fn raw_string(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        self.advance(); // r
        let triple = self.peek(1) == Some('"') && self.peek(2) == Some('"');
        let quotes = if triple { 3 } else { 1 };
        for _ in 0..quotes {
            self.advance();
        }
        if triple && self.cur() == Some('\n') {
            self.advance();
        }
        let mut lit = String::new();
        loop {
            match self.cur() {
                None => return Err(AdarshError::syntax("Raw string band nahi hui: closing quote missing hai.", pos)),
                Some('"') if !triple || (self.peek(1) == Some('"') && self.peek(2) == Some('"')) => {
                    for _ in 0..quotes {
                        self.advance();
                    }
                    break;
                }
                Some(c) => {
                    lit.push(c);
                    self.advance();
                }
            }
        }
        Ok(Token { ty: Tok::Str, value: TokValue::Text(lit), pos, end: Pos::default(), doc: None })
    }

    fn identifier(&mut self) -> Token {
        let pos = self.pos();
        let mut s = String::new();
        while let Some(c) = self.cur() {
            if !(c.is_alphanumeric() || c == '_' || is_devanagari(c)) {
                break;
            }
            s.push(c);
            self.advance();
        }
        let ty = keyword(&s).unwrap_or(Tok::Ident);
        Token { ty, value: TokValue::Text(s), pos, end: Pos::default(), doc: None }
    }

    fn operator(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        let c = self.cur().unwrap();
        if c == '.' && self.peek(1) == Some('.') && self.peek(2) == Some('.') {
            for _ in 0..3 {
                self.advance();
            }
            return Ok(Token {
                ty: Tok::Ellipsis,
                value: TokValue::Text("...".into()),
                pos,
                end: Pos::default(),
                doc: None,
            });
        }
        if let Some(n) = self.peek(1) {
            if let Some(ty) = multi_op(c, n) {
                self.advance();
                self.advance();
                return Ok(Token { ty, value: TokValue::Text(format!("{c}{n}")), pos, end: Pos::default(), doc: None });
            }
        }
        match single_op(c) {
            Some(ty) => {
                self.advance();
                Ok(Token { ty, value: TokValue::Text(c.to_string()), pos, end: Pos::default(), doc: None })
            }
            None => {
                let hint = match c {
                    '&' => " 'aur' ya '&&' likho.",
                    '|' => " 'ya' ya '||' likho.",
                    '?' => " '?.' (khali-safe field) ya '??' (default value) likho.",
                    _ => "",
                };
                Err(AdarshError::syntax(format!("Anjaan character '{c}'.{hint}"), pos))
            }
        }
    }

    fn next_token(&mut self) -> Result<Token, AdarshError> {
        while let Some(c) = self.cur() {
            if c.is_whitespace() {
                self.advance();
                continue;
            }
            if c == '#' && self.peek(1) == Some('[') {
                let start = self.pos();
                self.advance();
                self.advance();
                loop {
                    match self.cur() {
                        None => {
                            return Err(AdarshError::syntax(
                                "Block comment '#[' band nahi hua: ']#' missing hai.",
                                start,
                            ))
                        }
                        Some(']') if self.peek(1) == Some('#') => {
                            self.advance();
                            self.advance();
                            break;
                        }
                        Some(_) => self.advance(),
                    }
                }
                continue;
            }
            if c == '#' {
                let doc = self.peek(1) == Some('#');
                let mut text = String::new();
                while self.cur().is_some_and(|c| c != '\n') {
                    text.push(self.cur().unwrap());
                    self.advance();
                }
                if doc {
                    let line = text.trim_start_matches('#').trim().to_string();
                    let d = self.pending_doc.get_or_insert_with(String::new);
                    if !d.is_empty() {
                        d.push('\n');
                    }
                    d.push_str(&line);
                }
                continue;
            }
            let tok = if c == 'r' && self.peek(1) == Some('"') {
                self.raw_string()?
            } else if c == '"' {
                self.string()?
            } else if c.is_ascii_digit()
                || (c == '.' && self.peek(1).is_some_and(|d| d.is_ascii_digit()) && !self.last.ends_value())
            {
                self.number()?
            } else if c.is_alphabetic() || c == '_' || is_devanagari(c) {
                self.identifier()
            } else {
                self.operator()?
            };
            return Ok(tok);
        }
        Ok(Token { ty: Tok::Eof, value: TokValue::None, pos: self.pos(), end: Pos::default(), doc: None })
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, AdarshError> {
        let mut out = Vec::new();
        loop {
            let mut t = self.next_token()?;
            t.end = self.pos();
            t.doc = self.pending_doc.take();
            self.last = t.ty;
            let eof = t.ty == Tok::Eof;
            out.push(t);
            if eof {
                return Ok(out);
            }
        }
    }
}
