//! Lexical analysis for AdarshLang (port of `adarsh_lang/lexer.py`).

use crate::error::AdarshError;
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
    /// The type name exactly as the Python implementation prints it in errors.
    pub fn name(self) -> &'static str {
        use Tok::*;
        match self {
            Eof => "EOF",
            Ident => "IDENT",
            Number => "NUMBER",
            Str => "STRING",
            Badlo => "BADLO",
            Kaam => "KAAM",
            Wapas => "WAPAS",
            Agar => "AGAR",
            Warna => "WARNA",
            Jabtak => "JABTAK",
            Dikhao => "DIKHAO",
            Bas => "BAS",
            AageBadho => "AAGE_BADHO",
            SahiHaiBe => "SAHI_HAI_BE",
            Jhuth => "JHUTH",
            Ginnati => "GINNATI",
            KeLiye => "KE_LIYE",
            In => "IN",
            Chuno => "CHUNO",
            Case => "CASE",
            WarnaCase => "WARNA_CASE",
            Pakdo => "PAKDO",
            Chhoddo => "CHHODDO",
            Lao => "LAO",
            Dhacha => "DHACHA",
            Baaki => "BAAKI",
            Khali => "KHALI",
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
            Aur => "&&",
            Ya => "||",
            Nahin => "!",
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

/// Literal payload carried by a token.
#[derive(Debug, Clone)]
pub enum TokValue {
    None,
    Int(i64),
    BigInt(BigInt),
    Float(f64),
    Text(String),
}

#[derive(Debug, Clone)]
pub struct Token {
    pub ty: Tok,
    pub value: TokValue,
    pub line: i64,
    pub column: i64,
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
    pos: usize,
    line: i64,
    column: i64,
}

impl Lexer {
    pub fn new(text: &str) -> Self {
        Lexer { chars: text.chars().collect(), pos: 0, line: 1, column: 1 }
    }

    fn cur(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos + 1).copied()
    }

    fn advance(&mut self) {
        if self.cur() == Some('\n') {
            self.line += 1;
            self.column = 0;
        }
        self.pos += 1;
        self.column += 1;
    }

    fn tok(&self, ty: Tok, value: TokValue, line: i64, column: i64) -> Token {
        Token { ty, value, line, column }
    }

    fn make_number(&mut self) -> Token {
        let (line, col) = (self.line, self.column);
        let mut s = String::new();
        let mut dots = 0;
        while let Some(c) = self.cur() {
            if !(c.is_ascii_digit() || c == '.') {
                break;
            }
            if c == '.' {
                dots += 1;
                if dots > 1 {
                    break;
                }
            }
            s.push(c);
            self.advance();
        }
        let value = if s.contains('.') {
            TokValue::Float(s.parse::<f64>().unwrap_or(0.0))
        } else {
            match s.parse::<i64>() {
                Ok(i) => TokValue::Int(i),
                Err(_) => TokValue::BigInt(s.parse::<BigInt>().unwrap_or_default()),
            }
        };
        self.tok(Tok::Number, value, line, col)
    }

    fn make_string(&mut self) -> Token {
        let (line, col) = (self.line, self.column);
        self.advance();
        let mut out = String::new();
        while let Some(c) = self.cur() {
            if c == '"' {
                break;
            }
            if c == '\\' {
                self.advance();
                match self.cur() {
                    Some('n') => out.push('\n'),
                    Some('t') => out.push('\t'),
                    Some('\\') => out.push('\\'),
                    Some('"') => out.push('"'),
                    Some('r') => out.push('\r'),
                    Some('0') => out.push('\0'),
                    Some(other) => {
                        out.push('\\');
                        out.push(other);
                    }
                    None => out.push('\\'),
                }
            } else {
                out.push(c);
            }
            self.advance();
        }
        self.advance();
        self.tok(Tok::Str, TokValue::Text(out), line, col)
    }

    fn make_identifier(&mut self) -> Token {
        let (line, col) = (self.line, self.column);
        let mut s = String::new();
        while let Some(c) = self.cur() {
            if !(c.is_alphanumeric() || c == '_') {
                break;
            }
            s.push(c);
            self.advance();
        }
        let ty = keyword(&s).unwrap_or(Tok::Ident);
        self.tok(ty, TokValue::Text(s), line, col)
    }

    fn make_operator(&mut self) -> Result<Token, AdarshError> {
        let (line, col) = (self.line, self.column);
        let op = self.cur().unwrap();
        if let Some(next) = self.peek() {
            if let Some(ty) = multi_op(op, next) {
                self.advance();
                self.advance();
                return Ok(self.tok(ty, TokValue::Text(format!("{op}{next}")), line, col));
            }
        }
        self.advance();
        match single_op(op) {
            Some(ty) => Ok(self.tok(ty, TokValue::Text(op.to_string()), line, col)),
            None => Err(AdarshError::Lex(format!(
                "Unexpected character '{op}' at line {line}, col {col}"
            ))),
        }
    }

    fn next_token(&mut self) -> Result<Token, AdarshError> {
        while let Some(c) = self.cur() {
            if c.is_whitespace() {
                self.advance();
                continue;
            }
            if c == '#' {
                while let Some(c) = self.cur() {
                    if c == '\n' {
                        break;
                    }
                    self.advance();
                }
                continue;
            }
            if c == '"' {
                return Ok(self.make_string());
            }
            if c.is_ascii_digit() {
                return Ok(self.make_number());
            }
            if c.is_alphabetic() || c == '_' {
                return Ok(self.make_identifier());
            }
            if single_op(c).is_some() || c == '&' || c == '|' {
                // '&' and '|' only exist as the first half of '&&' / '||'.
                return self.make_operator();
            }
            return Err(AdarshError::Lex(format!(
                "Unexpected character '{c}' at line {}, col {}",
                self.line, self.column
            )));
        }
        Ok(self.tok(Tok::Eof, TokValue::None, self.line, self.column))
    }

    pub fn tokenize(mut self) -> Result<Vec<Token>, AdarshError> {
        let mut out = Vec::new();
        loop {
            let t = self.next_token()?;
            let eof = t.ty == Tok::Eof;
            out.push(t);
            if eof {
                return Ok(out);
            }
        }
    }
}
