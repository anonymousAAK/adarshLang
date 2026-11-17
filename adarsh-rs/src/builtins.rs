//! Builtin functions.

use crate::ast::BinOp;
use crate::error::AdarshError;
use crate::interp::Interpreter;
use crate::semantics::Signature;
use crate::value::*;

/// Default value of an optional builtin parameter.
#[derive(Clone, Copy)]
pub enum Def {
    Int(i64),
    Str(&'static str),
    Null,
}

impl Def {
    fn value(self) -> Value {
        match self {
            Def::Int(i) => Value::Int(i),
            Def::Str(s) => Value::str(s),
            Def::Null => Value::Null,
        }
    }
}

pub struct BuiltinSpec {
    pub name: &'static str,
    pub required: usize,
    pub defaults: &'static [Def],
}

macro_rules! b {
    ($name:literal, $req:literal) => {
        BuiltinSpec { name: $name, required: $req, defaults: &[] }
    };
    ($name:literal, $req:literal, [$($d:expr),*]) => {
        BuiltinSpec { name: $name, required: $req, defaults: &[$($d),*] }
    };
}

pub static BUILTINS: &[BuiltinSpec] = &[
    b!("length", 1),
    b!("push", 2),
    b!("pop", 1, [Def::Int(-1)]),
    b!("rakho", 3),
    b!("nikalo", 2, [Def::Null]),
    b!("hatao", 2),
    b!("map", 2),
    b!("filter", 2),
    b!("reduce", 2, [Def::Null]),
    b!("random_number", 0, [Def::Int(0), Def::Int(1)]),
    b!("current_time", 0),
    b!("abs", 1),
    b!("floor", 1),
    b!("ceil", 1),
    b!("upper", 1),
    b!("lower", 1),
    b!("join", 1, [Def::Str("")]),
    b!("split", 1, [Def::Str(" ")]),
    b!("prakar", 1),
    b!("shabdme", 1),
    b!("sankhya", 1),
    b!("range", 1, [Def::Null, Def::Int(1)]),
    b!("keys", 1),
    b!("values", 1),
    b!("contains", 2),
    b!("sort", 1),
    b!("reverse", 1),
    b!("slice", 1, [Def::Int(0), Def::Null]),
    b!("replace", 3),
    b!("trim", 1),
    b!("find", 2),
    b!("min_val", 2),
    b!("max_val", 2),
    b!("round_val", 1),
    b!("padho", 0, [Def::Str("")]),
];

pub fn builtin_name(id: usize) -> &'static str {
    BUILTINS.get(id).map(|b| b.name).unwrap_or("?")
}

pub fn signatures() -> impl Iterator<Item = (&'static str, Signature)> {
    BUILTINS.iter().map(|b| (b.name, Signature::from_counts(b.required, b.required + b.defaults.len())))
}

fn err(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

/// "'push' ko pehla argument list chahiye, number mila."
fn wants(func: &str, nth: &str, what: &str, got: &Value) -> AdarshError {
    err(format!("'{func}' ko {nth} argument {what} chahiye, {} mila.", got.type_desc()))
}

pub fn is_iterable(v: &Value) -> bool {
    matches!(v, Value::List(_) | Value::Str(_) | Value::Dict(_))
}

/// Snapshot of what `ke_liye` visits: list items, string characters, dict keys / record fields.
pub fn iter_items(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(l) => Some(l.borrow().clone()),
        Value::Str(s) => Some(s.chars().map(|c| Value::Str(c.to_string().into())).collect()),
        Value::Dict(d) => Some(d.borrow().keys()),
        _ => None,
    }
}

/// An index must be a whole number. `Ok(None)` means "too big to be in range".
pub fn normalize_index(v: &Value) -> Result<Option<i64>, AdarshError> {
    match v {
        Value::Int(i) => Ok(Some(*i)),
        Value::Big(_) => Ok(None),
        Value::Float(f) if f.is_finite() && f.fract() == 0.0 => {
            Ok(if f.abs() < 9.0e18 { Some(*f as i64) } else { None })
        }
        Value::Float(_) => Err(err(format!("Index poora number hona chahiye, {} nahi.", display(v)))),
        _ => Err(err(format!("Index poora number hona chahiye, {} nahi.", v.type_desc()))),
    }
}

/// Resolve a (possibly negative) index against `len`.
pub fn resolve(idx: Option<i64>, len: usize) -> Option<usize> {
    let i = idx?;
    let len = len as i64;
    let j = if i < 0 { i + len } else { i };
    (0..len).contains(&j).then_some(j as usize)
}

pub fn out_of_range(idx: &Value, len: usize, what: &str) -> AdarshError {
    let hint = if len == 0 {
        format!("{what} khali hai")
    } else {
        format!("{what} ki length {len} hai, isliye index 0 se {} tak (ya -1 se -{len})", len - 1)
    };
    err(format!("Index {} bahar hai: {hint}.", display(idx)))
}

fn len_of(v: &Value) -> Result<usize, AdarshError> {
    match v {
        Value::Str(s) => Ok(s.chars().count()),
        Value::List(l) => Ok(l.borrow().len()),
        Value::Dict(d) => Ok(d.borrow().len()),
        _ => Err(err(format!("{} ki length nahi hoti.", v.type_desc()))),
    }
}

/// Python-style slice clamping for `[start:end]`.
fn slice_bounds(start: &Value, end: &Value, len: usize) -> (usize, usize) {
    let len_i = len as i128;
    let clamp = |v: &Value| -> i128 {
        let raw: i128 = match v {
            Value::Int(i) => *i as i128,
            Value::Big(b) if num_traits::Signed::is_negative(&**b) => i128::MIN / 2,
            Value::Big(_) => i128::MAX / 2,
            _ => 0,
        };
        let r = if raw < 0 { raw + len_i } else { raw };
        r.clamp(0, len_i)
    };
    let (s, e) = (clamp(start), clamp(end));
    (s as usize, e.max(s) as usize)
}

/// Stable merge sort driven by `<`, so mixed types give a clear error.
fn sort_values(items: Vec<Value>) -> Result<Vec<Value>, AdarshError> {
    if items.len() <= 1 {
        return Ok(items);
    }
    let mut items = items;
    let right = items.split_off(items.len() / 2);
    let (l, r) = (sort_values(items)?, sort_values(right)?);
    let mut out = Vec::with_capacity(l.len() + r.len());
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        if compare(BinOp::Lt, &r[j], &l[i])? {
            out.push(r[j].clone());
            j += 1;
        } else {
            out.push(l[i].clone());
            i += 1;
        }
    }
    out.extend_from_slice(&l[i..]);
    out.extend_from_slice(&r[j..]);
    Ok(out)
}

fn need_number(func: &str, v: &Value) -> AdarshError {
    err(format!("{func}() ko number chahiye, {} mila.", v.type_desc()))
}

fn round_value(func: &str, v: &Value, f: fn(f64) -> f64) -> Result<Value, AdarshError> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(x) => float_to_int(f(*x)),
        _ => Err(need_number(func, v)),
    }
}

impl Interpreter {
    /// Call builtin `id` with already-evaluated arguments.
    pub fn call_builtin(&mut self, id: usize, args: Vec<Value>) -> Result<Value, AdarshError> {
        let spec = &BUILTINS[id];
        let max = spec.required + spec.defaults.len();
        if args.len() < spec.required || args.len() > max {
            let sig = Signature::from_counts(spec.required, max);
            return Err(err(sig.arity_error(spec.name, args.len())));
        }
        let given = args.len();
        let mut v = args;
        for i in given..max {
            v.push(spec.defaults[i - spec.required].value());
        }
        let provided = |i: usize| i < given;
        self.dispatch(spec.name, v, &provided)
    }

    fn call_value(&mut self, func: &str, f: &Value, args: Vec<Value>) -> Result<Value, AdarshError> {
        if !f.is_callable() {
            return Err(wants(func, "pehla", "kaam (function)", f));
        }
        self.call(f, args)
    }

    fn dispatch(&mut self, name: &str, v: Vec<Value>, provided: &dyn Fn(usize) -> bool) -> Result<Value, AdarshError> {
        let a = &v;
        Ok(match name {
            "length" => Value::Int(len_of(&a[0])? as i64),
            "push" => {
                let Value::List(l) = &a[0] else { return Err(wants(name, "pehla", "list", &a[0])) };
                l.borrow_mut().push(a[1].clone());
                a[0].clone()
            }
            "pop" => {
                let Value::List(l) = &a[0] else { return Err(wants(name, "pehla", "list", &a[0])) };
                let idx = normalize_index(&a[1])?;
                let mut list = l.borrow_mut();
                if list.is_empty() {
                    return Err(err("Khali list se pop() nahi kar sakte."));
                }
                match resolve(idx, list.len()) {
                    Some(i) => list.remove(i),
                    None => return Err(out_of_range(&a[1], list.len(), "List")),
                }
            }
            "rakho" => {
                let Value::Dict(d) = &a[0] else { return Err(wants(name, "pehla", "dict", &a[0])) };
                d.borrow_mut().insert(a[1].clone(), a[2].clone())?;
                a[0].clone()
            }
            "nikalo" => {
                let has_default = provided(2);
                match &a[0] {
                    Value::Dict(d) => match d.borrow().get(&a[1])? {
                        Some(x) => x,
                        None if has_default => a[2].clone(),
                        None => return Err(err(format!("Key {} dict mein nahi hai.", repr(&a[1])))),
                    },
                    Value::List(l) => {
                        let idx = normalize_index(&a[1])?;
                        let list = l.borrow();
                        match resolve(idx, list.len()) {
                            Some(i) => list[i].clone(),
                            None if has_default => a[2].clone(),
                            None => return Err(out_of_range(&a[1], list.len(), "List")),
                        }
                    }
                    Value::Str(s) => {
                        let idx = normalize_index(&a[1])?;
                        let n = s.chars().count();
                        match resolve(idx, n) {
                            Some(i) => Value::Str(s.chars().nth(i).unwrap().to_string().into()),
                            None if has_default => a[2].clone(),
                            None => return Err(out_of_range(&a[1], n, "String")),
                        }
                    }
                    other => return Err(wants(name, "pehla", "dict, list ya string", other)),
                }
            }
            "hatao" => match &a[0] {
                Value::Dict(d) => {
                    if let Some(t) = &d.borrow().type_name {
                        return Err(err(format!("Dhacha {t} se field nahi hata sakte.")));
                    }
                    let removed = d.borrow_mut().remove(&a[1])?;
                    removed.ok_or_else(|| err(format!("Key {} dict mein nahi hai.", repr(&a[1]))))?
                }
                Value::List(l) => {
                    let idx = normalize_index(&a[1])?;
                    let mut list = l.borrow_mut();
                    match resolve(idx, list.len()) {
                        Some(i) => list.remove(i),
                        None => return Err(out_of_range(&a[1], list.len(), "List")),
                    }
                }
                other => return Err(wants(name, "pehla", "dict ya list", other)),
            },
            "map" | "filter" => {
                if !is_iterable(&a[1]) {
                    return Err(wants(name, "doosra", "list, string ya dict", &a[1]));
                }
                let mut out = Vec::new();
                let mut i = 0;
                while let Some(item) = self.nth_item(&a[1], i) {
                    i += 1;
                    let r = self.call_value(name, &a[0], vec![item.clone()])?;
                    if name == "map" {
                        out.push(r);
                    } else if r.truthy() {
                        out.push(item);
                    }
                }
                Value::list(out)
            }
            "reduce" => {
                if !is_iterable(&a[1]) {
                    return Err(wants(name, "doosra", "list, string ya dict", &a[1]));
                }
                let mut i = 0;
                let mut acc = if provided(2) {
                    a[2].clone()
                } else {
                    i = 1;
                    self.nth_item(&a[1], 0).ok_or_else(|| {
                        err("reduce() ko khali list mili aur shuruaati value (teesra argument) nahi di.")
                    })?
                };
                while let Some(item) = self.nth_item(&a[1], i) {
                    i += 1;
                    acc = self.call_value(name, &a[0], vec![acc, item])?;
                }
                acc
            }
            "random_number" => {
                let (mut s, mut e) = (a[0].clone(), a[1].clone());
                for x in [&s, &e] {
                    if as_num(x).is_none() {
                        return Err(need_number("random_number", x));
                    }
                }
                if compare(BinOp::Gt, &s, &e)? {
                    std::mem::swap(&mut s, &mut e);
                }
                let r = self.next_random();
                let span = arith(BinOp::Sub, &e, &s)?;
                arith(BinOp::Add, &s, &arith(BinOp::Mul, &span, &Value::Float(r))?)?
            }
            "current_time" => Value::Float((self.clock)()),
            "abs" => abs_value(&a[0])?,
            "floor" => round_value("floor", &a[0], f64::floor)?,
            "ceil" => round_value("ceil", &a[0], f64::ceil)?,
            "round_val" => round_value("round_val", &a[0], f64::round_ties_even)?,
            "upper" => Value::Str(display(&a[0]).to_uppercase().into()),
            "lower" => Value::Str(display(&a[0]).to_lowercase().into()),
            "join" => {
                let items = iter_items(&a[0]).ok_or_else(|| wants(name, "pehla", "list", &a[0]))?;
                let parts: Vec<String> = items.iter().map(display).collect();
                Value::Str(parts.join(&display(&a[1])).into())
            }
            "split" => {
                let (s, sep) = (display(&a[0]), display(&a[1]));
                if sep.is_empty() {
                    return Err(err("split() ka separator khali string nahi ho sakta."));
                }
                Value::list(s.split(sep.as_str()).map(Value::str).collect())
            }
            "prakar" => Value::str(a[0].prakar()),
            "shabdme" => Value::Str(display(&a[0]).into()),
            "sankhya" => match &a[0] {
                Value::Int(_) | Value::Big(_) | Value::Float(_) => a[0].clone(),
                Value::Bool(b) => Value::Int(*b as i64),
                Value::Str(s) => {
                    let bad = || err(format!("{} ko number mein nahi badal sakte.", repr(&a[0])));
                    match parse_int(s) {
                        Some(v) => v,
                        None => Value::Float(parse_float(s).ok_or_else(bad)?),
                    }
                }
                other => return Err(err(format!("{} ko number mein nahi badal sakte.", other.type_desc()))),
            },
            "range" => {
                let (start, end) = if provided(1) && !matches!(a[1], Value::Null) {
                    (to_int(&a[0], "range()")?, to_int(&a[1], "range()")?)
                } else {
                    (Value::Int(0), to_int(&a[0], "range()")?)
                };
                let step = to_int(&a[2], "range()")?;
                let as_i = |v: &Value| match v {
                    Value::Int(i) => Ok(*i),
                    _ => Err(err("range() ke number bahut bade hain.")),
                };
                let (s, e, st) = (as_i(&start)?, as_i(&end)?, as_i(&step)?);
                if st == 0 {
                    return Err(err("range() ka step 0 nahi ho sakta."));
                }
                let count: i128 = if st > 0 {
                    if e > s {
                        (e as i128 - s as i128 + st as i128 - 1) / st as i128
                    } else {
                        0
                    }
                } else if s > e {
                    (s as i128 - e as i128 - st as i128 - 1) / (-(st as i128))
                } else {
                    0
                };
                if count > 100_000_000 {
                    return Err(err("range() bahut badi list banayega (10 crore se zyada items)."));
                }
                Value::list((0..count as i64).map(|k| Value::Int(s + k * st)).collect())
            }
            "keys" | "values" => {
                let Value::Dict(d) = &a[0] else { return Err(wants(name, "pehla", "dict", &a[0])) };
                let d = d.borrow();
                Value::list(
                    d.entries().iter().map(|(k, v)| if name == "keys" { k.clone() } else { v.clone() }).collect(),
                )
            }
            "contains" => Value::Bool(match &a[0] {
                Value::List(l) => {
                    let items = l.borrow().clone();
                    items.iter().any(|x| values_eq(x, &a[1]))
                }
                Value::Str(s) => s.contains(display(&a[1]).as_str()),
                Value::Dict(d) => d.borrow().contains(&a[1])?,
                other => return Err(wants(name, "pehla", "list, string ya dict", other)),
            }),
            "sort" => {
                let Value::List(l) = &a[0] else { return Err(wants(name, "pehla", "list", &a[0])) };
                let items = l.borrow().clone();
                Value::list(sort_values(items)?)
            }
            "reverse" => match &a[0] {
                Value::List(l) => Value::list(l.borrow().iter().rev().cloned().collect()),
                Value::Str(s) => Value::Str(s.chars().rev().collect::<String>().into()),
                other => return Err(wants(name, "pehla", "list ya string", other)),
            },
            "slice" => {
                let start = to_int(&a[1], "slice()")?;
                let end = match &a[2] {
                    Value::Null => Value::Int(i64::MAX),
                    other => to_int(other, "slice()")?,
                };
                match &a[0] {
                    Value::List(l) => {
                        let list = l.borrow();
                        let (s, e) = slice_bounds(&start, &end, list.len());
                        Value::list(list[s..e].to_vec())
                    }
                    Value::Str(st) => {
                        let chars: Vec<char> = st.chars().collect();
                        let (s, e) = slice_bounds(&start, &end, chars.len());
                        Value::Str(chars[s..e].iter().collect::<String>().into())
                    }
                    other => return Err(wants(name, "pehla", "list ya string", other)),
                }
            }
            "replace" => Value::Str(display(&a[0]).replace(&display(&a[1]), &display(&a[2])).into()),
            "trim" => Value::Str(display(&a[0]).trim().into()),
            "find" => match &a[0] {
                Value::List(l) => {
                    let items = l.borrow().clone();
                    Value::Int(items.iter().position(|x| values_eq(x, &a[1])).map(|p| p as i64).unwrap_or(-1))
                }
                Value::Str(s) => {
                    let needle = display(&a[1]);
                    Value::Int(match s.find(needle.as_str()) {
                        Some(byte) => s[..byte].chars().count() as i64,
                        None => -1,
                    })
                }
                other => return Err(wants(name, "pehla", "list ya string", other)),
            },
            "min_val" => {
                if compare(BinOp::Lt, &a[1], &a[0])? {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "max_val" => {
                if compare(BinOp::Gt, &a[1], &a[0])? {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "padho" => {
                let prompt = display(&a[0]);
                if !prompt.is_empty() {
                    self.write_str(&prompt);
                    self.flush();
                }
                match (self.input)() {
                    Some(line) => Value::Str(line.trim_end_matches(['\n', '\r']).into()),
                    None => Value::Null,
                }
            }
            _ => unreachable!("unknown builtin {name}"),
        })
    }

    /// Item `i` of an iterable. Lists are read live, so items pushed during a
    /// loop are visited too.
    pub fn nth_item(&self, v: &Value, i: usize) -> Option<Value> {
        match v {
            Value::List(l) => l.borrow().get(i).cloned(),
            Value::Str(s) => s.chars().nth(i).map(|c| Value::Str(c.to_string().into())),
            Value::Dict(d) => d.borrow().entries().get(i).map(|(k, _)| k.clone()),
            _ => None,
        }
    }
}
