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
        _ => return None,
    })
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
        Ok(Token { ty: Tok::Number, value, pos, end: Pos::default() })
    }

    fn string(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        self.advance(); // opening quote
        let mut parts: Vec<RawPart> = Vec::new();
        let mut lit = String::new();
        loop {
            let Some(c) = self.cur() else {
                return Err(AdarshError::syntax("String band nahi hui: closing \" missing hai.", pos));
            };
            match c {
                '"' => {
                    self.advance();
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
                                "String ke andar '{' band nahi hua: '}' missing hai.",
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
            return Ok(Token { ty: Tok::Str, value: TokValue::Text(lit), pos, end: Pos::default() });
        }
        if !lit.is_empty() {
            parts.push(RawPart::Lit(lit));
        }
        Ok(Token { ty: Tok::Str, value: TokValue::Interp(parts), pos, end: Pos::default() })
    }

    fn identifier(&mut self) -> Token {
        let pos = self.pos();
        let mut s = String::new();
        while let Some(c) = self.cur() {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            s.push(c);
            self.advance();
        }
        let ty = keyword(&s).unwrap_or(Tok::Ident);
        Token { ty, value: TokValue::Text(s), pos, end: Pos::default() }
    }

    fn operator(&mut self) -> Result<Token, AdarshError> {
        let pos = self.pos();
        let c = self.cur().unwrap();
        if let Some(n) = self.peek(1) {
            if let Some(ty) = multi_op(c, n) {
                self.advance();
                self.advance();
                return Ok(Token { ty, value: TokValue::Text(format!("{c}{n}")), pos, end: Pos::default() });
            }
        }
        match single_op(c) {
            Some(ty) => {
                self.advance();
                Ok(Token { ty, value: TokValue::Text(c.to_string()), pos, end: Pos::default() })
            }
            None => {
                let hint = match c {
                    '&' => " 'aur' ya '&&' likho.",
                    '|' => " 'ya' ya '||' likho.",
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
            if c == '#' {
                while self.cur().is_some_and(|c| c != '\n') {
                    self.advance();
                }
                continue;
            }
            let tok = if c == '"' {
                self.string()?
            } else if c.is_ascii_digit()
                || (c == '.' && self.peek(1).is_some_and(|d| d.is_ascii_digit()) && !self.last.ends_value())
            {
                self.number()?
            } else if c.is_alphabetic() || c == '_' {
                self.identifier()
            } else {
                self.operator()?
            };
            return Ok(tok);
        }
        Ok(Token { ty: Tok::Eof, value: TokValue::None, pos: self.pos(), end: Pos::default() })
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, AdarshError> {
        let mut out = Vec::new();
        loop {
            let mut t = self.next_token()?;
            t.end = self.pos();
            self.last = t.ty;
            let eof = t.ty == Tok::Eof;
            out.push(t);
            if eof {
                return Ok(out);
            }
        }
    }
}
