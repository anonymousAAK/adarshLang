//! Bytecode for simple functions.
//!
//! A `kaam` whose body only uses a common subset of the language (local variables, numbers,
//! strings, operators, `agar`, `jabtak`, `ke_liye` over a range or a collection, `wapas`,
//! `bas`/`aage_badho`, `dikhao`, indexing, lists and calls) is compiled once, on its first
//! call, into a flat list of instructions. Its locals live in an array instead of a chain of
//! scope objects, which removes most of the per-statement cost of the tree-walking
//! interpreter.
//!
//! Everything else (lambdas and nested functions, methods, generators, `pakdo`, `chuno`,
//! records, typed or `pakka` locals, labelled loops, named or spread arguments, dot-calls,
//! ...) makes the compiler give up, and that function keeps running on the tree-walker. So
//! the two engines never have to agree on a feature only one of them implements. Errors carry
//! the same positions and call chains as in the tree-walker, and `ADARSH_NO_VM=1` turns the
//! VM off for comparisons.

use crate::ast::*;
use crate::error::{AdarshError, Pos};
use crate::interp::{get_var_cached, int_fast, Interpreter, ScopeRef};
use crate::value::*;
use std::rc::Rc;

thread_local! {
    static OFF_HERE: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Should new interpreters use the VM? Off with `ADARSH_NO_VM=1`, or on threads that called
/// [`set_enabled_on_this_thread`] with `false` (the test suite runs every program both ways).
pub fn enabled() -> bool {
    !OFF_HERE.with(|o| o.get()) && std::env::var_os("ADARSH_NO_VM").is_none()
}

pub fn set_enabled_on_this_thread(on: bool) {
    OFF_HERE.with(|o| o.set(!on));
}

#[derive(Debug, Clone)]
enum Op {
    Const(u32),
    Load(u32),
    Store(u32),
    LoadGlobal(u32),
    StoreGlobal(u32),
    Pop,
    Dup,
    Bin(BinOp),
    Neg,
    Not,
    /// replace the top with its truthiness as a boolean
    Truthy,
    Jump(u32),
    JumpIfFalse(u32),
    /// `a && b`: when the top is falsy, replace it with jhuth and jump; else pop it
    AndJump(u32),
    /// `a || b`: when the top is truthy, replace it with sahi and jump; else pop it
    OrJump(u32),
    /// `a ?? b`: when the top is not khali, keep it and jump; else pop it
    CoalesceJump(u32),
    Call {
        argc: u32,
        callee: Option<Rc<str>>,
        callee_pos: Pos,
    },
    /// `target.name`: pops the target, pushes the function, the receiver (or khali) and
    /// whether there is a receiver; for `?.` on khali pushes khali and jumps to `skip`
    DotResolve {
        name: Rc<str>,
        safe: bool,
        local: Option<u32>,
        global: u32,
        skip: u32,
        callee_pos: Pos,
    },
    /// call after `DotResolve` and the arguments
    DotCall {
        argc: u32,
        callee_pos: Pos,
    },
    Attr(Rc<str>),
    /// `x?.f`: khali stays khali
    SafeAttr(Rc<str>),
    AttrStore(Rc<str>),
    AttrCompound(Rc<str>, BinOp),
    Index,
    IndexStore,
    IndexCompound(BinOp),
    MakeList(u32),
    NewDict,
    /// pops value and key, inserts them into the dict below
    DictInsert,
    /// turn the top into its display text, optionally with a `{x:spec}` format
    Show(Option<Rc<str>>),
    Concat(u32),
    Print,
    Return,
    /// pops hi and lo; sets the counter, end and step locals
    RangeInit {
        counter: u32,
    },
    /// next number of a range loop into `var`, or jump to `exit` when done
    RangeNext {
        counter: u32,
        var: u32,
        exit: u32,
    },
    /// pops the collection for loop `iter`
    IterInit {
        iter: u32,
    },
    IterNext {
        iter: u32,
        var: u32,
        exit: u32,
    },
}

/// A compiled function body.
pub struct Code {
    ops: Vec<Op>,
    pos: Vec<Pos>,
    consts: Vec<Value>,
    globals: Vec<(Rc<str>, Slot)>,
    nlocals: usize,
    niters: usize,
}

impl std::fmt::Debug for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<bytecode: {} ops>", self.ops.len())
    }
}

// ------------------------------------------------------------------ compiler

struct Loop {
    breaks: Vec<usize>,
    continues: Vec<usize>,
}

struct Compiler {
    code: Code,
    scopes: Vec<Vec<(Rc<str>, u32)>>,
    loops: Vec<Loop>,
}

/// `None` means "not supported here": the function runs on the tree-walker instead.
type C = Option<()>;

/// Compile a function, or `None` when it uses something the VM leaves to the tree-walker.
pub fn compile(def: &FuncDef) -> Option<Code> {
    if def.generator || def.method_of.is_some() || def.params.iter().any(|p| p.is_varargs) {
        return None;
    }
    let mut c = Compiler {
        code: Code { ops: vec![], pos: vec![], consts: vec![], globals: vec![], nlocals: 0, niters: 0 },
        scopes: vec![vec![]],
        loops: vec![],
    };
    for p in &def.params {
        c.declare(&p.name);
    }
    // the body runs in the same scope as the parameters
    for n in &def.body {
        c.stmt(n)?;
    }
    let k = c.constant(Value::Null);
    c.emit(Op::Const(k), def.pos);
    c.emit(Op::Return, def.pos);
    Some(c.code)
}

impl Compiler {
    fn emit(&mut self, op: Op, pos: Pos) -> usize {
        self.code.ops.push(op);
        self.code.pos.push(pos);
        self.code.ops.len() - 1
    }

    fn here(&self) -> u32 {
        self.code.ops.len() as u32
    }

    fn patch(&mut self, at: usize, target: u32) {
        match &mut self.code.ops[at] {
            Op::Jump(t) | Op::JumpIfFalse(t) | Op::AndJump(t) | Op::OrJump(t) | Op::CoalesceJump(t) => *t = target,
            Op::DotResolve { skip, .. } => *skip = target,
            Op::RangeNext { exit, .. } | Op::IterNext { exit, .. } => *exit = target,
            _ => unreachable!("patching a non-jump"),
        }
    }

    fn constant(&mut self, v: Value) -> u32 {
        self.code.consts.push(v);
        (self.code.consts.len() - 1) as u32
    }

    fn new_local(&mut self) -> u32 {
        self.code.nlocals += 1;
        (self.code.nlocals - 1) as u32
    }

    /// Declare in the innermost scope; declaring a name twice there reuses its slot.
    fn declare(&mut self, name: &Rc<str>) -> u32 {
        if let Some((_, s)) = self.scopes.last().unwrap().iter().find(|(n, _)| n == name) {
            return *s;
        }
        let s = self.new_local();
        self.scopes.last_mut().unwrap().push((name.clone(), s));
        s
    }

    fn local(&self, name: &str) -> Option<u32> {
        self.scopes.iter().rev().find_map(|sc| sc.iter().rev().find(|(n, _)| &**n == name).map(|(_, s)| *s))
    }

    fn global(&mut self, name: &Rc<str>) -> u32 {
        self.code.globals.push((name.clone(), Slot::new()));
        (self.code.globals.len() - 1) as u32
    }

    fn block(&mut self, nodes: &[Node]) -> C {
        self.scopes.push(vec![]);
        for n in nodes {
            self.stmt(n)?;
        }
        self.scopes.pop();
        Some(())
    }

    fn loop_body(&mut self, body: &[Node]) -> Option<Loop> {
        self.loops.push(Loop { breaks: vec![], continues: vec![] });
        let ok = self.block(body);
        let l = self.loops.pop().unwrap();
        ok.map(|_| l)
    }

    fn stmt(&mut self, n: &Node) -> C {
        let pos = n.pos;
        match &n.kind {
            NodeKind::Expr(e) => {
                self.expr(e)?;
                self.emit(Op::Pop, pos);
            }
            NodeKind::Badlo { name, init, ty: None, constant: false } => {
                match init {
                    Some(e) => self.expr(e)?,
                    None => {
                        let k = self.constant(Value::Null);
                        self.emit(Op::Const(k), pos);
                    }
                }
                // declared after the initialiser: `badlo x = x + 1` reads the outer x
                let s = self.declare(name);
                self.emit(Op::Store(s), pos);
            }
            NodeKind::Dikhao(e) => {
                self.expr(e)?;
                self.emit(Op::Print, pos);
            }
            NodeKind::Wapas(e) => {
                match e {
                    Some(e) => self.expr(e)?,
                    None => {
                        let k = self.constant(Value::Null);
                        self.emit(Op::Const(k), pos);
                    }
                }
                self.emit(Op::Return, pos);
            }
            NodeKind::Block(b) => self.block(b)?,
            NodeKind::If { cond, then, otherwise } => {
                self.expr(cond)?;
                let jf = self.emit(Op::JumpIfFalse(0), pos);
                self.block(then)?;
                match otherwise {
                    None => {
                        let end = self.here();
                        self.patch(jf, end);
                    }
                    Some(o) => {
                        let j = self.emit(Op::Jump(0), pos);
                        let els = self.here();
                        self.patch(jf, els);
                        match &o.kind {
                            NodeKind::Block(b) => self.block(b)?,
                            _ => self.stmt(o)?,
                        }
                        let end = self.here();
                        self.patch(j, end);
                    }
                }
            }
            NodeKind::While { cond, body, label: None } => {
                let top = self.here();
                self.expr(cond)?;
                let jf = self.emit(Op::JumpIfFalse(0), pos);
                let l = self.loop_body(body)?;
                self.emit(Op::Jump(top), pos);
                let end = self.here();
                self.patch(jf, end);
                self.finish_loop(l, top, end);
            }
            NodeKind::ForEach { index: None, var, declare: true, iterable, body, label: None } => {
                // the loop variable lives in its own scope around the body, like the tree-walker's
                self.scopes.push(vec![]);
                let (top, next) = if let ExprKind::Range(lo, hi) = &iterable.kind {
                    self.expr(lo)?;
                    self.expr(hi)?;
                    // counter, end and step are three consecutive hidden locals
                    let counter = self.new_local();
                    self.new_local();
                    self.new_local();
                    self.emit(Op::RangeInit { counter }, iterable.pos);
                    let v = self.declare(var);
                    let top = self.here();
                    (top, self.emit(Op::RangeNext { counter, var: v, exit: 0 }, pos))
                } else {
                    self.expr(iterable)?;
                    let iter = self.code.niters as u32;
                    self.code.niters += 1;
                    self.emit(Op::IterInit { iter }, iterable.pos);
                    let v = self.declare(var);
                    let top = self.here();
                    (top, self.emit(Op::IterNext { iter, var: v, exit: 0 }, pos))
                };
                let l = self.loop_body(body)?;
                self.emit(Op::Jump(top), pos);
                let end = self.here();
                self.patch(next, end);
                self.finish_loop(l, top, end);
                self.scopes.pop();
            }
            NodeKind::Break(None) => {
                self.loops.last()?;
                let j = self.emit(Op::Jump(0), pos);
                self.loops.last_mut()?.breaks.push(j);
            }
            NodeKind::Continue(None) => {
                self.loops.last()?;
                let j = self.emit(Op::Jump(0), pos);
                self.loops.last_mut()?.continues.push(j);
            }
            _ => return None,
        }
        Some(())
    }

    fn finish_loop(&mut self, l: Loop, cont: u32, end: u32) {
        for b in l.breaks {
            self.patch(b, end);
        }
        for c in l.continues {
            self.patch(c, cont);
        }
    }

    fn expr(&mut self, e: &Expr) -> C {
        let pos = e.pos;
        match &e.kind {
            ExprKind::Int(i) => self.push_const(Value::Int(*i), pos),
            ExprKind::BigInt(b) => self.push_const(Value::Big(b.clone()), pos),
            ExprKind::Float(f) => self.push_const(Value::Float(*f), pos),
            ExprKind::Str(s) => self.push_const(Value::Str(s.clone()), pos),
            ExprKind::Bool(b) => self.push_const(Value::Bool(*b), pos),
            ExprKind::Null => self.push_const(Value::Null, pos),
            ExprKind::Interp(parts) => {
                for p in parts {
                    match p {
                        Part::Lit(l) => self.push_const(Value::Str(l.clone()), pos),
                        Part::Expr(x, spec) => {
                            self.expr(x)?;
                            self.emit(Op::Show(spec.clone()), x.pos);
                        }
                    }
                }
                self.emit(Op::Concat(parts.len() as u32), pos);
            }
            ExprKind::Var(name, _) => match self.local(name) {
                Some(s) => {
                    self.emit(Op::Load(s), pos);
                }
                None => {
                    let g = self.global(name);
                    self.emit(Op::LoadGlobal(g), pos);
                }
            },
            ExprKind::Assign(name, rhs, _) => {
                self.expr(rhs)?;
                self.emit(Op::Dup, pos);
                self.store(name, pos);
            }
            ExprKind::CompoundAssign(name, op, rhs, _) => {
                match self.local(name) {
                    Some(s) => self.emit(Op::Load(s), pos),
                    None => {
                        let g = self.global(name);
                        self.emit(Op::LoadGlobal(g), pos)
                    }
                };
                self.expr(rhs)?;
                self.emit(Op::Bin(*op), pos);
                self.emit(Op::Dup, pos);
                self.store(name, pos);
            }
            ExprKind::Binary(l, op, r) => {
                self.expr(l)?;
                let jump = match op {
                    BinOp::And => Some(Op::AndJump(0)),
                    BinOp::Or => Some(Op::OrJump(0)),
                    BinOp::Coalesce => Some(Op::CoalesceJump(0)),
                    _ => None,
                };
                match jump {
                    Some(j) => {
                        let at = self.emit(j, pos);
                        self.expr(r)?;
                        if !matches!(op, BinOp::Coalesce) {
                            self.emit(Op::Truthy, pos);
                        }
                        let end = self.here();
                        self.patch(at, end);
                    }
                    None => {
                        self.expr(r)?;
                        self.emit(Op::Bin(*op), pos);
                    }
                }
            }
            ExprKind::Unary(op, x) => {
                self.expr(x)?;
                self.emit(
                    match op {
                        UnOp::Neg => Op::Neg,
                        UnOp::Not => Op::Not,
                    },
                    pos,
                );
            }
            ExprKind::IfExpr(c, a, b) => {
                self.expr(c)?;
                let jf = self.emit(Op::JumpIfFalse(0), pos);
                self.expr(a)?;
                let j = self.emit(Op::Jump(0), pos);
                let els = self.here();
                self.patch(jf, els);
                self.expr(b)?;
                let end = self.here();
                self.patch(j, end);
            }
            ExprKind::Call(callee, args) => {
                if let ExprKind::Attr(t, name) | ExprKind::SafeAttr(t, name) = &callee.kind {
                    if args.iter().any(|a| matches!(a.kind, ExprKind::Spread(_) | ExprKind::NamedArg(..))) {
                        return None;
                    }
                    self.expr(t)?;
                    let local = self.local(name);
                    let global = self.global(name);
                    let safe = matches!(callee.kind, ExprKind::SafeAttr(..));
                    let at = self.emit(
                        Op::DotResolve { name: name.clone(), safe, local, global, skip: 0, callee_pos: callee.pos },
                        pos,
                    );
                    for a in args {
                        self.expr(a)?;
                    }
                    self.emit(Op::DotCall { argc: args.len() as u32, callee_pos: callee.pos }, pos);
                    let end = self.here();
                    self.patch(at, end);
                    return Some(());
                }
                self.expr(callee)?;
                for a in args {
                    if matches!(a.kind, ExprKind::Spread(_) | ExprKind::NamedArg(..)) {
                        return None;
                    }
                    self.expr(a)?;
                }
                let name = match &callee.kind {
                    ExprKind::Var(n, _) => Some(n.clone()),
                    _ => None,
                };
                self.emit(Op::Call { argc: args.len() as u32, callee: name, callee_pos: callee.pos }, pos);
            }
            ExprKind::Attr(t, name) => {
                self.expr(t)?;
                self.emit(Op::Attr(name.clone()), pos);
            }
            ExprKind::SafeAttr(t, name) => {
                self.expr(t)?;
                self.emit(Op::SafeAttr(name.clone()), pos);
            }
            ExprKind::AttrAssign(t, name, v) => {
                self.expr(t)?;
                self.expr(v)?;
                self.emit(Op::AttrStore(name.clone()), pos);
            }
            ExprKind::AttrCompoundAssign(t, name, op, v) => {
                self.expr(t)?;
                self.expr(v)?;
                self.emit(Op::AttrCompound(name.clone(), *op), pos);
            }
            ExprKind::Index(c, i) => {
                self.expr(c)?;
                self.expr(i)?;
                self.emit(Op::Index, pos);
            }
            ExprKind::IndexAssign(c, i, v) => {
                self.expr(c)?;
                self.expr(i)?;
                self.expr(v)?;
                self.emit(Op::IndexStore, pos);
            }
            ExprKind::IndexCompoundAssign(c, i, op, v) => {
                self.expr(c)?;
                self.expr(i)?;
                self.expr(v)?;
                self.emit(Op::IndexCompound(*op), pos);
            }
            ExprKind::Dict(entries) => {
                self.emit(Op::NewDict, pos);
                for (k, v) in entries {
                    self.expr(k)?;
                    self.expr(v)?;
                    self.emit(Op::DictInsert, k.pos);
                }
            }
            ExprKind::List(items) => {
                for x in items {
                    if matches!(x.kind, ExprKind::Spread(_)) {
                        return None;
                    }
                    self.expr(x)?;
                }
                self.emit(Op::MakeList(items.len() as u32), pos);
            }
            _ => return None,
        }
        Some(())
    }

    fn push_const(&mut self, v: Value, pos: Pos) {
        let k = self.constant(v);
        self.emit(Op::Const(k), pos);
    }

    fn store(&mut self, name: &Rc<str>, pos: Pos) {
        match self.local(name) {
            Some(s) => self.emit(Op::Store(s), pos),
            None => {
                let g = self.global(name);
                self.emit(Op::StoreGlobal(g), pos)
            }
        };
    }
}

// ------------------------------------------------------------------ machine

enum Iter {
    /// lists are read live, like the tree-walker (items added during the loop are visited)
    Live(Value, usize),
    Snapshot(Vec<Value>, usize),
}

impl Interpreter {
    /// Run compiled code for one call. `args` are the parameter values, already checked.
    pub(crate) fn run_code(&mut self, code: &Code, scope: &ScopeRef, args: Vec<Value>) -> Result<Value, AdarshError> {
        let base = self.vstack.len();
        self.vstack.extend(args);
        self.run_frame(code, scope, base)
    }

    /// Run a frame whose parameters are already on the value stack from `base` on.
    fn run_frame(&mut self, code: &Code, scope: &ScopeRef, base: usize) -> Result<Value, AdarshError> {
        self.vstack.resize(base + code.nlocals, Value::Null);
        let r = self.exec_frame(code, scope, base);
        self.vstack.truncate(base);
        r
    }

    fn exec_frame(&mut self, code: &Code, scope: &ScopeRef, base: usize) -> Result<Value, AdarshError> {
        let mut iters: Vec<Option<Iter>> = Vec::new();
        iters.resize_with(code.niters, || None);
        let mut pc = 0usize;
        macro_rules! pop {
            () => {
                self.vstack.pop().expect("vm stack")
            };
        }
        macro_rules! at {
            ($e:expr) => {
                $e.map_err(|err: AdarshError| err.at(code.pos[pc - 1]))?
            };
        }
        loop {
            let op = &code.ops[pc];
            pc += 1;
            match op {
                Op::Const(k) => self.vstack.push(code.consts[*k as usize].clone()),
                Op::Load(s) => self.vstack.push(self.vstack[base + *s as usize].clone()),
                Op::Store(s) => self.vstack[base + *s as usize] = pop!(),
                Op::LoadGlobal(g) => {
                    let (name, slot) = &code.globals[*g as usize];
                    self.vstack.push(at!(get_var_cached(scope, name, slot)));
                }
                Op::StoreGlobal(g) => {
                    let (name, slot) = &code.globals[*g as usize];
                    let v = pop!();
                    at!(self.set_var_cached(scope, name, slot, v));
                }
                Op::Pop => {
                    self.vstack.pop();
                }
                Op::Dup => {
                    let v = self.vstack.last().expect("vm stack").clone();
                    self.vstack.push(v);
                }
                Op::Bin(op) => {
                    let b = pop!();
                    let a = pop!();
                    let fast = match (&a, &b) {
                        (Value::Int(x), Value::Int(y)) => int_fast(*x, *op, *y),
                        _ => None,
                    };
                    let v = match fast {
                        Some(v) => v,
                        None => at!(self.binary_values(&a, *op, &b)),
                    };
                    self.vstack.push(v);
                }
                Op::Neg => {
                    let v = pop!();
                    self.vstack.push(at!(negate(&v)));
                }
                Op::Not => {
                    let v = pop!();
                    self.vstack.push(Value::Bool(!v.truthy()));
                }
                Op::Truthy => {
                    let v = pop!();
                    self.vstack.push(Value::Bool(v.truthy()));
                }
                Op::Jump(t) => pc = *t as usize,
                Op::JumpIfFalse(t) => {
                    if !pop!().truthy() {
                        pc = *t as usize;
                    }
                }
                Op::AndJump(t) => {
                    if !self.vstack.last().expect("vm stack").truthy() {
                        *self.vstack.last_mut().unwrap() = Value::Bool(false);
                        pc = *t as usize;
                    } else {
                        self.vstack.pop();
                    }
                }
                Op::OrJump(t) => {
                    if self.vstack.last().expect("vm stack").truthy() {
                        *self.vstack.last_mut().unwrap() = Value::Bool(true);
                        pc = *t as usize;
                    } else {
                        self.vstack.pop();
                    }
                }
                Op::CoalesceJump(t) => {
                    if !matches!(self.vstack.last(), Some(Value::Null)) {
                        pc = *t as usize;
                    } else {
                        self.vstack.pop();
                    }
                }
                Op::Call { argc, callee, callee_pos } => {
                    let fpos = self.vstack.len() - *argc as usize - 1;
                    if let Some(c) = self.direct_closure(&self.vstack[fpos], *argc as usize) {
                        // bytecode calling bytecode: the arguments on the stack become the callee's locals
                        if self.depth >= self.max_depth {
                            return Err(self.too_deep(&c.def).at(code.pos[pc - 1]));
                        }
                        let callee_code = compiled(&c.def).expect("checked by direct_closure");
                        self.depth += 1;
                        let r = self.run_frame(callee_code, &c.scope, fpos + 1);
                        self.depth -= 1;
                        self.vstack.pop();
                        match r {
                            Ok(v) => self.vstack.push(v),
                            Err(mut e) => {
                                if e.catchable() && e.pos.is_some() && e.trace.len() < 64 {
                                    e.trace.push((c.def.display_name(), *callee_pos));
                                }
                                return Err(e.at(code.pos[pc - 1]));
                            }
                        }
                        continue;
                    }
                    let args = self.vstack.split_off(self.vstack.len() - *argc as usize);
                    let f = pop!();
                    let v = at!(self.vm_call(f, args, callee, *callee_pos));
                    self.vstack.push(v);
                }
                Op::DotResolve { name, safe, local, global, skip, callee_pos } => {
                    let target = pop!();
                    let (gname, gslot) = &code.globals[*global as usize];
                    let local_value = local.map(|s| self.vstack[base + s as usize].clone());
                    let mut lookup = || match &local_value {
                        Some(v) => Some(v.clone()),
                        None => get_var_cached(scope, gname, gslot).ok(),
                    };
                    match at!(self.dot_callee(target, name, *safe, *callee_pos, &mut lookup)) {
                        None => {
                            self.vstack.push(Value::Null);
                            pc = *skip as usize;
                        }
                        Some((f, receiver)) => {
                            self.vstack.push(f);
                            self.vstack.push(Value::Bool(receiver.is_some()));
                            self.vstack.push(receiver.unwrap_or(Value::Null));
                        }
                    }
                }
                Op::DotCall { argc, callee_pos } => {
                    let args = self.vstack.split_off(self.vstack.len() - *argc as usize);
                    let receiver = pop!();
                    let has_receiver = matches!(pop!(), Value::Bool(true));
                    let f = pop!();
                    let mut all = Vec::with_capacity(args.len() + 1);
                    if has_receiver {
                        all.push(receiver);
                    }
                    all.extend(args);
                    let v = at!(self.vm_call(f, all, &None, *callee_pos));
                    self.vstack.push(v);
                }
                Op::Attr(name) => {
                    let t = pop!();
                    let v = at!(self.attr(&t, name));
                    self.vstack.push(v);
                }
                Op::SafeAttr(name) => {
                    let t = pop!();
                    let v = if matches!(t, Value::Null) { Value::Null } else { at!(self.attr(&t, name)) };
                    self.vstack.push(v);
                }
                Op::AttrStore(name) => {
                    let v = pop!();
                    let t = pop!();
                    let Value::Dict(d) = &t else {
                        let err = AdarshError::runtime(format!("{} par '.{name}' set nahi kar sakte.", t.type_desc()));
                        return Err(err.at(code.pos[pc - 1]));
                    };
                    d.borrow_mut().insert_str(name, v.clone());
                    self.vstack.push(v);
                }
                Op::AttrCompound(name, op) => {
                    let r = pop!();
                    let t = pop!();
                    let cur = at!(self.attr(&t, name));
                    let nv = at!(self.binary_values(&cur, *op, &r));
                    let Value::Dict(d) = &t else { unreachable!("attr() succeeded on a dict") };
                    d.borrow_mut().insert_str(name, nv.clone());
                    self.vstack.push(nv);
                }
                Op::Index => {
                    let i = pop!();
                    let c = pop!();
                    let v = at!(self.index(&c, &i));
                    self.vstack.push(v);
                }
                Op::IndexStore => {
                    let v = pop!();
                    let i = pop!();
                    let c = pop!();
                    at!(self.store_index(&c, i, v.clone()));
                    self.vstack.push(v);
                }
                Op::IndexCompound(op) => {
                    let r = pop!();
                    let i = pop!();
                    let c = pop!();
                    let cur = at!(self.index(&c, &i));
                    let nv = at!(self.binary_values(&cur, *op, &r));
                    at!(self.store_index(&c, i, nv.clone()));
                    self.vstack.push(nv);
                }
                Op::MakeList(n) => {
                    let items = self.vstack.split_off(self.vstack.len() - *n as usize);
                    self.vstack.push(Value::list(items));
                }
                Op::NewDict => self.vstack.push(Value::dict(Dict::new())),
                Op::DictInsert => {
                    let v = pop!();
                    let k = pop!();
                    let Some(Value::Dict(d)) = self.vstack.last() else { unreachable!("dict below its entries") };
                    let d = d.clone();
                    at!(d.borrow_mut().insert(k, v));
                }
                Op::Show(spec) => {
                    let v = pop!();
                    let text = self.show(&v);
                    let text = match spec {
                        None => text,
                        Some(spec) => at!(crate::builtins::format_spec(&v, &text, spec)),
                    };
                    self.vstack.push(Value::Str(text.into()));
                }
                Op::Concat(n) => {
                    let parts = self.vstack.split_off(self.vstack.len() - *n as usize);
                    let mut s = String::new();
                    for p in &parts {
                        if let Value::Str(t) = p {
                            s.push_str(t);
                        }
                    }
                    self.vstack.push(Value::Str(s.into()));
                }
                Op::Print => {
                    let v = pop!();
                    let mut s = self.show(&v);
                    s.push('\n');
                    self.write_str(&s);
                }
                Op::Return => return Ok(pop!()),
                Op::RangeInit { counter } => {
                    let hi = pop!();
                    let lo = pop!();
                    let (a, b) = at!(range_bounds(&lo, &hi));
                    let c = *counter as usize;
                    self.vstack[base + c] = Value::Int(a);
                    self.vstack[base + c + 1] = Value::Int(b);
                    self.vstack[base + c + 2] = Value::Int(if a <= b { 1 } else { -1 });
                }
                Op::RangeNext { counter, var, exit } => {
                    let c = *counter as usize;
                    let (Value::Int(cur), Value::Int(end), Value::Int(step)) =
                        (&self.vstack[base + c], &self.vstack[base + c + 1], &self.vstack[base + c + 2])
                    else {
                        // the counter is khali once it has passed the end
                        pc = *exit as usize;
                        continue;
                    };
                    let (cur, end, step) = (*cur, *end, *step);
                    if (step > 0 && cur > end) || (step < 0 && cur < end) {
                        pc = *exit as usize;
                        continue;
                    }
                    self.vstack[base + *var as usize] = Value::Int(cur);
                    self.vstack[base + c] = match cur.checked_add(step) {
                        Some(n) => Value::Int(n),
                        None => Value::Null,
                    };
                }
                Op::IterInit { iter } => {
                    let it = pop!();
                    if !crate::builtins::is_iterable(&it) {
                        let hint = if matches!(it, Value::Int(_)) { " Ginti ke liye 1..n likho." } else { "" };
                        let err = AdarshError::runtime(format!(
                            "'ke_liye' sirf list, string, dict, tuple ya set par chalta hai, {} par nahi.{hint}",
                            it.type_desc()
                        ));
                        return Err(err.at(code.pos[pc - 1]));
                    }
                    iters[*iter as usize] = Some(match &it {
                        Value::List(_) => Iter::Live(it, 0),
                        other => Iter::Snapshot(crate::builtins::iter_items(other).unwrap_or_default(), 0),
                    });
                }
                Op::IterNext { iter, var, exit } => {
                    let item = match iters[*iter as usize].as_mut().expect("iter set up") {
                        Iter::Live(v, i) => {
                            let item = self.nth_item(v, *i);
                            *i += 1;
                            item
                        }
                        Iter::Snapshot(items, i) => {
                            let item = items.get(*i).cloned();
                            *i += 1;
                            item
                        }
                    };
                    match item {
                        Some(v) => self.vstack[base + *var as usize] = v,
                        None => pc = *exit as usize,
                    }
                }
            }
        }
    }

    /// The closure to call directly from bytecode: a compiled plain function given exactly its
    /// parameters, with no types to check. Anything else goes through `call_full`.
    fn direct_closure(&self, f: &Value, argc: usize) -> Option<Rc<Closure>> {
        let Value::Func(c) = f else { return None };
        let d = &c.def;
        if d.params.len() != argc
            || d.ret.is_some()
            || d.params.iter().any(|p| p.ty.is_some() || p.is_varargs)
            || !self.use_vm
            || self.on_step.is_some()
        {
            return None;
        }
        compiled(d)?;
        Some(c.clone())
    }

    /// A call from compiled code, with the same checks and call-chain entry as `eval_call`.
    fn vm_call(
        &mut self,
        f: Value,
        args: Vec<Value>,
        callee: &Option<Rc<str>>,
        callee_pos: Pos,
    ) -> Result<Value, AdarshError> {
        if !f.is_callable() {
            let what = match callee {
                Some(n) => format!("'{n}' ({})", f.type_desc()),
                None => f.type_desc(),
            };
            return Err(AdarshError::runtime(format!("{what} function nahi hai, isliye call nahi kar sakte."))
                .with_code("E009"));
        }
        match self.call_full(&f, args, Vec::new()) {
            Err(mut e) if e.catchable() && e.pos.is_some() && e.trace.len() < 64 => {
                let name = match &f {
                    Value::Func(c) => Some(c.def.display_name()),
                    Value::Method(m) => Some(m.func.def.display_name()),
                    _ => None,
                };
                if let Some(name) = name {
                    e.trace.push((name, callee_pos));
                }
                Err(e)
            }
            other => other,
        }
    }
}

/// The function's bytecode, compiled on first use; `None` when it runs on the tree-walker.
pub(crate) fn compiled(def: &FuncDef) -> Option<&Code> {
    def.compiled
        .get_or_init(|| {
            let code = compile(def).map(Box::new);
            if std::env::var_os("ADARSH_VM_LOG").is_some() {
                let how = if code.is_some() { "bytecode" } else { "tree-walker" };
                eprintln!("vm: '{}' -> {how}", def.display_name());
            }
            code
        })
        .as_deref()
}

/// The ends of `lo..hi`, with the same rules and messages as the tree-walker's ranges.
fn range_bounds(lo: &Value, hi: &Value) -> Result<(i64, i64), AdarshError> {
    let as_int = |v: &Value| match v {
        Value::Int(i) => Ok(*i),
        Value::Bool(b) => Ok(*b as i64),
        Value::Float(f) if f.fract() == 0.0 && f.abs() < 9e18 => Ok(*f as i64),
        other => Err(AdarshError::runtime(format!("'..' range ko poore number chahiye, {} mila.", display(other)))),
    };
    let (a, b) = (as_int(lo)?, as_int(hi)?);
    if (b as i128 - a as i128).unsigned_abs() + 1 > 100_000_000 {
        return Err(AdarshError::runtime("Range bahut badi hai (10 crore se zyada numbers)."));
    }
    Ok((a, b))
}
