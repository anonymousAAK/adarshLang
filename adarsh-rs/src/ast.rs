//! Abstract syntax tree. Every expression and statement carries its source [`Pos`].

pub use crate::error::Pos;
use num_bigint::BigInt;
use std::rc::Rc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    IntDiv,
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
    /// `a ?? b`: `b` only when `a` is khali
    Coalesce,
    /// `x in coll`
    In,
}

impl BinOp {
    pub fn symbol(self) -> &'static str {
        match self {
            BinOp::Add => "+",
            BinOp::Sub => "-",
            BinOp::Mul => "*",
            BinOp::Div => "/",
            BinOp::IntDiv => "//",
            BinOp::Mod => "%",
            BinOp::Pow => "**",
            BinOp::Eq => "==",
            BinOp::Neq => "!=",
            BinOp::Lt => "<",
            BinOp::Gt => ">",
            BinOp::Lte => "<=",
            BinOp::Gte => ">=",
            BinOp::And => "aur",
            BinOp::Or => "ya",
            BinOp::Coalesce => "??",
            BinOp::In => "in",
        }
    }

    /// Method a record can define to overload this operator.
    pub fn method_name(self) -> Option<&'static str> {
        Some(match self {
            BinOp::Add => "jodo",
            BinOp::Sub => "ghatao",
            BinOp::Mul => "guna",
            BinOp::Div => "bhaag",
            BinOp::Eq => "barabar",
            BinOp::Lt => "chhota",
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnOp {
    Neg,
    Not,
}

/// A type annotation such as `number`, `string?` or `Vyakti`.
#[derive(Debug, Clone)]
pub struct TypeAnn {
    pub name: Rc<str>,
    pub nullable: bool,
    pub pos: Pos,
}

#[derive(Debug)]
pub struct Param {
    pub name: Rc<str>,
    pub default: Option<Expr>,
    pub is_varargs: bool,
    pub ty: Option<TypeAnn>,
}

/// A function definition, shared by every closure created from it.
#[derive(Debug)]
pub struct FuncDef {
    pub name: Option<Rc<str>>,
    /// `Some(type)` for a method declared as `kaam Type.name(...)`.
    pub method_of: Option<Rc<str>>,
    pub params: Vec<Param>,
    pub ret: Option<TypeAnn>,
    pub body: Vec<Node>,
    /// Contains `dete_raho`: calling it returns the list of yielded values.
    pub generator: bool,
    /// `## doc comment` lines written just above the definition.
    pub doc: Option<Rc<str>>,
    pub pos: Pos,
}

impl FuncDef {
    pub fn display_name(&self) -> String {
        match (&self.method_of, &self.name) {
            (Some(t), Some(n)) => format!("{t}.{n}"),
            (None, Some(n)) => n.to_string(),
            _ => "<anonymous>".into(),
        }
    }
}

#[derive(Debug)]
pub enum Part {
    Lit(Rc<str>),
    /// `{expr}` or `{expr:spec}` inside a string
    Expr(Expr, Option<Rc<str>>),
}

#[derive(Debug)]
pub struct Expr {
    pub kind: ExprKind,
    pub pos: Pos,
}

#[derive(Debug)]
pub enum ExprKind {
    Int(i64),
    BigInt(Rc<BigInt>),
    Float(f64),
    Str(Rc<str>),
    Interp(Vec<Part>),
    Bool(bool),
    Null,
    Var(Rc<str>),
    List(Vec<Expr>),
    Tuple(Vec<Expr>),
    Dict(Vec<(Expr, Expr)>),
    Dhacha {
        type_name: Rc<str>,
        fields: Vec<(Rc<str>, Expr)>,
    },
    Binary(Box<Expr>, BinOp, Box<Expr>),
    /// `a < b <= c`: each middle operand is evaluated once
    Chain(Vec<Expr>, Vec<BinOp>),
    Unary(UnOp, Box<Expr>),
    /// `a..b`, inclusive on both ends
    Range(Box<Expr>, Box<Expr>),
    /// `agar (c) a warna b` used as a value
    IfExpr(Box<Expr>, Box<Expr>, Box<Expr>),
    /// `x hai number`
    TypeCheck(Box<Expr>, TypeAnn),
    Call(Box<Expr>, Vec<Expr>),
    /// `...xs` inside a list literal or call
    Spread(Box<Expr>),
    /// `naam: value` inside a call
    NamedArg(Rc<str>, Box<Expr>),
    Index(Box<Expr>, Box<Expr>),
    Slice(Box<Expr>, Option<Box<Expr>>, Option<Box<Expr>>, Option<Box<Expr>>),
    Attr(Box<Expr>, Rc<str>),
    /// `obj?.field`: khali when obj is khali
    SafeAttr(Box<Expr>, Rc<str>),
    Func(Rc<FuncDef>),
    Assign(Rc<str>, Box<Expr>),
    CompoundAssign(Rc<str>, BinOp, Box<Expr>),
    IndexAssign(Box<Expr>, Box<Expr>, Box<Expr>),
    IndexCompoundAssign(Box<Expr>, Box<Expr>, BinOp, Box<Expr>),
    AttrAssign(Box<Expr>, Rc<str>, Box<Expr>),
    AttrCompoundAssign(Box<Expr>, Rc<str>, BinOp, Box<Expr>),
}

/// A `chuno` case pattern.
#[derive(Debug)]
pub enum Pattern {
    /// compare with `==` (any expression, including variables)
    Value(Expr),
    /// `lo..hi`
    Range(Expr, Expr),
    /// a name that captures the value (only with a guard, or inside a list pattern)
    Bind(Rc<str>),
    /// `_`
    Wildcard,
    /// `[a, 2, _]`; `rest` captures remaining items after `...`
    List(Vec<Pattern>, Option<Rc<str>>),
    /// `Vyakti` (type name, possibly a parent type) or `number` / `string` / ...
    Type(TypeAnn),
}

#[derive(Debug)]
pub struct SwitchCase {
    pub patterns: Vec<Pattern>,
    pub guard: Option<Expr>,
    pub body: Vec<Node>,
}

#[derive(Debug)]
pub struct Catch {
    pub name: Rc<str>,
    /// `chhoddo (e: Type)`: only errors of this type
    pub ty: Option<TypeAnn>,
    pub body: Vec<Node>,
}

/// Target of `badlo [a, b] = ...` / `badlo { naam, umar } = ...`.
#[derive(Debug)]
pub enum Destructure {
    /// names (`None` for `_`) and an optional `...rest`
    List(Vec<Option<Rc<str>>>, Option<Rc<str>>),
    Fields(Vec<Rc<str>>),
}

#[derive(Debug)]
pub struct FieldDef {
    pub name: Rc<str>,
    pub default: Option<Expr>,
}

#[derive(Debug)]
pub struct DhachaDef {
    pub name: Rc<str>,
    pub parent: Option<Rc<str>>,
    pub fields: Vec<FieldDef>,
    pub pos: Pos,
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,
    pub pos: Pos,
}

#[derive(Debug)]
pub enum NodeKind {
    Expr(Expr),
    Badlo {
        name: Rc<str>,
        init: Option<Expr>,
        ty: Option<TypeAnn>,
        constant: bool,
    },
    Destructure {
        target: Destructure,
        init: Expr,
        constant: bool,
    },
    /// several declarations in one statement (`badlo a = 1, b = 2;`), same scope
    Seq(Vec<Node>),
    Dikhao(Expr),
    Break(Option<Rc<str>>),
    Continue(Option<Rc<str>>),
    Wapas(Option<Expr>),
    Yield(Expr),
    Block(Vec<Node>),
    If {
        cond: Expr,
        then: Vec<Node>,
        otherwise: Option<Box<Node>>,
    },
    While {
        cond: Expr,
        body: Vec<Node>,
        label: Option<Rc<str>>,
    },
    DoWhile {
        body: Vec<Node>,
        cond: Expr,
        label: Option<Rc<str>>,
    },
    For {
        init: Option<Box<Node>>,
        cond: Option<Expr>,
        update: Option<Expr>,
        body: Vec<Node>,
        label: Option<Rc<str>>,
    },
    ForEach {
        index: Option<Rc<str>>,
        var: Rc<str>,
        declare: bool,
        iterable: Expr,
        body: Vec<Node>,
        label: Option<Rc<str>>,
    },
    Switch {
        subject: Expr,
        cases: Vec<SwitchCase>,
        default: Option<Vec<Node>>,
    },
    TryCatch {
        body: Vec<Node>,
        catches: Vec<Catch>,
        finally: Option<Vec<Node>>,
    },
    Throw(Expr),
    Import {
        path: Expr,
        alias: Option<Rc<str>>,
    },
    KaamDef(Rc<FuncDef>),
    DhachaDef(Rc<DhachaDef>),
    /// `vikalp Rang { Laal, Hara };`
    EnumDef(Rc<str>, Rc<Vec<Rc<str>>>),
}
