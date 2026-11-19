//! Runtime values: printing, equality, ordering, truthiness and arithmetic.

use crate::ast::{BinOp, FuncDef};
use crate::error::AdarshError;
use crate::interp::ScopeRef;
use num_bigint::BigInt;
use num_integer::Integer;
use num_traits::{FromPrimitive, Signed, ToPrimitive, Zero};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::Write as _;
use std::rc::Rc;

pub type R<T> = Result<T, AdarshError>;

pub struct Closure {
    pub def: Rc<FuncDef>,
    pub scope: ScopeRef,
}

impl std::fmt::Debug for Closure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "<kaam {}>", self.def.display_name())
    }
}

/// A method looked up on a record: calling it binds `yeh` to `receiver`.
#[derive(Debug)]
pub struct BoundMethod {
    pub receiver: Value,
    pub func: Rc<Closure>,
}

#[derive(Clone, Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Big(Rc<BigInt>),
    Float(f64),
    Str(Rc<str>),
    List(Rc<RefCell<Vec<Value>>>),
    /// Dictionaries and `dhacha` records (records have `type_name` set).
    Dict(Rc<RefCell<Dict>>),
    Func(Rc<Closure>),
    Method(Rc<BoundMethod>),
    Builtin(usize),
    /// Fixed-size, unchangeable sequence: `(1, 2)`.
    Tuple(Rc<Vec<Value>>),
    /// Unique values in insertion order (stored as dict keys).
    Set(Rc<RefCell<Dict>>),
    /// One variant of a `vikalp`: `Rang.Laal`.
    Enum(Rc<EnumValue>),
    /// The `vikalp` itself; `.Variant` gives a value.
    EnumType(Rc<EnumType>),
}

#[derive(Debug)]
pub struct EnumValue {
    pub type_name: Rc<str>,
    pub name: Rc<str>,
    pub index: usize,
}

#[derive(Debug)]
pub struct EnumType {
    pub name: Rc<str>,
    pub variants: Vec<Rc<EnumValue>>,
}

impl Value {
    pub fn str(s: &str) -> Value {
        Value::Str(Rc::from(s))
    }
    pub fn list(v: Vec<Value>) -> Value {
        Value::List(Rc::new(RefCell::new(v)))
    }
    pub fn dict(d: Dict) -> Value {
        Value::Dict(Rc::new(RefCell::new(d)))
    }
    pub fn from_big(b: BigInt) -> Value {
        match b.to_i64() {
            Some(i) => Value::Int(i),
            None => Value::Big(Rc::new(b)),
        }
    }

    /// The name `prakar()` returns.
    pub fn prakar(&self) -> &'static str {
        match self {
            Value::Null => "khali",
            Value::Bool(_) => "boolean",
            Value::Int(_) | Value::Big(_) | Value::Float(_) => "number",
            Value::Str(_) => "string",
            Value::List(_) => "list",
            Value::Dict(d) => {
                if d.borrow().type_name.is_some() {
                    "dhacha"
                } else {
                    "dict"
                }
            }
            Value::Func(_) | Value::Method(_) | Value::Builtin(_) => "kaam",
            Value::Tuple(_) => "tuple",
            Value::Set(_) => "set",
            Value::Enum(_) | Value::EnumType(_) => "vikalp",
        }
    }

    /// Type description for error messages, e.g. `string`, `dhacha Vyakti`.
    pub fn type_desc(&self) -> String {
        if let Value::Enum(e) = self {
            return format!("vikalp {}", e.type_name);
        }
        if let Value::Dict(d) = self {
            if let Some(t) = &d.borrow().type_name {
                return format!("dhacha {t}");
            }
        }
        self.prakar().to_string()
    }

    pub fn truthy(&self) -> bool {
        match self {
            Value::Null => false,
            Value::Bool(b) => *b,
            Value::Int(i) => *i != 0,
            Value::Big(b) => !b.is_zero(),
            Value::Float(f) => *f != 0.0,
            Value::Str(s) => !s.is_empty(),
            Value::List(l) => !l.borrow().is_empty(),
            Value::Dict(d) => d.borrow().type_name.is_some() || !d.borrow().is_empty(),
            Value::Func(_) | Value::Method(_) | Value::Builtin(_) => true,
            Value::Tuple(t) => !t.is_empty(),
            Value::Set(s) => !s.borrow().is_empty(),
            Value::Enum(_) | Value::EnumType(_) => true,
        }
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, Value::Func(_) | Value::Method(_) | Value::Builtin(_))
    }
}

pub fn type_err(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

// ---------------------------------------------------------------- dict

/// Hash key: `sahi_hai_be == 1 == 1.0`, so they share a slot.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HKey {
    Null,
    Int(i64),
    Big(BigInt),
    Float(u64),
    Str(Rc<str>),
    Ptr(usize),
    Tuple(Vec<HKey>),
    Enum(Rc<str>, usize),
}

pub fn hash_key(v: &Value) -> R<HKey> {
    Ok(match v {
        Value::Null => HKey::Null,
        Value::Bool(b) => HKey::Int(*b as i64),
        Value::Int(i) => HKey::Int(*i),
        Value::Big(b) => HKey::Big((**b).clone()),
        Value::Float(f) => {
            if f.is_finite() && f.fract() == 0.0 {
                if f.abs() <= 9.2e18 {
                    HKey::Int(*f as i64)
                } else {
                    HKey::Big(BigInt::from_f64(*f).unwrap_or_default())
                }
            } else {
                HKey::Float(f.to_bits())
            }
        }
        Value::Str(s) => HKey::Str(s.clone()),
        Value::Func(c) => HKey::Ptr(Rc::as_ptr(c) as *const u8 as usize),
        Value::Method(m) => HKey::Ptr(Rc::as_ptr(m) as *const u8 as usize),
        Value::Builtin(i) => HKey::Ptr(*i),
        Value::Tuple(items) => HKey::Tuple(items.iter().map(hash_key).collect::<R<Vec<_>>>()?),
        Value::Enum(e) => HKey::Enum(e.type_name.clone(), e.index),
        Value::EnumType(t) => HKey::Ptr(Rc::as_ptr(t) as *const u8 as usize),
        Value::List(_) | Value::Dict(_) | Value::Set(_) => {
            return Err(type_err(format!("{} ko dict ki key nahi bana sakte.", v.type_desc())))
        }
    })
}

/// Insertion-ordered dictionary. Records are dicts with a `type_name`.
#[derive(Default, Debug)]
pub struct Dict {
    entries: Vec<(Value, Value)>,
    index: HashMap<HKey, usize>,
    pub type_name: Option<Rc<str>>,
}

impl Dict {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn record(type_name: Rc<str>) -> Self {
        Dict { type_name: Some(type_name), ..Default::default() }
    }
    pub fn len(&self) -> usize {
        self.entries.len()
    }
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
    pub fn get(&self, k: &Value) -> R<Option<Value>> {
        let hk = hash_key(k)?;
        Ok(self.index.get(&hk).map(|&i| self.entries[i].1.clone()))
    }
    pub fn get_str(&self, k: &str) -> Option<Value> {
        self.index.get(&HKey::Str(Rc::from(k))).map(|&i| self.entries[i].1.clone())
    }
    pub fn contains(&self, k: &Value) -> R<bool> {
        Ok(self.index.contains_key(&hash_key(k)?))
    }
    pub fn insert(&mut self, k: Value, v: Value) -> R<()> {
        let hk = hash_key(&k)?;
        if let Some(&i) = self.index.get(&hk) {
            self.entries[i].1 = v;
        } else {
            self.index.insert(hk, self.entries.len());
            self.entries.push((k, v));
        }
        Ok(())
    }
    pub fn insert_str(&mut self, k: &str, v: Value) {
        self.insert(Value::str(k), v).expect("string keys are hashable")
    }
    /// Remove a key, returning its value.
    pub fn remove(&mut self, k: &Value) -> R<Option<Value>> {
        let hk = hash_key(k)?;
        let Some(i) = self.index.remove(&hk) else { return Ok(None) };
        let (_, v) = self.entries.remove(i);
        for slot in self.index.values_mut() {
            if *slot > i {
                *slot -= 1;
            }
        }
        Ok(Some(v))
    }
    pub fn entries(&self) -> &[(Value, Value)] {
        &self.entries
    }
    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().map(|(k, _)| k.clone()).collect()
    }
}

// ---------------------------------------------------------------- printing

/// Shortest round-trip float text; scientific outside 1e-4..1e16 (`1e+16`, `1e-05`).
pub fn float_repr(f: f64) -> String {
    if f.is_nan() {
        return "nan".into();
    }
    if f.is_infinite() {
        return if f > 0.0 { "inf".into() } else { "-inf".into() };
    }
    if f == 0.0 {
        return if f.is_sign_negative() { "-0.0".into() } else { "0.0".into() };
    }
    let sci = format!("{:e}", f);
    let (mant, exp) = sci.split_once('e').unwrap();
    let exp: i32 = exp.parse().unwrap();
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut out = String::new();
    if mant.starts_with('-') {
        out.push('-');
    }
    if (-4..16).contains(&exp) {
        if exp >= 0 {
            let e = exp as usize;
            if digits.len() <= e + 1 {
                out.push_str(&digits);
                out.extend(std::iter::repeat_n('0', e + 1 - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..=e]);
                out.push('.');
                out.push_str(&digits[e + 1..]);
            }
        } else {
            out.push_str("0.");
            out.extend(std::iter::repeat_n('0', (-exp - 1) as usize));
            out.push_str(&digits);
        }
    } else {
        out.push_str(&digits[..1]);
        if digits.len() > 1 {
            out.push('.');
            out.push_str(&digits[1..]);
        }
        let _ = write!(out, "e{}{:02}", if exp < 0 { '-' } else { '+' }, exp.abs());
    }
    out
}

/// String as an AdarshLang literal: `"say \"hi\"\n"`.
fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\r' => out.push_str("\\r"),
            '\0' => out.push_str("\\0"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            c => out.push(c),
        }
    }
    out.push('"');
}

/// Custom text for a value (records with a `shabd()` method), supplied by the interpreter.
pub type ShowHook<'a> = &'a mut dyn FnMut(&Value) -> Option<String>;

fn write_value(v: &Value, nested: bool, out: &mut String, seen: &mut Vec<usize>, hook: ShowHook) {
    match v {
        Value::Null => out.push_str("khali"),
        Value::Bool(b) => out.push_str(if *b { "sahi_hai_be" } else { "jhuth" }),
        Value::Int(i) => {
            let _ = write!(out, "{i}");
        }
        Value::Big(b) => {
            let _ = write!(out, "{b}");
        }
        Value::Float(f) => out.push_str(&float_repr(*f)),
        Value::Str(s) => {
            if nested {
                quote(s, out)
            } else {
                out.push_str(s)
            }
        }
        Value::List(l) => {
            let id = Rc::as_ptr(l) as *const u8 as usize;
            if seen.contains(&id) {
                out.push_str("[...]");
                return;
            }
            seen.push(id);
            out.push('[');
            for (i, item) in l.borrow().iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(item, true, out, seen, hook);
            }
            out.push(']');
            seen.pop();
        }
        Value::Dict(d) => {
            let id = Rc::as_ptr(d) as *const u8 as usize;
            if seen.contains(&id) {
                out.push_str("{...}");
                return;
            }
            if d.borrow().type_name.is_some() {
                if let Some(text) = hook(v) {
                    out.push_str(&text);
                    return;
                }
            }
            seen.push(id);
            let d = d.borrow();
            match &d.type_name {
                Some(t) => {
                    out.push_str(t);
                    if d.is_empty() {
                        out.push_str(" {}");
                    } else {
                        out.push_str(" { ");
                        for (i, (k, val)) in d.entries().iter().enumerate() {
                            if i > 0 {
                                out.push_str(", ");
                            }
                            write_value(k, false, out, seen, hook);
                            out.push_str(": ");
                            write_value(val, true, out, seen, hook);
                        }
                        out.push_str(" }");
                    }
                }
                None => {
                    out.push('{');
                    for (i, (k, val)) in d.entries().iter().enumerate() {
                        if i > 0 {
                            out.push_str(", ");
                        }
                        write_value(k, true, out, seen, hook);
                        out.push_str(": ");
                        write_value(val, true, out, seen, hook);
                    }
                    out.push('}');
                }
            }
            seen.pop();
        }
        Value::Func(c) => {
            let _ = write!(out, "<kaam {}>", c.def.display_name());
        }
        Value::Method(m) => {
            let _ = write!(out, "<kaam {}>", m.func.def.display_name());
        }
        Value::Builtin(i) => {
            let _ = write!(out, "<kaam {}>", crate::builtins::builtin_name(*i));
        }
        Value::Tuple(items) => {
            out.push('(');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(item, true, out, seen, hook);
            }
            if items.len() == 1 {
                out.push(',');
            }
            out.push(')');
        }
        Value::Set(set) => {
            out.push_str("set([");
            for (i, (k, _)) in set.borrow().entries().iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(k, true, out, seen, hook);
            }
            out.push_str("])");
        }
        Value::Enum(e) => {
            let _ = write!(out, "{}.{}", e.type_name, e.name);
        }
        Value::EnumType(t) => {
            let _ = write!(out, "<vikalp {}>", t.name);
        }
    }
}

/// How `dikhao` and string interpolation show a value (strings unquoted).
pub fn display(v: &Value) -> String {
    display_with(v, &mut |_| None)
}

pub fn display_with(v: &Value, hook: ShowHook) -> String {
    if let Value::Str(s) = v {
        return s.to_string();
    }
    let mut out = String::new();
    write_value(v, false, &mut out, &mut Vec::new(), hook);
    out
}

/// Source-like form, used by the REPL and inside containers (strings quoted).
pub fn repr(v: &Value) -> String {
    repr_with(v, &mut |_| None)
}

pub fn repr_with(v: &Value, hook: ShowHook) -> String {
    let mut out = String::new();
    write_value(v, true, &mut out, &mut Vec::new(), hook);
    out
}

// ---------------------------------------------------------------- numbers

#[derive(Clone)]
pub enum Num {
    I(i64),
    B(BigInt),
    F(f64),
}

pub fn as_num(v: &Value) -> Option<Num> {
    Some(match v {
        Value::Bool(b) => Num::I(*b as i64),
        Value::Int(i) => Num::I(*i),
        Value::Big(b) => Num::B((**b).clone()),
        Value::Float(f) => Num::F(*f),
        _ => return None,
    })
}

fn big(n: &Num) -> BigInt {
    match n {
        Num::I(i) => BigInt::from(*i),
        Num::B(b) => b.clone(),
        Num::F(_) => unreachable!(),
    }
}

fn too_big() -> AdarshError {
    AdarshError::runtime("Number bahut bada hai.")
}

pub fn big_to_f64(b: &BigInt) -> R<f64> {
    match b.to_f64() {
        Some(f) if f.is_finite() => Ok(f),
        _ => Err(too_big()),
    }
}

fn to_f64(n: &Num) -> R<f64> {
    match n {
        Num::I(i) => Ok(*i as f64),
        Num::B(b) => big_to_f64(b),
        Num::F(f) => Ok(*f),
    }
}

fn is_float(n: &Num) -> bool {
    matches!(n, Num::F(_))
}

fn unsupported(op: BinOp, a: &Value, b: &Value) -> AdarshError {
    let (ta, tb) = (a.type_desc(), b.type_desc());
    let mut msg = format!("'{}' {ta} aur {tb} ke beech nahi chalta.", op.symbol());
    if op == BinOp::Add && (matches!(a, Value::Str(_)) || matches!(b, Value::Str(_))) {
        msg = format!(
            "String mein sirf string jod sakte ho, yahan {ta} + {tb} hai. Interpolation \"..{{x}}..\" ya shabdme() use karo."
        );
    }
    type_err(msg)
}

/// Correctly rounded `a / b` for integers.
fn int_true_div(a: &Num, b: &Num) -> R<f64> {
    const EXACT: i64 = 1 << 53;
    if let (Num::I(x), Num::I(y)) = (a, b) {
        if x.abs() <= EXACT && y.abs() <= EXACT {
            return Ok(*x as f64 / *y as f64);
        }
    }
    let (x, y) = (big(a), big(b));
    let negative = x.is_negative() != y.is_negative();
    let (x, y) = (x.abs(), y.abs());
    if x.is_zero() {
        return Ok(if negative { -0.0 } else { 0.0 });
    }
    let k: i64 = 64 + y.bits() as i64 - x.bits() as i64;
    let (q, r) = if k >= 0 { (&x << k as usize).div_rem(&y) } else { x.div_rem(&(&y << (-k) as usize)) };
    let mut sticky = !r.is_zero();
    let shift = q.bits() as i64 - 55;
    let mut q = q;
    if shift > 0 {
        let mask = (BigInt::from(1) << shift as usize) - 1;
        sticky |= !(&q & &mask).is_zero();
        q >>= shift as usize;
    }
    let m = q.to_u64().unwrap() | sticky as u64;
    let mut f = m as f64;
    let mut e = shift.max(0) - k;
    while e > 0 {
        let step = e.min(1000);
        f *= 2f64.powi(step as i32);
        e -= step;
    }
    while e < 0 {
        let step = (-e).min(1000);
        f /= 2f64.powi(step as i32);
        e += step;
    }
    if f.is_infinite() {
        return Err(too_big());
    }
    Ok(if negative { -f } else { f })
}

fn floor_mod_f(a: f64, b: f64) -> f64 {
    let mut r = a % b;
    if r != 0.0 {
        if (r < 0.0) != (b < 0.0) {
            r += b;
        }
    } else {
        r = 0.0f64.copysign(b);
    }
    r
}

fn repeat_count(n: &Value) -> Option<i64> {
    match n {
        Value::Int(i) => Some(*i),
        Value::Bool(b) => Some(*b as i64),
        _ => None,
    }
}

const MAX_REPEAT: usize = 100_000_000;

/// Binary arithmetic for `+ - * / % **` (also used by compound assignment).
pub fn arith(op: BinOp, a: &Value, b: &Value) -> R<Value> {
    match op {
        BinOp::Add => {
            match (a, b) {
                (Value::Str(x), Value::Str(y)) => {
                    let mut s = String::with_capacity(x.len() + y.len());
                    s.push_str(x);
                    s.push_str(y);
                    return Ok(Value::Str(s.into()));
                }
                (Value::List(x), Value::List(y)) => {
                    let mut v = x.borrow().clone();
                    v.extend(y.borrow().iter().cloned());
                    return Ok(Value::list(v));
                }
                _ => {}
            }
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            if is_float(&x) || is_float(&y) {
                return Ok(Value::Float(to_f64(&x)? + to_f64(&y)?));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if let Some(r) = p.checked_add(*q) {
                    return Ok(Value::Int(r));
                }
            }
            Ok(Value::from_big(big(&x) + big(&y)))
        }
        BinOp::Sub => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            if is_float(&x) || is_float(&y) {
                return Ok(Value::Float(to_f64(&x)? - to_f64(&y)?));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if let Some(r) = p.checked_sub(*q) {
                    return Ok(Value::Int(r));
                }
            }
            Ok(Value::from_big(big(&x) - big(&y)))
        }
        BinOp::Mul => {
            match (a, b) {
                (Value::Str(s), n) | (n, Value::Str(s)) if repeat_count(n).is_some() => {
                    let k = repeat_count(n).unwrap().max(0) as usize;
                    if s.len().saturating_mul(k) > MAX_REPEAT {
                        return Err(too_big());
                    }
                    return Ok(Value::Str(s.repeat(k).into()));
                }
                (Value::List(l), n) | (n, Value::List(l)) if repeat_count(n).is_some() => {
                    let k = repeat_count(n).unwrap().max(0) as usize;
                    let src = l.borrow();
                    if src.len().saturating_mul(k) > MAX_REPEAT {
                        return Err(too_big());
                    }
                    let mut v = Vec::with_capacity(src.len() * k);
                    for _ in 0..k {
                        v.extend(src.iter().cloned());
                    }
                    return Ok(Value::list(v));
                }
                _ => {}
            }
            let (x, y) = nums(a, b).ok_or_else(|| {
                let seq = |v: &Value| matches!(v, Value::Str(_) | Value::List(_));
                if seq(a) || seq(b) {
                    let (s, other) = if seq(a) { (a, b) } else { (b, a) };
                    type_err(format!(
                        "{} ko sirf poore number se guna kar sakte ho, {} se nahi.",
                        s.type_desc(),
                        other.type_desc()
                    ))
                } else {
                    unsupported(op, a, b)
                }
            })?;
            if is_float(&x) || is_float(&y) {
                return Ok(Value::Float(to_f64(&x)? * to_f64(&y)?));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if let Some(r) = p.checked_mul(*q) {
                    return Ok(Value::Int(r));
                }
            }
            Ok(Value::from_big(big(&x) * big(&y)))
        }
        BinOp::Div => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            let zero = || AdarshError::runtime("Zero se divide nahi kar sakte.");
            if is_float(&x) || is_float(&y) {
                let d = to_f64(&y)?;
                if d == 0.0 {
                    return Err(zero());
                }
                return Ok(Value::Float(to_f64(&x)? / d));
            }
            if big(&y).is_zero() {
                return Err(zero());
            }
            Ok(Value::Float(int_true_div(&x, &y)?))
        }
        BinOp::IntDiv => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            let zero = || AdarshError::runtime("Zero se divide nahi kar sakte.");
            if is_float(&x) || is_float(&y) {
                let d = to_f64(&y)?;
                if d == 0.0 {
                    return Err(zero());
                }
                return Ok(Value::Float((to_f64(&x)? / d).floor()));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if *q == 0 {
                    return Err(zero());
                }
                if let Some(t) = p.checked_div(*q) {
                    // round towards minus infinity, like Python's //
                    let floor = if p % q != 0 && ((*p < 0) != (*q < 0)) { t - 1 } else { t };
                    return Ok(Value::Int(floor));
                }
            }
            let by = big(&y);
            if by.is_zero() {
                return Err(zero());
            }
            Ok(Value::from_big(big(&x).div_floor(&by)))
        }
        BinOp::Mod => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            let zero = || AdarshError::runtime("Zero se modulo (%) nahi kar sakte.");
            if is_float(&x) || is_float(&y) {
                let d = to_f64(&y)?;
                if d == 0.0 {
                    return Err(zero());
                }
                return Ok(Value::Float(floor_mod_f(to_f64(&x)?, d)));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if *q == 0 {
                    return Err(zero());
                }
                if let Some(r) = p.checked_rem_euclid(*q) {
                    // the result takes the divisor's sign (floor modulo)
                    return Ok(Value::Int(if r != 0 && *q < 0 { r + q } else { r }));
                }
            }
            let by = big(&y);
            if by.is_zero() {
                return Err(zero());
            }
            Ok(Value::from_big(big(&x).mod_floor(&by)))
        }
        BinOp::Pow => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported(op, a, b))?;
            if !is_float(&x) && !is_float(&y) {
                let e = big(&y);
                if !e.is_negative() {
                    if let (Num::I(p), Some(q)) = (&x, e.to_u32()) {
                        if let Some(r) = p.checked_pow(q) {
                            return Ok(Value::Int(r));
                        }
                    }
                    let base = big(&x);
                    if base.abs() <= BigInt::from(1) {
                        // 0, 1 and -1 stay small for any exponent
                        let b = base.to_i64().unwrap();
                        let r = if e.is_zero() || b == 1 {
                            1
                        } else if b == 0 {
                            0
                        } else if e.is_even() {
                            1
                        } else {
                            -1
                        };
                        return Ok(Value::Int(r));
                    }
                    // cap results at roughly 64 million bits
                    let exp = e
                        .to_u32()
                        .filter(|e| (base.bits()).saturating_mul(*e as u64) < 64_000_000)
                        .ok_or_else(too_big)?;
                    return Ok(Value::from_big(num_traits::pow(base, exp as usize)));
                }
            }
            let (fx, fy) = (to_f64(&x)?, to_f64(&y)?);
            if fx == 0.0 && fy < 0.0 {
                return Err(AdarshError::runtime("0 ki negative power nahi hoti (zero se divide jaisa)."));
            }
            if fx < 0.0 && fy.fract() != 0.0 {
                return Err(AdarshError::runtime("Negative number ki fractional power real number nahi hoti."));
            }
            let r = fx.powf(fy);
            if r.is_infinite() && fx.is_finite() && fy.is_finite() {
                return Err(too_big());
            }
            Ok(Value::Float(r))
        }
        _ => unreachable!("not an arithmetic operator"),
    }
}

fn nums(a: &Value, b: &Value) -> Option<(Num, Num)> {
    Some((as_num(a)?, as_num(b)?))
}

pub fn negate(v: &Value) -> R<Value> {
    match as_num(v) {
        Some(Num::I(i)) => Ok(i.checked_neg().map(Value::Int).unwrap_or_else(|| Value::from_big(-BigInt::from(i)))),
        Some(Num::B(b)) => Ok(Value::from_big(-b)),
        Some(Num::F(f)) => Ok(Value::Float(-f)),
        None => Err(type_err(format!("'-' {} par nahi chalta, sirf number par.", v.type_desc()))),
    }
}

pub fn abs_value(v: &Value) -> R<Value> {
    match as_num(v) {
        Some(Num::I(i)) => {
            Ok(i.checked_abs().map(Value::Int).unwrap_or_else(|| Value::from_big(BigInt::from(i).abs())))
        }
        Some(Num::B(b)) => Ok(Value::from_big(b.abs())),
        Some(Num::F(f)) => Ok(Value::Float(f.abs())),
        None => Err(type_err(format!("abs() ko number chahiye, {} mila.", v.type_desc()))),
    }
}

/// Convert a float to an int value (nan/inf are errors).
pub fn float_to_int(f: f64) -> R<Value> {
    if !f.is_finite() {
        return Err(AdarshError::runtime(format!("{} ko poore number mein nahi badal sakte.", float_repr(f))));
    }
    if f.abs() < 9.0e18 {
        Ok(Value::Int(f as i64))
    } else {
        Ok(Value::from_big(BigInt::from_f64(f).unwrap_or_default()))
    }
}

// ---------------------------------------------------------------- equality & ordering

pub fn num_cmp(a: &Num, b: &Num) -> Option<Ordering> {
    match (a, b) {
        (Num::I(x), Num::I(y)) => Some(x.cmp(y)),
        (Num::F(x), Num::F(y)) => x.partial_cmp(y),
        (Num::F(x), other) => {
            if x.is_nan() {
                return None;
            }
            if x.is_infinite() {
                return Some(if *x > 0.0 { Ordering::Greater } else { Ordering::Less });
            }
            if x.fract() == 0.0 {
                BigInt::from_f64(*x).map(|bx| bx.cmp(&big(other)))
            } else {
                let o = match other {
                    Num::I(i) => *i as f64,
                    Num::B(b) => b.to_f64().unwrap_or(if b.is_negative() { f64::NEG_INFINITY } else { f64::INFINITY }),
                    Num::F(_) => unreachable!(),
                };
                x.partial_cmp(&o)
            }
        }
        (other, Num::F(_)) => num_cmp(b, other).map(Ordering::reverse),
        _ => Some(big(a).cmp(&big(b))),
    }
}

/// Structural equality (`==`).
pub fn values_eq(a: &Value, b: &Value) -> bool {
    if let (Some(x), Some(y)) = (as_num(a), as_num(b)) {
        return num_cmp(&x, &y) == Some(Ordering::Equal);
    }
    match (a, b) {
        (Value::Null, Value::Null) => true,
        (Value::Str(x), Value::Str(y)) => x == y,
        (Value::List(x), Value::List(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| values_eq(p, q))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let (x, y) = (x.borrow(), y.borrow());
            x.type_name == y.type_name
                && x.len() == y.len()
                && x.entries().iter().all(|(k, v)| matches!(y.get(k), Ok(Some(w)) if values_eq(v, &w)))
        }
        (Value::Func(x), Value::Func(y)) => Rc::ptr_eq(x, y),
        (Value::Method(x), Value::Method(y)) => Rc::ptr_eq(&x.func, &y.func) && values_eq(&x.receiver, &y.receiver),
        (Value::Builtin(x), Value::Builtin(y)) => x == y,
        (Value::Tuple(x), Value::Tuple(y)) => {
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| values_eq(p, q))
        }
        (Value::Set(x), Value::Set(y)) => {
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len() && x.entries().iter().all(|(k, _)| y.contains(k).unwrap_or(false))
        }
        (Value::Enum(x), Value::Enum(y)) => x.type_name == y.type_name && x.index == y.index,
        (Value::EnumType(x), Value::EnumType(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// `< > <= >=` on numbers, strings, and lists (lexicographic).
pub fn compare(op: BinOp, a: &Value, b: &Value) -> R<bool> {
    let test = |o: Option<Ordering>| -> bool {
        match o {
            None => false,
            Some(o) => match op {
                BinOp::Lt => o == Ordering::Less,
                BinOp::Gt => o == Ordering::Greater,
                BinOp::Lte => o != Ordering::Greater,
                BinOp::Gte => o != Ordering::Less,
                _ => unreachable!(),
            },
        }
    };
    if let (Some(x), Some(y)) = (as_num(a), as_num(b)) {
        return Ok(test(num_cmp(&x, &y)));
    }
    match (a, b) {
        (Value::Str(x), Value::Str(y)) => Ok(test(Some(x.cmp(y)))),
        (Value::Tuple(x), Value::Tuple(y)) => {
            for (p, q) in x.iter().zip(y.iter()) {
                if !values_eq(p, q) {
                    return compare(op, p, q);
                }
            }
            Ok(test(Some(x.len().cmp(&y.len()))))
        }
        (Value::Enum(x), Value::Enum(y)) if x.type_name == y.type_name => Ok(test(Some(x.index.cmp(&y.index)))),
        (Value::List(x), Value::List(y)) => {
            let (xs, ys) = (x.borrow().clone(), y.borrow().clone());
            for (p, q) in xs.iter().zip(ys.iter()) {
                if !values_eq(p, q) {
                    return compare(op, p, q);
                }
            }
            Ok(test(Some(xs.len().cmp(&ys.len()))))
        }
        _ => Err(type_err(format!(
            "'{}' se {} aur {} ko compare nahi kar sakte.",
            op.symbol(),
            a.type_desc(),
            b.type_desc()
        ))),
    }
}

// ---------------------------------------------------------------- conversions

fn strip_underscores(s: &str) -> Option<String> {
    if !s.contains('_') {
        return Some(s.to_string());
    }
    let cs: Vec<char> = s.chars().collect();
    for (i, c) in cs.iter().enumerate() {
        if *c == '_' {
            let ok = i > 0 && i + 1 < cs.len() && cs[i - 1].is_ascii_digit() && cs[i + 1].is_ascii_digit();
            if !ok {
                return None;
            }
        }
    }
    Some(s.replace('_', ""))
}

/// Parse an integer from text (`" 42 "`, `-7`, `1_000`).
pub fn parse_int(s: &str) -> Option<Value> {
    let t = s.trim();
    let body = t.strip_prefix(['+', '-']).unwrap_or(t);
    if body.is_empty() || !body.chars().all(|c| c.is_ascii_digit() || c == '_') {
        return None;
    }
    let clean = strip_underscores(t)?;
    match clean.parse::<i64>() {
        Ok(i) => Some(Value::Int(i)),
        Err(_) => clean.parse::<BigInt>().ok().map(Value::from_big),
    }
}

/// Parse a float from text (`3.5`, `.5`, `1e5`, `inf`).
pub fn parse_float(s: &str) -> Option<f64> {
    let clean = strip_underscores(s.trim())?;
    let lower = clean.to_ascii_lowercase();
    let body = lower.strip_prefix(['+', '-']).unwrap_or(&lower);
    if !(body == "inf" || body == "infinity" || body == "nan")
        && !body.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == '+' || c == '-')
    {
        return None;
    }
    clean.parse::<f64>().ok()
}

/// Convert to a whole number (`int()`): truncates floats, parses strings.
pub fn to_int(v: &Value, what: &str) -> R<Value> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(f) => float_to_int(*f),
        Value::Str(s) => {
            parse_int(s).ok_or_else(|| type_err(format!("{what} ko poora number chahiye, {} mila.", repr(v))))
        }
        _ => Err(type_err(format!("{what} ko poora number chahiye, {} mila.", v.type_desc()))),
    }
}
