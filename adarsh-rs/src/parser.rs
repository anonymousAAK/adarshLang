//! Recursive-descent parser.
//!
//! Expression precedence, lowest first:
//! `??` · `ya` · `aur` · `== !=` · `< > <= >= in hai` (chainable) · `..` ·
//! `+ -` · `* / // %` · unary `- nahin` · `**` (right-assoc) ·
//! postfix: call, `[index]`, `[a:b:c]`, `.field`, `?.field`, `Type { ... }`.

use crate::ast::*;
use crate::error::{AdarshError, Pos};
use crate::lexer::{Lexer, RawPart, Tok, TokValue, Token};
use std::rc::Rc;

type PResult<T> = Result<T, AdarshError>;

pub struct Parser {
    tokens: Vec<Token>,
    i: usize,
    /// One entry per function being parsed: whether it contains `dete_raho`.
    yields: Vec<bool>,
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

/// Move type and function definitions to the front of a block (keeping their
/// order), so they can be used before the line that defines them.
fn hoist(stmts: Vec<Node>) -> Vec<Node> {
    let (mut types, mut funcs, mut rest) = (Vec::new(), Vec::new(), Vec::new());
    for s in stmts {
        match s.kind {
            NodeKind::DhachaDef(_) | NodeKind::EnumDef(..) => types.push(s),
            NodeKind::KaamDef(_) => funcs.push(s),
            _ => rest.push(s),
        }
    }
    types.extend(funcs);
    types.extend(rest);
    types
}

/// Split `expr:spec` inside an interpolation, if the part after the last
/// top-level `:` looks like a format spec (`.2`, `>8`, `,`, `08.3`).
fn split_format_spec(code: &str) -> (&str, Option<&str>) {
    let (mut depth, mut in_str, mut esc, mut last_colon) = (0i32, false, false, None);
    for (i, c) in code.char_indices() {
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (false, '\\') => esc = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            ':' if depth == 0 => last_colon = Some(i),
            _ => {}
        }
    }
    if let Some(i) = last_colon {
        let spec = code[i + 1..].trim();
        let ok = !spec.is_empty()
            && spec.chars().all(|c| c.is_ascii_digit() || matches!(c, '.' | '<' | '>' | '^' | ',' | '%' | '+'));
        if ok {
            return (&code[..i], Some(spec));
        }
    }
    (code, None)
}

const LOOP_TOKENS: [Tok; 4] = [Tok::Jabtak, Tok::Ginnati, Tok::KeLiye, Tok::Karo];

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, i: 0, yields: Vec::new() }
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
    fn accept(&mut self, t: Tok) -> bool {
        if self.ty() == t {
            self.bump();
            true
        } else {
            false
        }
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
                return AdarshError::syntax("Line ke ant mein ';' missing hai.", prev.end).with_code("E101");
            }
        }
        AdarshError::syntax(msg, c.pos).with_code("E100")
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
            Ok((crate::ast::intern(&t.text()), t.pos))
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

    // ---------------------------------------------------------------- types

    fn type_ann(&mut self) -> PResult<TypeAnn> {
        let pos = self.pos();
        let name: Rc<str> = match self.ty() {
            Tok::Ident => crate::ast::intern(&self.bump().text()),
            Tok::Khali => {
                self.bump();
                "khali".into()
            }
            Tok::Kaam | Tok::Dhacha | Tok::Vikalp => self.bump().ty.describe().into(),
            _ => return Err(self.expected("type ka naam (jaise number, string, list)")),
        };
        Ok(TypeAnn { name, nullable: false, pos })
    }

    // ---------------------------------------------------------------- functions

    fn params(&mut self) -> PResult<Vec<Param>> {
        self.eat(Tok::LParen)?;
        let mut params: Vec<Param> = Vec::new();
        while self.ty() != Tok::RParen {
            let is_varargs = if self.ty() == Tok::Baaki || self.ty() == Tok::Ellipsis {
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
            let ty = if self.accept(Tok::Colon) { Some(self.type_ann()?) } else { None };
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
            params.push(Param { name, default, is_varargs, ty });
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        self.eat(Tok::RParen)?;
        Ok(params)
    }

    /// Optional `-> type` after a parameter list.
    fn return_type(&mut self) -> PResult<Option<TypeAnn>> {
        if self.ty() == Tok::Minus && self.peek_ty(1) == Tok::Gt {
            self.bump();
            self.bump();
            return Ok(Some(self.type_ann()?));
        }
        Ok(None)
    }

    /// Parse a function body, noting whether it uses `dete_raho`.
    fn function_body(&mut self) -> PResult<(Vec<Node>, bool)> {
        self.yields.push(false);
        let body = self.block();
        let generator = self.yields.pop().unwrap_or(false);
        Ok((body?, generator))
    }

    fn kaam_def(&mut self) -> PResult<Node> {
        let kw = self.eat(Tok::Kaam)?;
        let pos = kw.pos;
        let (first, _) = self.ident()?;
        let (name, method_of) = if self.accept(Tok::Dot) {
            let (m, _) = self.ident()?;
            (m, Some(first))
        } else {
            (first, None)
        };
        let params = self.params()?;
        let ret = self.return_type()?;
        let (body, generator) = self.function_body()?;
        let doc = kw.doc.map(Rc::from);
        let def = FuncDef { name: Some(name), method_of, params, ret, body, generator, doc, pos };
        Ok(node(NodeKind::KaamDef(Rc::new(def)), pos))
    }

    fn dhacha_def(&mut self) -> PResult<Node> {
        let pos = self.eat(Tok::Dhacha)?.pos;
        let (name, _) = self.ident()?;
        let parent = if self.accept(Tok::Colon) { Some(self.ident()?.0) } else { None };
        self.eat(Tok::LBrace)?;
        let mut fields: Vec<FieldDef> = Vec::new();
        while self.ty() != Tok::RBrace {
            let (f, fpos) = self.ident()?;
            if fields.iter().any(|x| x.name == f) {
                return Err(AdarshError::syntax(format!("Field '{f}' do baar likha hai."), fpos));
            }
            let default = if self.accept(Tok::Assign) { Some(self.expression()?) } else { None };
            fields.push(FieldDef { name: f, default });
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        self.eat(Tok::RBrace)?;
        self.accept(Tok::Semi);
        Ok(node(NodeKind::DhachaDef(Rc::new(DhachaDef { name, parent, fields, pos })), pos))
    }

    fn vikalp_def(&mut self) -> PResult<Node> {
        let pos = self.eat(Tok::Vikalp)?.pos;
        let (name, _) = self.ident()?;
        self.eat(Tok::LBrace)?;
        let mut variants: Vec<Rc<str>> = Vec::new();
        while self.ty() != Tok::RBrace {
            let (v, vpos) = self.ident()?;
            if variants.contains(&v) {
                return Err(AdarshError::syntax(format!("Vikalp '{v}' do baar likha hai."), vpos));
            }
            variants.push(v);
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        self.eat(Tok::RBrace)?;
        self.accept(Tok::Semi);
        Ok(node(NodeKind::EnumDef(name, Rc::new(variants)), pos))
    }

    // ---------------------------------------------------------------- statements

    fn semi(&mut self) -> PResult<()> {
        self.eat(Tok::Semi).map(|_| ())
    }

    fn optional_label(&mut self) -> Option<Rc<str>> {
        if self.ty() == Tok::Ident {
            Some(self.bump().text().into())
        } else {
            None
        }
    }

    fn statement(&mut self) -> PResult<Node> {
        let pos = self.pos();
        // `naam: jabtak (...)` labels a loop for `bas naam;`
        if self.ty() == Tok::Ident && self.peek_ty(1) == Tok::Colon && LOOP_TOKENS.contains(&self.peek_ty(2)) {
            let label: Rc<str> = self.bump().text().into();
            self.bump();
            return self.loop_statement(Some(label));
        }
        let kind = match self.ty() {
            Tok::Kaam if self.peek_ty(1) == Tok::Ident => return self.kaam_def(),
            Tok::Dhacha => return self.dhacha_def(),
            Tok::Vikalp => return self.vikalp_def(),
            Tok::Badlo | Tok::Pakka => {
                let n = self.declaration()?;
                self.semi()?;
                return Ok(n);
            }
            Tok::Agar => return self.agar(),
            t if LOOP_TOKENS.contains(&t) => return self.loop_statement(None),
            Tok::Chuno => self.chuno()?,
            Tok::Dikhao => {
                self.bump();
                let e = self.expression()?;
                self.semi()?;
                NodeKind::Dikhao(e)
            }
            Tok::Bas => {
                self.bump();
                let l = self.optional_label();
                self.semi()?;
                NodeKind::Break(l)
            }
            Tok::AageBadho => {
                self.bump();
                let l = self.optional_label();
                self.semi()?;
                NodeKind::Continue(l)
            }
            Tok::Pakdo => self.pakdo()?,
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
                let path = self.expression()?;
                let alias = if self.accept(Tok::As) { Some(self.ident()?.0) } else { None };
                self.semi()?;
                NodeKind::Import { path, alias }
            }
            Tok::Wapas => {
                self.bump();
                let e = if self.ty() == Tok::Semi { None } else { Some(self.expression()?) };
                self.semi()?;
                NodeKind::Wapas(e)
            }
            Tok::DeteRaho => {
                let p = self.bump().pos;
                match self.yields.last_mut() {
                    Some(y) => *y = true,
                    None => {
                        return Err(AdarshError::syntax("'dete_raho' sirf kaam (function) ke andar chalta hai.", p))
                    }
                }
                let e = self.expression()?;
                self.semi()?;
                NodeKind::Yield(e)
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

    /// `badlo` / `pakka` declarations, including `badlo a = 1, b = 2` and destructuring.
    fn declaration(&mut self) -> PResult<Node> {
        let start = self.bump();
        let constant = start.ty == Tok::Pakka;
        let pos = start.pos;
        // destructuring
        if self.ty() == Tok::LBracket || self.ty() == Tok::LBrace {
            let target = if self.accept(Tok::LBracket) {
                let mut names = Vec::new();
                let mut rest = None;
                while self.ty() != Tok::RBracket {
                    if self.accept(Tok::Ellipsis) {
                        rest = Some(self.ident()?.0);
                        break;
                    }
                    let (n, _) = self.ident()?;
                    names.push(if &*n == "_" { None } else { Some(n) });
                    if !self.accept(Tok::Comma) {
                        break;
                    }
                }
                self.eat(Tok::RBracket)?;
                Destructure::List(names, rest)
            } else {
                self.bump();
                let mut names = Vec::new();
                while self.ty() != Tok::RBrace {
                    names.push(self.ident()?.0);
                    if !self.accept(Tok::Comma) {
                        break;
                    }
                }
                self.eat(Tok::RBrace)?;
                Destructure::Fields(names)
            };
            self.eat(Tok::Assign)?;
            let init = self.expression()?;
            return Ok(node(NodeKind::Destructure { target, init, constant }, pos));
        }
        let mut decls = Vec::new();
        loop {
            let (name, npos) = self.ident()?;
            let ty = if self.accept(Tok::Colon) { Some(self.type_ann()?) } else { None };
            let init = if self.accept(Tok::Assign) { Some(self.expression()?) } else { None };
            if constant && init.is_none() {
                return Err(AdarshError::syntax(format!("'pakka {name}' ko shuru mein hi value deni hogi."), npos));
            }
            decls.push(node(NodeKind::Badlo { name, init, ty, constant }, npos));
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        if decls.len() == 1 {
            return Ok(decls.pop().unwrap());
        }
        Ok(node(NodeKind::Seq(decls), pos))
    }

    fn loop_statement(&mut self, label: Option<Rc<str>>) -> PResult<Node> {
        let pos = self.pos();
        let kind = match self.ty() {
            Tok::Jabtak => {
                self.bump();
                self.eat(Tok::LParen)?;
                let cond = self.expression()?;
                self.eat(Tok::RParen)?;
                NodeKind::While { cond, body: self.block()?, label }
            }
            Tok::Karo => {
                self.bump();
                let body = self.block()?;
                self.eat(Tok::Jabtak)?;
                self.eat(Tok::LParen)?;
                let cond = self.expression()?;
                self.eat(Tok::RParen)?;
                self.semi()?;
                NodeKind::DoWhile { body, cond, label }
            }
            Tok::Ginnati => self.ginnati(label)?,
            Tok::KeLiye => self.ke_liye(label)?,
            _ => return Err(self.expected("loop (jabtak / ginnati / ke_liye / karo)")),
        };
        Ok(node(kind, pos))
    }

    fn pakdo(&mut self) -> PResult<NodeKind> {
        let pos = self.eat(Tok::Pakdo)?.pos;
        let body = self.block()?;
        let mut catches = Vec::new();
        while self.ty() == Tok::Chhoddo {
            self.bump();
            self.eat(Tok::LParen)?;
            let (name, _) = self.ident()?;
            let ty = if self.accept(Tok::Colon) { Some(self.type_ann()?) } else { None };
            self.eat(Tok::RParen)?;
            catches.push(Catch { name, ty, body: self.block()? });
        }
        let finally = if self.accept(Tok::Aakhir) { Some(self.block()?) } else { None };
        if catches.is_empty() && finally.is_none() {
            return Err(AdarshError::syntax("'pakdo { }' ke baad 'chhoddo (err) { }' ya 'aakhir { }' chahiye.", pos));
        }
        Ok(NodeKind::TryCatch { body, catches, finally })
    }

    fn assignment(&mut self) -> PResult<Expr> {
        let target = self.expression()?;
        let pos = self.pos();
        if self.ty() == Tok::Assign {
            self.bump();
            let rhs = Box::new(self.expression()?);
            let kind = match target.kind {
                ExprKind::Var(n, _) => ExprKind::Assign(n, rhs, Slot::new()),
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
                ExprKind::Var(n, _) => ExprKind::CompoundAssign(n, op, rhs, Slot::new()),
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

    // ---------------------------------------------------------------- expressions

    pub fn expression(&mut self) -> PResult<Expr> {
        self.coalesce()
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

    fn coalesce(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::QQ, BinOp::Coalesce)], Self::or)
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

    /// `< > <= >=` chain (`0 < x < 10`); `in` and `hai` live at this level too.
    fn comparison(&mut self) -> PResult<Expr> {
        let first = self.range()?;
        let mut operands = vec![first];
        let mut ops: Vec<BinOp> = Vec::new();
        let mut op_pos = Vec::new();
        loop {
            let op = match self.ty() {
                Tok::Lt => BinOp::Lt,
                Tok::Gt => BinOp::Gt,
                Tok::Lte => BinOp::Lte,
                Tok::Gte => BinOp::Gte,
                Tok::In => {
                    let pos = self.bump().pos;
                    let rhs = self.range()?;
                    let lhs = Self::fold(operands, ops, op_pos);
                    operands = vec![expr(ExprKind::Binary(Box::new(lhs), BinOp::In, Box::new(rhs)), pos)];
                    ops = Vec::new();
                    op_pos = Vec::new();
                    continue;
                }
                Tok::Hai => {
                    let pos = self.bump().pos;
                    let ty = self.type_ann()?;
                    let lhs = Self::fold(operands, ops, op_pos);
                    operands = vec![expr(ExprKind::TypeCheck(Box::new(lhs), ty), pos)];
                    ops = Vec::new();
                    op_pos = Vec::new();
                    continue;
                }
                _ => break,
            };
            op_pos.push(self.bump().pos);
            ops.push(op);
            operands.push(self.range()?);
        }
        Ok(Self::fold(operands, ops, op_pos))
    }

    fn fold(mut operands: Vec<Expr>, ops: Vec<BinOp>, op_pos: Vec<Pos>) -> Expr {
        match ops.len() {
            0 => operands.pop().unwrap(),
            1 => {
                let rhs = operands.pop().unwrap();
                let lhs = operands.pop().unwrap();
                expr(ExprKind::Binary(Box::new(lhs), ops[0], Box::new(rhs)), op_pos[0])
            }
            _ => {
                let pos = operands[0].pos;
                expr(ExprKind::Chain(operands, ops), pos)
            }
        }
    }

    fn range(&mut self) -> PResult<Expr> {
        let lo = self.term()?;
        if self.ty() == Tok::DotDot {
            let pos = self.bump().pos;
            let hi = self.term()?;
            return Ok(expr(ExprKind::Range(Box::new(lo), Box::new(hi)), pos));
        }
        Ok(lo)
    }
    fn term(&mut self) -> PResult<Expr> {
        self.binary_level(&[(Tok::Plus, BinOp::Add), (Tok::Minus, BinOp::Sub)], Self::factor)
    }
    fn factor(&mut self) -> PResult<Expr> {
        self.binary_level(
            &[(Tok::Mul, BinOp::Mul), (Tok::Div, BinOp::Div), (Tok::IntDiv, BinOp::IntDiv), (Tok::Mod, BinOp::Mod)],
            Self::unary,
        )
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

    /// Call arguments: plain, `...spread` or `naam: value`.
    fn call_args(&mut self) -> PResult<Vec<Expr>> {
        let mut args = Vec::new();
        while self.ty() != Tok::RParen {
            let pos = self.pos();
            if self.accept(Tok::Ellipsis) {
                let e = self.expression()?;
                args.push(expr(ExprKind::Spread(Box::new(e)), pos));
            } else if self.ty() == Tok::Ident && self.peek_ty(1) == Tok::Colon {
                let (name, _) = self.ident()?;
                self.bump();
                let e = self.expression()?;
                args.push(expr(ExprKind::NamedArg(name, Box::new(e)), pos));
            } else {
                if args.iter().any(|a: &Expr| matches!(a.kind, ExprKind::NamedArg(..))) {
                    return Err(AdarshError::syntax(
                        "Naam waale arguments (naam: value) sabse aakhir mein aate hain.",
                        pos,
                    ));
                }
                args.push(self.expression()?);
            }
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        self.eat(Tok::RParen)?;
        Ok(args)
    }

    fn postfix(&mut self) -> PResult<Expr> {
        let mut e = self.primary()?;
        loop {
            match self.ty() {
                Tok::LParen => {
                    let pos = self.bump().pos;
                    let args = self.call_args()?;
                    e = expr(ExprKind::Call(Box::new(e), args), pos);
                }
                Tok::LBracket => {
                    let pos = self.bump().pos;
                    let opt = |p: &mut Self| -> PResult<Option<Box<Expr>>> {
                        if matches!(p.ty(), Tok::Colon | Tok::RBracket) {
                            Ok(None)
                        } else {
                            Ok(Some(Box::new(p.expression()?)))
                        }
                    };
                    let start = opt(self)?;
                    if self.accept(Tok::Colon) {
                        let end = opt(self)?;
                        let step = if self.accept(Tok::Colon) { opt(self)? } else { None };
                        self.eat(Tok::RBracket)?;
                        e = expr(ExprKind::Slice(Box::new(e), start, end, step), pos);
                    } else {
                        let idx = start.ok_or_else(|| self.expected("index"))?;
                        self.eat(Tok::RBracket)?;
                        e = expr(ExprKind::Index(Box::new(e), idx), pos);
                    }
                }
                Tok::Dot => {
                    let pos = self.bump().pos;
                    let (attr, _) = self.ident()?;
                    e = expr(ExprKind::Attr(Box::new(e), attr), pos);
                }
                Tok::QDot => {
                    let pos = self.bump().pos;
                    let (attr, _) = self.ident()?;
                    e = expr(ExprKind::SafeAttr(Box::new(e), attr), pos);
                }
                // `Naam { field: value }` record construction
                Tok::LBrace
                    if matches!(&e.kind, ExprKind::Var(..))
                        && (self.peek_ty(1) == Tok::RBrace
                            || (self.peek_ty(1) == Tok::Ident && self.peek_ty(2) == Tok::Colon)) =>
                {
                    let ExprKind::Var(type_name, _) = e.kind else { unreachable!() };
                    let pos = e.pos;
                    self.bump();
                    let mut fields = Vec::new();
                    while self.ty() != Tok::RBrace {
                        let (f, _) = self.ident()?;
                        self.eat(Tok::Colon)?;
                        fields.push((f, self.expression()?));
                        if !self.accept(Tok::Comma) {
                            break;
                        }
                    }
                    self.eat(Tok::RBrace)?;
                    e = expr(ExprKind::Dhacha { type_name, fields }, pos);
                }
                _ => return Ok(e),
            }
        }
    }

    /// Is the `(` at the cursor the start of `(params) => body`?
    fn at_lambda(&self) -> bool {
        let mut depth = 0;
        let mut j = self.i;
        while let Some(t) = self.tokens.get(j) {
            match t.ty {
                Tok::LParen => depth += 1,
                Tok::RParen => {
                    depth -= 1;
                    if depth == 0 {
                        return self.tokens.get(j + 1).is_some_and(|t| t.ty == Tok::Arrow);
                    }
                }
                Tok::Eof | Tok::Semi | Tok::LBrace => return false,
                _ => {}
            }
            j += 1;
        }
        false
    }

    fn lambda_body(&mut self, pos: Pos) -> PResult<(Vec<Node>, bool)> {
        if self.ty() == Tok::LBrace {
            return self.function_body();
        }
        self.yields.push(false);
        let e = self.expression();
        self.yields.pop();
        let e = e?;
        Ok((vec![node(NodeKind::Wapas(Some(e)), pos)], false))
    }

    fn lambda(&mut self, params: Vec<Param>, pos: Pos) -> PResult<ExprKind> {
        self.eat(Tok::Arrow)?;
        let (body, generator) = self.lambda_body(pos)?;
        let def = FuncDef { name: None, method_of: None, params, ret: None, body, generator, doc: None, pos };
        Ok(ExprKind::Func(Rc::new(def)))
    }

    fn list_items(&mut self, close: Tok) -> PResult<Vec<Expr>> {
        let mut items = Vec::new();
        while self.ty() != close {
            let pos = self.pos();
            if self.accept(Tok::Ellipsis) {
                let e = self.expression()?;
                items.push(expr(ExprKind::Spread(Box::new(e)), pos));
            } else {
                items.push(self.expression()?);
            }
            if !self.accept(Tok::Comma) {
                break;
            }
        }
        self.eat(close)?;
        Ok(items)
    }

    fn interpolation(&mut self, parts: Vec<RawPart>) -> PResult<ExprKind> {
        let mut out = Vec::new();
        for p in parts {
            out.push(match p {
                RawPart::Lit(s) => Part::Lit(s.into()),
                RawPart::Code(code, cpos) => {
                    let (code, spec) = split_format_spec(&code);
                    let tokens = Lexer::at(code, cpos).tokenize()?;
                    let mut sub = Parser::new(tokens);
                    let e = sub.expression()?;
                    if sub.ty() != Tok::Eof {
                        return Err(sub.expected("'}'"));
                    }
                    Part::Expr(e, spec.map(Rc::from))
                }
            });
        }
        Ok(ExprKind::Interp(out))
    }

    fn primary(&mut self) -> PResult<Expr> {
        let pos = self.pos();
        // `x => x * 2`
        if self.ty() == Tok::Ident && self.peek_ty(1) == Tok::Arrow {
            let (name, _) = self.ident()?;
            let params = vec![Param { name, default: None, is_varargs: false, ty: None }];
            return Ok(expr(self.lambda(params, pos)?, pos));
        }
        if self.ty() == Tok::LParen && self.at_lambda() {
            let params = self.params()?;
            return Ok(expr(self.lambda(params, pos)?, pos));
        }
        let tok = self.bump();
        let kind = match tok.ty {
            Tok::Ident => ExprKind::Var(crate::ast::intern(&tok.text()), Slot::new()),
            Tok::Number => match tok.value {
                TokValue::Int(i) => ExprKind::Int(i),
                TokValue::BigInt(b) => ExprKind::BigInt(Rc::new(b)),
                TokValue::Float(f) => ExprKind::Float(f),
                _ => unreachable!(),
            },
            Tok::Str => match tok.value {
                TokValue::Interp(parts) => self.interpolation(parts)?,
                _ => ExprKind::Str(tok.text().into()),
            },
            Tok::SahiHaiBe => ExprKind::Bool(true),
            Tok::Jhuth => ExprKind::Bool(false),
            Tok::Khali => ExprKind::Null,
            Tok::LBracket => ExprKind::List(self.list_items(Tok::RBracket)?),
            Tok::LBrace => {
                let mut entries = Vec::new();
                while self.ty() != Tok::RBrace {
                    let k = self.expression()?;
                    self.eat(Tok::Colon)?;
                    let v = self.expression()?;
                    entries.push((k, v));
                    if !self.accept(Tok::Comma) {
                        break;
                    }
                }
                self.eat(Tok::RBrace)?;
                ExprKind::Dict(entries)
            }
            Tok::LParen => {
                // `()` is the empty tuple, `(a, b)` a tuple, `(a)` grouping
                if self.accept(Tok::RParen) {
                    ExprKind::Tuple(Vec::new())
                } else {
                    let first = self.expression()?;
                    if self.accept(Tok::Comma) {
                        let mut items = vec![first];
                        items.extend(self.list_items(Tok::RParen)?);
                        ExprKind::Tuple(items)
                    } else {
                        self.eat(Tok::RParen)?;
                        return Ok(first);
                    }
                }
            }
            Tok::Kaam => {
                let params = self.params()?;
                let ret = self.return_type()?;
                let (body, generator) = self.function_body()?;
                ExprKind::Func(Rc::new(FuncDef {
                    name: None,
                    method_of: None,
                    params,
                    ret,
                    body,
                    generator,
                    doc: None,
                    pos,
                }))
            }
            // `agar (c) a warna b` as a value
            Tok::Agar => {
                self.eat(Tok::LParen)?;
                let cond = self.expression()?;
                self.eat(Tok::RParen)?;
                let then = self.expression()?;
                if self.ty() != Tok::Warna {
                    return Err(self.expected("'warna' (value waale agar ko dono hisse chahiye)"));
                }
                self.bump();
                let otherwise = self.expression()?;
                ExprKind::IfExpr(Box::new(cond), Box::new(then), Box::new(otherwise))
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
                return Err(AdarshError::syntax(what, pos).with_code("E102"));
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

    fn ginnati(&mut self, label: Option<Rc<str>>) -> PResult<NodeKind> {
        self.eat(Tok::Ginnati)?;
        self.eat(Tok::LParen)?;
        let mut init = None;
        if self.ty() != Tok::Semi {
            let ipos = self.pos();
            let n = if self.ty() == Tok::Badlo || self.ty() == Tok::Pakka {
                self.declaration()?
            } else {
                node(NodeKind::Expr(self.assignment()?), ipos)
            };
            init = Some(Box::new(n));
        }
        self.semi()?;
        let cond = if self.ty() != Tok::Semi { Some(self.expression()?) } else { None };
        self.semi()?;
        let update = if self.ty() != Tok::RParen { Some(self.assignment()?) } else { None };
        self.eat(Tok::RParen)?;
        Ok(NodeKind::For { init, cond, update, body: self.block()?, label })
    }

    fn ke_liye(&mut self, label: Option<Rc<str>>) -> PResult<NodeKind> {
        self.eat(Tok::KeLiye)?;
        self.eat(Tok::LParen)?;
        let declare = self.accept(Tok::Badlo);
        let (first, _) = self.ident()?;
        let (index, var) = if self.accept(Tok::Comma) {
            let (second, _) = self.ident()?;
            (Some(first), second)
        } else {
            (None, first)
        };
        if !(self.accept(Tok::In) || self.accept(Tok::Colon)) {
            return Err(self.expected("'in'"));
        }
        let iterable = self.expression()?;
        self.eat(Tok::RParen)?;
        Ok(NodeKind::ForEach { index, var, declare, iterable, body: self.block()?, label })
    }

    fn pattern(&mut self, in_list: bool) -> PResult<Pattern> {
        if self.ty() == Tok::Ident && self.cur().text() == "_" {
            self.bump();
            return Ok(Pattern::Wildcard);
        }
        if self.accept(Tok::LBracket) {
            let mut items = Vec::new();
            let mut rest = None;
            while self.ty() != Tok::RBracket {
                if self.accept(Tok::Ellipsis) {
                    rest = Some(self.ident()?.0);
                    break;
                }
                items.push(self.pattern(true)?);
                if !self.accept(Tok::Comma) {
                    break;
                }
            }
            self.eat(Tok::RBracket)?;
            return Ok(Pattern::List(items, rest));
        }
        // inside a list pattern a bare name captures; at the top it captures only before `agar`
        if self.ty() == Tok::Ident
            && matches!(self.peek_ty(1), Tok::Comma | Tok::RBracket | Tok::Colon | Tok::Agar)
            && (in_list || self.peek_ty(1) == Tok::Agar)
        {
            return Ok(Pattern::Bind(crate::ast::intern(&self.bump().text())));
        }
        let e = self.expression()?;
        Ok(match e.kind {
            ExprKind::Range(lo, hi) => Pattern::Range(*lo, *hi),
            _ => Pattern::Value(e),
        })
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
                    let mut patterns = vec![self.pattern(false)?];
                    while self.accept(Tok::Comma) {
                        patterns.push(self.pattern(false)?);
                    }
                    let guard = if self.accept(Tok::Agar) { Some(self.expression()?) } else { None };
                    self.eat(Tok::Colon)?;
                    cases.push(SwitchCase { patterns, guard, body: self.case_body()? });
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
