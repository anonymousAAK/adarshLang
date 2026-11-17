//! Static checks before a program runs: every name is declared, named
//! functions get the right number of arguments, `bas` / `aage_badho` only
//! inside loops, and `dhacha` constructions use declared fields.
//!
//! Function bodies are checked at the end of the block that defines them, so a
//! function may use variables and functions declared later in that block.

use crate::ast::*;
use crate::error::{AdarshError, Pos};
use crate::loader::ModuleLoader;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Arity of a parameter list.
#[derive(Clone, Copy, Debug)]
pub struct Signature {
    pub min: usize,
    pub max: Option<usize>,
}

impl Signature {
    pub fn from_params(params: &[Param]) -> Self {
        let mut min = 0;
        let mut max = Some(0usize);
        for p in params {
            if p.is_varargs {
                max = None;
                break;
            }
            max = max.map(|m| m + 1);
            if p.default.is_none() {
                min += 1;
            }
        }
        Signature { min, max }
    }

    pub fn from_counts(required: usize, total: usize) -> Self {
        Signature { min: required, max: Some(total) }
    }

    pub fn allows(&self, n: usize) -> bool {
        n >= self.min && self.max.is_none_or(|m| n <= m)
    }

    /// "2", "1 se 3", "kam se kam 1".
    pub fn describe(&self) -> String {
        match self.max {
            Some(m) if m == self.min => format!("{m}"),
            Some(m) => format!("{} se {m}", self.min),
            None => format!("kam se kam {}", self.min),
        }
    }

    pub fn arity_error(&self, name: &str, given: usize) -> String {
        let plural = |n: usize| if n == 1 { "argument" } else { "arguments" };
        format!(
            "'{name}' ko {given} {} diye, par yeh {} {} leta hai.",
            plural(given),
            self.describe(),
            plural(self.max.unwrap_or(2))
        )
    }
}

#[derive(Default)]
pub struct SemScope {
    vars: HashSet<Rc<str>>,
    functions: HashMap<Rc<str>, Signature>,
    types: HashMap<Rc<str>, Rc<Vec<Rc<str>>>>,
    parent: Option<SemScopeRef>,
}

pub type SemScopeRef = Rc<RefCell<SemScope>>;

fn child(parent: &SemScopeRef) -> SemScopeRef {
    Rc::new(RefCell::new(SemScope { parent: Some(parent.clone()), ..Default::default() }))
}

fn lookup<T>(s: &SemScopeRef, f: &dyn Fn(&SemScope) -> Option<T>) -> Option<T> {
    let mut cur = Some(s.clone());
    while let Some(sc) = cur {
        let b = sc.borrow();
        if let Some(v) = f(&b) {
            return Some(v);
        }
        cur = b.parent.clone();
    }
    None
}

pub fn undeclared_message(name: &str) -> String {
    format!("'{name}' naam ka variable nahi mila. Pehle 'badlo {name} = ...;' se banao.")
}

fn declare(s: &SemScopeRef, name: &Rc<str>) {
    s.borrow_mut().vars.insert(name.clone());
}

fn check_var(s: &SemScopeRef, name: &str, pos: Pos) -> Result<(), AdarshError> {
    lookup(s, &|sc| sc.vars.contains(name).then_some(()))
        .ok_or_else(|| AdarshError::semantic(undeclared_message(name)).at(pos))
}

fn find_type(s: &SemScopeRef, name: &str) -> Option<Rc<Vec<Rc<str>>>> {
    lookup(s, &|sc| sc.types.get(name).cloned())
}

/// A fresh root scope with every builtin declared.
pub fn root_scope() -> SemScopeRef {
    let scope = Rc::new(RefCell::new(SemScope::default()));
    {
        let mut s = scope.borrow_mut();
        for (name, sig) in crate::builtins::signatures() {
            let n: Rc<str> = Rc::from(name);
            s.functions.insert(n.clone(), sig);
            s.vars.insert(n);
        }
    }
    scope
}

type SResult = Result<(), AdarshError>;

pub struct Analyzer {
    imported: HashSet<String>,
    loader: Rc<dyn ModuleLoader>,
    /// Function bodies waiting for the end of their enclosing block.
    deferred: Vec<Vec<(Rc<FuncDef>, SemScopeRef)>>,
}

impl Analyzer {
    pub fn new(loader: Rc<dyn ModuleLoader>) -> Self {
        Analyzer { imported: HashSet::new(), loader, deferred: Vec::new() }
    }

    pub fn analyze_program(&mut self, program: &[Node], scope: &SemScopeRef) -> SResult {
        self.block(program, scope, 0)
    }

    fn block(&mut self, nodes: &[Node], scope: &SemScopeRef, depth: usize) -> SResult {
        self.deferred.push(Vec::new());
        let mut result = nodes.iter().try_for_each(|n| self.node(n, scope, depth));
        let pending = self.deferred.pop().unwrap_or_default();
        if result.is_ok() {
            result = pending.into_iter().try_for_each(|(def, sc)| self.function_body(&def, &sc));
        }
        result
    }

    fn defer(&mut self, def: &Rc<FuncDef>, scope: &SemScopeRef) {
        match self.deferred.last_mut() {
            Some(frame) => frame.push((def.clone(), scope.clone())),
            None => self.deferred.push(vec![(def.clone(), scope.clone())]),
        }
    }

    fn function_body(&mut self, def: &FuncDef, scope: &SemScopeRef) -> SResult {
        let fs = child(scope);
        if def.method_of.is_some() {
            declare(&fs, &Rc::from("yeh"));
        }
        for p in &def.params {
            if let Some(d) = &p.default {
                self.expr(d, scope, 0)?;
            }
            declare(&fs, &p.name);
        }
        self.block(&def.body, &fs, 0)
    }

    fn node(&mut self, node: &Node, scope: &SemScopeRef, depth: usize) -> SResult {
        let pos = node.pos;
        match &node.kind {
            NodeKind::Expr(e) => self.expr(e, scope, depth),
            NodeKind::Badlo(name, init) => {
                // the initialiser may not refer to the variable being declared
                if let Some(e) = init {
                    let shadowed = scope.borrow().vars.contains(name);
                    if !shadowed {
                        self.expr(e, scope, depth).map_err(|err| {
                            if err.message == undeclared_message(name) {
                                AdarshError::semantic(format!(
                                    "'{name}' ko usi ki line mein use kiya, jab woh abhi bana bhi nahi."
                                ))
                                .at(err.pos.unwrap_or(pos))
                            } else {
                                err
                            }
                        })?;
                    } else {
                        self.expr(e, scope, depth)?;
                    }
                }
                declare(scope, name);
                Ok(())
            }
            NodeKind::Dikhao(e) | NodeKind::Throw(e) => self.expr(e, scope, depth),
            NodeKind::Break | NodeKind::Continue if depth == 0 => {
                let word = if matches!(node.kind, NodeKind::Break) { "bas" } else { "aage_badho" };
                Err(AdarshError::semantic(format!(
                    "'{word}' sirf loop (jabtak / ginnati / ke_liye) ke andar chal sakta hai."
                ))
                .at(pos))
            }
            NodeKind::Break | NodeKind::Continue => Ok(()),
            NodeKind::Wapas(e) => e.as_ref().map_or(Ok(()), |e| self.expr(e, scope, depth)),
            NodeKind::Block(b) => self.block(b, scope, depth),
            NodeKind::If { cond, then, otherwise } => {
                self.expr(cond, scope, depth)?;
                self.block(then, &child(scope), depth)?;
                if let Some(o) = otherwise {
                    let cs = child(scope);
                    match &o.kind {
                        NodeKind::Block(b) => self.block(b, &cs, depth)?,
                        _ => self.node(o, &cs, depth)?,
                    }
                }
                Ok(())
            }
            NodeKind::While { cond, body } => {
                self.expr(cond, scope, depth)?;
                self.block(body, &child(scope), depth + 1)
            }
            NodeKind::For { init, cond, update, body } => {
                let ls = child(scope);
                if let Some(i) = init {
                    self.node(i, &ls, depth)?;
                }
                if let Some(c) = cond {
                    self.expr(c, &ls, depth)?;
                }
                if let Some(u) = update {
                    self.expr(u, &ls, depth)?;
                }
                self.block(body, &child(&ls), depth + 1)
            }
            NodeKind::ForEach { var, declare: decl, iterable, body } => {
                self.expr(iterable, scope, depth)?;
                let ls = child(scope);
                if *decl {
                    declare(&ls, var);
                } else {
                    check_var(scope, var, pos)?;
                }
                self.block(body, &child(&ls), depth + 1)
            }
            NodeKind::Switch { subject, cases, default } => {
                self.expr(subject, scope, depth)?;
                for c in cases {
                    for m in &c.matches {
                        self.expr(m, scope, depth)?;
                    }
                    self.block(&c.body, &child(scope), depth)?;
                }
                if let Some(d) = default {
                    self.block(d, &child(scope), depth)?;
                }
                Ok(())
            }
            NodeKind::TryCatch { body, err_name, handler } => {
                self.block(body, &child(scope), depth)?;
                let cs = child(scope);
                declare(&cs, err_name);
                self.block(handler, &cs, depth)
            }
            NodeKind::Import(e) => {
                self.expr(e, scope, depth)?;
                if let ExprKind::Str(name) = &e.kind {
                    let path = crate::loader::module_path(name);
                    if !self.imported.contains(&path) {
                        if let Some(src) = self.loader.load(&path) {
                            self.imported.insert(path.clone());
                            let file = crate::error::register_source(&path, &src);
                            let module = crate::parser::parse(&src, file)?;
                            // module definitions become visible to the importing scope
                            for stmt in &module {
                                self.node(stmt, scope, depth)?;
                            }
                        }
                    }
                }
                Ok(())
            }
            NodeKind::KaamDef(def) => {
                let name = def.name.clone().expect("named function");
                match &def.method_of {
                    Some(t) => {
                        if find_type(scope, t).is_none() {
                            return Err(AdarshError::semantic(format!(
                                "'{t}' naam ka dhacha nahi mila, isliye '{t}.{name}' method nahi bana sakte."
                            ))
                            .at(pos));
                        }
                    }
                    None => {
                        let mut s = scope.borrow_mut();
                        s.functions.insert(name.clone(), Signature::from_params(&def.params));
                        s.vars.insert(name);
                    }
                }
                self.defer(def, scope);
                Ok(())
            }
            NodeKind::DhachaDef(name, fields) => {
                scope.borrow_mut().types.insert(name.clone(), fields.clone());
                Ok(())
            }
        }
    }

    // `depth` is threaded through so loop context is preserved uniformly
    #[allow(clippy::only_used_in_recursion)]
    fn expr(&mut self, e: &Expr, scope: &SemScopeRef, depth: usize) -> SResult {
        let pos = e.pos;
        match &e.kind {
            ExprKind::Int(_)
            | ExprKind::BigInt(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null => Ok(()),
            ExprKind::Interp(parts) => parts.iter().try_for_each(|p| match p {
                Part::Expr(x) => self.expr(x, scope, depth),
                Part::Lit(_) => Ok(()),
            }),
            ExprKind::Var(name) => check_var(scope, name, pos),
            ExprKind::Assign(name, rhs) | ExprKind::CompoundAssign(name, _, rhs) => {
                check_var(scope, name, pos).map_err(|_| {
                    AdarshError::semantic(format!(
                        "'{name}' naam ka variable nahi mila. Naya variable banane ke liye 'badlo {name} = ...;' likho."
                    ))
                    .at(pos)
                })?;
                self.expr(rhs, scope, depth)
            }
            ExprKind::IndexAssign(c, i, v) | ExprKind::IndexCompoundAssign(c, i, _, v) => {
                self.expr(c, scope, depth)?;
                self.expr(i, scope, depth)?;
                self.expr(v, scope, depth)
            }
            ExprKind::AttrAssign(t, _, v) | ExprKind::AttrCompoundAssign(t, _, _, v) => {
                self.expr(t, scope, depth)?;
                self.expr(v, scope, depth)
            }
            ExprKind::Binary(l, _, r) => {
                self.expr(l, scope, depth)?;
                self.expr(r, scope, depth)
            }
            ExprKind::Unary(_, x) => self.expr(x, scope, depth),
            ExprKind::List(items) => items.iter().try_for_each(|i| self.expr(i, scope, depth)),
            ExprKind::Dict(entries) => entries.iter().try_for_each(|(k, v)| {
                self.expr(k, scope, depth)?;
                self.expr(v, scope, depth)
            }),
            ExprKind::Dhacha { type_name, fields } => {
                let declared = find_type(scope, type_name).ok_or_else(|| {
                    AdarshError::semantic(format!(
                        "'{type_name}' naam ka dhacha nahi mila. Pehle 'dhacha {type_name} {{ ... }};' likho."
                    ))
                    .at(pos)
                })?;
                let mut seen: HashSet<&str> = HashSet::new();
                for (fname, fe) in fields {
                    if !declared.iter().any(|d| d == fname) {
                        let list: Vec<&str> = declared.iter().map(|s| &**s).collect();
                        return Err(AdarshError::semantic(format!(
                            "Dhacha '{type_name}' mein '{fname}' field nahi hai. Fields: {}.",
                            list.join(", ")
                        ))
                        .at(fe.pos));
                    }
                    if !seen.insert(fname) {
                        return Err(AdarshError::semantic(format!("Field '{fname}' do baar diya gaya.")).at(fe.pos));
                    }
                    self.expr(fe, scope, depth)?;
                }
                Ok(())
            }
            ExprKind::Attr(t, _) => self.expr(t, scope, depth),
            ExprKind::Index(c, i) => {
                self.expr(c, scope, depth)?;
                self.expr(i, scope, depth)
            }
            ExprKind::Func(def) => {
                self.defer(def, scope);
                Ok(())
            }
            ExprKind::Call(callee, args) => {
                self.expr(callee, scope, depth)?;
                for a in args {
                    self.expr(a, scope, depth)?;
                }
                if let ExprKind::Var(name) = &callee.kind {
                    let sig = lookup(scope, &|sc| {
                        // a plain variable of the same name in a nearer scope shadows the function
                        if sc.functions.contains_key(&**name) {
                            Some(sc.functions.get(&**name).copied())
                        } else if sc.vars.contains(&**name) {
                            Some(None)
                        } else {
                            None
                        }
                    })
                    .flatten();
                    if let Some(sig) = sig {
                        if !sig.allows(args.len()) {
                            return Err(AdarshError::semantic(sig.arity_error(name, args.len())).at(pos));
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
