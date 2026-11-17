//! Recursive-descent parser.
//!
//! Precedence (low → high): `ya` · `aur` · `== !=` · `< > <= >=` · `+ -` ·
//! `* / %` · unary `- nahin` · `**` (right-assoc) · postfix call/index/attr.

use crate::ast::*;
use crate::error::{AdarshError, Pos};
use crate::lexer::{Lexer, RawPart, Tok, TokValue, Token};
use std::rc::Rc;

type PResult<T> = Result<T, AdarshError>;

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
}

fn compound_op(t: Tok) -> Option<BinOp> {
    Some(match t {
        Tok::PlusAssign => BinOp::Add,
        Tok::MinusAssign => BinOp::Sub,
        Tok::MulAssign => BinOp::Mul,
        Tok::DivAssign => BinOp::Div,
        Tok::ModAssign => BinOp::Mod,
        _ => return None,
    })
}

fn expr(kind: ExprKind, pos: Pos) -> Expr {
    Expr { kind, pos }
}

fn node(kind: NodeKind, pos: Pos) -> Node {
    Node { kind, pos }
}

/// Move `dhacha` and `kaam` definitions to the front of a block (stable), so
/// types and functions can be used before the line that defines them.
fn hoist(stmts: Vec<Node>) -> Vec<Node> {
    let (mut types, mut funcs, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    for s in stmts {
        match s.kind {
            NodeKind::DhachaDef(..) => types.push(s),
            NodeKind::KaamDef(_) => funcs.push(s),
            _ => rest.push(s),
        }
    }
    types.extend(funcs);
    types.extend(rest);
    types
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, i: 0 }
    }

    fn cur(&self) -> &Token {
        &self.tokens[self.i.min(self.tokens.len() - 1)]
    }
    fn ty(&self) -> Tok {
        self.cur().ty
    }
    fn peek_ty(&self, n: usize) -> Tok {
        self.tokens.get(self.i + n).map(|t| t.ty).unwrap_or(Tok::Eof)
    }
    fn pos(&self) -> Pos {
        self.cur().pos
    }
    fn bump(&mut self) -> Token {
        let t = self.cur().clone();
        if self.i < self.tokens.len() - 1 {
            self.i += 1;
        }
        t
    }

    fn expected(&self, what: &str) -> AdarshError {
        let c = self.cur();
        let got = match c.ty {
            Tok::Ident => format!("naam '{}'", c.text()),
            other => format!("'{}'", other.describe()),
        };
        let msg = format!("Yahan {what} hona chahiye tha, par {got} mila.");
        // a missing ';' belongs at the end of the previous line, not the start of the next
        if what == "';'" && self.i > 0 {
            let prev = &self.tokens[self.i - 1];
            if prev.end.line < c.pos.line {
                return AdarshError::syntax("Line ke ant mein ';' missing hai.", prev.end);
            }
        }
        AdarshError::syntax(msg, c.pos)
    }

    fn eat(&mut self, t: Tok) -> PResult<Token> {
        if self.ty() == t {
            Ok(self.bump())
        } else {
            Err(self.expected(&format!("'{}'", t.describe())))
        }
    }

    fn ident(&mut self) -> PResult<(Rc<str>, Pos)> {
        if self.ty() == Tok::Ident {
            let t = self.bump();
            Ok((t.text().into(), t.pos))
        } else {
            Err(self.expected("ek naam"))
        }
    }

    pub fn parse_program(&mut self) -> PResult<Vec<Node>> {
        let mut stmts = Vec::new();
        while self.ty() != Tok::Eof {
            stmts.push(self.statement()?);
        }
        Ok(hoist(stmts))
    }

    fn block(&mut self) -> PResult<Vec<Node>> {
        self.eat(Tok::LBrace)?;
        let mut stmts = Vec::new();
        while self.ty() != Tok::RBrace {
            if self.ty() == Tok::Eof {
                return Err(self.expected("'}'"));
            }
            stmts.push(self.statement()?);
        }
        self.eat(Tok::RBrace)?;
        Ok(hoist(stmts))
    }

    fn params(&mut self) -> PResult<Vec<Param>> {
        self.eat(Tok::LParen)?;
        let mut params: Vec<Param> = Vec::new();
        while self.ty() != Tok::RParen {
            let is_varargs = if self.ty() == Tok::Baaki {
                let p = self.bump().pos;
                if params.iter().any(|p| p.is_varargs) {
                    return Err(AdarshError::syntax("Sirf ek 'baaki' parameter ho sakta hai.", p));
                }
                true
            } else {
                false
            };
            if params.last().is_some_and(|p| p.is_varargs) {
                return Err(AdarshError::syntax("'baaki' parameter sabse aakhri hona chahiye.", self.pos()));
            }
            let (name, npos) = self.ident()?;
            if params.iter().any(|p| p.name == name) {
                return Err(AdarshError::syntax(format!("Parameter '{name}' do baar likha hai."), npos));
            }
            let mut default = None;
            if self.ty() == Tok::Assign {
                if is_varargs {
                    return Err(AdarshError::syntax("'baaki' parameter ka default nahi ho sakta.", self.pos()));
                }
                self.bump();
                default = Some(self.expression()?);
            } else if !is_varargs && params.iter().any(|p| p.default.is_some()) {
                return Err(AdarshError::syntax(
                    format!(
                        "Parameter '{name}' default waale parameters ke baad hai, isliye iska bhi default chahiye."
                    ),
                    npos,
                ));
            }
            params.push(Param { name, default, is_varargs });
            if self.ty() != Tok::Comma {
                break;
            }
            self.bump();
        }
        self.eat(Tok::RParen)?;
        Ok(params)
    }

    fn kaam_def(&mut self) -> PResult<Node> {
        let pos = self.eat(Tok::Kaam)?.pos;
        let (first, _) = self.ident()?;
        let (name, method_of) = if self.ty() == Tok::Dot {
            self.bump();
            let (m, _) = self.ident()?;
            (m, Some(first))
        } else {
            (first, None)
        };
        let params = self.params()?;
        let body = self.block()?;
        Ok(node(NodeKind::KaamDef(Rc::new(FuncDef { name: Some(name), method_of, params, body, pos })), pos))
    }

    fn dhacha_def(&mut self) -> PResult<Node> {
        let pos = self.eat(Tok::Dhacha)?.pos;
        let (name, _) = self.ident()?;
        self.eat(Tok::LBrace)?;
        let mut fields: Vec<Rc<str>> = Vec::new();
        while self.ty() != Tok::RBrace {
            let (f, fpos) = self.ident()?;
            if fields.contains(&f) {
                return Err(AdarshError::syntax(format!("Field '{f}' do baar likha hai."), fpos));
            }
            fields.push(f);
            if self.ty() != Tok::Comma {
                break;
            }
            self.bump();
        }
        self.eat(Tok::RBrace)?;
        if self.ty() == Tok::Semi {
            self.bump();
        }
        Ok(node(NodeKind::DhachaDef(name, Rc::new(fields)), pos))
    }

    fn semi(&mut self) -> PResult<()> {
        self.eat(Tok::Semi).map(|_| ())
    }

    fn statement(&mut self) -> PResult<Node> {
        let pos = self.pos();
        let kind = match self.ty() {
            Tok::Kaam if self.peek_ty(1) == Tok::Ident => return self.kaam_def(),
            Tok::Dhacha => return self.dhacha_def(),
            Tok::Badlo => {
                let n = self.badlo()?;
                self.semi()?;
                n
            }
            Tok::Agar => return self.agar(),
            Tok::Jabtak => {
                self.bump();
                self.eat(Tok::LParen)?;
                let cond = self.expression()?;
                self.eat(Tok::RParen)?;
                NodeKind::While { cond, body: self.block()? }
            }
            Tok::Ginnati => self.ginnati()?,
            Tok::KeLiye => self.ke_liye()?,
            Tok::Chuno => self.chuno()?,
            Tok::Dikhao => {
                self.bump();
                let e = self.expression()?;
                self.semi()?;
                NodeKind::Dikhao(e)
            }
            Tok::Bas => {
                self.bump();
                self.semi()?;
                NodeKind::Break
            }
            Tok::AageBadho => {
                self.bump();
                self.semi()?;
                NodeKind::Continue
            }
            Tok::Pakdo => {
                self.bump();
                let body = self.block()?;
                self.eat(Tok::Chhoddo)?;
                self.eat(Tok::LParen)?;
                let (err_name, _) = self.ident()?;
                self.eat(Tok::RParen)?;
                NodeKind::TryCatch { body, err_name, handler: self.block()? }
            }
            Tok::Chhoddo => {
                self.bump();
                self.eat(Tok::LParen)?;
                let e = self.expression()?;
                self.eat(Tok::RParen)?;
                self.semi()?;
                NodeKind::Throw(e)
            }
            Tok::Lao => {
                self.bump();
                let e = self.expression()?;
                self.semi()?;
                NodeKind::Import(e)
            }
            Tok::Wapas => {
                self.bump();
                let e = if self.ty() == Tok::Semi { None } else { Some(self.expression()?) };
                self.semi()?;
                NodeKind::Wapas(e)
            }
            Tok::LBrace => NodeKind::Block(self.block()?),
            _ => {
                let e = self.assignment()?;
                self.semi()?;
                NodeKind::Expr(e)
            }
        };
        Ok(node(kind, pos))
    }

    fn badlo(&mut self) -> PResult<NodeKind> {
        self.eat(Tok::Badlo)?;
        let (name, _) = self.ident()?;
        let init = if self.ty() == Tok::Assign {
            self.bump();
            Some(self.expression()?)
        } else {
            None
        };
        Ok(NodeKind::Badlo(name, init))
    }

    fn assignment(&mut self) -> PResult<Expr> {
        let target = self.expression()?;
        let pos = self.pos();
        if self.ty() == Tok::Assign {
            self.bump();
            let rhs = Box::new(self.expression()?);
            let kind = match target.kind {
                ExprKind::Var(n) => ExprKind::Assign(n, rhs),
                ExprKind::Index(c, i) => ExprKind::IndexAssign(c, i, rhs),
                ExprKind::Attr(t, a) => ExprKind::AttrAssign(t, a, rhs),
                _ => {
                    return Err(AdarshError::syntax(
                        "'=' ke left mein variable, list[index] ya object.field hona chahiye.",
                        target.pos,
                    ))
                }
            };
            return Ok(expr(kind, pos));
        }
        if let Some(op) = compound_op(self.ty()) {
            self.bump();
            let rhs = Box::new(self.expression()?);
            let kind = match target.kind {
                ExprKind::Var(n) => ExprKind::CompoundAssign(n, op, rhs),
                ExprKind::Index(c, i) => ExprKind::IndexCompoundAssign(c, i, op, rhs),
                ExprKind::Attr(t, a) => ExprKind::AttrCompoundAssign(t, a, op, rhs),
                _ => {
                    return Err(AdarshError::syntax(
                        format!("'{}=' ke left mein variable, list[index] ya object.field hona chahiye.", op.symbol()),
                        target.pos,
                    ))
                }
            };
            return Ok(expr(kind, pos));
        }
        Ok(target)
    }

    pub fn expression(&mut self) -> PResult<Expr> {
        self.or()
    }

    fn binary_level(&mut self, ops: &[(Tok, BinOp)], next: fn(&mut Self) -> PResult<Expr>) -> PResult<Expr> {
        let mut lhs = next(self)?;
        'outer: loop {
            for (t, op) in ops {
                if self.ty() == *t {
                    let pos = self.bump().pos;
                    let rhs = next(self)?;
                    lhs = expr(ExprKind::Binary(Box::new(lhs), *op, Box::new(rhs)), pos);
                    continue 'outer;
                }
            }
            return Ok(lhs);
        }
    }

    fn or(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Ya, BinOp::Or)], Self::and)
    }
    fn and(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Aur, BinOp::And)], Self::equality)
    }
    fn equality(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Eq, BinOp::Eq), (Tok::Neq, BinOp::Neq)], Self::comparison)
    }
    fn comparison(&mut self) -> PResult<Expr> {
        self.binary_level(
            &[(Tok::Lt, BinOp::Lt), (Tok::Gt, BinOp::Gt), (Tok::Lte, BinOp::Lte), (Tok::Gte, BinOp::Gte)],
            Self::term,
        )
    }
    fn term(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Plus, BinOp::Add), (Tok::Minus, BinOp::Sub)], Self::factor)
    }
    fn factor(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Mul, BinOp::Mul), (Tok::Div, BinOp::Div), (Tok::Mod, BinOp::Mod)], Self::unary)
    }

    fn unary(&mut self) -> PResult<Expr> {
        let op = match self.ty() {
            Tok::Nahin => UnOp::Not,
            Tok::Minus => UnOp::Neg,
            _ => return self.power(),
        };
        let pos = self.bump().pos;
        let operand = self.unary()?;
        Ok(expr(ExprKind::Unary(op, Box::new(operand)), pos))
    }

    fn power(&mut self) -> PResult<Expr> {
        let base = self.postfix()?;
        if self.ty() == Tok::Power {
            let pos = self.bump().pos;
            let exp = self.unary()?;
            return Ok(expr(ExprKind::Binary(Box::new(base), BinOp::Pow, Box::new(exp)), pos));
        }
        Ok(base)
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let mut e = self.primary()?;
        loop {
            match self.ty() {
                Tok::LParen => {
                    let pos = self.bump().pos;
                    let mut args = Vec::new();
                    while self.ty() != Tok::RParen {
                        args.push(self.expression()?);
                        if self.ty() != Tok::Comma {
                            break;
                        }
                        self.bump();
                    }
                    self.eat(Tok::RParen)?;
                    e = expr(ExprKind::Call(Box::new(e), args), pos);
                }
                Tok::LBracket => {
                    let pos = self.bump().pos;
                    let idx = self.expression()?;
                    self.eat(Tok::RBracket)?;
                    e = expr(ExprKind::Index(Box::new(e), Box::new(idx)), pos);
                }
                Tok::Dot => {
                    let pos = self.bump().pos;
                    let (attr, _) = self.ident()?;
                    e = expr(ExprKind::Attr(Box::new(e), attr), pos);
                }
                // `Naam { field: value }` record construction
                Tok::LBrace
                    if matches!(&e.kind, ExprKind::Var(_))
                        && (self.peek_ty(1) == Tok::RBrace
                            || (self.peek_ty(1) == Tok::Ident && self.peek_ty(2) == Tok::Colon)) =>
                {
                    let ExprKind::Var(type_name) = e.kind else { unreachable!() };
                    let pos = e.pos;
                    self.bump();
                    let mut fields = Vec::new();
                    while self.ty() != Tok::RBrace {
                        let (f, _) = self.ident()?;
                        self.eat(Tok::Colon)?;
                        fields.push((f, self.expression()?));
                        if self.ty() != Tok::Comma {
                            break;
                        }
                        self.bump();
                    }
                    self.eat(Tok::RBrace)?;
                    e = expr(ExprKind::Dhacha { type_name, fields }, pos);
                }
                _ => return Ok(e),
            }
        }
    }

    fn primary(&mut self) -> PResult<Expr> {
        let tok = self.bump();
        let pos = tok.pos;
        let kind = match tok.ty {
            Tok::Ident => ExprKind::Var(tok.text().into()),
            Tok::Number => match tok.value {
                TokValue::Int(i) => ExprKind::Int(i),
                TokValue::BigInt(b) => ExprKind::BigInt(Rc::new(b)),
                TokValue::Float(f) => ExprKind::Float(f),
                _ => unreachable!(),
            },
            Tok::Str => match tok.value {
                TokValue::Interp(parts) => {
                    let mut out = Vec::new();
                    for p in parts {
                        out.push(match p {
                            RawPart::Lit(s) => Part::Lit(s.into()),
                            RawPart::Code(code, cpos) => {
                                let tokens = Lexer::at(&code, cpos).tokenize()?;
                                let mut sub = Parser::new(tokens);
                                let e = sub.expression()?;
                                if sub.ty() != Tok::Eof {
                                    return Err(sub.expected("'}'"));
                                }
                                Part::Expr(e)
                            }
                        });
                    }
                    ExprKind::Interp(out)
                }
                _ => ExprKind::Str(tok.text().into()),
            },
            Tok::SahiHaiBe => ExprKind::Bool(true),
            Tok::Jhuth => ExprKind::Bool(false),
            Tok::Khali => ExprKind::Null,
            Tok::LBracket => {
                let mut items = Vec::new();
                while self.ty() != Tok::RBracket {
                    items.push(self.expression()?);
                    if self.ty() != Tok::Comma {
                        break;
                    }
                    self.bump();
                }
                self.eat(Tok::RBracket)?;
                ExprKind::List(items)
            }
            Tok::LBrace => {
                let mut entries = Vec::new();
                while self.ty() != Tok::RBrace {
                    let k = self.expression()?;
                    self.eat(Tok::Colon)?;
                    let v = self.expression()?;
                    entries.push((k, v));
                    if self.ty() != Tok::Comma {
                        break;
                    }
                    self.bump();
                }
                self.eat(Tok::RBrace)?;
                ExprKind::Dict(entries)
            }
            Tok::LParen => {
                let e = self.expression()?;
                self.eat(Tok::RParen)?;
                return Ok(e);
            }
            Tok::Kaam => {
                let params = self.params()?;
                let body = self.block()?;
                ExprKind::Func(Rc::new(FuncDef { name: None, method_of: None, params, body, pos }))
            }
            other => {
                self.i -= 1;
                let what = match other {
                    Tok::Eof => "Program adhoora reh gaya: yahan koi value ya expression chahiye tha.".to_string(),
                    _ => format!(
                        "'{}' yahan samajh nahi aaya. Yahan koi value ya expression chahiye tha.",
                        other.describe()
                    ),
                };
                return Err(AdarshError::syntax(what, pos));
            }
        };
        Ok(expr(kind, pos))
    }

    fn agar(&mut self) -> PResult<Node> {
        let pos = self.eat(Tok::Agar)?.pos;
        self.eat(Tok::LParen)?;
        let cond = self.expression()?;
        self.eat(Tok::RParen)?;
        let then = self.block()?;
        let mut otherwise = None;
        if self.ty() == Tok::Warna {
            let wpos = self.bump().pos;
            otherwise = Some(Box::new(if self.ty() == Tok::Agar {
                self.agar()?
            } else {
                node(NodeKind::Block(self.block()?), wpos)
            }));
        }
        Ok(node(NodeKind::If { cond, then, otherwise }, pos))
    }

    fn ginnati(&mut self) -> PResult<NodeKind> {
        self.eat(Tok::Ginnati)?;
        self.eat(Tok::LParen)?;
        let mut init = None;
        if self.ty() != Tok::Semi {
            let ipos = self.pos();
            let kind = if self.ty() == Tok::Badlo { self.badlo()? } else { NodeKind::Expr(self.assignment()?) };
            init = Some(Box::new(node(kind, ipos)));
        }
        self.semi()?;
        let cond = if self.ty() != Tok::Semi { Some(self.expression()?) } else { None };
        self.semi()?;
        let update = if self.ty() != Tok::RParen { Some(self.assignment()?) } else { None };
        self.eat(Tok::RParen)?;
        Ok(NodeKind::For { init, cond, update, body: self.block()? })
    }

    fn ke_liye(&mut self) -> PResult<NodeKind> {
        self.eat(Tok::KeLiye)?;
        self.eat(Tok::LParen)?;
        let declare = if self.ty() == Tok::Badlo {
            self.bump();
            true
        } else {
            false
        };
        let (var, _) = self.ident()?;
        if self.ty() == Tok::In || self.ty() == Tok::Colon {
            self.bump();
        } else {
            return Err(self.expected("'in'"));
        }
        let iterable = self.expression()?;
        self.eat(Tok::RParen)?;
        Ok(NodeKind::ForEach { var, declare, iterable, body: self.block()? })
    }

    fn chuno(&mut self) -> PResult<NodeKind> {
        self.eat(Tok::Chuno)?;
        self.eat(Tok::LParen)?;
        let subject = self.expression()?;
        self.eat(Tok::RParen)?;
        self.eat(Tok::LBrace)?;
        let mut cases = Vec::new();
        let mut default = None;
        loop {
            match self.ty() {
                Tok::Case => {
                    self.bump();
                    let mut matches = vec![self.expression()?];
                    while self.ty() == Tok::Comma {
                        self.bump();
                        matches.push(self.expression()?);
                    }
                    self.eat(Tok::Colon)?;
                    cases.push(SwitchCase { matches, body: self.case_body()? });
                }
                Tok::WarnaCase => {
                    self.bump();
                    self.eat(Tok::Colon)?;
                    default = Some(self.case_body()?);
                }
                Tok::RBrace => break,
                _ => return Err(self.expected("'case', 'warna_case' ya '}'")),
            }
        }
        self.eat(Tok::RBrace)?;
        Ok(NodeKind::Switch { subject, cases, default })
    }

    /// A braced block, or statements up to the next `case` / `warna_case` / `}`.
    fn case_body(&mut self) -> PResult<Vec<Node>> {
        if self.ty() == Tok::LBrace {
            return self.block();
        }
        let mut stmts = Vec::new();
        while !matches!(self.ty(), Tok::Case | Tok::WarnaCase | Tok::RBrace | Tok::Eof) {
            stmts.push(self.statement()?);
        }
        Ok(stmts)
    }
}

/// Lex and parse a whole program. `file` is the id from [`crate::error::register_source`].
pub fn parse(source: &str, file: u32) -> Result<Vec<Node>, AdarshError> {
    let tokens = Lexer::new(source, file).tokenize()?;
    Parser::new(tokens).parse_program()
}
