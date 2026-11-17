//! AST definitions (port of `adarsh_lang/ast.py`).

use num_bigint::BigInt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Pow,
    Eq,
    Neq,
    Lt,
    Gt,
    Lte,
    Gte,
    And,
    Or,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::Eq => "==",
            BinOp::Neq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Lte => "<=",
            BinOp::Gte => ">=",
            BinOp::And => "&&",
            BinOp::Or => "||",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

#[derive(Debug)]
pub struct Param {
    pub name: Rc<str>,
    pub default: Option<Expr>,
    pub is_varargs: bool,
}

/// Shared function definition: params + body, referenced by every closure value.
#[derive(Debug)]
pub struct FuncDef {
    pub name: Option<Rc<str>>,
    pub params: Vec<Param>,
    pub body: Vec<Node>,
}

#[derive(Debug)]
pub enum Expr {
    Int(i64),
    BigInt(Rc<BigInt>),
    Float(f64),
    Str(Rc<str>),
    Bool(bool),
    Null,
    Var(Rc<str>),
    List(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Dhacha { type_name: Rc<str>, fields: Vec<(Rc<str>, Expr)> },
    Binary(Box<Expr>, BinOp, Box<Expr>),
    Unary(UnOp, Box<Expr>),
    Call(Box<Expr>, Vec<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Attr(Box<Expr>, Rc<str>),
    Func(Rc<FuncDef>),
    // assignments are expressions in the Python AST (they return the value)
    Assign(Rc<str>, Box<Expr>),
    CompoundAssign(Rc<str>, BinOp, Box<Expr>),
    IndexAssign(Box<Expr>, Box<Expr>, Box<Expr>),
    IndexCompoundAssign(Box<Expr>, Box<Expr>, BinOp, Box<Expr>),
    AttrAssign(Box<Expr>, Rc<str>, Box<Expr>),
    AttrCompoundAssign(Box<Expr>, Rc<str>, BinOp, Box<Expr>),
}

#[derive(Debug)]
pub struct SwitchCase {
    pub matches: Vec<Expr>,
    pub body: Vec<Node>,
}

#[derive(Debug)]
pub enum Node {
    Expr(Expr),
    Badlo(Rc<str>, Option<Expr>),
    Dikhao(Expr),
    Break,
    Continue,
    Wapas(Option<Expr>),
    Block(Vec<Node>),
    If { cond: Expr, then: Vec<Node>, otherwise: Option<Box<Node>> },
    While { cond: Expr, body: Vec<Node> },
    For { init: Option<Box<Node>>, cond: Option<Expr>, update: Option<Expr>, body: Vec<Node> },
    ForEach { var: Rc<str>, declare: bool, iterable: Expr, body: Vec<Node> },
    Switch { subject: Expr, cases: Vec<SwitchCase>, default: Option<Vec<Node>> },
    TryCatch { body: Vec<Node>, err_name: Rc<str>, handler: Vec<Node> },
    Throw(Expr),
    Import(Expr),
    KaamDef(Rc<FuncDef>),
    DhachaDef(Rc<str>, Rc<Vec<Rc<str>>>),
}
