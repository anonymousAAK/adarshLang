//! Tree-walking interpreter.

use crate::ast::*;
use crate::builtins::{is_iterable, normalize_index, out_of_range, resolve, BUILTINS};
use crate::error::{AdarshError, ErrorKind};
use crate::loader::{module_path, ModuleLoader};
use crate::semantics::{root_scope, Analyzer, Signature};
use crate::value::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::io::Write;
use std::rc::Rc;

// ------------------------------------------------------------ fast hasher for scopes

/// FxHash: much faster than SipHash for short identifier keys.
#[derive(Default)]
pub struct FxHasher(u64);

impl Hasher for FxHasher {
    fn write(&mut self, bytes: &[u8]) {
        for &b in bytes {
            self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x51_7c_c1_b7_27_22_0a_95);
        }
    }
    fn finish(&self) -> u64 {
        self.0
    }
}

type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

// ------------------------------------------------------------ scopes

#[derive(Default)]
pub struct Scope {
    vars: FxMap<Rc<str>, Value>,
    types: FxMap<Rc<str>, Rc<Vec<Rc<str>>>>,
    parent: Option<ScopeRef>,
}

pub type ScopeRef = Rc<RefCell<Scope>>;

pub fn new_scope(parent: Option<&ScopeRef>) -> ScopeRef {
    Rc::new(RefCell::new(Scope { parent: parent.cloned(), ..Default::default() }))
}

fn undeclared(name: &str) -> AdarshError {
    AdarshError::runtime(crate::semantics::undeclared_message(name))
}

fn define(scope: &ScopeRef, name: &Rc<str>, value: Value) {
    scope.borrow_mut().vars.insert(name.clone(), value);
}

fn get_var(scope: &ScopeRef, name: &str) -> Result<Value, AdarshError> {
    if let Some(v) = scope.borrow().vars.get(name) {
        return Ok(v.clone());
    }
    let mut cur = scope.borrow().parent.clone();
    while let Some(s) = cur {
        if let Some(v) = s.borrow().vars.get(name) {
            return Ok(v.clone());
        }
        cur = s.borrow().parent.clone();
    }
    Err(undeclared(name))
}

fn set_var(scope: &ScopeRef, name: &str, value: Value) -> Result<(), AdarshError> {
    let mut cur = scope.clone();
    loop {
        let next = {
            let mut s = cur.borrow_mut();
            if let Some(slot) = s.vars.get_mut(name) {
                *slot = value;
                return Ok(());
            }
            s.parent.clone()
        };
        match next {
            Some(p) => cur = p,
            None => return Err(undeclared(name)),
        }
    }
}

fn get_type(scope: &ScopeRef, name: &str) -> Option<Rc<Vec<Rc<str>>>> {
    let mut cur = Some(scope.clone());
    while let Some(s) = cur {
        let b = s.borrow();
        if let Some(t) = b.types.get(name) {
            return Some(t.clone());
        }
        cur = b.parent.clone();
    }
    None
}

// ------------------------------------------------------------ control flow

/// Result of executing a statement: a value, or a control-flow signal.
pub enum Flow {
    Normal(Value),
    Return(Value),
    Break,
    Continue,
}

type Exec = Result<Flow, AdarshError>;
type Eval = Result<Value, AdarshError>;

fn rt(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

// ------------------------------------------------------------ interpreter

pub struct Interpreter {
    pub global: ScopeRef,
    loaded_modules: HashSet<String>,
    loader: Rc<dyn ModuleLoader>,
    out: Box<dyn Write>,
    /// Source of lines for `padho()`; `None` means end of input.
    pub input: Box<dyn FnMut() -> Option<String>>,
    /// Methods declared with `kaam Type.name(...)`, by type then method name.
    methods: FxMap<Rc<str>, FxMap<Rc<str>, Rc<Closure>>>,
    depth: usize,
    /// Maximum nesting of function calls before a recursion error.
    pub max_depth: usize,
    rng: u64,
    pub clock: fn() -> f64,
}

fn system_clock() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        0.0
    }
}

impl Interpreter {
    pub fn new(loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Self {
        let global = new_scope(None);
        {
            let mut g = global.borrow_mut();
            for (id, spec) in BUILTINS.iter().enumerate() {
                g.vars.insert(Rc::from(spec.name), Value::Builtin(id));
            }
        }
        let seed = (system_clock() * 1e6) as u64 ^ 0x9E37_79B9_7F4A_7C15;
        Interpreter {
            global,
            loaded_modules: HashSet::new(),
            loader,
            out,
            input: Box::new(|| None),
            methods: FxMap::default(),
            depth: 0,
            // browsers give WebAssembly a much smaller native stack than the CLI thread
            max_depth: if cfg!(target_arch = "wasm32") { 300 } else { 10_000 },
            rng: seed | 1,
            clock: system_clock,
        }
    }

    pub fn set_seed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }

    /// xorshift64* → uniform float in [0, 1)
    pub fn next_random(&mut self) -> f64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        (x.wrapping_mul(0x2545_F491_4F6C_DD1D) >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn flush(&mut self) {
        let _ = self.out.flush();
    }

    pub fn write_str(&mut self, s: &str) {
        let _ = self.out.write_all(s.as_bytes());
    }

    /// Run a program in the global scope.
    pub fn run(&mut self, program: &[Node]) -> Exec {
        let g = self.global.clone();
        let r = self.exec_block(program, &g);
        self.flush();
        r
    }

    pub fn exec_block(&mut self, nodes: &[Node], scope: &ScopeRef) -> Exec {
        let mut result = Flow::Normal(Value::Null);
        for n in nodes {
            result = self.exec(n, scope)?;
            if !matches!(result, Flow::Normal(_)) {
                return Ok(result);
            }
        }
        Ok(result)
    }

    fn exec(&mut self, node: &Node, scope: &ScopeRef) -> Exec {
        self.exec_kind(node, scope).map_err(|e| e.at(node.pos))
    }

    /// Run a loop body; `None` means keep looping.
    fn loop_body(&mut self, body: &[Node], scope: &ScopeRef) -> Result<Option<Flow>, AdarshError> {
        Ok(match self.exec_block(body, &new_scope(Some(scope)))? {
            Flow::Return(v) => Some(Flow::Return(v)),
            Flow::Break => Some(Flow::Normal(Value::Null)),
            Flow::Continue | Flow::Normal(_) => None,
        })
    }

    fn exec_kind(&mut self, node: &Node, scope: &ScopeRef) -> Exec {
        match &node.kind {
            NodeKind::Expr(e) => Ok(Flow::Normal(self.eval(e, scope)?)),
            NodeKind::Badlo(name, init) => {
                let v = match init {
                    Some(e) => self.eval(e, scope)?,
                    None => Value::Null,
                };
                define(scope, name, v);
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Dikhao(e) => {
                let v = self.eval(e, scope)?;
                let mut s = display(&v);
                s.push('\n');
                self.write_str(&s);
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Break => Ok(Flow::Break),
            NodeKind::Continue => Ok(Flow::Continue),
            NodeKind::Wapas(e) => {
                let v = match e {
                    Some(e) => self.eval(e, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(v))
            }
            NodeKind::Block(b) => self.exec_block(b, &new_scope(Some(scope))),
            NodeKind::If { cond, then, otherwise } => {
                if self.eval(cond, scope)?.truthy() {
                    self.exec_block(then, &new_scope(Some(scope)))
                } else if let Some(o) = otherwise {
                    self.exec(o, scope)
                } else {
                    Ok(Flow::Normal(Value::Null))
                }
            }
            NodeKind::While { cond, body } => {
                while self.eval(cond, scope)?.truthy() {
                    if let Some(f) = self.loop_body(body, scope)? {
                        return Ok(f);
                    }
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::For { init, cond, update, body } => {
                let ls = new_scope(Some(scope));
                if let Some(i) = init {
                    self.exec(i, &ls)?;
                }
                loop {
                    if let Some(c) = cond {
                        if !self.eval(c, &ls)?.truthy() {
                            break;
                        }
                    }
                    if let Some(f) = self.loop_body(body, &ls)? {
                        return Ok(f);
                    }
                    if let Some(u) = update {
                        self.eval(u, &ls)?;
                    }
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::ForEach { var, declare, iterable, body } => {
                self.exec_foreach(var, declare, iterable, body, scope)
            }
            NodeKind::Switch { subject, cases, default } => self.exec_switch(subject, cases, default, scope),
            NodeKind::TryCatch { body, err_name, handler } => {
                let depth = self.depth;
                match self.exec_block(body, &new_scope(Some(scope))) {
                    Err(e) if e.kind != ErrorKind::Syntax && e.kind != ErrorKind::Semantic => {
                        self.depth = depth;
                        let cs = new_scope(Some(scope));
                        define(&cs, err_name, e.catch_value());
                        self.exec_block(handler, &cs)
                    }
                    other => other,
                }
            }
            NodeKind::Throw(e) => {
                let v = self.eval(e, scope)?;
                Err(AdarshError::thrown(v).at(e.pos))
            }
            NodeKind::Import(e) => self.exec_import(e, scope),
            NodeKind::KaamDef(def) => {
                let closure = Rc::new(Closure { def: def.clone(), scope: scope.clone() });
                let name = def.name.clone().expect("named function");
                match &def.method_of {
                    Some(t) => {
                        self.methods.entry(t.clone()).or_default().insert(name, closure);
                    }
                    None => define(scope, &name, Value::Func(closure)),
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::DhachaDef(name, fields) => {
                scope.borrow_mut().types.insert(name.clone(), fields.clone());
                Ok(Flow::Normal(Value::Null))
            }
        }
    }

    pub fn eval(&mut self, e: &Expr, scope: &ScopeRef) -> Eval {
        self.eval_kind(e, scope).map_err(|err| err.at(e.pos))
    }

    fn eval_kind(&mut self, e: &Expr, scope: &ScopeRef) -> Eval {
        match &e.kind {
            ExprKind::Int(i) => Ok(Value::Int(*i)),
            ExprKind::BigInt(b) => Ok(Value::Big(b.clone())),
            ExprKind::Float(f) => Ok(Value::Float(*f)),
            ExprKind::Str(s) => Ok(Value::Str(s.clone())),
            ExprKind::Interp(parts) => self.eval_interp(parts, scope),
            ExprKind::Bool(b) => Ok(Value::Bool(*b)),
            ExprKind::Null => Ok(Value::Null),
            ExprKind::Var(name) => get_var(scope, name),
            ExprKind::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for i in items {
                    v.push(self.eval(i, scope)?);
                }
                Ok(Value::list(v))
            }
            ExprKind::Dict(entries) => self.eval_dict(entries, scope),
            ExprKind::Dhacha { type_name, fields } => self.eval_record(type_name, fields, scope),
            ExprKind::Binary(l, op, r) => {
                let a = self.eval(l, scope)?;
                match op {
                    // short-circuit: the right side runs only when needed
                    BinOp::And => {
                        if !a.truthy() {
                            return Ok(Value::Bool(false));
                        }
                        return Ok(Value::Bool(self.eval(r, scope)?.truthy()));
                    }
                    BinOp::Or => {
                        if a.truthy() {
                            return Ok(Value::Bool(true));
                        }
                        return Ok(Value::Bool(self.eval(r, scope)?.truthy()));
                    }
                    _ => {}
                }
                let b = self.eval(r, scope)?;
                match op {
                    BinOp::Eq => Ok(Value::Bool(values_eq(&a, &b))),
                    BinOp::Neq => Ok(Value::Bool(!values_eq(&a, &b))),
                    BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Ok(Value::Bool(compare(*op, &a, &b)?)),
                    _ => arith(*op, &a, &b),
                }
            }
            ExprKind::Unary(op, x) => {
                let v = self.eval(x, scope)?;
                match op {
                    UnOp::Neg => negate(&v),
                    UnOp::Not => Ok(Value::Bool(!v.truthy())),
                }
            }
            ExprKind::Assign(name, rhs) => {
                let v = self.eval(rhs, scope)?;
                set_var(scope, name, v.clone())?;
                Ok(v)
            }
            ExprKind::CompoundAssign(name, op, rhs) => {
                let cur = get_var(scope, name)?;
                let r = self.eval(rhs, scope)?;
                let v = arith(*op, &cur, &r)?;
                set_var(scope, name, v.clone())?;
                Ok(v)
            }
            ExprKind::Index(c, i) => {
                let coll = self.eval(c, scope)?;
                let idx = self.eval(i, scope)?;
                self.index(&coll, &idx)
            }
            ExprKind::IndexAssign(c, i, v) => self.eval_index_assign(c, i, v, scope),
            ExprKind::IndexCompoundAssign(c, i, op, v) => self.eval_index_compound(c, i, op, v, scope),
            ExprKind::Attr(t, attr) => {
                let target = self.eval(t, scope)?;
                self.attr(&target, attr)
            }
            ExprKind::AttrAssign(t, attr, v) => self.eval_attr_assign(t, attr, v, scope),
            ExprKind::AttrCompoundAssign(t, attr, op, v) => self.eval_attr_compound(t, attr, op, v, scope),
            ExprKind::Func(def) => Ok(Value::Func(Rc::new(Closure { def: def.clone(), scope: scope.clone() }))),
            ExprKind::Call(callee, args) => {
                let f = self.eval(callee, scope)?;
                if !f.is_callable() {
                    let what = match &callee.kind {
                        ExprKind::Var(n) => format!("'{n}' ({})", f.type_desc()),
                        _ => f.type_desc(),
                    };
                    return Err(rt(format!("{what} function nahi hai, isliye call nahi kar sakte.")));
                }
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, scope)?);
                }
                self.call(&f, vals)
            }
        }
    }

    #[inline(never)]
    fn exec_foreach(
        &mut self,
        var: &Rc<str>,
        declare: &bool,
        iterable: &Expr,
        body: &[Node],
        scope: &ScopeRef,
    ) -> Exec {
        let it = self.eval(iterable, scope)?;
        if !is_iterable(&it) {
            return Err(rt(format!(
                "'ke_liye' sirf list, string ya dict par chalta hai, {} par nahi.",
                it.type_desc()
            ))
            .at(iterable.pos));
        }
        let ls = new_scope(Some(scope));
        if *declare {
            define(&ls, var, Value::Null);
        }
        // lists are read live; strings and dicts iterate a snapshot
        let snapshot = match &it {
            Value::List(_) => None,
            other => crate::builtins::iter_items(other),
        };
        let mut i = 0usize;
        loop {
            let item = match &snapshot {
                Some(items) => items.get(i).cloned(),
                None => self.nth_item(&it, i),
            };
            let Some(item) = item else { break };
            i += 1;
            set_var(if *declare { &ls } else { scope }, var, item)?;
            if let Some(f) = self.loop_body(body, &ls)? {
                return Ok(f);
            }
        }
        Ok(Flow::Normal(Value::Null))
    }

    #[inline(never)]
    fn exec_switch(
        &mut self,
        subject: &Expr,
        cases: &[SwitchCase],
        default: &Option<Vec<Node>>,
        scope: &ScopeRef,
    ) -> Exec {
        let sv = self.eval(subject, scope)?;
        for c in cases {
            for m in &c.matches {
                if values_eq(&self.eval(m, scope)?, &sv) {
                    return self.exec_block(&c.body, &new_scope(Some(scope)));
                }
            }
        }
        match default {
            Some(d) => self.exec_block(d, &new_scope(Some(scope))),
            None => Ok(Flow::Normal(Value::Null)),
        }
    }

    #[inline(never)]
    fn exec_import(&mut self, e: &Expr, scope: &ScopeRef) -> Exec {
        let mv = self.eval(e, scope)?;
        let Value::Str(name) = mv else {
            return Err(rt("'lao' ko file ka naam (string) chahiye.").at(e.pos));
        };
        let path = module_path(&name);
        if self.loaded_modules.contains(&path) {
            return Ok(Flow::Normal(Value::Null));
        }
        let Some(src) = self.loader.load(&path) else {
            return Err(rt(format!("Module '{path}' nahi mila.")).at(e.pos));
        };
        self.loaded_modules.insert(path.clone());
        let file = crate::error::register_source(&path, &src);
        let module = crate::parser::parse(&src, file)?;
        let mut sema = Analyzer::new(self.loader.clone());
        sema.analyze_program(&module, &root_scope())?;
        let g = self.global.clone();
        self.exec_block(&module, &g)?;
        Ok(Flow::Normal(Value::Null))
    }

    #[inline(never)]
    fn eval_interp(&mut self, parts: &[Part], scope: &ScopeRef) -> Eval {
        let mut s = String::new();
        for p in parts {
            match p {
                Part::Lit(l) => s.push_str(l),
                Part::Expr(x) => s.push_str(&display(&self.eval(x, scope)?)),
            }
        }
        Ok(Value::Str(s.into()))
    }

    #[inline(never)]
    fn eval_dict(&mut self, entries: &[(Expr, Expr)], scope: &ScopeRef) -> Eval {
        let mut d = Dict::new();
        for (k, v) in entries {
            let key = self.eval(k, scope)?;
            let val = self.eval(v, scope)?;
            d.insert(key, val).map_err(|e| e.at(k.pos))?;
        }
        Ok(Value::dict(d))
    }

    #[inline(never)]
    fn eval_record(&mut self, type_name: &Rc<str>, fields: &[(Rc<str>, Expr)], scope: &ScopeRef) -> Eval {
        let Some(declared) = get_type(scope, type_name) else {
            return Err(rt(format!("'{type_name}' naam ka dhacha nahi mila.")));
        };
        let mut d = Dict::record(type_name.clone());
        for f in declared.iter() {
            d.insert_str(f, Value::Null);
        }
        for (name, fe) in fields {
            let v = self.eval(fe, scope)?;
            d.insert_str(name, v);
        }
        Ok(Value::dict(d))
    }

    #[inline(never)]
    fn eval_index_assign(&mut self, c: &Expr, i: &Expr, v: &Expr, scope: &ScopeRef) -> Eval {
        let coll = self.eval(c, scope)?;
        let idx = self.eval(i, scope)?;
        let val = self.eval(v, scope)?;
        self.store_index(&coll, idx, val.clone())?;
        Ok(val)
    }

    #[inline(never)]
    fn eval_index_compound(&mut self, c: &Expr, i: &Expr, op: &BinOp, v: &Expr, scope: &ScopeRef) -> Eval {
        let coll = self.eval(c, scope)?;
        let idx = self.eval(i, scope)?;
        let right = self.eval(v, scope)?;
        let cur = self.index(&coll, &idx)?;
        let nv = arith(*op, &cur, &right)?;
        self.store_index(&coll, idx, nv.clone())?;
        Ok(nv)
    }

    #[inline(never)]
    fn eval_attr_assign(&mut self, t: &Expr, attr: &Rc<str>, v: &Expr, scope: &ScopeRef) -> Eval {
        let target = self.eval(t, scope)?;
        let val = self.eval(v, scope)?;
        let Value::Dict(d) = &target else {
            return Err(rt(format!("{} par '.{attr}' set nahi kar sakte.", target.type_desc())));
        };
        d.borrow_mut().insert_str(attr, val.clone());
        Ok(val)
    }

    #[inline(never)]
    fn eval_attr_compound(&mut self, t: &Expr, attr: &Rc<str>, op: &BinOp, v: &Expr, scope: &ScopeRef) -> Eval {
        let target = self.eval(t, scope)?;
        let right = self.eval(v, scope)?;
        let cur = self.attr(&target, attr)?;
        let nv = arith(*op, &cur, &right)?;
        let Value::Dict(d) = &target else { unreachable!("attr() succeeded on a dict") };
        d.borrow_mut().insert_str(attr, nv.clone());
        Ok(nv)
    }

    fn attr(&mut self, target: &Value, attr: &str) -> Eval {
        let Value::Dict(d) = target else {
            return Err(rt(format!(
                "{} par '.{attr}' nahi chalta. Sirf dict aur dhacha ke fields hote hain.",
                target.type_desc()
            )));
        };
        let d_ref = d.borrow();
        if let Some(v) = d_ref.get_str(attr) {
            return Ok(v);
        }
        if let Some(t) = &d_ref.type_name {
            if let Some(m) = self.methods.get(t).and_then(|ms| ms.get(attr)) {
                return Ok(Value::Method(Rc::new(BoundMethod { receiver: target.clone(), func: m.clone() })));
            }
            return Err(rt(format!("Dhacha {t} mein '{attr}' naam ka field ya method nahi hai.")));
        }
        Err(rt(format!("Dict mein '{attr}' key nahi hai.")))
    }

    fn index(&mut self, coll: &Value, idx: &Value) -> Eval {
        match coll {
            Value::Dict(d) => match d.borrow().get(idx)? {
                Some(v) => Ok(v),
                None => Err(rt(format!("Key {} dict mein nahi hai.", repr(idx)))),
            },
            Value::List(l) => {
                let n = normalize_index(idx)?;
                let list = l.borrow();
                resolve(n, list.len()).map(|k| list[k].clone()).ok_or_else(|| out_of_range(idx, list.len(), "List"))
            }
            Value::Str(s) => {
                let n = normalize_index(idx)?;
                let len = s.chars().count();
                resolve(n, len)
                    .map(|k| Value::Str(s.chars().nth(k).unwrap().to_string().into()))
                    .ok_or_else(|| out_of_range(idx, len, "String"))
            }
            other => Err(rt(format!("{} mein [index] nahi lagta.", other.type_desc()))),
        }
    }

    fn store_index(&mut self, coll: &Value, idx: Value, val: Value) -> Result<(), AdarshError> {
        match coll {
            Value::Dict(d) => d.borrow_mut().insert(idx, val),
            Value::List(l) => {
                let n = normalize_index(&idx)?;
                let mut list = l.borrow_mut();
                let len = list.len();
                let k = resolve(n, len).ok_or_else(|| out_of_range(&idx, len, "List"))?;
                list[k] = val;
                Ok(())
            }
            Value::Str(_) => {
                Err(rt("String ke andar ka character badal nahi sakte. Nayi string banao (jaise replace() se)."))
            }
            other => Err(rt(format!("{} mein [index] se value nahi rakh sakte.", other.type_desc()))),
        }
    }

    /// Call any callable value with evaluated arguments.
    pub fn call(&mut self, f: &Value, args: Vec<Value>) -> Eval {
        match f {
            Value::Builtin(id) => self.call_builtin(*id, args),
            Value::Func(c) => self.call_closure(c.clone(), None, args),
            Value::Method(m) => self.call_closure(m.func.clone(), Some(m.receiver.clone()), args),
            other => Err(rt(format!("{} function nahi hai, isliye call nahi kar sakte.", other.type_desc()))),
        }
    }

    fn call_closure(&mut self, c: Rc<Closure>, receiver: Option<Value>, args: Vec<Value>) -> Eval {
        let def = &c.def;
        let sig = Signature::from_params(&def.params);
        if !sig.allows(args.len()) {
            return Err(rt(sig.arity_error(&def.display_name(), args.len())));
        }
        if self.depth >= self.max_depth {
            return Err(rt(format!(
                "Recursion bahut gehri ho gayi ({} calls). Shayad '{}' khud ko bina ruke call kar raha hai?",
                self.max_depth,
                def.display_name()
            )));
        }
        let call_scope = new_scope(Some(&c.scope));
        if let Some(r) = receiver {
            define(&call_scope, &Rc::from("yeh"), r);
        }
        let mut args = args.into_iter();
        for p in &def.params {
            let v = if p.is_varargs {
                Value::list(args.by_ref().collect())
            } else {
                match args.next() {
                    Some(a) => a,
                    None => self.eval(p.default.as_ref().expect("arity checked"), &c.scope)?,
                }
            };
            define(&call_scope, &p.name, v);
        }
        self.depth += 1;
        let r = self.exec_block(&def.body, &call_scope);
        self.depth -= 1;
        match r? {
            Flow::Return(v) => Ok(v),
            // a function without `wapas` returns khali
            _ => Ok(Value::Null),
        }
    }
}
