//! Recursive-descent parser (port of `adarsh_lang/parser.py`).
//!
//! The grammar, precedence and error messages follow the Python implementation.

use crate::ast::*;
use crate::error::AdarshError;
use crate::lexer::{Tok, TokValue, Token};
use std::rc::Rc;

type PResult<T> = Result<T, AdarshError>;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    eof: Token,
}

fn perr(msg: impl Into<String>) -> AdarshError {
    AdarshError::Parse(msg.into())
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

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser {
            tokens,
            pos: 0,
            eof: Token { ty: Tok::Eof, value: TokValue::None, line: -1, column: -1 },
        }
    }

    fn cur(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof)
    }

    fn ty(&self) -> Tok {
        self.cur().ty
    }

    fn eat(&mut self, t: Tok) -> PResult<()> {
        if self.ty() == t {
            self.pos += 1;
            Ok(())
        } else {
            let c = self.cur();
            Err(perr(format!(
                "Expected token type '{}' but got '{}' at line {}, col {}",
                t.name(),
                c.ty.name(),
                c.line,
                c.column
            )))
        }
    }

    fn eat_ident(&mut self) -> PResult<Rc<str>> {
        let name = self.cur().text();
        self.eat(Tok::Ident)?;
        Ok(name.into())
    }

    pub fn parse_program(&mut self) -> PResult<Vec<Node>> {
        let mut stmts = Vec::new();
        while self.ty() != Tok::Eof {
            let s = match self.ty() {
                Tok::Kaam => self.parse_kaam_def()?,
                Tok::Dhacha => self.parse_dhacha_def()?,
                _ => self.parse_statement()?,
            };
            stmts.push(s);
        }
        Ok(stmts)
    }

    fn parse_kaam_def(&mut self) -> PResult<Node> {
        self.eat(Tok::Kaam)?;
        let name = self.eat_ident()?;
        self.eat(Tok::LParen)?;
        let mut params = Vec::new();
        if self.ty() != Tok::RParen {
            params = self.parse_param_list()?;
        }
        self.eat(Tok::RParen)?;
        let body = self.parse_block()?;
        Ok(Node::KaamDef(Rc::new(FuncDef { name: Some(name), params, body })))
    }

    fn parse_param_list(&mut self) -> PResult<Vec<Param>> {
        let mut params = Vec::new();
        let mut seen_varargs = false;
        loop {
            let mut is_varargs = false;
            if self.ty() == Tok::Baaki {
                if seen_varargs {
                    return Err(perr(
                        "Multiple 'baaki' varargs parameters are not allowed in AdarshLang.",
                    ));
                }
                self.eat(Tok::Baaki)?;
                is_varargs = true;
                seen_varargs = true;
            }
            let name = self.eat_ident()?;
            let mut default = None;
            if self.ty() == Tok::Assign {
                if is_varargs {
                    return Err(perr("Varargs parameter cannot have a default value in AdarshLang."));
                }
                self.eat(Tok::Assign)?;
                default = Some(self.parse_expression()?);
            }
            params.push(Param { name, default, is_varargs });
            if self.ty() != Tok::Comma {
                break;
            }
            self.eat(Tok::Comma)?;
            if self.ty() == Tok::RParen {
                break;
            }
        }
        Ok(params)
    }

    fn parse_statement(&mut self) -> PResult<Node> {
        match self.ty() {
            Tok::Badlo => self.parse_badlo(),
            Tok::Agar => self.parse_agar(),
            Tok::Jabtak => {
                self.eat(Tok::Jabtak)?;
                self.eat(Tok::LParen)?;
                let cond = self.parse_expression()?;
                self.eat(Tok::RParen)?;
                let body = self.parse_block()?;
                Ok(Node::While { cond, body })
            }
            Tok::Ginnati => self.parse_ginnati(),
            Tok::KeLiye => self.parse_ke_liye(),
            Tok::Chuno => self.parse_chuno(),
            Tok::Dikhao => {
                self.eat(Tok::Dikhao)?;
                let e = self.parse_expression()?;
                self.eat(Tok::Semi)?;
                Ok(Node::Dikhao(e))
            }
            Tok::Bas => {
                self.eat(Tok::Bas)?;
                self.eat(Tok::Semi)?;
                Ok(Node::Break)
            }
            Tok::AageBadho => {
                self.eat(Tok::AageBadho)?;
                self.eat(Tok::Semi)?;
                Ok(Node::Continue)
            }
            Tok::Pakdo => {
                self.eat(Tok::Pakdo)?;
                let body = self.parse_block()?;
                self.eat(Tok::Chhoddo)?;
                self.eat(Tok::LParen)?;
                let err_name = self.eat_ident()?;
                self.eat(Tok::RParen)?;
                let handler = self.parse_block()?;
                Ok(Node::TryCatch { body, err_name, handler })
            }
            Tok::Chhoddo => {
                self.eat(Tok::Chhoddo)?;
                self.eat(Tok::LParen)?;
                let e = self.parse_expression()?;
                self.eat(Tok::RParen)?;
                self.eat(Tok::Semi)?;
                Ok(Node::Throw(e))
            }
            Tok::Lao => {
                self.eat(Tok::Lao)?;
                let e = self.parse_expression()?;
                self.eat(Tok::Semi)?;
                Ok(Node::Import(e))
            }
            Tok::Wapas => {
                self.eat(Tok::Wapas)?;
                let mut e = None;
                if self.ty() != Tok::Semi {
                    e = Some(self.parse_expression()?);
                }
                self.eat(Tok::Semi)?;
                Ok(Node::Wapas(e))
            }
            Tok::LBrace => Ok(Node::Block(self.parse_block()?)),
            _ => {
                let e = self.parse_assignment_expression()?;
                self.eat(Tok::Semi)?;
                Ok(Node::Expr(e))
            }
        }
    }

    fn parse_badlo(&mut self) -> PResult<Node> {
        self.eat(Tok::Badlo)?;
        let name = self.eat_ident()?;
        let mut init = None;
        if self.ty() == Tok::Assign {
            self.eat(Tok::Assign)?;
            init = Some(self.parse_expression()?);
        }
        self.eat(Tok::Semi)?;
        Ok(Node::Badlo(name, init))
    }

    fn parse_block(&mut self) -> PResult<Vec<Node>> {
        self.eat(Tok::LBrace)?;
        let mut stmts = Vec::new();
        while self.ty() != Tok::RBrace {
            if self.ty() == Tok::Eof {
                // Python would raise on the EOF token inside parse_statement; keep the same message.
                return Err(perr(format!(
                    "Unexpected token 'EOF' at line {}, col {}",
                    self.cur().line,
                    self.cur().column
                )));
            }
            stmts.push(self.parse_statement()?);
        }
        self.eat(Tok::RBrace)?;
        Ok(stmts)
    }

    fn parse_assignment_expression(&mut self) -> PResult<Expr> {
        let expr = self.parse_expression()?;
        if self.ty() == Tok::Assign {
            self.eat(Tok::Assign)?;
            let rhs = Box::new(self.parse_expression()?);
            return match expr {
                Expr::Var(name) => Ok(Expr::Assign(name, rhs)),
                Expr::Index(c, i) => Ok(Expr::IndexAssign(c, i, rhs)),
                Expr::Attr(t, a) => Ok(Expr::AttrAssign(t, a, rhs)),
                _ => Err(perr("Invalid assignment target in AdarshLang.")),
            };
        }
        if let Some(op) = compound_op(self.ty()) {
            self.pos += 1;
            let rhs = Box::new(self.parse_expression()?);
            return match expr {
                Expr::Var(name) => Ok(Expr::CompoundAssign(name, op, rhs)),
                Expr::Index(c, i) => Ok(Expr::IndexCompoundAssign(c, i, op, rhs)),
                Expr::Attr(t, a) => Ok(Expr::AttrCompoundAssign(t, a, op, rhs)),
                _ => Err(perr("Invalid compound assignment target in AdarshLang.")),
            };
        }
        Ok(expr)
    }

    pub fn parse_expression(&mut self) -> PResult<Expr> {
        self.parse_or()
    }

    fn binary_level(
        &mut self,
        ops: &[(Tok, BinOp)],
        next: fn(&mut Self) -> PResult<Expr>,
    ) -> PResult<Expr> {
        let mut node = next(self)?;
        'outer: loop {
            for (t, op) in ops {
                if self.ty() == *t {
                    self.pos += 1;
                    let rhs = next(self)?;
                    node = Expr::Binary(Box::new(node), *op, Box::new(rhs));
                    continue 'outer;
                }
            }
            return Ok(node);
        }
    }

    fn parse_or(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Ya, BinOp::Or)], Self::parse_and)
    }
    fn parse_and(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Aur, BinOp::And)], Self::parse_equality)
    }
    fn parse_equality(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Eq, BinOp::Eq), (Tok::Neq, BinOp::Neq)], Self::parse_comparison)
    }
    fn parse_comparison(&mut self) -> PResult<Expr> {
        self.binary_level(
            &[(Tok::Lt, BinOp::Lt), (Tok::Gt, BinOp::Gt), (Tok::Lte, BinOp::Lte), (Tok::Gte, BinOp::Gte)],
            Self::parse_term,
        )
    }
    fn parse_term(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Plus, BinOp::Add), (Tok::Minus, BinOp::Sub)], Self::parse_factor)
    }
    fn parse_factor(&mut self) -> PResult<Expr> {
        self.binary_level(
            &[(Tok::Mul, BinOp::Mul), (Tok::Div, BinOp::Div), (Tok::Mod, BinOp::Mod)],
            Self::parse_unary,
        )
    }

    fn parse_unary(&mut self) -> PResult<Expr> {
        let op = match self.ty() {
            Tok::Nahin => UnOp::Not,
            Tok::Minus => UnOp::Neg,
            _ => return self.parse_power(),
        };
        self.pos += 1;
        let operand = self.parse_unary()?;
        Ok(Expr::Unary(op, Box::new(operand)))
    }

    fn parse_power(&mut self) -> PResult<Expr> {
        let node = self.parse_postfix()?;
        if self.ty() == Tok::Power {
            self.pos += 1;
            let rhs = self.parse_unary()?;
            return Ok(Expr::Binary(Box::new(node), BinOp::Pow, Box::new(rhs)));
        }
        Ok(node)
    }

    fn parse_postfix(&mut self) -> PResult<Expr> {
        let mut node = self.parse_primary()?;
        loop {
            match self.ty() {
                Tok::LParen => {
                    self.eat(Tok::LParen)?;
                    let mut args = Vec::new();
                    if self.ty() != Tok::RParen {
                        args.push(self.parse_expression()?);
                        while self.ty() == Tok::Comma {
                            self.eat(Tok::Comma)?;
                            args.push(self.parse_expression()?);
                        }
                    }
                    self.eat(Tok::RParen)?;
                    node = Expr::Call(Box::new(node), args);
                }
                Tok::LBracket => {
                    self.eat(Tok::LBracket)?;
                    let idx = self.parse_expression()?;
                    self.eat(Tok::RBracket)?;
                    node = Expr::Index(Box::new(node), Box::new(idx));
                }
                Tok::Dot => {
                    self.eat(Tok::Dot)?;
                    let attr = self.eat_ident()?;
                    node = Expr::Attr(Box::new(node), attr);
                }
                Tok::LBrace if matches!(node, Expr::Var(_)) => {
                    let Expr::Var(type_name) = node else { unreachable!() };
                    node = self.parse_dhacha_construction(type_name)?;
                }
                _ => return Ok(node),
            }
        }
    }

    fn parse_primary(&mut self) -> PResult<Expr> {
        let tok = self.cur().clone();
        let node = match tok.ty {
            Tok::Ident => {
                self.pos += 1;
                Expr::Var(tok.text().into())
            }
            Tok::Number => {
                self.pos += 1;
                match tok.value {
                    TokValue::Int(i) => Expr::Int(i),
                    TokValue::BigInt(b) => Expr::BigInt(Rc::new(b)),
                    TokValue::Float(f) => Expr::Float(f),
                    _ => Expr::Int(0),
                }
            }
            Tok::SahiHaiBe => {
                self.pos += 1;
                Expr::Bool(true)
            }
            Tok::Jhuth => {
                self.pos += 1;
                Expr::Bool(false)
            }
            Tok::Khali => {
                self.pos += 1;
                Expr::Null
            }
            Tok::Str => {
                self.pos += 1;
                Expr::Str(tok.text().into())
            }
            Tok::LBracket => {
                self.eat(Tok::LBracket)?;
                let mut items = Vec::new();
                if self.ty() != Tok::RBracket {
                    items.push(self.parse_expression()?);
                    while self.ty() == Tok::Comma {
                        self.eat(Tok::Comma)?;
                        items.push(self.parse_expression()?);
                    }
                }
                self.eat(Tok::RBracket)?;
                Expr::List(items)
            }
            Tok::LBrace => {
                self.eat(Tok::LBrace)?;
                let mut entries = Vec::new();
                if self.ty() != Tok::RBrace {
                    loop {
                        let k = self.parse_expression()?;
                        self.eat(Tok::Colon)?;
                        let v = self.parse_expression()?;
                        entries.push((k, v));
                        if self.ty() != Tok::Comma {
                            break;
                        }
                        self.eat(Tok::Comma)?;
                    }
                }
                self.eat(Tok::RBrace)?;
                Expr::Dict(entries)
            }
            Tok::LParen => {
                self.eat(Tok::LParen)?;
                let e = self.parse_expression()?;
                self.eat(Tok::RParen)?;
                e
            }
            Tok::Kaam => {
                self.eat(Tok::Kaam)?;
                self.eat(Tok::LParen)?;
                let mut params = Vec::new();
                if self.ty() != Tok::RParen {
                    params = self.parse_param_list()?;
                }
                self.eat(Tok::RParen)?;
                let body = self.parse_block()?;
                Expr::Func(Rc::new(FuncDef { name: None, params, body }))
            }
            _ => {
                return Err(perr(format!(
                    "Unexpected token '{}' at line {}, col {}",
                    tok.ty.name(),
                    tok.line,
                    tok.column
                )))
            }
        };
        Ok(node)
    }

    fn parse_dhacha_construction(&mut self, type_name: Rc<str>) -> PResult<Expr> {
        self.eat(Tok::LBrace)?;
        let mut fields = Vec::new();
        if self.ty() != Tok::RBrace {
            loop {
                let name = self.eat_ident()?;
                self.eat(Tok::Colon)?;
                let e = self.parse_expression()?;
                fields.push((name, e));
                if self.ty() != Tok::Comma {
                    break;
                }
                self.eat(Tok::Comma)?;
            }
        }
        self.eat(Tok::RBrace)?;
        Ok(Expr::Dhacha { type_name, fields })
    }

    fn parse_agar(&mut self) -> PResult<Node> {
        self.eat(Tok::Agar)?;
        self.eat(Tok::LParen)?;
        let cond = self.parse_expression()?;
        self.eat(Tok::RParen)?;
        let then = self.parse_block()?;
        let mut otherwise = None;
        if self.ty() == Tok::Warna {
            self.eat(Tok::Warna)?;
            otherwise = Some(Box::new(if self.ty() == Tok::Agar {
                self.parse_agar()?
            } else {
                Node::Block(self.parse_block()?)
            }));
        }
        Ok(Node::If { cond, then, otherwise })
    }

    fn parse_ginnati(&mut self) -> PResult<Node> {
        self.eat(Tok::Ginnati)?;
        self.eat(Tok::LParen)?;
        let mut init = None;
        if self.ty() != Tok::Semi {
            if self.ty() == Tok::Badlo {
                init = Some(Box::new(self.parse_badlo()?));
            } else {
                init = Some(Box::new(Node::Expr(self.parse_assignment_expression()?)));
                self.eat(Tok::Semi)?;
            }
        } else {
            self.eat(Tok::Semi)?;
        }
        let mut cond = None;
        if self.ty() != Tok::Semi {
            cond = Some(self.parse_expression()?);
        }
        self.eat(Tok::Semi)?;
        let mut update = None;
        if self.ty() != Tok::RParen {
            update = Some(self.parse_assignment_expression()?);
        }
        self.eat(Tok::RParen)?;
        let body = self.parse_block()?;
        Ok(Node::For { init, cond, update, body })
    }

    fn parse_ke_liye(&mut self) -> PResult<Node> {
        self.eat(Tok::KeLiye)?;
        self.eat(Tok::LParen)?;
        let mut declare = false;
        if self.ty() == Tok::Badlo {
            self.eat(Tok::Badlo)?;
            declare = true;
        }
        let var = self.eat_ident()?;
        if self.ty() == Tok::In {
            self.eat(Tok::In)?;
        } else {
            self.eat(Tok::Colon)?;
        }
        let iterable = self.parse_expression()?;
        self.eat(Tok::RParen)?;
        let body = self.parse_block()?;
        Ok(Node::ForEach { var, declare, iterable, body })
    }

    fn parse_chuno(&mut self) -> PResult<Node> {
        self.eat(Tok::Chuno)?;
        self.eat(Tok::LParen)?;
        let subject = self.parse_expression()?;
        self.eat(Tok::RParen)?;
        self.eat(Tok::LBrace)?;
        let mut cases = Vec::new();
        let mut default = None;
        loop {
            match self.ty() {
                Tok::Case => {
                    self.eat(Tok::Case)?;
                    let mut matches = vec![self.parse_expression()?];
                    while self.ty() == Tok::Comma {
                        self.eat(Tok::Comma)?;
                        matches.push(self.parse_expression()?);
                    }
                    self.eat(Tok::Colon)?;
                    let body = self.parse_case_body()?;
                    cases.push(SwitchCase { matches, body });
                }
                Tok::WarnaCase => {
                    self.eat(Tok::WarnaCase)?;
                    self.eat(Tok::Colon)?;
                    default = Some(self.parse_case_body()?);
                    break;
                }
                _ => break,
            }
        }
        self.eat(Tok::RBrace)?;
        Ok(Node::Switch { subject, cases, default })
    }

    fn parse_case_body(&mut self) -> PResult<Vec<Node>> {
        if self.ty() == Tok::LBrace {
            return self.parse_block();
        }
        Ok(vec![self.parse_statement()?])
    }

    fn parse_dhacha_def(&mut self) -> PResult<Node> {
        self.eat(Tok::Dhacha)?;
        let name = self.eat_ident()?;
        self.eat(Tok::LBrace)?;
        let mut fields = Vec::new();
        if self.ty() != Tok::RBrace {
            fields.push(self.eat_ident()?);
            while self.ty() == Tok::Comma {
                self.eat(Tok::Comma)?;
                fields.push(self.eat_ident()?);
            }
        }
        self.eat(Tok::RBrace)?;
        self.eat(Tok::Semi)?;
        Ok(Node::DhachaDef(name, Rc::new(fields)))
    }
}

pub fn parse(source: &str) -> Result<Vec<Node>, AdarshError> {
    let tokens = crate::lexer::Lexer::new(source).tokenize()?;
    Parser::new(tokens).parse_program()
}
