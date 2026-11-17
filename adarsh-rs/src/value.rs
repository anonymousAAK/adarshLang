//! Runtime values with Python-compatible semantics (printing, equality,
//! ordering, truthiness, arithmetic), mirroring how the reference
//! implementation leaned on Python objects.

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
        write!(f, "<kaam {}>", self.def.name.as_deref().unwrap_or("<anonymous>"))
    }
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
    Dict(Rc<RefCell<Dict>>),
    Func(Rc<Closure>),
    Builtin(usize),
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

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "NoneType",
            Value::Bool(_) => "bool",
            Value::Int(_) | Value::Big(_) => "int",
            Value::Float(_) => "float",
            Value::Str(_) => "str",
            Value::List(_) => "list",
            Value::Dict(_) => "dict",
            Value::Func(_) => "AdarshFunctionValue",
            Value::Builtin(_) => "AdarshBuiltinFunction",
        }
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
            Value::Dict(d) => !d.borrow().is_empty(),
            Value::Func(_) | Value::Builtin(_) => true,
        }
    }

    pub fn is_callable(&self) -> bool {
        matches!(self, Value::Func(_) | Value::Builtin(_))
    }
}

// ---------------------------------------------------------------- dict

/// Hash key normalised the way Python hashes: `True == 1 == 1.0` share a slot.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HKey {
    Null,
    Int(i64),
    Big(BigInt),
    Float(u64),
    Str(Rc<str>),
    Ptr(usize),
}

pub fn hash_key(v: &Value) -> R<HKey> {
    Ok(match v {
        Value::Null => HKey::Null,
        Value::Bool(b) => HKey::Int(*b as i64),
        Value::Int(i) => HKey::Int(*i),
        Value::Big(b) => HKey::Big((**b).clone()),
        Value::Float(f) => {
            if f.is_finite() && f.fract() == 0.0 {
                if *f >= -9.2e18 && *f <= 9.2e18 {
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
        Value::Builtin(i) => HKey::Ptr(*i),
        Value::List(_) | Value::Dict(_) => {
            return Err(AdarshError::type_error(format!("unhashable type: '{}'", v.type_name())))
        }
    })
}

/// Insertion-ordered dictionary (Python 3.7+ semantics). Deletion is not part
/// of the language, so a vector plus an index map is enough.
#[derive(Default, Debug)]
pub struct Dict {
    entries: Vec<(Value, Value)>,
    index: HashMap<HKey, usize>,
}

impl Dict {
    pub fn new() -> Self {
        Self::default()
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
        self.insert(Value::str(k), v).expect("str keys are hashable")
    }
    pub fn entries(&self) -> &[(Value, Value)] {
        &self.entries
    }
    pub fn keys(&self) -> Vec<Value> {
        self.entries.iter().map(|(k, _)| k.clone()).collect()
    }
}

// ---------------------------------------------------------------- printing

/// Python's `repr(float)`: shortest round-trip digits, scientific outside 1e-4..1e16.
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
    let neg = mant.starts_with('-');
    let digits: String = mant.chars().filter(|c| c.is_ascii_digit()).collect();
    let mut out = String::new();
    if neg {
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

fn str_repr(s: &str, out: &mut String) {
    let quote = if s.contains('\'') && !s.contains('"') { '"' } else { '\'' };
    out.push(quote);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c if c.is_control() => {
                let n = c as u32;
                if n < 0x100 {
                    let _ = write!(out, "\\x{:02x}", n);
                } else if n < 0x10000 {
                    let _ = write!(out, "\\u{:04x}", n);
                } else {
                    let _ = write!(out, "\\U{:08x}", n);
                }
            }
            c => out.push(c),
        }
    }
    out.push(quote);
}

fn write_value(v: &Value, repr: bool, out: &mut String, seen: &mut Vec<usize>, names: &dyn Fn(usize) -> &'static str) {
    match v {
        Value::Null => out.push_str("None"),
        Value::Bool(b) => out.push_str(if *b { "True" } else { "False" }),
        Value::Int(i) => {
            let _ = write!(out, "{i}");
        }
        Value::Big(b) => {
            let _ = write!(out, "{b}");
        }
        Value::Float(f) => out.push_str(&float_repr(*f)),
        Value::Str(s) => {
            if repr {
                str_repr(s, out)
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
                write_value(item, true, out, seen, names);
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
            seen.push(id);
            out.push('{');
            for (i, (k, val)) in d.borrow().entries().iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_value(k, true, out, seen, names);
                out.push_str(": ");
                write_value(val, true, out, seen, names);
            }
            out.push('}');
            seen.pop();
        }
        Value::Func(c) => {
            let _ = write!(out, "<kaam {}>", c.def.name.as_deref().unwrap_or("<anonymous>"));
        }
        Value::Builtin(i) => {
            let _ = write!(out, "<builtin kaam {}>", names(*i));
        }
    }
}

/// Python `str(value)`.
pub fn py_str(v: &Value) -> String {
    if let Value::Str(s) = v {
        return s.to_string();
    }
    let mut out = String::new();
    write_value(v, false, &mut out, &mut Vec::new(), &crate::builtins::builtin_name);
    out
}

/// Python `repr(value)`.
pub fn py_repr(v: &Value) -> String {
    let mut out = String::new();
    write_value(v, true, &mut out, &mut Vec::new(), &crate::builtins::builtin_name);
    out
}

/// What `dikhao` prints: `khali` / `sahi_hai_be` / `jhuth` at top level, else `str()`.
pub fn display(v: &Value) -> String {
    match v {
        Value::Null => "khali".into(),
        Value::Bool(true) => "sahi_hai_be".into(),
        Value::Bool(false) => "jhuth".into(),
        other => py_str(other),
    }
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

pub fn big_to_f64(b: &BigInt) -> R<f64> {
    match b.to_f64() {
        Some(f) if f.is_finite() => Ok(f),
        _ => Err(AdarshError::overflow("int too large to convert to float")),
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

fn unsupported(op: &str, a: &Value, b: &Value) -> AdarshError {
    AdarshError::type_error(format!(
        "unsupported operand type(s) for {op}: '{}' and '{}'",
        a.type_name(),
        b.type_name()
    ))
}

fn int_true_div(a: &Num, b: &Num) -> R<f64> {
    const EXACT: i64 = 1 << 53;
    if let (Num::I(x), Num::I(y)) = (a, b) {
        if x.abs() <= EXACT && y.abs() <= EXACT {
            return Ok(*x as f64 / *y as f64);
        }
    }
    // Correctly rounded a / b for arbitrary ints (what CPython does).
    let (x, y) = (big(a), big(b));
    let negative = x.is_negative() != y.is_negative();
    let (x, y) = (x.abs(), y.abs());
    if x.is_zero() {
        return Ok(if negative { -0.0 } else { 0.0 });
    }
    // scale so the integer quotient has at least 64 significant bits
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
    // 55-bit mantissa: 53 bits + round bit + sticky bit, rounded by the u64 -> f64 conversion
    let m = q.to_u64().unwrap() | sticky as u64;
    let exp = shift.max(0) - k;
    let mut f = m as f64;
    // apply 2^exp in steps to avoid intermediate overflow/underflow
    let mut e = exp;
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
        return Err(AdarshError::overflow("integer division result too large for a float"));
    }
    Ok(if negative { -f } else { f })
}

fn py_fmod(a: f64, b: f64) -> f64 {
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
                (Value::Str(_), _) => {
                    return Err(AdarshError::type_error(format!(
                        "can only concatenate str (not \"{}\") to str",
                        b.type_name()
                    )))
                }
                (Value::List(_), _) => {
                    return Err(AdarshError::type_error(format!(
                        "can only concatenate list (not \"{}\") to list",
                        b.type_name()
                    )))
                }
                _ => {}
            }
            let (x, y) = nums(a, b).ok_or_else(|| unsupported("+", a, b))?;
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
            let (x, y) = nums(a, b).ok_or_else(|| unsupported("-", a, b))?;
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
                    let k = repeat_count(n).unwrap();
                    return Ok(Value::Str(s.repeat(k.max(0) as usize).into()));
                }
                (Value::List(l), n) | (n, Value::List(l)) if repeat_count(n).is_some() => {
                    let k = repeat_count(n).unwrap().max(0) as usize;
                    let src = l.borrow();
                    let mut v = Vec::with_capacity(src.len() * k);
                    for _ in 0..k {
                        v.extend(src.iter().cloned());
                    }
                    return Ok(Value::list(v));
                }
                _ => {}
            }
            let (x, y) = nums(a, b).ok_or_else(|| {
                if matches!(a, Value::Str(_) | Value::List(_)) || matches!(b, Value::Str(_) | Value::List(_)) {
                    let other = if matches!(a, Value::Str(_) | Value::List(_)) { b } else { a };
                    AdarshError::type_error(format!(
                        "can't multiply sequence by non-int of type '{}'",
                        other.type_name()
                    ))
                } else {
                    unsupported("*", a, b)
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
            let (x, y) = nums(a, b).ok_or_else(|| unsupported("/", a, b))?;
            if is_float(&x) || is_float(&y) {
                let d = to_f64(&y)?;
                if d == 0.0 {
                    return Err(AdarshError::zero_div("float division by zero"));
                }
                return Ok(Value::Float(to_f64(&x)? / d));
            }
            if big(&y).is_zero() {
                return Err(AdarshError::zero_div("division by zero"));
            }
            Ok(Value::Float(int_true_div(&x, &y)?))
        }
        BinOp::Mod => {
            if let Value::Str(_) = a {
                return Err(AdarshError::type_error("not all arguments converted during string formatting"));
            }
            let (x, y) = nums(a, b).ok_or_else(|| unsupported("%", a, b))?;
            if is_float(&x) || is_float(&y) {
                let d = to_f64(&y)?;
                if d == 0.0 {
                    return Err(AdarshError::zero_div("float modulo by zero"));
                }
                return Ok(Value::Float(py_fmod(to_f64(&x)?, d)));
            }
            if let (Num::I(p), Num::I(q)) = (&x, &y) {
                if *q == 0 {
                    return Err(AdarshError::zero_div("integer modulo by zero"));
                }
                if let Some(r) = p.checked_rem_euclid(*q) {
                    // rem_euclid is always >= 0; Python's result takes the divisor's sign.
                    let r = if r != 0 && *q < 0 { r + q } else { r };
                    return Ok(Value::Int(r));
                }
            }
            let (bx, by) = (big(&x), big(&y));
            if by.is_zero() {
                return Err(AdarshError::zero_div("integer modulo by zero"));
            }
            Ok(Value::from_big(bx.mod_floor(&by)))
        }
        BinOp::Pow => {
            let (x, y) = nums(a, b).ok_or_else(|| unsupported("** or pow()", a, b))?;
            if !is_float(&x) && !is_float(&y) {
                let e = big(&y);
                if !e.is_negative() {
                    let base = big(&x);
                    if let (Num::I(p), Some(q)) = (&x, e.to_u32()) {
                        if let Some(r) = p.checked_pow(q) {
                            return Ok(Value::Int(r));
                        }
                    }
                    let exp = e
                        .to_u32()
                        .ok_or_else(|| AdarshError::Host(crate::error::HostKind::MemoryError, String::new()))?;
                    return Ok(Value::from_big(num_traits::pow(base, exp as usize)));
                }
            }
            let (fx, fy) = (to_f64(&x)?, to_f64(&y)?);
            if fx == 0.0 && fy < 0.0 {
                return Err(AdarshError::zero_div("0.0 cannot be raised to a negative power"));
            }
            let r = fx.powf(fy);
            if r.is_infinite() && fx.is_finite() && fy.is_finite() {
                return Err(AdarshError::overflow("(34, 'Numerical result out of range')"));
            }
            Ok(Value::Float(r))
        }
        _ => Err(AdarshError::runtime(format!("Unknown operator '{}' in AdarshLang.", op.symbol()))),
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
        None => Err(AdarshError::type_error(format!("bad operand type for unary -: '{}'", v.type_name()))),
    }
}

pub fn abs_value(v: &Value) -> R<Value> {
    match as_num(v) {
        Some(Num::I(i)) => Ok(i.checked_abs().map(Value::Int).unwrap_or_else(|| Value::from_big(BigInt::from(i).abs()))),
        Some(Num::B(b)) => Ok(Value::from_big(b.abs())),
        Some(Num::F(f)) => Ok(Value::Float(f.abs())),
        None => Err(AdarshError::type_error(format!("bad operand type for abs(): '{}'", v.type_name()))),
    }
}

/// Convert a float to an int value, with Python's errors for nan/inf.
pub fn float_to_int(f: f64) -> R<Value> {
    if f.is_nan() {
        return Err(AdarshError::value_error("cannot convert float NaN to integer"));
    }
    if f.is_infinite() {
        return Err(AdarshError::overflow("cannot convert float infinity to integer"));
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
            // compare exactly via big ints when x is integral, else via f64
            if x.fract() == 0.0 {
                BigInt::from_f64(*x).map(|bx| bx.cmp(&big(other)))
            } else {
                let o = match other {
                    Num::I(i) => *i as f64,
                    Num::B(b) => b.to_f64().unwrap_or(f64::INFINITY * if b.is_negative() { -1.0 } else { 1.0 }),
                    Num::F(_) => unreachable!(),
                };
                x.partial_cmp(&o)
            }
        }
        (other, Num::F(_)) => num_cmp(b, other).map(Ordering::reverse),
        _ => Some(big(a).cmp(&big(b))),
    }
}

/// Python `==`.
pub fn py_eq(a: &Value, b: &Value) -> bool {
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
            x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| identical(p, q) || py_eq(p, q))
        }
        (Value::Dict(x), Value::Dict(y)) => {
            if Rc::ptr_eq(x, y) {
                return true;
            }
            let (x, y) = (x.borrow(), y.borrow());
            x.len() == y.len()
                && x.entries().iter().all(|(k, v)| match y.get(k) {
                    Ok(Some(w)) => identical(v, &w) || py_eq(v, &w),
                    _ => false,
                })
        }
        (Value::Func(x), Value::Func(y)) => Rc::ptr_eq(x, y),
        (Value::Builtin(x), Value::Builtin(y)) => x == y,
        _ => false,
    }
}

/// Python's `is` for container elements (identity shortcut used by list/dict `==`).
fn identical(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::List(x), Value::List(y)) => Rc::ptr_eq(x, y),
        (Value::Dict(x), Value::Dict(y)) => Rc::ptr_eq(x, y),
        (Value::Func(x), Value::Func(y)) => Rc::ptr_eq(x, y),
        _ => false,
    }
}

/// Python rich comparison for `< > <= >=`.
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
        (Value::List(x), Value::List(y)) => {
            let (xs, ys) = (x.borrow().clone(), y.borrow().clone());
            for (p, q) in xs.iter().zip(ys.iter()) {
                if !(identical(p, q) || py_eq(p, q)) {
                    return compare(op, p, q);
                }
            }
            Ok(test(Some(xs.len().cmp(&ys.len()))))
        }
        _ => Err(AdarshError::type_error(format!(
            "'{}' not supported between instances of '{}' and '{}'",
            op.symbol(),
            a.type_name(),
            b.type_name()
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

/// Python `int(str)` (base 10).
pub fn parse_py_int(s: &str) -> Option<Value> {
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

/// Python `float(str)`.
pub fn parse_py_float(s: &str) -> Option<f64> {
    let t = s.trim();
    let clean = strip_underscores(t)?;
    let lower = clean.to_ascii_lowercase();
    let body = lower.strip_prefix(['+', '-']).unwrap_or(&lower);
    if !(body == "inf" || body == "infinity" || body == "nan")
        && !body.chars().all(|c| c.is_ascii_digit() || c == '.' || c == 'e' || c == '+' || c == '-')
    {
        return None;
    }
    clean.parse::<f64>().ok()
}

/// Python `int(value)`.
pub fn py_int(v: &Value) -> R<Value> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(f) => float_to_int(*f),
        Value::Str(s) => parse_py_int(s).ok_or_else(|| {
            AdarshError::value_error(format!("invalid literal for int() with base 10: {}", py_repr(v)))
        }),
        _ => Err(AdarshError::type_error(format!(
            "int() argument must be a string, a bytes-like object or a real number, not '{}'",
            v.type_name()
        ))),
    }
}
