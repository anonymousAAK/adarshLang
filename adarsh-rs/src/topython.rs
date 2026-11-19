//! "Python mein dekho": translate an AdarshLang program into readable Python,
//! so learners can see how the same ideas look in a mainstream language.
//!
//! The goal is teaching, not exact equivalence: printing, dict-vs-attribute
//! access and some builtins behave slightly differently in Python.

use crate::ast::*;
use std::fmt::Write as _;

pub fn translate(program: &[Node]) -> String {
    let mut t = Py::default();
    t.block(program, 0);
    let mut head = String::from("# AdarshLang se bana Python (seekhne ke liye; har cheez bilkul same nahi chalti)\n");
    let mut imports: Vec<&str> = t.imports.iter().map(String::as_str).collect();
    imports.sort();
    imports.dedup();
    for i in imports {
        let _ = writeln!(head, "{i}");
    }
    if !t.imports.is_empty() {
        head.push('\n');
    }
    head + &t.out
}

#[derive(Default)]
struct Py {
    out: String,
    imports: Vec<String>,
    lambda_count: usize,
    /// definitions that must appear before the statement being written
    pending: Vec<String>,
}

fn ind(n: usize) -> String {
    "    ".repeat(n)
}

fn py_string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn py_type(name: &str) -> String {
    match name {
        "number" => "(int, float)".into(),
        "string" => "str".into(),
        "boolean" => "bool".into(),
        "list" => "list".into(),
        "dict" => "dict".into(),
        "tuple" => "tuple".into(),
        "set" => "set".into(),
        "kaam" => "Callable".into(),
        "khali" => "type(None)".into(),
        "koi" => "object".into(),
        other => other.into(),
    }
}

fn py_annotation(name: &str) -> String {
    match name {
        "number" => "float".into(),
        other => py_type(other),
    }
}

fn op_str(op: BinOp) -> &'static str {
    match op {
        BinOp::And => "and",
        BinOp::Or => "or",
        BinOp::In => "in",
        other => other.symbol(),
    }
}

impl Py {
    fn line(&mut self, level: usize, text: &str) {
        for p in std::mem::take(&mut self.pending) {
            for l in p.lines() {
                let _ = writeln!(self.out, "{}{l}", ind(level));
            }
        }
        let _ = writeln!(self.out, "{}{text}", ind(level));
    }

    fn import(&mut self, what: &str) {
        self.imports.push(what.to_string());
    }

    fn block(&mut self, nodes: &[Node], level: usize) {
        if nodes.is_empty() {
            self.line(level, "pass");
            return;
        }
        for n in nodes {
            self.node(n, level);
        }
    }

    fn params(&mut self, params: &[Param]) -> String {
        params
            .iter()
            .map(|p| {
                let ty = p.ty.as_ref().map(|t| format!(": {}", py_annotation(&t.name))).unwrap_or_default();
                if p.is_varargs {
                    format!("*{}", p.name)
                } else if let Some(d) = &p.default {
                    format!("{}{ty}={}", p.name, self.expr(d))
                } else {
                    format!("{}{ty}", p.name)
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn func(&mut self, name: &str, def: &FuncDef, level: usize, method: bool) {
        let mut params = self.params(&def.params);
        if method {
            params = if params.is_empty() { "yeh".into() } else { format!("yeh, {params}") };
        }
        let ret = def.ret.as_ref().map(|t| format!(" -> {}", py_annotation(&t.name))).unwrap_or_default();
        if let Some(doc) = &def.doc {
            self.line(level, &format!("def {name}({params}){ret}:"));
            self.line(level + 1, &format!("\"\"\"{}\"\"\"", doc.replace('\n', " ")));
        } else {
            self.line(level, &format!("def {name}({params}){ret}:"));
        }
        self.block(&def.body, level + 1);
    }

    fn node(&mut self, n: &Node, level: usize) {
        match &n.kind {
            NodeKind::Expr(e) => {
                let s = self.expr(e);
                self.line(level, &s);
            }
            NodeKind::Badlo { name, init, ty, constant } => {
                let v = init.as_ref().map(|e| self.expr(e)).unwrap_or_else(|| "None".into());
                let ty = ty.as_ref().map(|t| format!(": {}", py_annotation(&t.name))).unwrap_or_default();
                let note = if *constant { "  # pakka (constant)" } else { "" };
                self.line(level, &format!("{name}{ty} = {v}{note}"));
            }
            NodeKind::Destructure { target, init, .. } => {
                let v = self.expr(init);
                match target {
                    Destructure::List(names, rest) => {
                        let mut parts: Vec<String> =
                            names.iter().map(|n| n.as_deref().unwrap_or("_").to_string()).collect();
                        if let Some(r) = rest {
                            parts.push(format!("*{r}"));
                        }
                        self.line(level, &format!("{} = {v}", parts.join(", ")));
                    }
                    Destructure::Fields(names) => {
                        let tmp = "_d";
                        self.line(level, &format!("{tmp} = {v}"));
                        for n in names {
                            self.line(level, &format!("{n} = {tmp}[\"{n}\"]"));
                        }
                    }
                }
            }
            NodeKind::Seq(nodes) => {
                for n in nodes {
                    self.node(n, level);
                }
            }
            NodeKind::Dikhao(e) => {
                let s = self.expr(e);
                self.line(level, &format!("print({s})"));
            }
            NodeKind::Break(l) => {
                let note = l
                    .as_ref()
                    .map(|l| format!("  # '{l}' wale loop se (Python mein label nahi hote)"))
                    .unwrap_or_default();
                self.line(level, &format!("break{note}"));
            }
            NodeKind::Continue(l) => {
                let note = l.as_ref().map(|l| format!("  # '{l}' wale loop ke liye")).unwrap_or_default();
                self.line(level, &format!("continue{note}"));
            }
            NodeKind::Wapas(e) => match e {
                Some(e) => {
                    let s = self.expr(e);
                    self.line(level, &format!("return {s}"));
                }
                None => self.line(level, "return"),
            },
            NodeKind::Yield(e) => {
                let s = self.expr(e);
                self.line(level, &format!("yield {s}"));
            }
            NodeKind::Block(b) => self.block(b, level),
            NodeKind::If { cond, then, otherwise } => {
                let c = self.expr(cond);
                self.line(level, &format!("if {c}:"));
                self.block(then, level + 1);
                let mut rest = otherwise.as_deref();
                while let Some(o) = rest {
                    match &o.kind {
                        NodeKind::If { cond, then, otherwise } => {
                            let c = self.expr(cond);
                            self.line(level, &format!("elif {c}:"));
                            self.block(then, level + 1);
                            rest = otherwise.as_deref();
                        }
                        NodeKind::Block(b) => {
                            self.line(level, "else:");
                            self.block(b, level + 1);
                            rest = None;
                        }
                        _ => rest = None,
                    }
                }
            }
            NodeKind::While { cond, body, .. } => {
                let c = self.expr(cond);
                self.line(level, &format!("while {c}:"));
                self.block(body, level + 1);
            }
            NodeKind::DoWhile { body, cond, .. } => {
                self.line(level, "while True:");
                self.block(body, level + 1);
                let c = self.expr(cond);
                self.line(level + 1, &format!("if not ({c}):"));
                self.line(level + 2, "break");
            }
            NodeKind::For { init, cond, update, body, .. } => {
                if let Some(i) = init {
                    self.node(i, level);
                }
                let c = cond.as_ref().map(|c| self.expr(c)).unwrap_or_else(|| "True".into());
                self.line(level, &format!("while {c}:"));
                self.block(body, level + 1);
                if let Some(u) = update {
                    let s = self.expr(u);
                    self.line(level + 1, &s);
                }
            }
            NodeKind::ForEach { index, var, iterable, body, .. } => {
                let it = self.expr(iterable);
                let header = match (index, &iterable.kind) {
                    (Some(i), ExprKind::Dict(_)) => format!("for {i}, {var} in {it}.items():"),
                    (Some(i), _) => format!("for {i}, {var} in enumerate({it}):"),
                    (None, _) => format!("for {var} in {it}:"),
                };
                self.line(level, &header);
                self.block(body, level + 1);
            }
            NodeKind::Switch { subject, cases, default } => {
                let s = self.expr(subject);
                self.line(level, &format!("match {s}:"));
                for c in cases {
                    let pats: Vec<String> = c.patterns.iter().map(|p| self.pattern(p)).collect();
                    let guard = c.guard.as_ref().map(|g| self.expr(g));
                    let head = if pats.len() == 1 && pats[0].contains(" if ") {
                        match guard {
                            Some(g) => format!("{} and {g}", pats[0]),
                            None => pats[0].clone(),
                        }
                    } else {
                        let joined: Vec<String> =
                            pats.iter().map(|p| p.split(" if ").next().unwrap_or(p).to_string()).collect();
                        match guard {
                            Some(g) => format!("{} if {g}", joined.join(" | ")),
                            None => joined.join(" | "),
                        }
                    };
                    self.line(level + 1, &format!("case {head}:"));
                    self.block(&c.body, level + 2);
                }
                if let Some(d) = default {
                    self.line(level + 1, "case _:");
                    self.block(d, level + 2);
                }
            }
            NodeKind::TryCatch { body, catches, finally } => {
                self.line(level, "try:");
                self.block(body, level + 1);
                for c in catches {
                    let ty = c.ty.as_ref().map(|t| t.name.to_string()).unwrap_or_else(|| "Exception".into());
                    let ty = if ty == "Galti" { "Exception".to_string() } else { ty };
                    self.line(level, &format!("except {ty} as {}:", c.name));
                    self.block(&c.body, level + 1);
                }
                if let Some(f) = finally {
                    self.line(level, "finally:");
                    self.block(f, level + 1);
                }
            }
            NodeKind::Throw(e) => {
                let s = self.expr(e);
                self.line(level, &format!("raise Exception({s})"));
            }
            NodeKind::Import { path, alias } => {
                let name = match &path.kind {
                    ExprKind::Str(s) => s.trim_end_matches(".aak").replace(['/', '-'], "_"),
                    _ => "module".into(),
                };
                match alias {
                    Some(a) => self.line(level, &format!("import {name} as {a}")),
                    None => self.line(level, &format!("from {name} import *")),
                }
            }
            NodeKind::KaamDef(def) => {
                let name = def.name.as_deref().unwrap_or("_kaam");
                match &def.method_of {
                    Some(t) => {
                        let fname = format!("_{t}_{name}");
                        self.func(&fname, def, level, true);
                        let pyname = match name {
                            "shabd" => "__str__",
                            "jodo" => "__add__",
                            "ghatao" => "__sub__",
                            "guna" => "__mul__",
                            "bhaag" => "__truediv__",
                            "barabar" => "__eq__",
                            "chhota" => "__lt__",
                            other => other,
                        };
                        self.line(level, &format!("{t}.{pyname} = {fname}"));
                    }
                    None => self.func(name, def, level, false),
                }
                self.line(level, "");
            }
            NodeKind::DhachaDef(d) => {
                self.import("from dataclasses import dataclass");
                self.line(level, "@dataclass");
                let parent = d.parent.as_ref().map(|p| format!("({p})")).unwrap_or_default();
                self.line(level, &format!("class {}{parent}:", d.name));
                if d.fields.is_empty() {
                    self.line(level + 1, "pass");
                }
                for f in &d.fields {
                    let v = f.default.as_ref().map(|e| self.expr(e)).unwrap_or_else(|| "None".into());
                    self.line(level + 1, &format!("{}: object = {v}", f.name));
                }
                self.line(level, "");
            }
            NodeKind::EnumDef(name, variants) => {
                self.import("from enum import Enum");
                self.line(level, &format!("class {name}(Enum):"));
                for (i, v) in variants.iter().enumerate() {
                    self.line(level + 1, &format!("{v} = {}", i + 1));
                }
                self.line(level, "");
            }
        }
    }

    fn pattern(&mut self, p: &Pattern) -> String {
        match p {
            Pattern::Value(e) => match &e.kind {
                ExprKind::Var(n, _)
                    if ["number", "string", "list", "dict", "boolean", "tuple", "set"].contains(&&**n) =>
                {
                    format!("{}()", py_annotation(n).trim_start_matches('(').split(',').next().unwrap_or("object"))
                }
                ExprKind::Var(n, _) if n.starts_with(|c: char| c.is_uppercase()) => format!("{n}()"),
                _ => self.expr(e),
            },
            Pattern::Range(lo, hi) => {
                // Python's match has no ranges: capture the value and test it in a guard
                format!("_v if {} <= _v <= {}", self.expr(lo), self.expr(hi))
            }
            Pattern::Bind(n) => n.to_string(),
            Pattern::Wildcard => "_".into(),
            Pattern::List(items, rest) => {
                let mut parts: Vec<String> = items.iter().map(|i| self.pattern(i)).collect();
                if let Some(r) = rest {
                    parts.push(format!("*{r}"));
                }
                format!("[{}]", parts.join(", "))
            }
            Pattern::Type(t) => format!("{}()", py_type(&t.name)),
        }
    }

    fn args(&mut self, args: &[Expr]) -> Vec<String> {
        args.iter().map(|a| self.expr(a)).collect()
    }

    /// Builtins that read better as Python methods or functions.
    fn builtin_call(&mut self, name: &str, args: &[String]) -> Option<String> {
        let a = |i: usize| args.get(i).cloned().unwrap_or_default();
        let named = |key: &str| args.iter().find_map(|x| x.strip_prefix(&format!("{key}=")).map(str::to_string));
        Some(match (name, args.len()) {
            ("length", 1) => format!("len({})", a(0)),
            ("push", 2) => format!("{}.append({})", a(0), a(1)),
            ("pop", 1) => format!("{}.pop()", a(0)),
            ("pop", 2) => format!("{}.pop({})", a(0), a(1)),
            ("insert", 3) => format!("{}.insert({}, {})", a(0), a(1), a(2)),
            ("rakho", 3) => format!("{}[{}] = {}", a(0), a(1), a(2)),
            ("nikalo", 2) => format!("{}[{}]", a(0), a(1)),
            ("nikalo", 3) => format!("{}.get({}, {})", a(0), a(1), a(2)),
            ("hatao", 2) => format!("{}.pop({})", a(0), a(1)),
            ("keys", 1) => format!("list({}.keys())", a(0)),
            ("values", 1) => format!("list({}.values())", a(0)),
            ("contains", 2) => format!("{} in {}", a(1), a(0)),
            ("find", 2) => format!("{}.index({})", a(0), a(1)),
            ("count", 2) => format!("{}.count({})", a(0), a(1)),
            ("sort", _) => {
                let mut extra = String::new();
                if let Some(b) = named("by") {
                    extra.push_str(&format!(", key={b}"));
                }
                if let Some(u) = named("ulta") {
                    extra.push_str(&format!(", reverse={u}"));
                }
                format!("sorted({}{extra})", a(0))
            }
            ("reverse", 1) => format!("{}[::-1]", a(0)),
            ("slice", 2) => format!("{}[{}:]", a(0), a(1)),
            ("slice", 3) => format!("{}[{}:{}]", a(0), a(1), a(2)),
            ("join", 1) => format!("\"\".join(map(str, {}))", a(0)),
            ("join", 2) => format!("{}.join(map(str, {}))", a(1), a(0)),
            ("map", 2) => format!("list(map({}, {}))", a(0), a(1)),
            ("filter", 2) => format!("list(filter({}, {}))", a(0), a(1)),
            ("reduce", _) => {
                self.import("from functools import reduce");
                format!("reduce({})", args.join(", "))
            }
            ("upper" | "lower" | "title", 1) => format!("str({}).{name}()", a(0)),
            ("trim", 1) => format!("str({}).strip()", a(0)),
            ("split", 1) => format!("{}.split(\" \")", a(0)),
            ("split", 2) => format!("{}.split({})", a(0), a(1)),
            ("replace", 3) => format!("{}.replace({}, {})", a(0), a(1), a(2)),
            ("starts_with", 2) => format!("{}.startswith({})", a(0), a(1)),
            ("ends_with", 2) => format!("{}.endswith({})", a(0), a(1)),
            ("pad_left", 2) => format!("str({}).rjust({})", a(0), a(1)),
            ("pad_left", 3) => format!("str({}).rjust({}, {})", a(0), a(1), a(2)),
            ("pad_right", 2) => format!("str({}).ljust({})", a(0), a(1)),
            ("pad_right", 3) => format!("str({}).ljust({}, {})", a(0), a(1), a(2)),
            ("repeat", 2) => format!("{} * {}", a(0), a(1)),
            ("char_code", 1) => format!("ord({})", a(0)),
            ("from_code", 1) => format!("chr({})", a(0)),
            ("format", 2) => format!("format({}, {})", a(0), a(1)),
            ("shabdme", 1) => format!("str({})", a(0)),
            ("sankhya", 1) => format!("float({})", a(0)),
            ("prakar", 1) => format!("type({}).__name__", a(0)),
            ("padho", 0) => "input()".into(),
            ("padho", 1) => format!("input({})", a(0)),
            ("padho_number", _) => format!("float(input({}))", args.first().cloned().unwrap_or_default()),
            ("min_val", 2) => format!("min({}, {})", a(0), a(1)),
            ("max_val", 2) => format!("max({}, {})", a(0), a(1)),
            ("round_val", 1) => format!("round({})", a(0)),
            ("sqrt" | "sin" | "cos" | "tan" | "asin" | "acos" | "atan" | "exp" | "log" | "gcd" | "lcm", _) => {
                self.import("import math");
                format!("math.{name}({})", args.join(", "))
            }
            ("floor" | "ceil", 1) => {
                self.import("import math");
                format!("math.{name}({})", a(0))
            }
            ("random_int", 2) => {
                self.import("import random");
                format!("random.randint({}, {})", a(0), a(1))
            }
            ("random_choice", 1) => {
                self.import("import random");
                format!("random.choice({})", a(0))
            }
            ("shuffle", 1) => {
                self.import("import random");
                format!("random.shuffle({})", a(0))
            }
            ("random_number", _) => {
                self.import("import random");
                format!("random.uniform({})", if args.is_empty() { "0, 1".into() } else { args.join(", ") })
            }
            ("json_banao", _) => {
                self.import("import json");
                format!("json.dumps({})", a(0))
            }
            ("json_padho", 1) => {
                self.import("import json");
                format!("json.loads({})", a(0))
            }
            ("file_padho", 1) => format!("open({}).read()", a(0)),
            ("file_likho", 2) => format!("open({}, \"w\").write({})", a(0), a(1)),
            ("file_jodo", 2) => format!("open({}, \"a\").write({})", a(0), a(1)),
            ("ruko", 1) => {
                self.import("import time");
                format!("time.sleep({})", a(0))
            }
            ("band_karo", _) => {
                self.import("import sys");
                format!("sys.exit({})", args.first().cloned().unwrap_or_else(|| "0".into()))
            }
            ("jaancho", 1) => format!("assert {}", a(0)),
            ("jaancho", 2) => format!("assert {}, {}", a(0), a(1)),
            ("unique", 1) => format!("list(dict.fromkeys({}))", a(0)),
            ("zip", 2) => format!("list(zip({}, {}))", a(0), a(1)),
            ("range" | "abs" | "sum" | "min" | "max" | "any" | "all" | "list" | "tuple" | "set" | "round", _) => {
                format!("{name}({})", args.join(", "))
            }
            _ => return None,
        })
    }

    fn expr(&mut self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(i) => i.to_string(),
            ExprKind::BigInt(b) => b.to_string(),
            ExprKind::Float(f) => crate::value::float_repr(*f),
            ExprKind::Str(s) => py_string(s),
            ExprKind::Interp(parts) => {
                let mut s = String::from("f\"");
                for p in parts {
                    match p {
                        Part::Lit(l) => {
                            let lit = py_string(l);
                            s.push_str(&lit[1..lit.len() - 1].replace('{', "{{").replace('}', "}}"));
                        }
                        Part::Expr(x, spec) => {
                            let inner = self.expr(x).replace('"', "'");
                            let spec = spec
                                .as_ref()
                                .map(|sp| {
                                    let mut sp = sp.to_string();
                                    if sp.contains('.') && sp.ends_with(|c: char| c.is_ascii_digit()) {
                                        sp.push('f');
                                    }
                                    format!(":{sp}")
                                })
                                .unwrap_or_default();
                            let _ = write!(s, "{{{inner}{spec}}}");
                        }
                    }
                }
                s.push('"');
                s
            }
            ExprKind::Bool(b) => if *b { "True" } else { "False" }.into(),
            ExprKind::Null => "None".into(),
            ExprKind::Var(n, _) => match &**n {
                "PI" => {
                    self.import("import math");
                    "math.pi".into()
                }
                "E" => {
                    self.import("import math");
                    "math.e".into()
                }
                "yeh" => "yeh".into(),
                other => other.into(),
            },
            ExprKind::List(items) => format!("[{}]", self.args(items).join(", ")),
            ExprKind::Tuple(items) => {
                let parts = self.args(items);
                if parts.len() == 1 {
                    format!("({},)", parts[0])
                } else {
                    format!("({})", parts.join(", "))
                }
            }
            ExprKind::Dict(entries) => {
                let parts: Vec<String> =
                    entries.iter().map(|(k, v)| format!("{}: {}", self.expr(k), self.expr(v))).collect();
                format!("{{{}}}", parts.join(", "))
            }
            ExprKind::Dhacha { type_name, fields } => {
                let parts: Vec<String> = fields.iter().map(|(n, v)| format!("{n}={}", self.expr(v))).collect();
                format!("{type_name}({})", parts.join(", "))
            }
            ExprKind::Binary(l, op, r) => {
                let (a, b) = (self.expr(l), self.expr(r));
                match op {
                    BinOp::Coalesce => format!("({a} if {a} is not None else {b})"),
                    _ => format!("({a} {} {b})", op_str(*op)),
                }
            }
            ExprKind::Chain(items, ops) => {
                let mut s = self.expr(&items[0]);
                for (op, item) in ops.iter().zip(items.iter().skip(1)) {
                    let _ = write!(s, " {} {}", op_str(*op), self.expr(item));
                }
                format!("({s})")
            }
            ExprKind::Unary(UnOp::Neg, x) => format!("-{}", self.expr(x)),
            ExprKind::Unary(UnOp::Not, x) => format!("(not {})", self.expr(x)),
            ExprKind::Range(lo, hi) => {
                let (a, b) = (self.expr(lo), self.expr(hi));
                match (&lo.kind, &hi.kind) {
                    (ExprKind::Int(x), ExprKind::Int(y)) if x > y => format!("range({a}, {}, -1)", y - 1),
                    (_, ExprKind::Int(y)) => format!("range({a}, {})", y + 1),
                    _ => format!("range({a}, {b} + 1)"),
                }
            }
            ExprKind::IfExpr(c, a, b) => format!("({} if {} else {})", self.expr(a), self.expr(c), self.expr(b)),
            ExprKind::TypeCheck(x, t) => {
                let v = self.expr(x);
                if &*t.name == "khali" {
                    format!("({v} is None)")
                } else {
                    format!("isinstance({v}, {})", py_type(&t.name))
                }
            }
            ExprKind::Spread(x) => format!("*{}", self.expr(x)),
            ExprKind::NamedArg(n, x) => format!("{n}={}", self.expr(x)),
            ExprKind::Index(c, i) => format!("{}[{}]", self.expr(c), self.expr(i)),
            ExprKind::Slice(c, a, b, s) => {
                let f = |me: &mut Self, x: &Option<Box<Expr>>| x.as_ref().map(|e| me.expr(e)).unwrap_or_default();
                let (ca, cb, cs) = (f(self, a), f(self, b), f(self, s));
                let step = if s.is_some() { format!(":{cs}") } else { String::new() };
                format!("{}[{ca}:{cb}{step}]", self.expr(c))
            }
            ExprKind::Attr(t, a) => {
                let target = self.expr(t);
                if target == "kachhua" {
                    self.import("import turtle as kachhua");
                }
                format!("{target}.{a}")
            }
            ExprKind::SafeAttr(t, a) => {
                let v = self.expr(t);
                format!("({v}.{a} if {v} is not None else None)")
            }
            ExprKind::Func(def) => {
                if def.body.len() == 1 && !def.generator {
                    if let NodeKind::Wapas(Some(ret)) = &def.body[0].kind {
                        let params = self.params(&def.params);
                        let body = self.expr(ret);
                        return format!("(lambda {params}: {body})");
                    }
                }
                self.lambda_count += 1;
                let name = format!("_kaam{}", self.lambda_count);
                let mut sub = Py { lambda_count: self.lambda_count + 100, ..Default::default() };
                sub.func(&name, def, 0, false);
                self.imports.extend(sub.imports);
                self.pending.push(sub.out);
                name
            }
            ExprKind::Assign(n, v, _) => format!("{n} = {}", self.expr(v)),
            ExprKind::CompoundAssign(n, op, v, _) => format!("{n} {}= {}", op.symbol(), self.expr(v)),
            ExprKind::IndexAssign(c, i, v) => format!("{}[{}] = {}", self.expr(c), self.expr(i), self.expr(v)),
            ExprKind::IndexCompoundAssign(c, i, op, v) => {
                format!("{}[{}] {}= {}", self.expr(c), self.expr(i), op.symbol(), self.expr(v))
            }
            ExprKind::AttrAssign(t, a, v) => format!("{}.{a} = {}", self.expr(t), self.expr(v)),
            ExprKind::AttrCompoundAssign(t, a, op, v) => {
                format!("{}.{a} {}= {}", self.expr(t), op.symbol(), self.expr(v))
            }
            ExprKind::Call(callee, args) => {
                // kachhua.aage(100) → kachhua.forward(100) with Python's turtle module
                if let ExprKind::Attr(t, m) = &callee.kind {
                    if matches!(&t.kind, ExprKind::Var(v, _) if &**v == "kachhua") {
                        self.import("import turtle as kachhua");
                        let py = match &**m {
                            "aage" => "forward",
                            "peeche" => "backward",
                            "daayen" => "right",
                            "baayen" => "left",
                            "kalam_upar" => "penup",
                            "kalam_neeche" => "pendown",
                            "rang" => "pencolor",
                            "mota" => "width",
                            "jaao" => "goto",
                            "ghar" => "home",
                            other => other,
                        };
                        let a = self.args(args);
                        return format!("kachhua.{py}({})", a.join(", "));
                    }
                }
                // dot-call on a builtin: xs.length() → len(xs)
                if let ExprKind::Attr(t, name) = &callee.kind {
                    if crate::builtins::builtin_spec(name).is_some()
                        && !matches!(&t.kind, ExprKind::Var(v, _) if &**v == "kachhua")
                    {
                        let mut all = vec![self.expr(t)];
                        all.extend(self.args(args));
                        // xs.map(f) → map(f, xs): these builtins take the function first
                        if matches!(&**name, "map" | "filter" | "reduce") && all.len() >= 2 {
                            all.swap(0, 1);
                        }
                        if let Some(s) = self.builtin_call(name, &all) {
                            return s;
                        }
                    }
                }
                let a = self.args(args);
                if let ExprKind::Var(name, _) = &callee.kind {
                    if let Some(s) = self.builtin_call(name, &a) {
                        return s;
                    }
                }
                format!("{}({})", self.expr(callee), a.join(", "))
            }
        }
    }
}
