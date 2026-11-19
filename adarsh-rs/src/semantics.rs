//! Static checks before a program runs: every name is declared, named
//! functions get the right number of arguments, `bas` / `aage_badho` only
//! inside loops (and to a real label), `pakka` values are never reassigned,
//! records use declared fields, type names exist.
//!
//! It also collects warnings (unused variables, unreachable code, hidden
//! builtins) and suggests the closest name for typos ("Kya matlab 'dikhao' tha?").
//!
//! Function bodies are checked at the end of the block that defines them, so a
//! function may use variables and functions declared later in that block.

use crate::ast::*;
use crate::error::{AdarshError, ErrorKind, Pos};
use crate::loader::ModuleLoader;
use std::cell::{Cell, RefCell};
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

/// Names of the built-in types usable in annotations, `hai` and typed `chhoddo`.
pub const BUILTIN_TYPES: &[&str] = &[
    "number", "string", "boolean", "list", "dict", "tuple", "set", "kaam", "khali", "koi", "dhacha", "vikalp", "Galti",
];

/// Every keyword, for "did you mean" suggestions.
pub const KEYWORDS: &[&str] = &[
    "badlo",
    "pakka",
    "kaam",
    "wapas",
    "agar",
    "warna",
    "jabtak",
    "karo",
    "dikhao",
    "bas",
    "aage_badho",
    "sahi_hai_be",
    "jhuth",
    "ginnati",
    "ke_liye",
    "in",
    "chuno",
    "case",
    "warna_case",
    "pakdo",
    "chhoddo",
    "aakhir",
    "lao",
    "dhacha",
    "vikalp",
    "baaki",
    "khali",
    "aur",
    "ya",
    "nahin",
    "hai",
    "dete_raho",
];

/// Levenshtein distance (small strings only).
fn distance(a: &str, b: &str) -> usize {
    let (a, b): (Vec<char>, Vec<char>) = (a.chars().collect(), b.chars().collect());
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.iter().enumerate() {
        let mut cur = vec![i + 1];
        for (j, cb) in b.iter().enumerate() {
            let cost = if ca == cb { 0 } else { 1 };
            cur.push((prev[j] + cost).min(prev[j + 1] + 1).min(cur[j] + 1));
        }
        prev = cur;
    }
    prev[b.len()]
}

/// The closest candidate to `name`, if it is a plausible typo.
pub fn closest<'a>(name: &str, candidates: impl Iterator<Item = &'a str>) -> Option<String> {
    let limit = if name.chars().count() <= 4 { 1 } else { 2 };
    candidates
        .filter(|c| *c != name)
        .map(|c| (distance(&name.to_lowercase(), &c.to_lowercase()), c))
        .filter(|(d, _)| *d <= limit)
        // ties go to the shorter, then alphabetically first name, so every run picks the same guess
        .min_by_key(|(d, c)| (*d, c.len(), *c))
        .map(|(_, c)| c.to_string())
}

pub fn suggestion_suffix(s: Option<String>) -> String {
    s.map(|s| format!(" Kya aapka matlab '{s}' tha?")).unwrap_or_default()
}

pub fn undeclared_message(name: &str) -> String {
    format!("'{name}' naam ka variable nahi mila. Pehle 'badlo {name} = ...;' se banao.")
}

#[derive(Debug)]
struct VarInfo {
    pos: Pos,
    used: Cell<bool>,
    constant: bool,
    /// only real `badlo` / `pakka` variables produce "unused" warnings
    warn_unused: bool,
}

#[derive(Default)]
pub struct SemScope {
    vars: HashMap<Rc<str>, VarInfo>,
    functions: HashMap<Rc<str>, Signature>,
    types: HashMap<Rc<str>, Rc<DhachaDef>>,
    enums: HashSet<Rc<str>>,
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

fn find_type(s: &SemScopeRef, name: &str) -> Option<Rc<DhachaDef>> {
    lookup(s, &|sc| sc.types.get(name).cloned())
}

fn is_enum(s: &SemScopeRef, name: &str) -> bool {
    lookup(s, &|sc| sc.enums.contains(name).then_some(())).is_some()
}

/// All field names of a record type, parents first.
fn all_fields(s: &SemScopeRef, def: &DhachaDef) -> Vec<Rc<str>> {
    let mut chain: Vec<Vec<Rc<str>>> = vec![def.fields.iter().map(|f| f.name.clone()).collect()];
    let mut seen: HashSet<Rc<str>> = HashSet::from([def.name.clone()]);
    let mut parent = def.parent.clone();
    while let Some(p) = parent {
        if !seen.insert(p.clone()) {
            break;
        }
        let Some(d) = find_type(s, &p) else { break };
        chain.push(d.fields.iter().map(|f| f.name.clone()).collect());
        parent = d.parent.clone();
    }
    let mut out: Vec<Rc<str>> = Vec::new();
    for names in chain.into_iter().rev() {
        for n in names {
            if !out.contains(&n) {
                out.push(n);
            }
        }
    }
    out
}

/// A fresh root scope with every builtin declared.
pub fn root_scope() -> SemScopeRef {
    let scope = Rc::new(RefCell::new(SemScope::default()));
    {
        let mut s = scope.borrow_mut();
        for (name, sig) in crate::builtins::signatures() {
            let n: Rc<str> = Rc::from(name);
            s.functions.insert(n.clone(), sig);
            s.vars
                .insert(n, VarInfo { pos: Pos::default(), used: Cell::new(true), constant: false, warn_unused: false });
        }
        for name in crate::builtins::GLOBAL_VALUES {
            s.vars.insert(
                Rc::from(*name),
                VarInfo { pos: Pos::default(), used: Cell::new(true), constant: true, warn_unused: false },
            );
        }
    }
    scope
}

fn is_builtin_name(name: &str) -> bool {
    crate::builtins::BUILTINS.iter().any(|b| b.name == name)
}

type SResult = Result<(), AdarshError>;

pub struct Analyzer {
    imported: HashSet<String>,
    loader: Rc<dyn ModuleLoader>,
    /// Function bodies waiting for the end of their enclosing block.
    deferred: Vec<Vec<(Rc<FuncDef>, SemScopeRef)>>,
    /// Loop labels visible at the current point (reset inside functions).
    labels: Vec<Vec<Option<Rc<str>>>>,
    pub warnings: Vec<AdarshError>,
}

impl Analyzer {
    pub fn new(loader: Rc<dyn ModuleLoader>) -> Self {
        Analyzer {
            imported: HashSet::new(),
            loader,
            deferred: Vec::new(),
            labels: vec![Vec::new()],
            warnings: Vec::new(),
        }
    }

    pub fn analyze_program(&mut self, program: &[Node], scope: &SemScopeRef) -> SResult {
        self.block(program, scope)?;
        self.unused_warnings(scope);
        Ok(())
    }

    fn warn(&mut self, msg: String, pos: Pos) {
        let mut w = AdarshError::new(ErrorKind::Warning, msg).at(pos);
        w.code = Some("W001");
        self.warnings.push(w);
    }

    fn unused_warnings(&mut self, scope: &SemScopeRef) {
        let mut found: Vec<(Pos, String)> = scope
            .borrow()
            .vars
            .iter()
            .filter(|(n, v)| v.warn_unused && !v.used.get() && !n.starts_with('_'))
            .map(|(n, v)| (v.pos, n.to_string()))
            .collect();
        found.sort_by_key(|(p, _)| (p.line, p.col));
        for (pos, n) in found {
            self.warn(format!("Variable '{n}' banaya par kabhi use nahi kiya."), pos);
        }
    }

    fn declare(&mut self, s: &SemScopeRef, name: &Rc<str>, pos: Pos, constant: bool, warn_unused: bool) {
        if warn_unused && is_builtin_name(name) {
            self.warn(format!("'{name}' builtin function ka naam hai; yeh variable usse chhupa dega."), pos);
        }
        s.borrow_mut().vars.insert(name.clone(), VarInfo { pos, used: Cell::new(false), constant, warn_unused });
    }

    fn visible_names(&self, s: &SemScopeRef) -> Vec<String> {
        let mut names = Vec::new();
        let mut cur = Some(s.clone());
        while let Some(sc) = cur {
            let b = sc.borrow();
            names.extend(b.vars.keys().map(|k| k.to_string()));
            cur = b.parent.clone();
        }
        names.extend(KEYWORDS.iter().map(|k| k.to_string()));
        names
    }

    fn undeclared(&self, s: &SemScopeRef, name: &str, pos: Pos) -> AdarshError {
        let names = self.visible_names(s);
        let msg = match closest(name, names.iter().map(String::as_str)) {
            Some(guess) => format!("'{name}' naam ka variable nahi mila. Kya aapka matlab '{guess}' tha?"),
            None => undeclared_message(name),
        };
        AdarshError::semantic(msg).at(pos).with_code("E001")
    }

    /// Mark a variable as used; error if it does not exist.
    fn use_var(&self, s: &SemScopeRef, name: &str, pos: Pos) -> SResult {
        let found = lookup(s, &|sc| {
            sc.vars.get(name).map(|v| {
                v.used.set(true);
            })
        });
        found.ok_or_else(|| self.undeclared(s, name, pos))
    }

    fn check_assignable(&self, s: &SemScopeRef, name: &str, pos: Pos) -> SResult {
        let info = lookup(s, &|sc| sc.vars.get(name).map(|v| v.constant));
        match info {
            None => {
                let names = self.visible_names(s);
                let hint = suggestion_suffix(closest(name, names.iter().map(String::as_str)));
                Err(AdarshError::semantic(format!(
                    "'{name}' naam ka variable nahi mila. Naya variable banane ke liye 'badlo {name} = ...;' likho.{hint}"
                ))
                .at(pos)
                .with_code("E001"))
            }
            Some(true) => Err(AdarshError::semantic(format!(
                "'{name}' pakka (constant) hai, isliye uski value badal nahi sakte. Badalni ho to 'badlo' se banao."
            ))
            .at(pos)
            .with_code("E003")),
            Some(false) => Ok(()),
        }
    }

    fn check_type(&self, s: &SemScopeRef, ty: &TypeAnn) -> SResult {
        let n = &*ty.name;
        if BUILTIN_TYPES.contains(&n) || find_type(s, n).is_some() || is_enum(s, n) {
            return Ok(());
        }
        let mut cands: Vec<String> = BUILTIN_TYPES.iter().map(|t| t.to_string()).collect();
        let mut cur = Some(s.clone());
        while let Some(sc) = cur {
            let b = sc.borrow();
            cands.extend(b.types.keys().map(|k| k.to_string()));
            cur = b.parent.clone();
        }
        let hint = suggestion_suffix(closest(n, cands.iter().map(String::as_str)));
        Err(AdarshError::semantic(format!("'{n}' naam ka koi type nahi hai.{hint}")).at(ty.pos).with_code("E004"))
    }

    fn block(&mut self, nodes: &[Node], scope: &SemScopeRef) -> SResult {
        self.deferred.push(Vec::new());
        let mut result = Ok(());
        for (i, n) in nodes.iter().enumerate() {
            result = self.node(n, scope);
            if result.is_err() {
                break;
            }
            let ends =
                matches!(n.kind, NodeKind::Wapas(_) | NodeKind::Break(_) | NodeKind::Continue(_) | NodeKind::Throw(_));
            if ends && i + 1 < nodes.len() {
                let what = match n.kind {
                    NodeKind::Wapas(_) => "wapas",
                    NodeKind::Break(_) => "bas",
                    NodeKind::Continue(_) => "aage_badho",
                    _ => "chhoddo",
                };
                self.warn(format!("Yeh code kabhi nahi chalega, kyunki upar '{what}' hai."), nodes[i + 1].pos);
            }
        }
        let pending = self.deferred.pop().unwrap_or_default();
        if result.is_ok() {
            result = pending.into_iter().try_for_each(|(def, sc)| self.function_body(&def, &sc));
        }
        result
    }

    fn scoped_block(&mut self, nodes: &[Node], parent: &SemScopeRef) -> SResult {
        let s = child(parent);
        self.block(nodes, &s)?;
        self.unused_warnings(&s);
        Ok(())
    }

    fn defer(&mut self, def: &Rc<FuncDef>, scope: &SemScopeRef) {
        match self.deferred.last_mut() {
            Some(frame) => frame.push((def.clone(), scope.clone())),
            None => self.deferred.push(vec![(def.clone(), scope.clone())]),
        }
    }

    fn function_body(&mut self, def: &FuncDef, scope: &SemScopeRef) -> SResult {
        let fs = child(scope);
        if let Some(t) = &def.method_of {
            self.declare(&fs, &Rc::from("yeh"), def.pos, false, false);
            if find_type(scope, t).is_none() {
                return Err(AdarshError::semantic(format!(
                    "'{t}' naam ka dhacha nahi mila, isliye '{t}.{}' method nahi bana sakte.",
                    def.name.as_deref().unwrap_or("?")
                ))
                .at(def.pos));
            }
        }
        for p in &def.params {
            if let Some(d) = &p.default {
                self.expr(d, scope)?;
            }
            if let Some(t) = &p.ty {
                self.check_type(scope, t)?;
            }
            self.declare(&fs, &p.name, def.pos, false, false);
        }
        if let Some(t) = &def.ret {
            self.check_type(scope, t)?;
        }
        self.labels.push(Vec::new());
        let r = self.block(&def.body, &fs);
        self.labels.pop();
        r?;
        self.unused_warnings(&fs);
        Ok(())
    }

    fn loop_body(&mut self, label: &Option<Rc<str>>, body: &[Node], scope: &SemScopeRef) -> SResult {
        self.labels.last_mut().unwrap().push(label.clone());
        let r = self.scoped_block(body, scope);
        self.labels.last_mut().unwrap().pop();
        r
    }

    fn check_jump(&self, word: &str, label: &Option<Rc<str>>, pos: Pos) -> SResult {
        let loops = self.labels.last().unwrap();
        if loops.is_empty() {
            return Err(AdarshError::semantic(format!(
                "'{word}' sirf loop (jabtak / ginnati / ke_liye / karo) ke andar chal sakta hai."
            ))
            .at(pos)
            .with_code("E005"));
        }
        if let Some(l) = label {
            if !loops.iter().any(|x| x.as_deref() == Some(&**l)) {
                return Err(AdarshError::semantic(format!(
                    "'{l}' naam ka koi loop yahan nahi hai, isliye '{word} {l};' nahi chal sakta."
                ))
                .at(pos));
            }
        }
        Ok(())
    }

    fn pattern(&mut self, p: &Pattern, scope: &SemScopeRef, case_scope: &SemScopeRef) -> SResult {
        match p {
            Pattern::Value(e) => {
                // a bare type name (`case Vyakti:` / `case number:`) is a type pattern
                if let ExprKind::Var(n, _) = &e.kind {
                    let is_type = BUILTIN_TYPES.contains(&&**n) || find_type(scope, n).is_some();
                    if is_type && lookup(scope, &|sc| sc.vars.get(&**n).map(|_| ())).is_none() {
                        return Ok(());
                    }
                }
                self.expr(e, scope)
            }
            Pattern::Range(lo, hi) => {
                self.expr(lo, scope)?;
                self.expr(hi, scope)
            }
            Pattern::Bind(n) => {
                self.declare(case_scope, n, Pos::default(), false, false);
                Ok(())
            }
            Pattern::Wildcard => Ok(()),
            Pattern::List(items, rest) => {
                for i in items {
                    self.pattern(i, scope, case_scope)?;
                }
                if let Some(r) = rest {
                    self.declare(case_scope, r, Pos::default(), false, false);
                }
                Ok(())
            }
            Pattern::Type(t) => self.check_type(scope, t),
        }
    }

    fn node(&mut self, node: &Node, scope: &SemScopeRef) -> SResult {
        let pos = node.pos;
        match &node.kind {
            NodeKind::Expr(e) => self.expr(e, scope),
            NodeKind::Badlo { name, init, ty, constant } => {
                if let Some(e) = init {
                    let already_here = scope.borrow().vars.contains_key(name);
                    self.expr(e, scope).map_err(|err| {
                        if !already_here && err.message.starts_with(&format!("'{name}' naam ka variable nahi mila.")) {
                            AdarshError::semantic(format!(
                                "'{name}' ko usi ki line mein use kiya, jab woh abhi bana bhi nahi."
                            ))
                            .at(err.pos.unwrap_or(pos))
                        } else {
                            err
                        }
                    })?;
                }
                if let Some(t) = ty {
                    self.check_type(scope, t)?;
                }
                self.declare(scope, name, pos, *constant, true);
                Ok(())
            }
            NodeKind::Destructure { target, init, constant } => {
                self.expr(init, scope)?;
                match target {
                    Destructure::List(names, rest) => {
                        for n in names.iter().flatten().chain(rest.iter()) {
                            self.declare(scope, n, pos, *constant, true);
                        }
                    }
                    Destructure::Fields(names) => {
                        for n in names {
                            self.declare(scope, n, pos, *constant, true);
                        }
                    }
                }
                Ok(())
            }
            NodeKind::Seq(nodes) => nodes.iter().try_for_each(|n| self.node(n, scope)),
            NodeKind::Dikhao(e) | NodeKind::Throw(e) | NodeKind::Yield(e) => self.expr(e, scope),
            NodeKind::Break(l) => self.check_jump("bas", l, pos),
            NodeKind::Continue(l) => self.check_jump("aage_badho", l, pos),
            NodeKind::Wapas(e) => e.as_ref().map_or(Ok(()), |e| self.expr(e, scope)),
            NodeKind::Block(b) => self.scoped_block(b, scope),
            NodeKind::If { cond, then, otherwise } => {
                self.expr(cond, scope)?;
                self.scoped_block(then, scope)?;
                if let Some(o) = otherwise {
                    match &o.kind {
                        NodeKind::Block(b) => self.scoped_block(b, scope)?,
                        _ => self.node(o, scope)?,
                    }
                }
                Ok(())
            }
            NodeKind::While { cond, body, label } => {
                self.expr(cond, scope)?;
                self.loop_body(label, body, scope)
            }
            NodeKind::DoWhile { body, cond, label } => {
                self.loop_body(label, body, scope)?;
                self.expr(cond, scope)
            }
            NodeKind::For { init, cond, update, body, label } => {
                let ls = child(scope);
                if let Some(i) = init {
                    self.node(i, &ls)?;
                }
                if let Some(c) = cond {
                    self.expr(c, &ls)?;
                }
                if let Some(u) = update {
                    self.expr(u, &ls)?;
                }
                self.loop_body(label, body, &ls)
            }
            NodeKind::ForEach { index, var, declare, iterable, body, label } => {
                self.expr(iterable, scope)?;
                let ls = child(scope);
                if *declare {
                    self.declare(&ls, var, pos, false, false);
                    if let Some(i) = index {
                        self.declare(&ls, i, pos, false, false);
                    }
                } else {
                    for n in index.iter().chain(std::iter::once(var)) {
                        self.check_assignable(scope, n, pos)?;
                    }
                }
                self.loop_body(label, body, &ls)
            }
            NodeKind::Switch { subject, cases, default } => {
                self.expr(subject, scope)?;
                for c in cases {
                    let cs = child(scope);
                    for p in &c.patterns {
                        self.pattern(p, scope, &cs)?;
                    }
                    if let Some(g) = &c.guard {
                        self.expr(g, &cs)?;
                    }
                    self.block(&c.body, &cs)?;
                }
                if let Some(d) = default {
                    self.scoped_block(d, scope)?;
                }
                Ok(())
            }
            NodeKind::TryCatch { body, catches, finally } => {
                self.scoped_block(body, scope)?;
                for c in catches {
                    if let Some(t) = &c.ty {
                        self.check_type(scope, t)?;
                    }
                    let cs = child(scope);
                    self.declare(&cs, &c.name, pos, false, false);
                    self.block(&c.body, &cs)?;
                }
                if let Some(f) = finally {
                    self.scoped_block(f, scope)?;
                }
                Ok(())
            }
            NodeKind::Import { path, alias } => {
                self.expr(path, scope)?;
                if let Some(a) = alias {
                    self.declare(scope, a, pos, true, false);
                    return Ok(());
                }
                if let ExprKind::Str(name) = &path.kind {
                    let p = crate::loader::module_path(name);
                    if !self.imported.contains(&p) {
                        if let Some(src) = self.loader.load(&p) {
                            self.imported.insert(p.clone());
                            let file = crate::error::register_source(&p, &src);
                            let module = crate::parser::parse(&src, file)?;
                            // module definitions become visible to the importing scope
                            self.deferred.push(Vec::new());
                            let r = module.iter().try_for_each(|stmt| self.node(stmt, scope));
                            let pending = self.deferred.pop().unwrap_or_default();
                            r?;
                            for (def, sc) in pending {
                                self.function_body(&def, &sc)?;
                            }
                            // names a module defines count as used
                            for v in scope.borrow().vars.values() {
                                v.used.set(true);
                            }
                        }
                    }
                }
                Ok(())
            }
            NodeKind::KaamDef(def) => {
                let name = def.name.clone().expect("named function");
                if def.method_of.is_none() {
                    scope.borrow_mut().functions.insert(name.clone(), Signature::from_params(&def.params));
                    self.declare(scope, &name, pos, false, false);
                }
                self.defer(def, scope);
                Ok(())
            }
            NodeKind::DhachaDef(def) => {
                if let Some(p) = &def.parent {
                    if find_type(scope, p).is_none() && !scope.borrow().types.contains_key(p) {
                        return Err(AdarshError::semantic(format!(
                            "'{}' ka parent '{p}' naam ka dhacha nahi mila.",
                            def.name
                        ))
                        .at(def.pos));
                    }
                }
                for f in &def.fields {
                    if let Some(d) = &f.default {
                        self.expr(d, scope)?;
                    }
                }
                scope.borrow_mut().types.insert(def.name.clone(), def.clone());
                Ok(())
            }
            NodeKind::EnumDef(name, _) => {
                scope.borrow_mut().enums.insert(name.clone());
                self.declare(scope, name, pos, true, false);
                Ok(())
            }
        }
    }

    fn expr(&mut self, e: &Expr, scope: &SemScopeRef) -> SResult {
        let pos = e.pos;
        match &e.kind {
            ExprKind::Int(_)
            | ExprKind::BigInt(_)
            | ExprKind::Float(_)
            | ExprKind::Str(_)
            | ExprKind::Bool(_)
            | ExprKind::Null => Ok(()),
            ExprKind::Interp(parts) => parts.iter().try_for_each(|p| match p {
                Part::Expr(x, _) => self.expr(x, scope),
                Part::Lit(_) => Ok(()),
            }),
            ExprKind::Var(name, _) => self.use_var(scope, name, pos),
            ExprKind::Assign(name, rhs, _) => {
                self.check_assignable(scope, name, pos)?;
                self.expr(rhs, scope)
            }
            ExprKind::CompoundAssign(name, _, rhs, _) => {
                self.check_assignable(scope, name, pos)?;
                self.use_var(scope, name, pos)?;
                self.expr(rhs, scope)
            }
            ExprKind::IndexAssign(c, i, v) | ExprKind::IndexCompoundAssign(c, i, _, v) => {
                self.expr(c, scope)?;
                self.expr(i, scope)?;
                self.expr(v, scope)
            }
            ExprKind::AttrAssign(t, _, v) | ExprKind::AttrCompoundAssign(t, _, _, v) => {
                self.expr(t, scope)?;
                self.expr(v, scope)
            }
            ExprKind::Binary(l, _, r) | ExprKind::Range(l, r) => {
                self.expr(l, scope)?;
                self.expr(r, scope)
            }
            ExprKind::Chain(items, _) | ExprKind::List(items) | ExprKind::Tuple(items) => {
                items.iter().try_for_each(|i| self.expr(i, scope))
            }
            ExprKind::IfExpr(c, a, b) => {
                self.expr(c, scope)?;
                self.expr(a, scope)?;
                self.expr(b, scope)
            }
            ExprKind::TypeCheck(x, t) => {
                self.expr(x, scope)?;
                self.check_type(scope, t)
            }
            ExprKind::Unary(_, x) | ExprKind::Spread(x) | ExprKind::NamedArg(_, x) => self.expr(x, scope),
            ExprKind::Dict(entries) => entries.iter().try_for_each(|(k, v)| {
                self.expr(k, scope)?;
                self.expr(v, scope)
            }),
            ExprKind::Dhacha { type_name, fields } => {
                let def = find_type(scope, type_name).ok_or_else(|| {
                    let mut cands = Vec::new();
                    let mut cur = Some(scope.clone());
                    while let Some(sc) = cur {
                        let b = sc.borrow();
                        cands.extend(b.types.keys().map(|k| k.to_string()));
                        cur = b.parent.clone();
                    }
                    let hint = suggestion_suffix(closest(type_name, cands.iter().map(String::as_str)));
                    AdarshError::semantic(format!(
                        "'{type_name}' naam ka dhacha nahi mila. Pehle 'dhacha {type_name} {{ ... }};' likho.{hint}"
                    ))
                    .at(pos)
                })?;
                let declared = all_fields(scope, &def);
                let mut seen: HashSet<&str> = HashSet::new();
                for (fname, fe) in fields {
                    if !declared.iter().any(|d| d == fname) {
                        let list: Vec<&str> = declared.iter().map(|s| &**s).collect();
                        let hint = suggestion_suffix(closest(fname, list.iter().copied()));
                        return Err(AdarshError::semantic(format!(
                            "Dhacha '{type_name}' mein '{fname}' field nahi hai. Fields: {}.{hint}",
                            list.join(", ")
                        ))
                        .at(fe.pos));
                    }
                    if !seen.insert(fname) {
                        return Err(AdarshError::semantic(format!("Field '{fname}' do baar diya gaya.")).at(fe.pos));
                    }
                    self.expr(fe, scope)?;
                }
                Ok(())
            }
            ExprKind::Attr(t, _) | ExprKind::SafeAttr(t, _) => self.expr(t, scope),
            ExprKind::Index(c, i) => {
                self.expr(c, scope)?;
                self.expr(i, scope)
            }
            ExprKind::Slice(c, a, b, s) => {
                self.expr(c, scope)?;
                for x in [a, b, s].into_iter().flatten() {
                    self.expr(x, scope)?;
                }
                Ok(())
            }
            ExprKind::Func(def) => {
                self.defer(def, scope);
                Ok(())
            }
            ExprKind::Call(callee, args) => {
                self.expr(callee, scope)?;
                for a in args {
                    self.expr(a, scope)?;
                }
                let simple = args.iter().all(|a| !matches!(a.kind, ExprKind::Spread(_) | ExprKind::NamedArg(..)));
                if let (ExprKind::Var(name, _), true) = (&callee.kind, simple) {
                    let sig = lookup(scope, &|sc| {
                        // a plain variable of the same name in a nearer scope shadows the function
                        if sc.functions.contains_key(&**name) {
                            Some(sc.functions.get(&**name).copied())
                        } else if sc.vars.contains_key(&**name) {
                            Some(None)
                        } else {
                            None
                        }
                    })
                    .flatten();
                    if let Some(sig) = sig {
                        if !sig.allows(args.len()) {
                            return Err(AdarshError::semantic(sig.arity_error(name, args.len()))
                                .at(pos)
                                .with_code("E002"));
                        }
                    }
                }
                Ok(())
            }
        }
    }
}
