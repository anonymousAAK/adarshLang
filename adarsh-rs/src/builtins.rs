//! Builtin functions (port of the `_builtin_*` methods in `adarsh_lang/runtime.py`).

use crate::error::{AdarshError, HostKind};
use crate::interp::Interpreter;
use crate::semantics::Signature;
use crate::value::*;

/// Default value of an optional builtin parameter.
#[derive(Clone, Copy)]
pub enum Def {
    Int(i64),
    False,
    Str(&'static str),
    Null,
}

impl Def {
    fn value(self) -> Value {
        match self {
            Def::Int(i) => Value::Int(i),
            Def::False => Value::Bool(false),
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
    b!("nikalo", 2, [Def::False]),
    b!("map", 2),
    b!("filter", 2),
    b!("reduce", 2, [Def::False]),
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
    b!("range", 1, [Def::Int(0), Def::Int(1)]),
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
];

pub fn builtin_name(id: usize) -> &'static str {
    BUILTINS.get(id).map(|b| b.name).unwrap_or("?")
}

pub fn signatures() -> impl Iterator<Item = (&'static str, Signature)> {
    BUILTINS
        .iter()
        .map(|b| (b.name, Signature::from_counts(b.required, b.required + b.defaults.len())))
}

fn rt(msg: &str) -> AdarshError {
    AdarshError::runtime(msg)
}

/// Values that `ke_liye`, `map`, `filter`, `reduce` and `join` can iterate.
pub fn is_iterable(v: &Value) -> bool {
    matches!(v, Value::List(_) | Value::Str(_) | Value::Dict(_))
}

/// Snapshot of the items Python would iterate (strings: chars, dicts: keys).
/// Lists are handled live by callers that need Python's live-iteration behaviour.
pub fn iter_items(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(l) => Some(l.borrow().clone()),
        Value::Str(s) => Some(s.chars().map(|c| Value::Str(c.to_string().into())).collect()),
        Value::Dict(d) => Some(d.borrow().keys()),
        _ => None,
    }
}

/// Python's `_normalize_index`.
pub fn normalize_index(v: &Value) -> Result<Option<i64>, AdarshError> {
    match v {
        Value::Bool(_) => Err(rt("Index must be an integer value in AdarshLang.")),
        Value::Float(f) => {
            if f.fract() != 0.0 || !f.is_finite() {
                if f.is_finite() {
                    return Err(rt("Index expressions must evaluate to whole numbers in AdarshLang."));
                }
                // inf.is_integer() is False in Python; nan too.
                return Err(rt("Index expressions must evaluate to whole numbers in AdarshLang."));
            }
            Ok(if f.abs() < 9.0e18 { Some(*f as i64) } else { None })
        }
        Value::Int(i) => Ok(Some(*i)),
        // a huge int is a valid index that is always out of range
        Value::Big(_) => Ok(None),
        _ => Err(rt("Index expressions must evaluate to integers in AdarshLang.")),
    }
}

/// Resolve a (possibly negative) Python index against `len`.
pub fn resolve(idx: Option<i64>, len: usize) -> Option<usize> {
    let i = idx?;
    let len = len as i64;
    let j = if i < 0 { i + len } else { i };
    (0..len).contains(&j).then_some(j as usize)
}

fn char_at(s: &str, idx: Option<i64>) -> Option<Value> {
    let n = s.chars().count();
    resolve(idx, n).map(|i| Value::Str(s.chars().nth(i).unwrap().to_string().into()))
}

fn py_len(v: &Value) -> Result<usize, AdarshError> {
    match v {
        Value::Str(s) => Ok(s.chars().count()),
        Value::List(l) => Ok(l.borrow().len()),
        Value::Dict(d) => Ok(d.borrow().len()),
        _ => Err(AdarshError::type_error(format!("object of type '{}' has no len()", v.type_name()))),
    }
}

/// Python slice clamping for `[start:end]` with step 1.
fn slice_bounds(start: &Value, end: &Value, len: usize) -> (usize, usize) {
    let len_i = len as i128;
    let clamp = |v: &Value| -> i128 {
        let raw: i128 = match v {
            Value::Int(i) => *i as i128,
            Value::Big(b) => {
                if num_traits::Signed::is_negative(&**b) {
                    i128::MIN / 2
                } else {
                    i128::MAX / 2
                }
            }
            _ => 0,
        };
        let r = if raw < 0 { raw + len_i } else { raw };
        r.clamp(0, len_i)
    };
    let (s, e) = (clamp(start), clamp(end));
    (s as usize, e.max(s) as usize)
}

/// Stable merge sort driven by Python's `<`, so mixed-type lists raise TypeError.
fn py_sort(items: Vec<Value>) -> Result<Vec<Value>, AdarshError> {
    if items.len() <= 1 {
        return Ok(items);
    }
    let mut items = items;
    let right = items.split_off(items.len() / 2);
    let (l, r) = (py_sort(items)?, py_sort(right)?);
    let mut out = Vec::with_capacity(l.len() + r.len());
    let (mut i, mut j) = (0, 0);
    while i < l.len() && j < r.len() {
        // take from the right only when strictly smaller (stability)
        if compare(crate::ast::BinOp::Lt, &r[j], &l[i])? {
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

fn round_half_even(v: &Value) -> Result<Value, AdarshError> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(f) => float_to_int(f.round_ties_even()),
        _ => Err(AdarshError::type_error(format!("type {} doesn't define __round__ method", v.type_name()))),
    }
}

fn floor_ceil(v: &Value, ceil: bool) -> Result<Value, AdarshError> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(f) => float_to_int(if ceil { f.ceil() } else { f.floor() }),
        _ => Err(AdarshError::type_error(format!("must be real number, not {}", v.type_name()))),
    }
}

impl Interpreter {
    /// Call builtin `id` with already-evaluated arguments.
    pub fn call_builtin(&mut self, id: usize, args: Vec<Value>) -> Result<Value, AdarshError> {
        let spec = &BUILTINS[id];
        let max = spec.required + spec.defaults.len();
        if args.len() < spec.required || args.len() > max {
            return Err(AdarshError::runtime(format!(
                "Builtin '{}' expected between {} and {} arguments in AdarshLang.",
                spec.name, spec.required, max
            )));
        }
        let given = args.len();
        let mut v = args;
        for i in given..max {
            v.push(spec.defaults[i - spec.required].value());
        }
        // provided[i] is true when the caller passed argument i explicitly
        let provided = |i: usize| i < given;
        self.dispatch(spec.name, v, &provided)
    }

    fn call_value(&mut self, f: &Value, args: Vec<Value>) -> Result<Value, AdarshError> {
        if !f.is_callable() {
            return Err(rt("Expected a callable function value in AdarshLang."));
        }
        self.call(f, args)
    }

    fn dispatch(&mut self, name: &str, v: Vec<Value>, provided: &dyn Fn(usize) -> bool) -> Result<Value, AdarshError> {
        let a = &v;
        Ok(match name {
            "length" => Value::Int(py_len(&a[0])? as i64),
            "push" => {
                let Value::List(l) = &a[0] else { return Err(rt("'push' expects a list target in AdarshLang.")) };
                l.borrow_mut().push(a[1].clone());
                a[0].clone()
            }
            "pop" => {
                let Value::List(l) = &a[0] else { return Err(rt("'pop' expects a list target in AdarshLang.")) };
                let idx = if provided(1) { normalize_index(&a[1])? } else { Some(-1) };
                let mut list = l.borrow_mut();
                match resolve(idx, list.len()) {
                    Some(i) => list.remove(i),
                    None => return Err(rt("Index out of range for 'pop' in AdarshLang.")),
                }
            }
            "rakho" => {
                let Value::Dict(d) = &a[0] else {
                    return Err(rt("'rakho' expects a dictionary target in AdarshLang."));
                };
                d.borrow_mut().insert(a[1].clone(), a[2].clone())?;
                a[0].clone()
            }
            "nikalo" => {
                let has_default = provided(2);
                match &a[0] {
                    Value::Dict(d) => match d.borrow().get(&a[1])? {
                        Some(x) => x,
                        None if has_default => a[2].clone(),
                        None => return Err(rt("Key not present in 'nikalo' dictionary lookup in AdarshLang.")),
                    },
                    Value::List(l) => {
                        let idx = normalize_index(&a[1])?;
                        let list = l.borrow();
                        match resolve(idx, list.len()) {
                            Some(i) => list[i].clone(),
                            None if has_default => a[2].clone(),
                            None => return Err(rt("Index out of range for 'nikalo' in AdarshLang.")),
                        }
                    }
                    Value::Str(s) => {
                        let idx = normalize_index(&a[1])?;
                        match char_at(s, idx) {
                            Some(c) => c,
                            None if has_default => a[2].clone(),
                            None => return Err(rt("Index out of range for 'nikalo' in AdarshLang.")),
                        }
                    }
                    _ => return Err(rt("'nikalo' expects a dictionary, list, or string target in AdarshLang.")),
                }
            }
            "map" | "filter" => {
                if !is_iterable(&a[1]) {
                    return Err(AdarshError::runtime(format!(
                        "'{name}' expects an iterable second argument in AdarshLang."
                    )));
                }
                let mut out = Vec::new();
                let mut i = 0;
                while let Some(item) = self.nth_item(&a[1], i) {
                    i += 1;
                    let r = self.call_value(&a[0], vec![item.clone()])?;
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
                    return Err(rt("'reduce' expects an iterable second argument in AdarshLang."));
                }
                let mut i = 0;
                let mut acc = if provided(2) {
                    a[2].clone()
                } else {
                    match self.nth_item(&a[1], 0) {
                        Some(x) => {
                            i = 1;
                            x
                        }
                        None => {
                            return Err(rt(
                                "'reduce' requires at least one item when no initial value is provided in AdarshLang.",
                            ))
                        }
                    }
                };
                while let Some(item) = self.nth_item(&a[1], i) {
                    i += 1;
                    acc = self.call_value(&a[0], vec![acc, item])?;
                }
                acc
            }
            "random_number" => {
                let (mut s, mut e) = (a[0].clone(), a[1].clone());
                if compare(crate::ast::BinOp::Gt, &s, &e)? {
                    std::mem::swap(&mut s, &mut e);
                }
                let r = self.next_random();
                // start + (end - start) * random()
                let span = crate::value::arith(crate::ast::BinOp::Sub, &e, &s)?;
                let scaled = crate::value::arith(crate::ast::BinOp::Mul, &span, &Value::Float(r))?;
                crate::value::arith(crate::ast::BinOp::Add, &s, &scaled)?
            }
            "current_time" => Value::Float((self.clock)()),
            "abs" => abs_value(&a[0])?,
            "floor" => floor_ceil(&a[0], false)?,
            "ceil" => floor_ceil(&a[0], true)?,
            "upper" => Value::Str(py_str(&a[0]).to_uppercase().into()),
            "lower" => Value::Str(py_str(&a[0]).to_lowercase().into()),
            "join" => {
                let items = iter_items(&a[0]).ok_or_else(|| rt("'join' expects an iterable of values in AdarshLang."))?;
                let sep = py_str(&a[1]);
                let parts: Vec<String> = items.iter().map(py_str).collect();
                Value::Str(parts.join(&sep).into())
            }
            "split" => {
                let (s, sep) = (py_str(&a[0]), py_str(&a[1]));
                if sep.is_empty() {
                    return Err(AdarshError::value_error("empty separator"));
                }
                Value::list(s.split(sep.as_str()).map(Value::str).collect())
            }
            "prakar" => Value::str(match &a[0] {
                Value::Null => "khali",
                Value::Bool(_) => "boolean",
                Value::Int(_) | Value::Big(_) | Value::Float(_) => "number",
                Value::Str(_) => "string",
                Value::List(_) => "list",
                Value::Dict(d) => {
                    if d.borrow().get_str("__type__").is_some() {
                        "dhacha"
                    } else {
                        "dict"
                    }
                }
                Value::Func(_) | Value::Builtin(_) => "kaam",
            }),
            "shabdme" => Value::Str(display(&a[0]).into()),
            "sankhya" => {
                let val = &a[0];
                match val {
                    Value::Int(_) | Value::Big(_) | Value::Float(_) | Value::Bool(_) => val.clone(),
                    _ => {
                        let text = py_str(val);
                        let err = || AdarshError::runtime(format!("Cannot convert '{text}' to number in AdarshLang."));
                        match val {
                            Value::Str(s) if text.contains('.') => Value::Float(parse_py_float(s).ok_or_else(err)?),
                            Value::Str(s) => parse_py_int(s).ok_or_else(err)?,
                            _ => return Err(err()),
                        }
                    }
                }
            }
            "range" => {
                let (start, end, step) = if !provided(1) {
                    (Value::Int(0), py_int(&a[0])?, Value::Int(1))
                } else {
                    (py_int(&a[0])?, py_int(&a[1])?, py_int(&a[2])?)
                };
                let as_i = |v: &Value| match v {
                    Value::Int(i) => Ok(*i),
                    _ => Err(AdarshError::overflow("Python int too large to convert to C ssize_t")),
                };
                let (s, e, st) = (as_i(&start)?, as_i(&end)?, as_i(&step)?);
                if st == 0 {
                    return Err(AdarshError::value_error("range() arg 3 must not be zero"));
                }
                let count: i128 = if st > 0 {
                    if e > s { ((e as i128 - s as i128) + st as i128 - 1) / st as i128 } else { 0 }
                } else if s > e {
                    ((s as i128 - e as i128) + (-(st as i128)) - 1) / (-(st as i128))
                } else {
                    0
                };
                if count > 100_000_000 {
                    return Err(AdarshError::Host(HostKind::MemoryError, String::new()));
                }
                Value::list((0..count as i64).map(|k| Value::Int(s + k * st)).collect())
            }
            "keys" | "values" => {
                let Value::Dict(d) = &a[0] else {
                    return Err(AdarshError::runtime(format!("'{name}' expects a dictionary in AdarshLang.")));
                };
                let d = d.borrow();
                let not_type = |k: &Value| !matches!(k, Value::Str(s) if &**s == "__type__");
                Value::list(
                    d.entries()
                        .iter()
                        .filter(|(k, _)| not_type(k))
                        .map(|(k, v)| if name == "keys" { k.clone() } else { v.clone() })
                        .collect(),
                )
            }
            "contains" => Value::Bool(match &a[0] {
                Value::List(l) => {
                    let items = l.borrow().clone();
                    items.iter().any(|x| py_eq(x, &a[1]))
                }
                Value::Str(s) => s.contains(py_str(&a[1]).as_str()),
                Value::Dict(d) => d.borrow().contains(&a[1])?,
                _ => return Err(rt("'contains' expects a list, string, or dictionary in AdarshLang.")),
            }),
            "sort" => {
                let Value::List(l) = &a[0] else { return Err(rt("'sort' expects a list in AdarshLang.")) };
                let items = l.borrow().clone();
                Value::list(py_sort(items)?)
            }
            "reverse" => match &a[0] {
                Value::List(l) => Value::list(l.borrow().iter().rev().cloned().collect()),
                Value::Str(s) => Value::Str(s.chars().rev().collect::<String>().into()),
                _ => return Err(rt("'reverse' expects a list or string in AdarshLang.")),
            },
            "slice" => {
                let start = py_int(&a[1])?;
                let end = match &a[2] {
                    Value::Null => Value::Int(py_len(&a[0])? as i64),
                    other => py_int(other)?,
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
                    _ => return Err(rt("'slice' expects a list or string in AdarshLang.")),
                }
            }
            "replace" => Value::Str(py_str(&a[0]).replace(&py_str(&a[1]), &py_str(&a[2])).into()),
            "trim" => Value::Str(py_str(&a[0]).trim().into()),
            "find" => match &a[0] {
                Value::List(l) => {
                    let items = l.borrow().clone();
                    Value::Int(items.iter().position(|x| py_eq(x, &a[1])).map(|p| p as i64).unwrap_or(-1))
                }
                Value::Str(s) => {
                    let needle = py_str(&a[1]);
                    Value::Int(match s.find(needle.as_str()) {
                        Some(byte) => s[..byte].chars().count() as i64,
                        None => -1,
                    })
                }
                _ => return Err(rt("'find' expects a list or string in AdarshLang.")),
            },
            "min_val" => {
                if compare(crate::ast::BinOp::Lt, &a[1], &a[0])? {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "max_val" => {
                if compare(crate::ast::BinOp::Gt, &a[1], &a[0])? {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "round_val" => round_half_even(&a[0])?,
            _ => return Err(AdarshError::runtime(format!("Unknown builtin '{name}' in AdarshLang."))),
        })
    }

    /// Item `i` of an iterable, reading lists live (Python iterates lists by index,
    /// so items appended during iteration are visited).
    pub fn nth_item(&self, v: &Value, i: usize) -> Option<Value> {
        match v {
            Value::List(l) => l.borrow().get(i).cloned(),
            Value::Str(s) => s.chars().nth(i).map(|c| Value::Str(c.to_string().into())),
            Value::Dict(d) => d.borrow().entries().get(i).map(|(k, _)| k.clone()),
            _ => None,
        }
    }
}
