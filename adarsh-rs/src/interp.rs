//! Tree-walking interpreter (port of `AdarshInterpreter` in `adarsh_lang/runtime.py`).

use crate::ast::*;
use crate::builtins::{is_iterable, normalize_index, resolve, BUILTINS};
use crate::error::{AdarshError, HostKind};
use crate::loader::{module_path, ModuleLoader};
use crate::semantics::{root_scope, Analyzer, Signature};
use crate::value::*;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasherDefault, Hasher};
use std::io::Write;
use std::rc::Rc;

// ------------------------------------------------------------ fast hasher for scopes

/// FxHash: much faster than SipHash for the short identifier keys we use.
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
    AdarshError::semantic(format!("Variable '{name}' not declared in AdarshLang scope."))
}

/// `declare_variable`: creates the slot (as khali) only if it doesn't exist yet.
fn declare(scope: &ScopeRef, name: &Rc<str>) {
    scope.borrow_mut().vars.entry(name.clone()).or_insert(Value::Null);
}

fn get_var(scope: &ScopeRef, name: &str) -> Result<Value, AdarshError> {
    let mut cur = scope.clone();
    loop {
        let next = {
            let s = cur.borrow();
            if let Some(v) = s.vars.get(name) {
                return Ok(v.clone());
            }
            s.parent.clone()
        };
        match next {
            Some(p) => cur = p,
            None => return Err(undeclared(name)),
        }
    }
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

fn rt(msg: &str) -> AdarshError {
    AdarshError::runtime(msg)
}

fn is_type_like(e: &AdarshError) -> bool {
    matches!(e, AdarshError::Host(HostKind::TypeError | HostKind::KeyError | HostKind::IndexError, _))
}

// ------------------------------------------------------------ interpreter

pub struct Interpreter {
    pub global: ScopeRef,
    loaded_modules: HashSet<String>,
    loader: Rc<dyn ModuleLoader>,
    out: Box<dyn Write>,
    depth: usize,
    /// Maximum nesting of AdarshLang function calls before a RecursionError.
    pub max_depth: usize,
    rng: u64,
    pub clock: fn() -> f64,
}

fn system_clock() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0)
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
            depth: 0,
            max_depth: 10_000,
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

    /// Run a whole program in the global scope (Python `interpret`).
    pub fn run(&mut self, program: &[Node]) -> Exec {
        let g = self.global.clone();
        let r = self.exec_block(program, &g);
        self.flush();
        r
    }

    fn print_line(&mut self, s: &str) {
        let _ = self.out.write_all(s.as_bytes());
        let _ = self.out.write_all(b"\n");
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
        match node {
            Node::Expr(e) => Ok(Flow::Normal(self.eval(e, scope)?)),
            Node::Badlo(name, init) => {
                declare(scope, name);
                let mut v = Value::Null;
                if let Some(e) = init {
                    v = self.eval(e, scope)?;
                    set_var(scope, name, v.clone())?;
                }
                Ok(Flow::Normal(v))
            }
            Node::Dikhao(e) => {
                let v = self.eval(e, scope)?;
                let s = display(&v);
                self.print_line(&s);
                Ok(Flow::Normal(Value::Null))
            }
            Node::Break => Ok(Flow::Break),
            Node::Continue => Ok(Flow::Continue),
            Node::Wapas(e) => {
                let v = match e {
                    Some(e) => self.eval(e, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(v))
            }
            Node::Block(b) => self.exec_block(b, scope),
            Node::If { cond, then, otherwise } => {
                if self.eval(cond, scope)?.truthy() {
                    self.exec_block(then, &new_scope(Some(scope)))
                } else if let Some(o) = otherwise {
                    self.exec(o, &new_scope(Some(scope)))
                } else {
                    Ok(Flow::Normal(Value::Null))
                }
            }
            Node::While { cond, body } => {
                let mut result = Value::Null;
                while self.eval(cond, scope)?.truthy() {
                    match self.exec_block(body, &new_scope(Some(scope)))? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Normal(Value::Null)),
                        // Python leaked a ContinueSignal out of the loop here when the last
                        // iteration ended with `aage_badho`, silently skipping later statements.
                        Flow::Continue => result = Value::Null,
                        Flow::Normal(v) => result = v,
                    }
                }
                Ok(Flow::Normal(result))
            }
            Node::For { init, cond, update, body } => {
                let ls = new_scope(Some(scope));
                if let Some(i) = init {
                    if let Flow::Return(v) = self.exec(i, &ls)? {
                        return Ok(Flow::Return(v));
                    }
                }
                loop {
                    if let Some(c) = cond {
                        if !self.eval(c, &ls)?.truthy() {
                            break;
                        }
                    }
                    match self.exec_block(body, &new_scope(Some(&ls)))? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Normal(Value::Null)),
                        _ => {}
                    }
                    if let Some(u) = update {
                        self.eval(u, &ls)?;
                    }
                }
                Ok(Flow::Normal(Value::Null))
            }
            Node::ForEach { var, declare: decl, iterable, body } => {
                let it = self.eval(iterable, scope)?;
                if !is_iterable(&it) {
                    return Err(rt("'ke_liye' expects an iterable expression in AdarshLang."));
                }
                let ls = new_scope(Some(scope));
                if *decl {
                    declare(&ls, var);
                }
                // strings and dicts iterate a snapshot, lists are read live like Python
                let snapshot = match &it {
                    Value::List(_) => None,
                    other => crate::builtins::iter_items(other),
                };
                let mut result = Value::Null;
                let mut i = 0usize;
                loop {
                    let item = match &snapshot {
                        Some(items) => items.get(i).cloned(),
                        None => self.nth_item(&it, i),
                    };
                    let Some(item) = item else { break };
                    i += 1;
                    if *decl {
                        set_var(&ls, var, item)?;
                    } else {
                        set_var(scope, var, item)?;
                    }
                    match self.exec_block(body, &new_scope(Some(&ls)))? {
                        Flow::Return(v) => return Ok(Flow::Return(v)),
                        Flow::Break => return Ok(Flow::Normal(Value::Null)),
                        Flow::Continue => continue,
                        Flow::Normal(v) => result = v,
                    }
                }
                Ok(Flow::Normal(result))
            }
            Node::Switch { subject, cases, default } => {
                let sv = self.eval(subject, scope)?;
                for c in cases {
                    for m in &c.matches {
                        let mv = self.eval(m, scope)?;
                        if py_eq(&mv, &sv) {
                            return self.exec_block(&c.body, &new_scope(Some(scope)));
                        }
                    }
                }
                match default {
                    Some(d) => self.exec_block(d, &new_scope(Some(scope))),
                    None => Ok(Flow::Normal(Value::Null)),
                }
            }
            Node::TryCatch { body, err_name, handler } => match self.exec_block(body, &new_scope(Some(scope))) {
                Err(AdarshError::User(v)) => {
                    let cs = new_scope(Some(scope));
                    declare(&cs, err_name);
                    set_var(&cs, err_name, v)?;
                    self.exec_block(handler, &cs)
                }
                other => other,
            },
            Node::Throw(e) => {
                let v = self.eval(e, scope)?;
                Err(AdarshError::User(v))
            }
            Node::Import(e) => {
                let mv = self.eval(e, scope)?;
                let Value::Str(name) = mv else {
                    return Err(rt("'lao' expects a string path in AdarshLang."));
                };
                let path = module_path(&name);
                if self.loaded_modules.contains(&path) {
                    return Ok(Flow::Normal(Value::Null));
                }
                let Some(src) = self.loader.load(&path) else {
                    return Err(AdarshError::runtime(format!("Module '{path}' not found for 'lao' in AdarshLang.")));
                };
                self.loaded_modules.insert(path);
                let module = crate::parser::parse(&src)?;
                let mut sema = Analyzer::new(self.loader.clone());
                sema.analyze_program(&module, &root_scope()).map_err(|e| match e {
                    AdarshError::Semantic(m) => AdarshError::Runtime(m),
                    other => other,
                })?;
                let g = self.global.clone();
                self.exec_block(&module, &g)?;
                Ok(Flow::Normal(Value::Null))
            }
            Node::KaamDef(def) => {
                let f = Value::Func(Rc::new(Closure { def: def.clone(), scope: scope.clone() }));
                let name = def.name.clone().expect("named function");
                declare(scope, &name);
                set_var(scope, &name, f.clone())?;
                Ok(Flow::Normal(f))
            }
            Node::DhachaDef(name, fields) => {
                scope.borrow_mut().types.insert(name.clone(), fields.clone());
                Ok(Flow::Normal(Value::Null))
            }
        }
    }

    pub fn eval(&mut self, e: &Expr, scope: &ScopeRef) -> Eval {
        match e {
            Expr::Int(i) => Ok(Value::Int(*i)),
            Expr::BigInt(b) => Ok(Value::Big(b.clone())),
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Str(s) => Ok(Value::Str(s.clone())),
            Expr::Bool(b) => Ok(Value::Bool(*b)),
            Expr::Null => Ok(Value::Null),
            Expr::Var(name) => get_var(scope, name),
            Expr::List(items) => {
                let mut v = Vec::with_capacity(items.len());
                for i in items {
                    v.push(self.eval(i, scope)?);
                }
                Ok(Value::list(v))
            }
            Expr::Dict(entries) => {
                let mut d = Dict::new();
                for (k, v) in entries {
                    let k = self.eval(k, scope)?;
                    let v = self.eval(v, scope)?;
                    d.insert(k, v)?;
                }
                Ok(Value::dict(d))
            }
            Expr::Dhacha { type_name, fields } => {
                let Some(declared) = get_type(scope, type_name) else {
                    return Err(AdarshError::runtime(format!(
                        "Dhacha '{type_name}' not declared before use in AdarshLang."
                    )));
                };
                let mut d = Dict::new();
                for f in declared.iter() {
                    d.insert_str(f, Value::Null);
                }
                for (name, fe) in fields {
                    let v = self.eval(fe, scope)?;
                    d.insert_str(name, v);
                }
                d.insert_str("__type__", Value::Str(type_name.clone()));
                Ok(Value::dict(d))
            }
            Expr::Binary(l, op, r) => {
                let a = self.eval(l, scope)?;
                let b = self.eval(r, scope)?;
                match op {
                    BinOp::Eq => Ok(Value::Bool(py_eq(&a, &b))),
                    BinOp::Neq => Ok(Value::Bool(!py_eq(&a, &b))),
                    BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Ok(Value::Bool(compare(*op, &a, &b)?)),
                    // Python evaluated both operands (no short-circuit) and returned a bool.
                    BinOp::And => Ok(Value::Bool(a.truthy() && b.truthy())),
                    BinOp::Or => Ok(Value::Bool(a.truthy() || b.truthy())),
                    _ => arith(*op, &a, &b),
                }
            }
            Expr::Unary(op, x) => {
                let v = self.eval(x, scope)?;
                match op {
                    UnOp::Neg => negate(&v),
                    UnOp::Not => Ok(Value::Bool(!v.truthy())),
                }
            }
            Expr::Assign(name, rhs) => {
                let v = self.eval(rhs, scope)?;
                set_var(scope, name, v.clone())?;
                Ok(v)
            }
            Expr::CompoundAssign(name, op, rhs) => {
                let cur = get_var(scope, name)?;
                let r = self.eval(rhs, scope)?;
                let v = arith(*op, &cur, &r)?;
                set_var(scope, name, v.clone())?;
                Ok(v)
            }
            Expr::Index(c, i) => {
                let coll = self.eval(c, scope)?;
                let idx = self.eval(i, scope)?;
                self.index(&coll, &idx)
            }
            Expr::IndexAssign(c, i, v) => {
                let coll = self.eval(c, scope)?;
                let idx = self.eval(i, scope)?;
                let val = self.eval(v, scope)?;
                let mutable_err = || rt("Only mutable collections like lists or dicts support indexed assignment in AdarshLang.");
                match &coll {
                    Value::Dict(d) => d.borrow_mut().insert(idx, val.clone()).map_err(|_| mutable_err())?,
                    _ => {
                        let n = normalize_index(&idx)?;
                        let Value::List(l) = &coll else { return Err(mutable_err()) };
                        let mut list = l.borrow_mut();
                        let Some(k) = resolve(n, list.len()) else {
                            return Err(rt("Index out of range while assigning into collection in AdarshLang."));
                        };
                        list[k] = val.clone();
                    }
                }
                Ok(val)
            }
            Expr::IndexCompoundAssign(c, i, op, v) => {
                let coll = self.eval(c, scope)?;
                let idx = self.eval(i, scope)?;
                let right = self.eval(v, scope)?;
                let fail = || rt("Compound assignment failed on collection in AdarshLang.");
                let wrap = |e: AdarshError| if is_type_like(&e) { fail() } else { e };
                match &coll {
                    Value::Dict(d) => {
                        let cur = d.borrow().get(&idx).map_err(wrap)?.ok_or_else(fail)?;
                        let nv = arith(*op, &cur, &right).map_err(wrap)?;
                        d.borrow_mut().insert(idx, nv.clone()).map_err(wrap)?;
                        Ok(nv)
                    }
                    _ => {
                        let n = normalize_index(&idx)?;
                        match &coll {
                            Value::List(l) => {
                                let len = l.borrow().len();
                                let k = resolve(n, len).ok_or_else(fail)?;
                                let cur = l.borrow()[k].clone();
                                let nv = arith(*op, &cur, &right).map_err(wrap)?;
                                let mut list = l.borrow_mut();
                                if k >= list.len() {
                                    return Err(fail());
                                }
                                list[k] = nv.clone();
                                Ok(nv)
                            }
                            Value::Str(s) => {
                                // reading works, item assignment is a TypeError
                                if resolve(n, s.chars().count()).is_none() {
                                    return Err(fail());
                                }
                                Err(fail())
                            }
                            _ => Err(fail()),
                        }
                    }
                }
            }
            Expr::Attr(t, attr) => {
                let target = self.eval(t, scope)?;
                let Value::Dict(d) = &target else {
                    return Err(rt("Attribute access only supported on dhacha objects or dictionaries in AdarshLang."));
                };
                let found = d.borrow().get_str(attr);
                found.ok_or_else(|| AdarshError::runtime(format!("Attribute '{attr}' missing on object in AdarshLang.")))
            }
            Expr::AttrAssign(t, attr, v) => {
                let target = self.eval(t, scope)?;
                let val = self.eval(v, scope)?;
                let Value::Dict(d) = &target else {
                    return Err(rt(
                        "Attribute assignment only supported on dhacha objects or dictionaries in AdarshLang.",
                    ));
                };
                d.borrow_mut().insert_str(attr, val.clone());
                Ok(val)
            }
            Expr::AttrCompoundAssign(t, attr, op, v) => {
                let target = self.eval(t, scope)?;
                let right = self.eval(v, scope)?;
                let Value::Dict(d) = &target else {
                    return Err(rt(
                        "Attribute compound assignment only supported on dhacha objects or dictionaries in AdarshLang.",
                    ));
                };
                let cur = d.borrow().get_str(attr).unwrap_or(Value::Null);
                let nv = arith(*op, &cur, &right)?;
                d.borrow_mut().insert_str(attr, nv.clone());
                Ok(nv)
            }
            Expr::Func(def) => Ok(Value::Func(Rc::new(Closure { def: def.clone(), scope: scope.clone() }))),
            Expr::Call(callee, args) => {
                let f = self.eval(callee, scope)?;
                if !f.is_callable() {
                    return Err(rt("Attempted to call a non-callable value in AdarshLang."));
                }
                let mut vals = Vec::with_capacity(args.len());
                for a in args {
                    vals.push(self.eval(a, scope)?);
                }
                self.call(&f, vals)
            }
        }
    }

    fn index(&mut self, coll: &Value, idx: &Value) -> Eval {
        let not_indexable = || rt("Only indexable values like lists, strings, or dicts support indexing in AdarshLang.");
        if let Value::Dict(d) = coll {
            return match d.borrow().get(idx) {
                Ok(Some(v)) => Ok(v),
                Ok(None) => Err(rt("Key not present during dictionary access in AdarshLang.")),
                Err(_) => Err(not_indexable()),
            };
        }
        let n = normalize_index(idx)?;
        let out_of_range = || rt("Index out of range while accessing collection in AdarshLang.");
        match coll {
            Value::List(l) => {
                let list = l.borrow();
                resolve(n, list.len()).map(|k| list[k].clone()).ok_or_else(out_of_range)
            }
            Value::Str(s) => {
                let len = s.chars().count();
                resolve(n, len)
                    .map(|k| Value::Str(s.chars().nth(k).unwrap().to_string().into()))
                    .ok_or_else(out_of_range)
            }
            _ => Err(not_indexable()),
        }
    }

    /// Call any callable value with evaluated arguments.
    pub fn call(&mut self, f: &Value, args: Vec<Value>) -> Eval {
        match f {
            Value::Builtin(id) => self.call_builtin(*id, args),
            Value::Func(c) => self.call_closure(c.clone(), args),
            _ => Err(rt("Attempted to call a non-callable value in AdarshLang.")),
        }
    }

    fn call_closure(&mut self, c: Rc<Closure>, args: Vec<Value>) -> Eval {
        let def = &c.def;
        let sig = Signature::from_params(&def.params);
        if !sig.allows(args.len()) {
            return Err(AdarshError::runtime(format!(
                "Function '{}' expected between {} and {} arguments in AdarshLang.",
                def.name.as_deref().unwrap_or("<anonymous>"),
                sig.min,
                sig.max_text()
            )));
        }
        if self.depth >= self.max_depth {
            return Err(AdarshError::Host(HostKind::RecursionError, "maximum recursion depth exceeded".into()));
        }
        // resolve arguments (defaults are evaluated in the closure scope, like Python)
        let mut values = Vec::with_capacity(def.params.len());
        let mut args = args.into_iter();
        let mut leftover = false;
        for p in &def.params {
            if p.is_varargs {
                values.push(Value::list(args.by_ref().collect()));
                break;
            }
            match args.next() {
                Some(a) => values.push(a),
                None => match &p.default {
                    Some(d) => values.push(self.eval(d, &c.scope)?),
                    None => return Err(rt("Missing required argument in AdarshLang function call.")),
                },
            }
        }
        if args.next().is_some() {
            leftover = true;
        }
        if leftover {
            return Err(rt("Too many arguments provided in AdarshLang function call."));
        }
        let call_scope = new_scope(Some(&c.scope));
        for (p, v) in def.params.iter().zip(values) {
            declare(&call_scope, &p.name);
            set_var(&call_scope, &p.name, v)?;
        }
        self.depth += 1;
        let r = self.exec_block(&def.body, &call_scope);
        self.depth -= 1;
        match r? {
            Flow::Return(v) | Flow::Normal(v) => Ok(v),
            Flow::Break => Err(rt("'bas' cannot exit a function directly in AdarshLang.")),
            Flow::Continue => Err(rt("'aage_badho' cannot escape to the caller in AdarshLang.")),
        }
    }
}

/// Values a REPL / embedding wants to show for the result of a snippet.
pub fn flow_value(f: &Flow) -> Option<&Value> {
    match f {
        Flow::Normal(v) | Flow::Return(v) => Some(v),
        _ => None,
    }
}
