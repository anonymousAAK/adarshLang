//! Tree-walking interpreter.

use crate::ast::*;
use crate::builtins::{is_iterable, normalize_index, out_of_range, resolve, BUILTINS};
use crate::error::{AdarshError, ErrorKind, Pos};
use crate::loader::{module_path, ModuleLoader};
use crate::semantics::{closest, root_scope, suggestion_suffix, Analyzer, Signature, BUILTIN_TYPES};
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

pub type FxMap<K, V> = HashMap<K, V, BuildHasherDefault<FxHasher>>;

// ------------------------------------------------------------ scopes

/// Variables of one scope. Small scopes (blocks, calls) are a plain vector: names are
/// interned by the parser, so a match is usually one pointer compare. Big scopes (the global
/// one holds every builtin) also get a hash index.
#[derive(Default)]
pub struct Vars {
    items: Vec<(Rc<str>, Value)>,
    index: Option<Box<FxMap<Rc<str>, u32>>>,
}

const INDEX_AT: usize = 12;

#[inline]
fn same_name(a: &str, b: &str) -> bool {
    a.len() == b.len() && (a.as_ptr() == b.as_ptr() || a.as_bytes() == b.as_bytes())
}

impl Vars {
    #[inline]
    pub fn position(&self, name: &str) -> Option<usize> {
        if let Some(ix) = &self.index {
            return ix.get(name).map(|&i| i as usize);
        }
        self.items.iter().position(|(n, _)| same_name(n, name))
    }

    /// Is this exact (interned) name defined here? Pointer compare only.
    #[inline]
    fn has_interned(&self, name: &str) -> bool {
        if let Some(ix) = &self.index {
            return ix.contains_key(name);
        }
        self.items.iter().any(|(n, _)| n.as_ptr() == name.as_ptr())
    }

    #[inline]
    fn name_at(&self, i: usize) -> Option<&str> {
        self.items.get(i).map(|(n, _)| &**n)
    }

    #[inline]
    pub fn get(&self, name: &str) -> Option<&Value> {
        self.position(name).map(|i| &self.items[i].1)
    }

    #[inline]
    pub fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.position(name).map(|i| &mut self.items[i].1)
    }

    #[inline]
    pub fn contains_key(&self, name: &str) -> bool {
        self.position(name).is_some()
    }

    /// Define or overwrite; returns the slot.
    pub fn insert(&mut self, name: Rc<str>, value: Value) -> usize {
        if let Some(i) = self.position(&name) {
            self.items[i].1 = value;
            return i;
        }
        let i = self.items.len();
        if let Some(ix) = &mut self.index {
            ix.insert(name.clone(), i as u32);
        } else if i + 1 >= INDEX_AT {
            let mut ix: FxMap<Rc<str>, u32> = FxMap::default();
            for (j, (n, _)) in self.items.iter().enumerate() {
                ix.insert(n.clone(), j as u32);
            }
            ix.insert(name.clone(), i as u32);
            self.index = Some(Box::new(ix));
        }
        if self.items.capacity() == 0 {
            self.items.reserve_exact(4);
        }
        self.items.push((name, value));
        i
    }

    pub fn iter(&self) -> impl Iterator<Item = (&Rc<str>, &Value)> {
        self.items.iter().map(|(n, v)| (n, v))
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

#[derive(Default)]
pub struct Scope {
    vars: Vars,
    /// `pakka` names and declared types live here only when used (rare)
    consts: Option<HashSet<Rc<str>>>,
    var_types: Option<Box<FxMap<Rc<str>, TypeAnn>>>,
    parent: Option<ScopeRef>,
}

pub type ScopeRef = Rc<RefCell<Scope>>;

/// Called before each statement with its position, scope and call depth; `false` stops the run.
pub type StepHook = Box<dyn FnMut(Pos, &ScopeRef, usize) -> bool>;

/// A record field and where its default value comes from.
type FieldSlot = (Rc<str>, Option<(Rc<TypeInfo>, usize)>);

impl Scope {
    /// Value of a variable defined directly in this scope.
    pub fn vars_get(&self, name: &str) -> Option<Value> {
        self.vars.get(name).cloned()
    }
}

pub fn new_scope(parent: Option<&ScopeRef>) -> ScopeRef {
    Rc::new(RefCell::new(Scope { parent: parent.cloned(), ..Default::default() }))
}

fn undeclared(name: &str) -> AdarshError {
    AdarshError::runtime(crate::semantics::undeclared_message(name)).with_code("E001")
}

fn define(scope: &ScopeRef, name: &Rc<str>, value: Value) {
    scope.borrow_mut().vars.insert(name.clone(), value);
}

fn get_var(scope: &ScopeRef, name: &str) -> Result<Value, AdarshError> {
    // walk the chain by reference: no Rc clones per level
    let s = scope.borrow();
    if let Some(v) = s.vars.get(name) {
        return Ok(v.clone());
    }
    match &s.parent {
        Some(p) => get_var(p, name),
        None => Err(undeclared(name)),
    }
}

/// Does running `nodes` add names to its scope? Blocks that don't can run in the enclosing scope.
fn declares(nodes: &[Node]) -> bool {
    nodes.iter().any(|n| {
        matches!(
            n.kind,
            NodeKind::Badlo { .. }
                | NodeKind::Destructure { .. }
                | NodeKind::Seq(_)
                | NodeKind::KaamDef(_)
                | NodeKind::DhachaDef(_)
                | NodeKind::EnumDef(..)
                | NodeKind::Import { .. }
        )
    })
}

/// A scope for running `nodes`: a fresh child only when the block declares something.
fn block_scope(scope: &ScopeRef, nodes: &[Node]) -> ScopeRef {
    if declares(nodes) {
        new_scope(Some(scope))
    } else {
        scope.clone()
    }
}

/// Find the scope and position holding `name`, using and refreshing the node's cached slot.
///
/// The cache says "`hops` scopes up, at position `index`". It is trusted only after checking
/// that no nearer scope has gained the name and that the slot still holds it, so a stale
/// hint costs one slow lookup, never a wrong answer.
#[inline]
fn resolve_slot<'a>(scope: &'a ScopeRef, name: &str, slot: &Slot) -> Option<(&'a RefCell<Scope>, usize)> {
    let (hops, index) = slot.get();
    if hops != u32::MAX {
        // SAFETY: each scope keeps its parent alive and `parent` is never reassigned, so every
        // scope on the chain lives at least as long as `scope`.
        let mut cur: &'a RefCell<Scope> = scope;
        let mut ok = true;
        for _ in 0..hops {
            let s = cur.borrow();
            if s.vars.has_interned(name) {
                ok = false;
                break;
            }
            match &s.parent {
                Some(p) => cur = unsafe { &*Rc::as_ptr(p) },
                None => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && cur.borrow().vars.name_at(index as usize).is_some_and(|n| same_name(n, name)) {
            return Some((cur, index as usize));
        }
    }
    let mut cur: &'a RefCell<Scope> = scope;
    let mut hops = 0u32;
    loop {
        let s = cur.borrow();
        if let Some(i) = s.vars.position(name) {
            slot.set(hops, i as u32);
            return Some((cur, i));
        }
        let p = s.parent.as_ref()?;
        cur = unsafe { &*Rc::as_ptr(p) };
        hops += 1;
    }
}

#[inline]
fn get_var_cached(scope: &ScopeRef, name: &str, slot: &Slot) -> Result<Value, AdarshError> {
    match resolve_slot(scope, name, slot) {
        Some((s, i)) => Ok(s.borrow().vars.items[i].1.clone()),
        None => Err(undeclared(name)),
    }
}

enum Assigned {
    /// stored; the old value is returned so it is dropped after the scope borrows end
    Done(Value),
    /// the variable has a declared type that must be checked first
    Typed(TypeAnn, Value),
}

fn assign_in(scope: &ScopeRef, name: &str, value: Value) -> Result<Assigned, AdarshError> {
    let mut s = scope.borrow_mut();
    if s.vars.contains_key(name) {
        if s.consts.as_ref().is_some_and(|c| c.contains(name)) {
            return Err(
                rt(format!("'{name}' pakka (constant) hai, isliye uski value badal nahi sakte.")).with_code("E003")
            );
        }
        if let Some(ty) = s.var_types.as_ref().and_then(|t| t.get(name).cloned()) {
            return Ok(Assigned::Typed(ty, value));
        }
        let slot = s.vars.get_mut(name).expect("checked above");
        return Ok(Assigned::Done(std::mem::replace(slot, value)));
    }
    match &s.parent {
        Some(p) => assign_in(p, name, value),
        None => Err(undeclared(name)),
    }
}

fn assign_in_unchecked(scope: &ScopeRef, name: &str, value: Value) -> Option<Value> {
    let mut s = scope.borrow_mut();
    if let Some(slot) = s.vars.get_mut(name) {
        return Some(std::mem::replace(slot, value));
    }
    match &s.parent {
        Some(p) => assign_in_unchecked(p, name, value),
        None => None,
    }
}

fn try_get_var(scope: &ScopeRef, name: &str) -> Option<Value> {
    get_var(scope, name).ok()
}

// ------------------------------------------------------------ control flow

/// Result of executing a statement: a value, or a control-flow signal.
pub enum Flow {
    Normal(Value),
    Return(Value),
    Break(Option<Rc<str>>),
    Continue(Option<Rc<str>>),
}

type Exec = Result<Flow, AdarshError>;
type Eval = Result<Value, AdarshError>;

fn rt(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

/// A declared record type (latest definition of a name wins).
pub struct TypeInfo {
    pub def: Rc<DhachaDef>,
    /// scope the type was defined in, for evaluating field defaults
    scope: ScopeRef,
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
    pub types: FxMap<Rc<str>, Rc<TypeInfo>>,
    /// one buffer per running generator function (`dete_raho`)
    yields: Vec<Vec<Value>>,
    pub depth: usize,
    /// Maximum nesting of function calls before a recursion error.
    pub max_depth: usize,
    rng: u64,
    pub clock: fn() -> f64,
    /// Called before each statement (debugger / step-through trace).
    pub on_step: Option<StepHook>,
    /// Commands recorded by the `kachhua` (turtle) builtins.
    pub turtle: crate::builtins::Turtle,
    /// Host hooks (files, sleeping, internet) that differ between CLI and browser.
    pub host: Rc<dyn crate::builtins::Host>,
}

fn system_clock() -> f64 {
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }
    #[cfg(target_arch = "wasm32")]
    {
        crate::builtins::wasm_clock()
    }
}

impl Interpreter {
    pub fn new(loader: Rc<dyn ModuleLoader>, out: Box<dyn Write>) -> Self {
        let global = new_scope(None);
        {
            let mut g = global.borrow_mut();
            for (id, spec) in BUILTINS.iter().enumerate() {
                g.vars.insert(crate::ast::intern(spec.name), Value::Builtin(id));
            }
            for (name, value) in crate::builtins::global_values() {
                g.vars.insert(crate::ast::intern(name), value);
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
            types: FxMap::default(),
            yields: Vec::new(),
            depth: 0,
            // browsers give WebAssembly a much smaller native stack than the CLI thread
            max_depth: if cfg!(target_arch = "wasm32") { 300 } else { 10_000 },
            rng: seed | 1,
            clock: system_clock,
            on_step: None,
            turtle: crate::builtins::Turtle::default(),
            host: crate::builtins::default_host(),
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

    // ---------------------------------------------------------------- variables

    fn set_var(&mut self, scope: &ScopeRef, name: &str, value: Value) -> Result<(), AdarshError> {
        match assign_in(scope, name, value)? {
            Assigned::Done(old) => {
                drop(old);
                Ok(())
            }
            // typed variable: check, then store (rare, so the second walk is fine)
            Assigned::Typed(ty, value) => {
                self.check_value_type(&value, &ty, &format!("'{name}'"))?;
                match assign_in_unchecked(scope, name, value) {
                    Some(old) => drop(old),
                    None => return Err(undeclared(name)),
                }
                Ok(())
            }
        }
    }

    /// Assignment through a cached slot; constants and typed variables take the checked path.
    #[inline]
    fn set_var_cached(&mut self, scope: &ScopeRef, name: &str, slot: &Slot, value: Value) -> Result<(), AdarshError> {
        if let Some((s, i)) = resolve_slot(scope, name, slot) {
            let mut b = s.borrow_mut();
            if b.consts.is_none() && b.var_types.is_none() {
                let old = std::mem::replace(&mut b.vars.items[i].1, value);
                drop(b);
                drop(old);
                return Ok(());
            }
        }
        self.set_var(scope, name, value)
    }

    fn declare_const(scope: &ScopeRef, name: &Rc<str>) {
        scope.borrow_mut().consts.get_or_insert_with(Default::default).insert(name.clone());
    }

    // ---------------------------------------------------------------- types

    /// Is `v` of type `ty`? Handles builtin names, records (with parents) and enums.
    pub fn value_is(&self, v: &Value, ty: &str) -> bool {
        match ty {
            "koi" => true,
            "number" => matches!(v, Value::Int(_) | Value::Big(_) | Value::Float(_)),
            "string" => matches!(v, Value::Str(_)),
            "boolean" => matches!(v, Value::Bool(_)),
            "list" => matches!(v, Value::List(_)),
            "dict" => matches!(v, Value::Dict(d) if d.borrow().type_name.is_none()),
            "tuple" => matches!(v, Value::Tuple(_)),
            "set" => matches!(v, Value::Set(_)),
            "kaam" => v.is_callable(),
            "khali" => matches!(v, Value::Null),
            "dhacha" => matches!(v, Value::Dict(d) if d.borrow().type_name.is_some()),
            "vikalp" => matches!(v, Value::Enum(_)),
            _ => match v {
                Value::Dict(d) => match &d.borrow().type_name {
                    Some(t) => self.is_subtype(t, ty),
                    None => false,
                },
                Value::Enum(e) => &*e.type_name == ty,
                _ => false,
            },
        }
    }

    fn is_subtype(&self, t: &str, ancestor: &str) -> bool {
        let mut cur: Option<Rc<str>> = Some(Rc::from(t));
        let mut steps = 0;
        while let Some(name) = cur {
            if &*name == ancestor {
                return true;
            }
            steps += 1;
            if steps > 64 {
                return false;
            }
            cur = self.types.get(&name).and_then(|i| i.def.parent.clone());
        }
        false
    }

    fn check_value_type(&self, v: &Value, ty: &TypeAnn, what: &str) -> Result<(), AdarshError> {
        if self.value_is(v, &ty.name) {
            return Ok(());
        }
        Err(rt(format!("{what} {} hona chahiye tha, par {} mila.", ty.name, v.type_desc())).with_code("E006"))
    }

    fn find_method(&self, type_name: &str, method: &str) -> Option<Rc<Closure>> {
        let mut cur: Option<Rc<str>> = Some(Rc::from(type_name));
        let mut steps = 0;
        while let Some(t) = cur {
            if let Some(m) = self.methods.get(&t).and_then(|ms| ms.get(method)) {
                return Some(m.clone());
            }
            steps += 1;
            if steps > 64 {
                return None;
            }
            cur = self.types.get(&t).and_then(|i| i.def.parent.clone());
        }
        None
    }

    fn record_type(v: &Value) -> Option<Rc<str>> {
        match v {
            Value::Dict(d) => d.borrow().type_name.clone(),
            _ => None,
        }
    }

    /// Text for `dikhao` / interpolation, using a record's `shabd()` method if it has one.
    pub fn show(&mut self, v: &Value) -> String {
        let mut hook = |x: &Value| self.custom_text(x);
        display_with(v, &mut hook)
    }

    pub fn show_repr(&mut self, v: &Value) -> String {
        let mut hook = |x: &Value| self.custom_text(x);
        repr_with(v, &mut hook)
    }

    fn custom_text(&mut self, v: &Value) -> Option<String> {
        let t = Self::record_type(v)?;
        let m = self.find_method(&t, "shabd")?;
        let r = self.call_closure(m, Some(v.clone()), vec![], vec![]).ok()?;
        Some(display(&r))
    }

    // ---------------------------------------------------------------- statements

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
        if self.on_step.is_some() && !matches!(node.kind, NodeKind::KaamDef(_) | NodeKind::DhachaDef(_)) {
            let depth = self.depth;
            let mut f = self.on_step.take().unwrap();
            let keep_going = f(node.pos, scope, depth);
            self.on_step = Some(f);
            if !keep_going {
                return Err(AdarshError::new(ErrorKind::Exit(0), "step limit"));
            }
        }
        self.exec_kind(node, scope).map_err(|e| e.at(node.pos))
    }

    /// Run a loop body; `Some(flow)` means leave the loop with that result.
    fn loop_body(
        &mut self,
        label: &Option<Rc<str>>,
        body: &[Node],
        scope: &ScopeRef,
    ) -> Result<Option<Flow>, AdarshError> {
        Ok(match self.exec_block(body, &block_scope(scope, body))? {
            Flow::Return(v) => Some(Flow::Return(v)),
            Flow::Break(None) => Some(Flow::Normal(Value::Null)),
            Flow::Break(Some(l)) if label.as_ref() == Some(&l) => Some(Flow::Normal(Value::Null)),
            Flow::Break(other) => Some(Flow::Break(other)),
            Flow::Continue(Some(l)) if label.as_ref() != Some(&l) => Some(Flow::Continue(Some(l))),
            Flow::Continue(_) | Flow::Normal(_) => None,
        })
    }

    fn exec_kind(&mut self, node: &Node, scope: &ScopeRef) -> Exec {
        match &node.kind {
            NodeKind::Expr(e) => Ok(Flow::Normal(self.eval(e, scope)?)),
            NodeKind::Badlo { name, init, ty, constant } => {
                let v = match init {
                    Some(e) => self.eval(e, scope)?,
                    None => Value::Null,
                };
                if let Some(t) = ty {
                    if init.is_some() {
                        self.check_value_type(&v, t, &format!("'{name}'"))?;
                    }
                    scope.borrow_mut().var_types.get_or_insert_with(Default::default).insert(name.clone(), t.clone());
                }
                define(scope, name, v);
                if *constant {
                    Self::declare_const(scope, name);
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Destructure { target, init, constant } => self.exec_destructure(target, init, *constant, scope),
            NodeKind::Seq(nodes) => {
                for n in nodes {
                    self.exec(n, scope)?;
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Dikhao(e) => {
                let v = self.eval(e, scope)?;
                let mut s = self.show(&v);
                s.push('\n');
                self.write_str(&s);
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Break(l) => Ok(Flow::Break(l.clone())),
            NodeKind::Continue(l) => Ok(Flow::Continue(l.clone())),
            NodeKind::Wapas(e) => {
                let v = match e {
                    Some(e) => self.eval(e, scope)?,
                    None => Value::Null,
                };
                Ok(Flow::Return(v))
            }
            NodeKind::Yield(e) => {
                let v = self.eval(e, scope)?;
                match self.yields.last_mut() {
                    Some(buf) => {
                        if buf.len() >= 10_000_000 {
                            return Err(rt(
                                "'dete_raho' ne 1 crore se zyada values di. Shayad loop kabhi khatam nahi hota?",
                            ));
                        }
                        buf.push(v);
                    }
                    None => return Err(rt("'dete_raho' sirf kaam (function) ke andar chalta hai.")),
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::Block(b) => self.exec_block(b, &block_scope(scope, b)),
            NodeKind::If { cond, then, otherwise } => {
                if self.eval(cond, scope)?.truthy() {
                    self.exec_block(then, &block_scope(scope, then))
                } else if let Some(o) = otherwise {
                    match &o.kind {
                        NodeKind::Block(b) => self.exec_block(b, &block_scope(scope, b)),
                        _ => self.exec(o, scope),
                    }
                } else {
                    Ok(Flow::Normal(Value::Null))
                }
            }
            NodeKind::While { cond, body, label } => {
                while self.eval(cond, scope)?.truthy() {
                    if let Some(f) = self.loop_body(label, body, scope)? {
                        return Ok(f);
                    }
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::DoWhile { body, cond, label } => loop {
                if let Some(f) = self.loop_body(label, body, scope)? {
                    return Ok(f);
                }
                if !self.eval(cond, scope)?.truthy() {
                    return Ok(Flow::Normal(Value::Null));
                }
            },
            NodeKind::For { init, cond, update, body, label } => {
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
                    if let Some(f) = self.loop_body(label, body, &ls)? {
                        return Ok(f);
                    }
                    if let Some(u) = update {
                        self.eval(u, &ls)?;
                    }
                }
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::ForEach { index, var, declare, iterable, body, label } => {
                self.exec_foreach(index, var, *declare, iterable, body, label, scope)
            }
            NodeKind::Switch { subject, cases, default } => self.exec_switch(subject, cases, default, scope),
            NodeKind::TryCatch { body, catches, finally } => self.exec_try(body, catches, finally, scope),
            NodeKind::Throw(e) => {
                let v = self.eval(e, scope)?;
                Err(AdarshError::thrown(v).at(e.pos))
            }
            NodeKind::Import { path, alias } => self.exec_import(path, alias, scope),
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
            NodeKind::DhachaDef(def) => {
                self.types.insert(def.name.clone(), Rc::new(TypeInfo { def: def.clone(), scope: scope.clone() }));
                Ok(Flow::Normal(Value::Null))
            }
            NodeKind::EnumDef(name, variants) => {
                let vs = variants
                    .iter()
                    .enumerate()
                    .map(|(i, v)| Rc::new(EnumValue { type_name: name.clone(), name: v.clone(), index: i }))
                    .collect();
                define(scope, name, Value::EnumType(Rc::new(EnumType { name: name.clone(), variants: vs })));
                Ok(Flow::Normal(Value::Null))
            }
        }
    }

    #[inline(never)]
    fn exec_destructure(&mut self, target: &Destructure, init: &Expr, constant: bool, scope: &ScopeRef) -> Exec {
        let v = self.eval(init, scope)?;
        let bind = |name: &Rc<str>, value: Value| {
            define(scope, name, value);
            if constant {
                Self::declare_const(scope, name);
            }
        };
        match target {
            Destructure::List(names, rest) => {
                let items: Vec<Value> = match &v {
                    Value::List(l) => l.borrow().clone(),
                    Value::Tuple(t) => t.to_vec(),
                    other => {
                        return Err(rt(format!(
                            "'badlo [..] = ...' ke right mein list ya tuple chahiye, {} mila.",
                            other.type_desc()
                        )))
                    }
                };
                let ok = if rest.is_some() { items.len() >= names.len() } else { items.len() == names.len() };
                if !ok {
                    return Err(rt(format!(
                        "List mein {} items hain, par {} naam diye{}.",
                        items.len(),
                        names.len(),
                        if rest.is_some() { " (aur ...baaki)" } else { "" }
                    )));
                }
                for (n, item) in names.iter().zip(items.iter()) {
                    if let Some(n) = n {
                        bind(n, item.clone());
                    }
                }
                if let Some(r) = rest {
                    bind(r, Value::list(items[names.len()..].to_vec()));
                }
            }
            Destructure::Fields(names) => {
                let Value::Dict(d) = &v else {
                    return Err(rt(format!(
                        "'badlo {{..}} = ...' ke right mein dict ya dhacha chahiye, {} mila.",
                        v.type_desc()
                    )));
                };
                for n in names {
                    let field = d.borrow().get_str(n);
                    match field {
                        Some(x) => bind(n, x),
                        None => return Err(rt(format!("'{n}' field/key nahi mila."))),
                    }
                }
            }
        }
        Ok(Flow::Normal(Value::Null))
    }

    #[allow(clippy::too_many_arguments)]
    #[inline(never)]
    fn exec_foreach(
        &mut self,
        index: &Option<Rc<str>>,
        var: &Rc<str>,
        declare: bool,
        iterable: &Expr,
        body: &[Node],
        label: &Option<Rc<str>>,
        scope: &ScopeRef,
    ) -> Exec {
        let it = self.eval(iterable, scope)?;
        if !is_iterable(&it) {
            let hint = if matches!(it, Value::Int(_)) { " Ginti ke liye 1..n likho." } else { "" };
            return Err(rt(format!(
                "'ke_liye' sirf list, string, dict, tuple ya set par chalta hai, {} par nahi.{hint}",
                it.type_desc()
            ))
            .at(iterable.pos));
        }
        let ls = new_scope(Some(scope));
        if declare {
            define(&ls, var, Value::Null);
            if let Some(i) = index {
                define(&ls, i, Value::Null);
            }
        }
        let target = if declare { ls.clone() } else { scope.clone() };
        // lists are read live (items added during the loop are visited); the rest iterate a snapshot
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
            if let Some(idx) = index {
                // dicts give (key, value); everything else (position, item)
                let (first, second) = match &it {
                    Value::Dict(d) => {
                        let val = d.borrow().get(&item)?.unwrap_or(Value::Null);
                        (item, val)
                    }
                    _ => (Value::Int(i as i64), item),
                };
                self.set_var(&target, idx, first)?;
                self.set_var(&target, var, second)?;
            } else {
                self.set_var(&target, var, item)?;
            }
            i += 1;
            if let Some(f) = self.loop_body(label, body, &ls)? {
                return Ok(f);
            }
        }
        Ok(Flow::Normal(Value::Null))
    }

    fn match_pattern(
        &mut self,
        p: &Pattern,
        v: &Value,
        scope: &ScopeRef,
        case_scope: &ScopeRef,
    ) -> Result<bool, AdarshError> {
        Ok(match p {
            Pattern::Wildcard => true,
            Pattern::Bind(n) => {
                define(case_scope, n, v.clone());
                true
            }
            Pattern::Value(e) => {
                if let ExprKind::Var(n, _) = &e.kind {
                    let is_type = BUILTIN_TYPES.contains(&&**n) || self.types.contains_key(n);
                    if is_type && try_get_var(scope, n).is_none() {
                        return Ok(self.value_is(v, n));
                    }
                }
                let target = self.eval(e, scope)?;
                values_eq(&target, v)
            }
            Pattern::Range(lo, hi) => {
                let (lo, hi) = (self.eval(lo, scope)?, self.eval(hi, scope)?);
                compare(BinOp::Gte, v, &lo).unwrap_or(false) && compare(BinOp::Lte, v, &hi).unwrap_or(false)
            }
            Pattern::Type(t) => self.value_is(v, &t.name),
            Pattern::List(items, rest) => {
                let values: Vec<Value> = match v {
                    Value::List(l) => l.borrow().clone(),
                    Value::Tuple(t) => t.to_vec(),
                    _ => return Ok(false),
                };
                let len_ok = if rest.is_some() { values.len() >= items.len() } else { values.len() == items.len() };
                if !len_ok {
                    return Ok(false);
                }
                for (pi, vi) in items.iter().zip(values.iter()) {
                    if !self.match_pattern(pi, vi, scope, case_scope)? {
                        return Ok(false);
                    }
                }
                if let Some(r) = rest {
                    define(case_scope, r, Value::list(values[items.len()..].to_vec()));
                }
                true
            }
        })
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
            for p in &c.patterns {
                let cs = new_scope(Some(scope));
                if !self.match_pattern(p, &sv, scope, &cs)? {
                    continue;
                }
                if let Some(g) = &c.guard {
                    if !self.eval(g, &cs)?.truthy() {
                        continue;
                    }
                }
                return self.exec_block(&c.body, &cs);
            }
        }
        match default {
            Some(d) => self.exec_block(d, &new_scope(Some(scope))),
            None => Ok(Flow::Normal(Value::Null)),
        }
    }

    #[inline(never)]
    fn exec_try(&mut self, body: &[Node], catches: &[Catch], finally: &Option<Vec<Node>>, scope: &ScopeRef) -> Exec {
        let depth = self.depth;
        let yields = self.yields.len();
        let mut result = self.exec_block(body, &new_scope(Some(scope)));
        if let Err(e) = &result {
            if e.catchable() {
                for c in catches {
                    let matches = match &c.ty {
                        None => true,
                        Some(t) if &*t.name == "Galti" => e.kind == ErrorKind::Runtime,
                        Some(t) => match &e.thrown {
                            Some(v) => self.value_is(v, &t.name),
                            None => &*t.name == "koi",
                        },
                    };
                    if matches {
                        self.depth = depth;
                        self.yields.truncate(yields);
                        let value = e.catch_value();
                        let cs = new_scope(Some(scope));
                        define(&cs, &c.name, value);
                        result = self.exec_block(&c.body, &cs);
                        break;
                    }
                }
            }
        }
        if let Some(f) = finally {
            self.depth = depth;
            match self.exec_block(f, &new_scope(Some(scope)))? {
                Flow::Normal(_) => {}
                other => return Ok(other),
            }
        }
        result
    }

    #[inline(never)]
    fn exec_import(&mut self, e: &Expr, alias: &Option<Rc<str>>, scope: &ScopeRef) -> Exec {
        let mv = self.eval(e, scope)?;
        let Value::Str(name) = mv else {
            return Err(rt("'lao' ko file ka naam (string) chahiye.").at(e.pos));
        };
        let path = module_path(&name);
        if alias.is_none() && self.loaded_modules.contains(&path) {
            return Ok(Flow::Normal(Value::Null));
        }
        let Some(src) = self.loader.load(&path) else {
            return Err(rt(format!("Module '{path}' nahi mila.")).at(e.pos).with_code("E007"));
        };
        let file = crate::error::register_source(&path, &src);
        let module = crate::parser::parse(&src, file)?;
        let mut sema = Analyzer::new(self.loader.clone());
        sema.analyze_program(&module, &root_scope())?;
        match alias {
            None => {
                self.loaded_modules.insert(path);
                let g = self.global.clone();
                self.exec_block(&module, &g)?;
            }
            Some(a) => {
                // run in its own scope and expose its names as `a.naam`
                let ms = new_scope(Some(&self.global));
                self.exec_block(&module, &ms)?;
                let mut ns = Dict::new();
                let mut names: Vec<(Rc<str>, Value)> =
                    ms.borrow().vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
                names.sort_by(|x, y| x.0.cmp(&y.0));
                for (k, v) in names {
                    ns.insert_str(&k, v);
                }
                define(scope, a, Value::dict(ns));
            }
        }
        Ok(Flow::Normal(Value::Null))
    }

    // ---------------------------------------------------------------- expressions

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
            ExprKind::Var(name, slot) => get_var_cached(scope, name, slot),
            ExprKind::List(items) => Ok(Value::list(self.eval_items(items, scope)?)),
            ExprKind::Tuple(items) => Ok(Value::Tuple(Rc::new(self.eval_items(items, scope)?))),
            ExprKind::Dict(entries) => self.eval_dict(entries, scope),
            ExprKind::Dhacha { type_name, fields } => self.eval_record(type_name, fields, scope),
            ExprKind::Binary(l, op, r) => self.eval_binary(l, *op, r, scope),
            ExprKind::Chain(items, ops) => {
                let mut lhs = self.eval(&items[0], scope)?;
                for (op, item) in ops.iter().zip(items.iter().skip(1)) {
                    let rhs = self.eval(item, scope)?;
                    if !compare(*op, &lhs, &rhs)? {
                        return Ok(Value::Bool(false));
                    }
                    lhs = rhs;
                }
                Ok(Value::Bool(true))
            }
            ExprKind::Unary(op, x) => {
                let v = self.eval(x, scope)?;
                match op {
                    UnOp::Neg => negate(&v),
                    UnOp::Not => Ok(Value::Bool(!v.truthy())),
                }
            }
            ExprKind::Range(lo, hi) => self.eval_range(lo, hi, scope),
            ExprKind::IfExpr(c, a, b) => {
                if self.eval(c, scope)?.truthy() {
                    self.eval(a, scope)
                } else {
                    self.eval(b, scope)
                }
            }
            ExprKind::TypeCheck(x, t) => {
                let v = self.eval(x, scope)?;
                Ok(Value::Bool(self.value_is(&v, &t.name)))
            }
            ExprKind::Spread(_) => Err(rt("'...' sirf list ke andar ya function call ke arguments mein chalta hai.")),
            ExprKind::NamedArg(n, _) => Err(rt(format!("'{n}: ...' sirf function call ke andar chalta hai."))),
            ExprKind::Assign(name, rhs, slot) => {
                let v = self.eval(rhs, scope)?;
                self.set_var_cached(scope, name, slot, v.clone())?;
                Ok(v)
            }
            ExprKind::CompoundAssign(name, op, rhs, slot) => {
                let cur = get_var_cached(scope, name, slot)?;
                let r = self.eval(rhs, scope)?;
                let v = self.binary_values(&cur, *op, &r)?;
                self.set_var_cached(scope, name, slot, v.clone())?;
                Ok(v)
            }
            ExprKind::Index(c, i) => {
                let coll = self.eval(c, scope)?;
                let idx = self.eval(i, scope)?;
                self.index(&coll, &idx)
            }
            ExprKind::Slice(c, a, b, s) => self.eval_slice(c, a, b, s, scope),
            ExprKind::IndexAssign(c, i, v) => self.eval_index_assign(c, i, v, scope),
            ExprKind::IndexCompoundAssign(c, i, op, v) => self.eval_index_compound(c, i, op, v, scope),
            ExprKind::Attr(t, attr) => {
                let target = self.eval(t, scope)?;
                self.attr(&target, attr)
            }
            ExprKind::SafeAttr(t, attr) => {
                let target = self.eval(t, scope)?;
                if matches!(target, Value::Null) {
                    return Ok(Value::Null);
                }
                self.attr(&target, attr)
            }
            ExprKind::AttrAssign(t, attr, v) => self.eval_attr_assign(t, attr, v, scope),
            ExprKind::AttrCompoundAssign(t, attr, op, v) => self.eval_attr_compound(t, attr, op, v, scope),
            ExprKind::Func(def) => Ok(Value::Func(Rc::new(Closure { def: def.clone(), scope: scope.clone() }))),
            ExprKind::Call(callee, args) => self.eval_call(callee, args, scope),
        }
    }

    fn eval_items(&mut self, items: &[Expr], scope: &ScopeRef) -> Result<Vec<Value>, AdarshError> {
        let mut v = Vec::with_capacity(items.len());
        for i in items {
            if let ExprKind::Spread(inner) = &i.kind {
                let x = self.eval(inner, scope)?;
                let items = crate::builtins::iter_items(&x).ok_or_else(|| {
                    rt(format!("'...' sirf list, tuple, set ya string ko faila sakta hai, {} ko nahi.", x.type_desc()))
                        .at(i.pos)
                })?;
                v.extend(items);
            } else {
                v.push(self.eval(i, scope)?);
            }
        }
        Ok(v)
    }

    #[inline(never)]
    fn eval_binary(&mut self, l: &Expr, op: BinOp, r: &Expr, scope: &ScopeRef) -> Eval {
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
            BinOp::Coalesce => {
                if !matches!(a, Value::Null) {
                    return Ok(a);
                }
                return self.eval(r, scope);
            }
            _ => {}
        }
        let b = self.eval(r, scope)?;
        self.binary_values(&a, op, &b)
    }

    /// Apply a binary operator, honouring record operator methods (`jodo`, `barabar`, `chhota`, ...).
    pub fn binary_values(&mut self, a: &Value, op: BinOp, b: &Value) -> Eval {
        if let Some(t) = Self::record_type(a) {
            let call = |me: &mut Self, name: &str, x: &Value, y: &Value| -> Option<Eval> {
                let m = me.find_method(&t, name)?;
                Some(me.call_closure(m, Some(x.clone()), vec![y.clone()], vec![]))
            };
            let via = match op {
                BinOp::Neq => call(self, "barabar", a, b).map(|r| r.map(|v| Value::Bool(!v.truthy()))),
                BinOp::Gt => call(self, "chhota", b, a),
                BinOp::Lte => call(self, "chhota", b, a).map(|r| r.map(|v| Value::Bool(!v.truthy()))),
                BinOp::Gte => call(self, "chhota", a, b).map(|r| r.map(|v| Value::Bool(!v.truthy()))),
                _ => op.method_name().and_then(|m| call(self, m, a, b)),
            };
            if let Some(r) = via {
                return r;
            }
        }
        match op {
            BinOp::Eq => Ok(Value::Bool(values_eq(a, b))),
            BinOp::Neq => Ok(Value::Bool(!values_eq(a, b))),
            BinOp::Lt | BinOp::Gt | BinOp::Lte | BinOp::Gte => Ok(Value::Bool(compare(op, a, b)?)),
            BinOp::In => Ok(Value::Bool(contains(b, a)?)),
            _ => arith(op, a, b),
        }
    }

    #[inline(never)]
    fn eval_range(&mut self, lo: &Expr, hi: &Expr, scope: &ScopeRef) -> Eval {
        let (a, b) = (self.eval(lo, scope)?, self.eval(hi, scope)?);
        let as_int = |v: &Value| match v {
            Value::Int(i) => Ok(*i),
            Value::Bool(b) => Ok(*b as i64),
            Value::Float(f) if f.fract() == 0.0 && f.abs() < 9e18 => Ok(*f as i64),
            other => Err(rt(format!("'..' range ko poore number chahiye, {} mila.", display(other)))),
        };
        let (a, b) = (as_int(&a)?, as_int(&b)?);
        let len = (b as i128 - a as i128).unsigned_abs() + 1;
        if len > 100_000_000 {
            return Err(rt("Range bahut badi hai (10 crore se zyada numbers)."));
        }
        // counts down when the end is smaller: 5..1 gives [5, 4, 3, 2, 1]
        let items = if a <= b { (a..=b).map(Value::Int).collect() } else { (b..=a).rev().map(Value::Int).collect() };
        Ok(Value::list(items))
    }

    #[inline(never)]
    fn eval_slice(
        &mut self,
        c: &Expr,
        a: &Option<Box<Expr>>,
        b: &Option<Box<Expr>>,
        s: &Option<Box<Expr>>,
        scope: &ScopeRef,
    ) -> Eval {
        let coll = self.eval(c, scope)?;
        let get = |x: &Option<Box<Expr>>, me: &mut Self| -> Result<Option<i64>, AdarshError> {
            match x {
                None => Ok(None),
                Some(e) => match me.eval(e, scope)? {
                    Value::Null => Ok(None),
                    v => normalize_index(&v).map(|n| Some(n.unwrap_or(i64::MAX / 4))),
                },
            }
        };
        let (start, end, step) = (get(a, self)?, get(b, self)?, get(s, self)?);
        let step = step.unwrap_or(1);
        if step == 0 {
            return Err(rt("Slice ka step 0 nahi ho sakta."));
        }
        let pick = |len: usize| crate::builtins::slice_positions(len, start, end, step);
        Ok(match &coll {
            Value::List(l) => {
                let l = l.borrow();
                Value::list(pick(l.len()).into_iter().map(|i| l[i].clone()).collect())
            }
            Value::Tuple(t) => Value::Tuple(Rc::new(pick(t.len()).into_iter().map(|i| t[i].clone()).collect())),
            Value::Str(st) => {
                let chars: Vec<char> = st.chars().collect();
                Value::Str(pick(chars.len()).into_iter().map(|i| chars[i]).collect::<String>().into())
            }
            other => return Err(rt(format!("{} ka slice [a:b] nahi hota.", other.type_desc()))),
        })
    }

    #[inline(never)]
    fn eval_interp(&mut self, parts: &[Part], scope: &ScopeRef) -> Eval {
        let mut s = String::new();
        for p in parts {
            match p {
                Part::Lit(l) => s.push_str(l),
                Part::Expr(x, spec) => {
                    let v = self.eval(x, scope)?;
                    match spec {
                        None => s.push_str(&self.show(&v)),
                        Some(spec) => {
                            let text = self.show(&v);
                            s.push_str(&crate::builtins::format_spec(&v, &text, spec).map_err(|e| e.at(x.pos))?)
                        }
                    }
                }
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

    /// Fields of a record type including its parents (parents first), with their defaults.
    fn record_layout(&self, type_name: &str) -> Result<Vec<FieldSlot>, AdarshError> {
        let mut chain: Vec<Rc<TypeInfo>> = Vec::new();
        let mut cur: Option<Rc<str>> = Some(Rc::from(type_name));
        while let Some(t) = cur {
            let Some(info) = self.types.get(&t).cloned() else {
                let hint = suggestion_suffix(closest(&t, self.types.keys().map(|k| &**k)));
                return Err(rt(format!("'{t}' naam ka dhacha nahi mila.{hint}")));
            };
            if chain.iter().any(|c| c.def.name == info.def.name) {
                return Err(rt(format!("Dhacha '{t}' apna hi parent ban gaya (gol ghera).")));
            }
            cur = info.def.parent.clone();
            chain.push(info);
        }
        let mut out: Vec<FieldSlot> = Vec::new();
        for info in chain.iter().rev() {
            for (i, f) in info.def.fields.iter().enumerate() {
                let default = f.default.as_ref().map(|_| (info.clone(), i));
                if let Some(slot) = out.iter_mut().find(|(n, _)| *n == f.name) {
                    slot.1 = default.or(slot.1.take());
                } else {
                    out.push((f.name.clone(), default));
                }
            }
        }
        Ok(out)
    }

    #[inline(never)]
    fn eval_record(&mut self, type_name: &Rc<str>, fields: &[(Rc<str>, Expr)], scope: &ScopeRef) -> Eval {
        let layout = self.record_layout(type_name)?;
        let mut d = Dict::record(type_name.clone());
        for (name, default) in &layout {
            if let Some((_, fe)) = fields.iter().find(|(n, _)| n == name) {
                let v = self.eval(fe, scope)?;
                d.insert_str(name, v);
            } else if let Some((info, i)) = default {
                let expr = info.def.fields[*i].default.as_ref().unwrap();
                let def_scope = info.scope.clone();
                let v = self.eval(expr, &def_scope)?;
                d.insert_str(name, v);
            } else {
                d.insert_str(name, Value::Null);
            }
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
        let nv = self.binary_values(&cur, *op, &right)?;
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
        let nv = self.binary_values(&cur, *op, &right)?;
        let Value::Dict(d) = &target else { unreachable!("attr() succeeded on a dict") };
        d.borrow_mut().insert_str(attr, nv.clone());
        Ok(nv)
    }

    pub fn attr(&mut self, target: &Value, attr: &str) -> Eval {
        match target {
            Value::Dict(d) => {
                let d_ref = d.borrow();
                if let Some(v) = d_ref.get_str(attr) {
                    return Ok(v);
                }
                if let Some(t) = d_ref.type_name.clone() {
                    drop(d_ref);
                    if let Some(m) = self.find_method(&t, attr) {
                        return Ok(Value::Method(Rc::new(BoundMethod { receiver: target.clone(), func: m })));
                    }
                    let fields: Vec<String> = d.borrow().entries().iter().map(|(k, _)| display(k)).collect();
                    let mut names = fields.clone();
                    if let Some(ms) = self.methods.get(&t) {
                        names.extend(ms.keys().map(|k| k.to_string()));
                    }
                    let hint = suggestion_suffix(closest(attr, names.iter().map(String::as_str)));
                    return Err(rt(format!("Dhacha {t} mein '{attr}' naam ka field ya method nahi hai.{hint}")));
                }
                let keys: Vec<String> = d_ref.entries().iter().map(|(k, _)| display(k)).collect();
                let hint = suggestion_suffix(closest(attr, keys.iter().map(String::as_str)));
                Err(rt(format!("Dict mein '{attr}' key nahi hai.{hint}")))
            }
            Value::EnumType(t) => match t.variants.iter().find(|v| &*v.name == attr) {
                Some(v) => Ok(Value::Enum(v.clone())),
                None => {
                    let hint = suggestion_suffix(closest(attr, t.variants.iter().map(|v| &*v.name)));
                    Err(rt(format!("Vikalp {} mein '{attr}' nahi hai.{hint}", t.name)))
                }
            },
            Value::Enum(e) => match attr {
                "naam" => Ok(Value::Str(e.name.clone())),
                "kram" => Ok(Value::Int(e.index as i64)),
                _ => Err(rt(format!("Vikalp value par sirf '.naam' aur '.kram' milte hain, '.{attr}' nahi."))),
            },
            other => Err(rt(format!(
                "{} par '.{attr}' nahi chalta. Sirf dict aur dhacha ke fields hote hain.",
                other.type_desc()
            ))),
        }
    }

    fn index(&mut self, coll: &Value, idx: &Value) -> Eval {
        match coll {
            Value::Dict(d) => match d.borrow().get(idx)? {
                Some(v) => Ok(v),
                None => {
                    let keys: Vec<String> = d.borrow().entries().iter().map(|(k, _)| display(k)).collect();
                    let hint = match idx {
                        Value::Str(s) => suggestion_suffix(closest(s, keys.iter().map(String::as_str))),
                        _ => String::new(),
                    };
                    Err(rt(format!("Key {} dict mein nahi hai.{hint}", repr(idx))).with_code("E008"))
                }
            },
            Value::List(l) => {
                let n = normalize_index(idx)?;
                let list = l.borrow();
                resolve(n, list.len()).map(|k| list[k].clone()).ok_or_else(|| out_of_range(idx, list.len(), "List"))
            }
            Value::Tuple(t) => {
                let n = normalize_index(idx)?;
                resolve(n, t.len()).map(|k| t[k].clone()).ok_or_else(|| out_of_range(idx, t.len(), "Tuple"))
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
            Value::Tuple(_) => Err(rt("Tuple badla nahi ja sakta. Badalna ho to list [..] use karo.")),
            other => Err(rt(format!("{} mein [index] se value nahi rakh sakte.", other.type_desc()))),
        }
    }

    // ---------------------------------------------------------------- calls

    #[inline(never)]
    fn eval_call(&mut self, callee: &Expr, args: &[Expr], scope: &ScopeRef) -> Eval {
        // `x.f(...)`: field / method, else the builtin or function `f` with x first ("dot-call")
        let (f, receiver) = match &callee.kind {
            ExprKind::Attr(t, name) | ExprKind::SafeAttr(t, name) => {
                let target = self.eval(t, scope)?;
                if matches!(callee.kind, ExprKind::SafeAttr(..)) && matches!(target, Value::Null) {
                    return Ok(Value::Null);
                }
                let own = match &target {
                    Value::Dict(d) => {
                        let has_field = d.borrow().get_str(name).is_some();
                        let is_method = Self::record_type(&target).and_then(|t| self.find_method(&t, name)).is_some();
                        has_field || is_method
                    }
                    Value::EnumType(_) | Value::Enum(_) => true,
                    _ => false,
                };
                if own {
                    (self.attr(&target, name).map_err(|e| e.at(callee.pos))?, None)
                } else {
                    match try_get_var(scope, name) {
                        Some(func) if func.is_callable() => (func, Some(target)),
                        _ => {
                            return Err(self
                                .attr(&target, name)
                                .err()
                                .unwrap_or_else(|| rt(format!("'{name}' naam ka function nahi mila."))));
                        }
                    }
                }
            }
            _ => (self.eval(callee, scope)?, None),
        };
        if !f.is_callable() {
            let what = match &callee.kind {
                ExprKind::Var(n, _) => format!("'{n}' ({})", f.type_desc()),
                _ => f.type_desc(),
            };
            return Err(rt(format!("{what} function nahi hai, isliye call nahi kar sakte.")).with_code("E009"));
        }
        let mut positional: Vec<Value> = Vec::with_capacity(args.len() + 1);
        if let Some(r) = receiver {
            positional.push(r);
        }
        let mut named: Vec<(Rc<str>, Value)> = Vec::new();
        for a in args {
            match &a.kind {
                ExprKind::Spread(inner) => {
                    let x = self.eval(inner, scope)?;
                    let items = crate::builtins::iter_items(&x).ok_or_else(|| {
                        rt(format!(
                            "'...' sirf list, tuple, set ya string ko faila sakta hai, {} ko nahi.",
                            x.type_desc()
                        ))
                        .at(a.pos)
                    })?;
                    positional.extend(items);
                }
                ExprKind::NamedArg(n, x) => {
                    let v = self.eval(x, scope)?;
                    if named.iter().any(|(k, _)| k == n) {
                        return Err(rt(format!("'{n}:' do baar diya gaya.")).at(a.pos));
                    }
                    named.push((n.clone(), v));
                }
                _ => positional.push(self.eval(a, scope)?),
            }
        }
        let name = match &f {
            Value::Func(c) => Some(c.def.display_name()),
            Value::Method(m) => Some(m.func.def.display_name()),
            _ => None,
        };
        match self.call_full(&f, positional, named) {
            // record which call the error passed through, for the "call chain" in the message
            Err(mut e) if name.is_some() && e.catchable() && e.pos.is_some() && e.trace.len() < 64 => {
                e.trace.push((name.unwrap(), callee.pos));
                Err(e)
            }
            other => other,
        }
    }

    /// Call any callable value with evaluated arguments.
    pub fn call(&mut self, f: &Value, args: Vec<Value>) -> Eval {
        self.call_full(f, args, Vec::new())
    }

    pub fn call_full(&mut self, f: &Value, args: Vec<Value>, named: Vec<(Rc<str>, Value)>) -> Eval {
        match f {
            Value::Builtin(id) => self.call_builtin(*id, args, named),
            Value::Func(c) => self.call_closure(c.clone(), None, args, named),
            Value::Method(m) => self.call_closure(m.func.clone(), Some(m.receiver.clone()), args, named),
            other => Err(rt(format!("{} function nahi hai, isliye call nahi kar sakte.", other.type_desc()))),
        }
    }

    pub fn call_closure(
        &mut self,
        c: Rc<Closure>,
        receiver: Option<Value>,
        args: Vec<Value>,
        named: Vec<(Rc<str>, Value)>,
    ) -> Eval {
        let def = &c.def;
        // place named arguments into their parameter slots
        let mut slots: Vec<Option<Value>> = Vec::with_capacity(def.params.len());
        let mut varargs: Vec<Value> = Vec::new();
        let mut args = args.into_iter();
        for p in &def.params {
            if p.is_varargs {
                varargs.extend(args.by_ref());
                slots.push(None);
            } else {
                slots.push(args.next());
            }
        }
        let extra = args.count();
        for (n, v) in named {
            match def.params.iter().position(|p| p.name == n && !p.is_varargs) {
                Some(i) if slots[i].is_none() => slots[i] = Some(v),
                Some(_) => return Err(rt(format!("'{n}' ki value do baar mili (position se aur naam se)."))),
                None => {
                    let hint = suggestion_suffix(closest(&n, def.params.iter().map(|p| &*p.name)));
                    return Err(rt(format!("'{}' mein '{n}' naam ka parameter nahi hai.{hint}", def.display_name())));
                }
            }
        }
        let sig = Signature::from_params(&def.params);
        let given = def.params.iter().zip(slots.iter()).filter(|(p, s)| !p.is_varargs && s.is_some()).count();
        let missing =
            def.params.iter().zip(slots.iter()).find(|(p, s)| !p.is_varargs && s.is_none() && p.default.is_none());
        if extra > 0 || missing.is_some() {
            if let Some((p, _)) = missing {
                if extra == 0 && given >= sig.min {
                    return Err(rt(format!("'{}' ko '{}' ki value nahi mili.", def.display_name(), p.name)));
                }
            }
            return Err(rt(sig.arity_error(&def.display_name(), given + extra + varargs.len())).with_code("E002"));
        }
        if self.depth >= self.max_depth {
            return Err(rt(format!(
                "Recursion bahut gehri ho gayi ({} calls). Shayad '{}' khud ko bina ruke call kar raha hai?",
                self.max_depth,
                def.display_name()
            ))
            .with_code("E010"));
        }
        let call_scope = new_scope(Some(&c.scope));
        if let Some(r) = receiver {
            define(&call_scope, &crate::ast::intern("yeh"), r);
        }
        for (p, slot) in def.params.iter().zip(slots) {
            let v = if p.is_varargs {
                Value::list(std::mem::take(&mut varargs))
            } else {
                match slot {
                    Some(v) => v,
                    None => self.eval(p.default.as_ref().expect("checked above"), &c.scope)?,
                }
            };
            if let Some(t) = &p.ty {
                self.check_value_type(&v, t, &format!("'{}' ka parameter '{}'", def.display_name(), p.name))?;
            }
            define(&call_scope, &p.name, v);
        }
        if def.generator {
            self.yields.push(Vec::new());
        }
        self.depth += 1;
        let r = self.exec_block(&def.body, &call_scope);
        self.depth -= 1;
        let produced = if def.generator { self.yields.pop() } else { None };
        let value = match r? {
            Flow::Return(v) => v,
            // a function without `wapas` returns khali
            _ => Value::Null,
        };
        let value = match produced {
            Some(items) => Value::list(items),
            None => value,
        };
        if let Some(t) = &def.ret {
            self.check_value_type(&value, t, &format!("'{}' ka result", def.display_name()))?;
        }
        Ok(value)
    }
}

/// `x in coll`
pub fn contains(coll: &Value, x: &Value) -> Result<bool, AdarshError> {
    Ok(match coll {
        Value::List(l) => {
            let items = l.borrow().clone();
            items.iter().any(|i| values_eq(i, x))
        }
        Value::Tuple(t) => t.iter().any(|i| values_eq(i, x)),
        Value::Str(s) => match x {
            Value::Str(n) => s.contains(&**n),
            other => return Err(rt(format!("String mein sirf string dhoondh sakte ho, {} nahi.", other.type_desc()))),
        },
        Value::Dict(d) => d.borrow().contains(x)?,
        Value::Set(s) => s.borrow().contains(x)?,
        other => return Err(rt(format!("'in' {} par nahi chalta.", other.type_desc()))),
    })
}

/// Variables visible from `scope` (innermost first), as text, for step-through views.
/// Builtins are left out.
pub fn visible_vars(scope: &ScopeRef) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = Vec::new();
    let mut cur = Some(scope.clone());
    while let Some(s) = cur {
        let b = s.borrow();
        let mut here: Vec<(String, String)> = b
            .vars
            .iter()
            .filter(|(k, v)| {
                !matches!(v, Value::Builtin(_))
                    && !(b.parent.is_none() && crate::builtins::GLOBAL_VALUES.contains(&&***k))
                    && !out.iter().any(|(n, _)| n == &***k)
            })
            .map(|(k, v)| {
                let mut text = repr(v);
                if text.chars().count() > 80 {
                    text = text.chars().take(77).collect::<String>() + "...";
                }
                (k.to_string(), text)
            })
            .collect();
        here.sort();
        out.extend(here);
        if out.len() > 40 {
            out.truncate(40);
            break;
        }
        cur = b.parent.clone();
    }
    out
}
