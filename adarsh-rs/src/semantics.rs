//! Static semantic analysis (port of `adarsh_lang/semantics.py`).

use crate::ast::*;
use crate::error::AdarshError;
use crate::loader::ModuleLoader;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

/// Arity information derived from a parameter list, same rules as Python's
/// `AdarshFunctionSignature`.
#[derive(Clone, Copy, Debug)]
pub struct Signature {
    pub min: usize,
    pub max: Option<usize>,
}

impl Signature {
    pub fn from_params(params: &[Param]) -> Self {
        let (mut min, mut max) = (0usize, Some(0usize));
        let mut defaults_started = false;
        for p in params {
            if p.is_varargs {
                max = None;
                break;
            }
            max = max.map(|m| m + 1);
            if p.default.is_none() && !defaults_started {
                min += 1;
            } else {
                defaults_started = true;
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

    pub fn max_text(&self) -> String {
        self.max.map(|m| m.to_string()).unwrap_or_else(|| "infinite".into())
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

fn undeclared(name: &str) -> AdarshError {
    AdarshError::semantic(format!("Variable '{name}' not declared in AdarshLang scope."))
}

impl SemScope {
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
}

fn declare_var(s: &SemScopeRef, name: &Rc<str>) {
    s.borrow_mut().vars.insert(name.clone());
}

fn check_var(s: &SemScopeRef, name: &str) -> Result<(), AdarshError> {
    SemScope::lookup(s, &|sc| sc.vars.contains(name).then_some(())).ok_or_else(|| undeclared(name))
}

fn has_type(s: &SemScopeRef, name: &str) -> Option<Rc<Vec<Rc<str>>>> {
    SemScope::lookup(s, &|sc| sc.types.get(name).cloned())
}

fn function_sig(s: &SemScopeRef, name: &str) -> Option<Signature> {
    SemScope::lookup(s, &|sc| sc.functions.get(name).copied())
}

pub struct Analyzer {
    imported: HashSet<String>,
    loader: Rc<dyn ModuleLoader>,
}

/// A fresh root scope with every builtin declared as a function and a variable.
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

impl Analyzer {
    pub fn new(loader: Rc<dyn ModuleLoader>) -> Self {
        Analyzer { imported: HashSet::new(), loader }
    }

    pub fn analyze_program(&mut self, program: &[Node], scope: &SemScopeRef) -> SResult {
        for stmt in program {
            self.node(stmt, scope, 0)?;
        }
        Ok(())
    }

    fn block(&mut self, nodes: &[Node], scope: &SemScopeRef, depth: usize) -> SResult {
        for n in nodes {
            self.node(n, scope, depth)?;
        }
        Ok(())
    }

    fn function(&mut self, def: &FuncDef, scope: &SemScopeRef) -> SResult {
        let fs = child(scope);
        for p in &def.params {
            declare_var(&fs, &p.name);
        }
        self.block(&def.body, &fs, 0)
    }

    fn node(&mut self, node: &Node, scope: &SemScopeRef, depth: usize) -> SResult {
        match node {
            Node::Expr(e) => self.expr(e, scope, depth),
            Node::Badlo(name, init) => {
                declare_var(scope, name);
                if let Some(e) = init {
                    self.expr(e, scope, depth)?;
                }
                Ok(())
            }
            Node::Dikhao(e) | Node::Throw(e) => self.expr(e, scope, depth),
            Node::Break => {
                if depth == 0 {
                    return Err(AdarshError::semantic("'bas' (break) can only be used inside loops in AdarshLang."));
                }
                Ok(())
            }
            Node::Continue => {
                if depth == 0 {
                    return Err(AdarshError::semantic(
                        "'aage_badho' (continue) can only be used inside loops in AdarshLang.",
                    ));
                }
                Ok(())
            }
            Node::Wapas(e) => {
                if let Some(e) = e {
                    self.expr(e, scope, depth)?;
                }
                Ok(())
            }
            Node::Block(b) => self.block(b, scope, depth),
            Node::If { cond, then, otherwise } => {
                self.expr(cond, scope, depth)?;
                self.block(then, &child(scope), depth)?;
                if let Some(o) = otherwise {
                    self.node(o, &child(scope), depth)?;
                }
                Ok(())
            }
            Node::While { cond, body } => {
                self.expr(cond, scope, depth)?;
                self.block(body, &child(scope), depth + 1)
            }
            Node::For { init, cond, update, body } => {
                let ls = child(scope);
                if let Some(i) = init {
                    self.node(i, &ls, depth + 1)?;
                }
                if let Some(c) = cond {
                    self.expr(c, &ls, depth + 1)?;
                }
                if let Some(u) = update {
                    self.expr(u, &ls, depth + 1)?;
                }
                self.block(body, &ls, depth + 1)
            }
            Node::ForEach { var, declare, iterable, body } => {
                let ls = child(scope);
                if *declare {
                    declare_var(&ls, var);
                } else {
                    check_var(scope, var)?;
                }
                self.expr(iterable, scope, depth)?;
                self.block(body, &ls, depth + 1)
            }
            Node::Switch { subject, cases, default } => {
                self.expr(subject, scope, depth)?;
                for c in cases {
                    let cs = child(scope);
                    for m in &c.matches {
                        self.expr(m, scope, depth)?;
                    }
                    self.block(&c.body, &cs, depth)?;
                }
                if let Some(d) = default {
                    self.block(d, &child(scope), depth)?;
                }
                Ok(())
            }
            Node::TryCatch { body, err_name, handler } => {
                self.block(body, &child(scope), depth)?;
                let cs = child(scope);
                declare_var(&cs, err_name);
                self.block(handler, &cs, depth)
            }
            Node::Import(e) => {
                self.expr(e, scope, depth)?;
                if let Expr::Str(name) = e {
                    if !name.is_empty() {
                        let path = crate::loader::module_path(name);
                        if !self.imported.contains(&path) {
                            if let Some(src) = self.loader.load(&path) {
                                self.imported.insert(path);
                                let module = crate::parser::parse(&src)?;
                                for stmt in &module {
                                    self.node(stmt, scope, depth)?;
                                }
                            }
                        }
                    }
                }
                Ok(())
            }
            Node::KaamDef(def) => {
                let name = def.name.clone().expect("named function");
                {
                    let mut s = scope.borrow_mut();
                    s.functions.insert(name.clone(), Signature::from_params(&def.params));
                    s.vars.insert(name);
                }
                self.function(def, scope)
            }
            Node::DhachaDef(name, fields) => {
                scope.borrow_mut().types.insert(name.clone(), fields.clone());
                Ok(())
            }
        }
    }

    #[allow(clippy::only_used_in_recursion)] // mirrors the Python analyzer, which threads loop depth through expressions
    fn expr(&mut self, e: &Expr, scope: &SemScopeRef, depth: usize) -> SResult {
        match e {
            Expr::Int(_) | Expr::BigInt(_) | Expr::Float(_) | Expr::Str(_) | Expr::Bool(_) | Expr::Null => Ok(()),
            Expr::Var(name) => check_var(scope, name),
            Expr::Assign(name, rhs) | Expr::CompoundAssign(name, _, rhs) => {
                check_var(scope, name)?;
                self.expr(rhs, scope, depth)
            }
            Expr::IndexAssign(c, i, v) | Expr::IndexCompoundAssign(c, i, _, v) => {
                self.expr(c, scope, depth)?;
                self.expr(i, scope, depth)?;
                self.expr(v, scope, depth)
            }
            Expr::AttrAssign(t, _, v) | Expr::AttrCompoundAssign(t, _, _, v) => {
                self.expr(t, scope, depth)?;
                self.expr(v, scope, depth)
            }
            Expr::Binary(l, _, r) => {
                self.expr(l, scope, depth)?;
                self.expr(r, scope, depth)
            }
            Expr::Unary(_, x) => self.expr(x, scope, depth),
            Expr::List(items) => {
                for i in items {
                    self.expr(i, scope, depth)?;
                }
                Ok(())
            }
            Expr::Dict(entries) => {
                for (k, v) in entries {
                    self.expr(k, scope, depth)?;
                    self.expr(v, scope, depth)?;
                }
                Ok(())
            }
            Expr::Dhacha { type_name, fields } => {
                let declared = has_type(scope, type_name).ok_or_else(|| {
                    AdarshError::semantic(format!("Dhacha '{type_name}' not declared in AdarshLang."))
                })?;
                let mut seen: HashSet<&str> = HashSet::new();
                for (fname, fe) in fields {
                    if !declared.iter().any(|d| d == fname) {
                        return Err(AdarshError::semantic(format!(
                            "Field '{fname}' not declared in dhacha '{type_name}' in AdarshLang."
                        )));
                    }
                    if !seen.insert(fname) {
                        return Err(AdarshError::semantic(format!(
                            "Field '{fname}' provided multiple times for dhacha '{type_name}' in AdarshLang."
                        )));
                    }
                    self.expr(fe, scope, depth)?;
                }
                Ok(())
            }
            Expr::Attr(t, _) => self.expr(t, scope, depth),
            Expr::Index(c, i) => {
                self.expr(c, scope, depth)?;
                self.expr(i, scope, depth)
            }
            Expr::Func(def) => self.function(def, scope),
            Expr::Call(callee, args) => {
                self.expr(callee, scope, depth)?;
                for a in args {
                    self.expr(a, scope, depth)?;
                }
                if let Expr::Var(name) = &**callee {
                    if let Some(sig) = function_sig(scope, name) {
                        if !sig.allows(args.len()) {
                            return Err(AdarshError::semantic(format!(
                                "Function '{name}' called with invalid argument count {} in AdarshLang.",
                                args.len()
                            )));
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
