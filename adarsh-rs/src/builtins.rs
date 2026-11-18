//! Builtin functions. Each one has named parameters, so both `sort(xs, by: f)`
//! and dot-calls like `xs.sort(by: f)` / `naam.upper()` work.

use crate::ast::BinOp;
use crate::error::{AdarshError, ErrorKind};
use crate::interp::Interpreter;
use crate::semantics::Signature;
use crate::stdlib::*;
pub use crate::stdlib::{default_host, format_spec, Host, Turtle};
use crate::value::*;
use num_integer::Integer;
use std::rc::Rc;

/// Default value of an optional builtin parameter.
#[derive(Clone, Copy)]
pub enum Def {
    Int(i64),
    Str(&'static str),
    Null,
    False,
}

impl Def {
    fn value(self) -> Value {
        match self {
            Def::Int(i) => Value::Int(i),
            Def::Str(s) => Value::str(s),
            Def::Null => Value::Null,
            Def::False => Value::Bool(false),
        }
    }
}

pub struct BuiltinSpec {
    pub name: &'static str,
    pub params: &'static [&'static str],
    pub required: usize,
    pub defaults: &'static [Def],
    pub doc: &'static str,
}

macro_rules! b {
    ($name:literal, [$($p:literal),*], $req:literal, $doc:literal) => {
        BuiltinSpec { name: $name, params: &[$($p),*], required: $req, defaults: &[], doc: $doc }
    };
    ($name:literal, [$($p:literal),*], $req:literal, [$($d:expr),*], $doc:literal) => {
        BuiltinSpec { name: $name, params: &[$($p),*], required: $req, defaults: &[$($d),*], doc: $doc }
    };
}

pub static BUILTINS: &[BuiltinSpec] = &[
    // ---- collections
    b!("length", ["cheez"], 1, "List, string, dict, tuple ya set ki length."),
    b!("push", ["list", "value"], 2, "List (ya set) ke ant mein value jodo."),
    b!("pop", ["list", "index"], 1, [Def::Int(-1)], "List se aakhri (ya index waali) value nikaalo."),
    b!("insert", ["list", "index", "value"], 3, "List mein index par value daalo."),
    b!("rakho", ["dict", "key", "value"], 3, "Dict mein key par value rakho."),
    b!("nikalo", ["cheez", "key", "default"], 2, [Def::Null], "Dict/list se value lo; na mile to default."),
    b!("hatao", ["cheez", "key"], 2, "Dict se key, list se index, ya set se value hatao."),
    b!("keys", ["dict"], 1, "Dict ki saari keys (list)."),
    b!("values", ["dict"], 1, "Dict ki saari values (list)."),
    b!("contains", ["cheez", "value"], 2, "Kya value andar hai? (in jaisa)"),
    b!("find", ["cheez", "value"], 2, "Value ka pehla index; na mile to -1."),
    b!("count", ["cheez", "value"], 2, "Value kitni baar aati hai."),
    b!(
        "sort",
        ["list", "by", "ulta"],
        1,
        [Def::Null, Def::False],
        "Sorted nayi list. by: kaam se key, ulta: sahi_hai_be se ulti."
    ),
    b!("reverse", ["cheez"], 1, "Ulti list ya string."),
    b!("slice", ["cheez", "start", "end"], 1, [Def::Int(0), Def::Null], "start se end tak ka hissa."),
    b!("range", ["start", "end", "step"], 1, [Def::Null, Def::Int(1)], "Numbers ki list: range(5), range(1, 10, 2)."),
    b!("join", ["items", "sep"], 1, [Def::Str("")], "Items ko sep ke saath jodkar string."),
    b!("map", ["kaam", "items"], 2, "Har item par kaam chalao, nayi list."),
    b!("filter", ["kaam", "items"], 2, "Sirf woh items jin par kaam sahi_hai_be de."),
    b!("reduce", ["kaam", "items", "shuru"], 2, [Def::Null], "Items ko ek value mein jodo."),
    b!("sum", ["items", "shuru"], 1, [Def::Int(0)], "Saare numbers ka jod."),
    b!("min", ["items", "by"], 1, [Def::Null], "Sabse chhoti value."),
    b!("max", ["items", "by"], 1, [Def::Null], "Sabse badi value."),
    b!("any", ["items", "kaam"], 1, [Def::Null], "Kya koi bhi item sahi hai?"),
    b!("all", ["items", "kaam"], 1, [Def::Null], "Kya saare items sahi hain?"),
    b!("zip", ["a", "b"], 2, "Do lists ko jodiyon (tuples) mein milao."),
    b!("unique", ["items"], 1, "Duplicate hata kar list (pehli baar ka kram)."),
    b!("flatten", ["items"], 1, "List ke andar ki lists ko ek list banao."),
    b!("group_by", ["items", "kaam"], 2, "kaam ke result ke hisaab se dict mein baanto."),
    b!("list", ["items"], 0, [Def::Null], "Nayi list (tuple/set/string se bhi)."),
    b!("tuple", ["items"], 0, [Def::Null], "Tuple banao."),
    b!("set", ["items"], 0, [Def::Null], "Set banao (unique values)."),
    b!("union", ["a", "b"], 2, "Dono sets ki saari values."),
    b!("intersection", ["a", "b"], 2, "Dono sets mein common values."),
    b!("difference", ["a", "b"], 2, "a mein hain par b mein nahi."),
    // ---- numbers
    b!("abs", ["x"], 1, "Bina minus ki value."),
    b!("floor", ["x"], 1, "Neeche ka poora number."),
    b!("ceil", ["x"], 1, "Upar ka poora number."),
    b!("round_val", ["x"], 1, "Paas ka poora number (2.5 → 2, banker's rounding)."),
    b!("round", ["x", "ank"], 1, [Def::Int(0)], "ank decimal places tak round."),
    b!("min_val", ["a", "b"], 2, "Do mein chhota."),
    b!("max_val", ["a", "b"], 2, "Do mein bada."),
    b!("sqrt", ["x"], 1, "Square root."),
    b!("sin", ["x"], 1, "sin (radians)."),
    b!("cos", ["x"], 1, "cos (radians)."),
    b!("tan", ["x"], 1, "tan (radians)."),
    b!("asin", ["x"], 1, "sin ulta (radians)."),
    b!("acos", ["x"], 1, "cos ulta (radians)."),
    b!("atan", ["x"], 1, "tan ulta (radians)."),
    b!("log", ["x", "base"], 1, [Def::Null], "Log (base na do to natural log)."),
    b!("exp", ["x"], 1, "e ki power x."),
    b!("gcd", ["a", "b"], 2, "Sabse bada common divisor."),
    b!("lcm", ["a", "b"], 2, "Sabse chhota common multiple."),
    b!("is_prime", ["n"], 1, "Kya n prime hai?"),
    b!("random_number", ["start", "end"], 0, [Def::Int(0), Def::Int(1)], "start aur end ke beech random decimal."),
    b!("random_int", ["start", "end"], 2, "start se end tak random poora number."),
    b!("random_choice", ["items"], 1, "List se koi ek random item."),
    b!("shuffle", ["list"], 1, "List ko randomly mila do (wahi list)."),
    // ---- strings
    b!("upper", ["text"], 1, "BADE akshar."),
    b!("lower", ["text"], 1, "chhote akshar."),
    b!("title", ["text"], 1, "Har Shabd Ka Pehla Akshar Bada."),
    b!("trim", ["text"], 1, "Shuru aur ant ki khali jagah hatao."),
    b!("split", ["text", "sep"], 1, [Def::Str(" ")], "String ko sep par todo."),
    b!("replace", ["text", "purana", "naya"], 3, "Purane hisse ko naye se badlo."),
    b!("starts_with", ["text", "shuru"], 2, "Kya text isse shuru hota hai?"),
    b!("ends_with", ["text", "ant"], 2, "Kya text isse khatam hota hai?"),
    b!("pad_left", ["text", "width", "fill"], 2, [Def::Str(" ")], "Left mein fill bharkar width tak."),
    b!("pad_right", ["text", "width", "fill"], 2, [Def::Str(" ")], "Right mein fill bharkar width tak."),
    b!("repeat", ["text", "n"], 2, "Text n baar."),
    b!("char_code", ["akshar"], 1, "Akshar ka Unicode number."),
    b!("from_code", ["n"], 1, "Unicode number se akshar."),
    b!("format", ["value", "spec"], 2, "Format karo: format(3.14159, \".2\") → \"3.14\"."),
    b!("lakh_format", ["n"], 1, "Indian style: 1500000 → \"15,00,000\"."),
    b!("rupaye", ["n"], 1, "Rupaye: 99.5 → \"₹99.50\"."),
    b!("shabdon_mein", ["n"], 1, "Number shabdon mein: 125 → \"ek sau pachchees\"."),
    // ---- types
    b!("prakar", ["value"], 1, "Value ka type ka naam."),
    b!("shabdme", ["value"], 1, "Value ko string banao."),
    b!("sankhya", ["value"], 1, "String ko number banao."),
    // ---- input / output / system
    b!("padho", ["sawaal"], 0, [Def::Str("")], "Ek line input padho (na ho to khali)."),
    b!("padho_number", ["sawaal"], 0, [Def::Str("")], "Number input padho; galat ho to dobara poochho."),
    b!("rang_mein", ["text", "rang"], 2, "Terminal mein rangeen text (laal, hara, peela, neela...)."),
    b!("current_time", [], 0, "1970 se ab tak ke seconds."),
    b!("ruko", ["second"], 1, "Itne second ruko."),
    b!("band_karo", ["code"], 0, [Def::Int(0)], "Program yahin band karo."),
    b!("jaancho", ["sharat", "sandesh"], 1, [Def::Str("")], "Test: sharat jhuth ho to galti do."),
    // ---- dates
    b!("aaj", [], 0, "Aaj ki tareekh (Tareekh { saal, mahina, din })."),
    b!("abhi", [], 0, "Abhi ka samay (Samay { saal, mahina, din, ghanta, minute, second })."),
    b!("tareekh", ["saal", "mahina", "din"], 3, "Tareekh banao: tareekh(2025, 8, 15)."),
    b!("din_ka_naam", ["tareekh"], 0, [Def::Null], "Hafte ka din: Somvaar, Mangalvaar..."),
    b!("tareekh_jodo", ["tareekh", "din"], 2, "Tareekh mein din jodo (minus bhi chalta hai)."),
    b!("din_ka_fark", ["a", "b"], 2, "Do tareekhon ke beech kitne din."),
    // ---- data / files / internet
    b!("json_banao", ["value", "sundar"], 1, [Def::False], "Value ko JSON text banao."),
    b!("json_padho", ["text"], 1, "JSON text se value banao."),
    b!("file_padho", ["path"], 1, "File ka text padho (CLI)."),
    b!("file_likho", ["path", "text"], 2, "File mein text likho, purana mit jaata hai (CLI)."),
    b!("file_jodo", ["path", "text"], 2, "File ke ant mein text jodo (CLI)."),
    b!("file_hai", ["path"], 1, "Kya file maujood hai? (CLI)"),
    b!("internet_se", ["url"], 1, "URL ka text download karo (CLI)."),
    // ---- turtle graphics (also as kachhua.aage(...) etc.)
    b!("kachhua_aage", ["kadam"], 1, "Kachhua aage chale."),
    b!("kachhua_peeche", ["kadam"], 1, "Kachhua peeche chale."),
    b!("kachhua_daayen", ["degree"], 1, "Kachhua daayein mude."),
    b!("kachhua_baayen", ["degree"], 1, "Kachhua baayein mude."),
    b!("kachhua_kalam_upar", [], 0, "Kalam utha lo (chalega par line nahi banegi)."),
    b!("kachhua_kalam_neeche", [], 0, "Kalam rakh do (line banegi)."),
    b!("kachhua_rang", ["rang"], 1, "Line ka rang: laal, hara, neela... ya #hex."),
    b!("kachhua_mota", ["width"], 1, "Line ki motai."),
    b!("kachhua_jaao", ["x", "y"], 2, "Seedha (x, y) par jaao."),
    b!("kachhua_ghar", [], 0, "Beech (0, 0) par wapas, upar ki taraf."),
];

/// Names other than functions that every program can use.
pub const GLOBAL_VALUES: &[&str] = &["PI", "E", "kachhua"];

pub fn global_values() -> Vec<(&'static str, Value)> {
    let mut turtle = Dict::new();
    for (id, spec) in BUILTINS.iter().enumerate() {
        if let Some(short) = spec.name.strip_prefix("kachhua_") {
            turtle.insert_str(short, Value::Builtin(id));
        }
    }
    vec![
        ("PI", Value::Float(std::f64::consts::PI)),
        ("E", Value::Float(std::f64::consts::E)),
        ("kachhua", Value::dict(turtle)),
    ]
}

pub fn builtin_name(id: usize) -> &'static str {
    BUILTINS.get(id).map(|b| b.name).unwrap_or("?")
}

pub fn builtin_spec(name: &str) -> Option<&'static BuiltinSpec> {
    BUILTINS.iter().find(|b| b.name == name)
}

pub fn signatures() -> impl Iterator<Item = (&'static str, Signature)> {
    BUILTINS.iter().map(|b| (b.name, Signature::from_counts(b.required, b.required + b.defaults.len())))
}

#[cfg(target_arch = "wasm32")]
pub fn wasm_clock() -> f64 {
    crate::wasm::now()
}

fn err(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

/// "'push' ko pehla argument list chahiye, number mila."
fn wants(func: &str, nth: &str, what: &str, got: &Value) -> AdarshError {
    err(format!("'{func}' ko {nth} argument {what} chahiye, {} mila.", got.type_desc()))
}

pub fn is_iterable(v: &Value) -> bool {
    matches!(v, Value::List(_) | Value::Str(_) | Value::Dict(_) | Value::Tuple(_) | Value::Set(_))
}

/// What `ke_liye` and `...` visit: list/tuple/set items, string characters, dict keys.
pub fn iter_items(v: &Value) -> Option<Vec<Value>> {
    match v {
        Value::List(l) => Some(l.borrow().clone()),
        Value::Tuple(t) => Some(t.to_vec()),
        Value::Str(s) => Some(s.chars().map(|c| Value::Str(c.to_string().into())).collect()),
        Value::Dict(d) | Value::Set(d) => Some(d.borrow().keys()),
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
    err(format!("Index {} bahar hai: {hint}.", display(idx))).with_code("E011")
}

/// Positions picked by a Python-style slice `[start:end:step]`.
pub fn slice_positions(len: usize, start: Option<i64>, end: Option<i64>, step: i64) -> Vec<usize> {
    let n = len as i64;
    let clamp = |v: i64, lo: i64, hi: i64| v.max(lo).min(hi);
    let norm = |v: i64| if v < 0 { v + n } else { v };
    let mut out = Vec::new();
    if step > 0 {
        let s = clamp(start.map(norm).unwrap_or(0), 0, n);
        let e = clamp(end.map(norm).unwrap_or(n), 0, n);
        let mut i = s;
        while i < e {
            out.push(i as usize);
            i += step;
        }
    } else {
        let s = clamp(start.map(norm).unwrap_or(n - 1), -1, n - 1);
        let e = clamp(end.map(norm).unwrap_or(-1), -1, n - 1);
        let mut i = s;
        while i > e {
            out.push(i as usize);
            i += step;
        }
    }
    out
}

fn len_of(v: &Value) -> Result<usize, AdarshError> {
    match v {
        Value::Str(s) => Ok(s.chars().count()),
        Value::List(l) => Ok(l.borrow().len()),
        Value::Dict(d) | Value::Set(d) => Ok(d.borrow().len()),
        Value::Tuple(t) => Ok(t.len()),
        _ => Err(err(format!("{} ki length nahi hoti.", v.type_desc()))),
    }
}

fn need_number(func: &str, v: &Value) -> AdarshError {
    err(format!("{func}() ko number chahiye, {} mila.", v.type_desc()))
}

fn to_f(func: &str, v: &Value) -> Result<f64, AdarshError> {
    match as_num(v) {
        Some(Num::I(i)) => Ok(i as f64),
        Some(Num::B(b)) => big_to_f64(&b),
        Some(Num::F(f)) => Ok(f),
        None => Err(need_number(func, v)),
    }
}

fn to_i(func: &str, v: &Value) -> Result<i64, AdarshError> {
    match to_int(v, &format!("{func}()"))? {
        Value::Int(i) => Ok(i),
        _ => Err(err(format!("{func}() ke liye number bahut bada hai."))),
    }
}

fn round_value(func: &str, v: &Value, f: fn(f64) -> f64) -> Result<Value, AdarshError> {
    match v {
        Value::Bool(b) => Ok(Value::Int(*b as i64)),
        Value::Int(_) | Value::Big(_) => Ok(v.clone()),
        Value::Float(x) => float_to_int(f(*x)),
        _ => Err(need_number(func, v)),
    }
}

fn float_result(func: &str, x: f64) -> Result<Value, AdarshError> {
    if x.is_nan() {
        return Err(err(format!("{func}() ka jawab real number nahi hai (jaise negative ka sqrt).")));
    }
    Ok(Value::Float(x))
}

/// A list (or other collection) as a Vec, for functions that read every item.
fn items_of(func: &str, nth: &str, v: &Value) -> Result<Vec<Value>, AdarshError> {
    iter_items(v).ok_or_else(|| wants(func, nth, "list (ya tuple, set, string)", v))
}

fn new_set(items: Vec<Value>) -> Result<Value, AdarshError> {
    let mut d = Dict::new();
    for i in items {
        d.insert(i, Value::Null)?;
    }
    Ok(Value::Set(Rc::new(std::cell::RefCell::new(d))))
}

impl Interpreter {
    /// Call builtin `id` with evaluated positional and named arguments.
    pub fn call_builtin(
        &mut self,
        id: usize,
        args: Vec<Value>,
        named: Vec<(Rc<str>, Value)>,
    ) -> Result<Value, AdarshError> {
        let spec = &BUILTINS[id];
        let max = spec.params.len();
        if args.len() > max {
            let sig = Signature::from_counts(spec.required, max);
            return Err(err(sig.arity_error(spec.name, args.len())).with_code("E002"));
        }
        let mut slots: Vec<Option<Value>> = args.into_iter().map(Some).collect();
        slots.resize(max, None);
        for (n, v) in named {
            match spec.params.iter().position(|p| *p == &*n) {
                Some(i) if slots[i].is_none() => slots[i] = Some(v),
                Some(_) => return Err(err(format!("'{n}' ki value do baar mili."))),
                None => {
                    let hint =
                        crate::semantics::suggestion_suffix(crate::semantics::closest(&n, spec.params.iter().copied()));
                    return Err(err(format!(
                        "'{}' mein '{n}' naam ka parameter nahi hai. Parameters: {}.{hint}",
                        spec.name,
                        spec.params.join(", ")
                    )));
                }
            }
        }
        let given: Vec<bool> = slots.iter().map(Option::is_some).collect();
        if let Some(i) = given.iter().take(spec.required).position(|g| !g) {
            return Err(err(format!("'{}' ko '{}' ki value nahi mili.", spec.name, spec.params[i])).with_code("E002"));
        }
        let v: Vec<Value> = slots
            .into_iter()
            .enumerate()
            .map(|(i, s)| s.unwrap_or_else(|| spec.defaults[i - spec.required].value()))
            .collect();
        let provided = move |i: usize| given.get(i).copied().unwrap_or(false);
        self.dispatch(spec.name, v, &provided)
    }

    fn call_fn(&mut self, func: &str, f: &Value, args: Vec<Value>) -> Result<Value, AdarshError> {
        if !f.is_callable() {
            return Err(wants(func, "ek", "kaam (function)", f));
        }
        self.call(f, args)
    }

    /// `map(f, xs)` and `xs.map(f)` both work: accept the function in either position.
    fn fn_and_items(a: &Value, b: &Value) -> (Value, Value) {
        if !a.is_callable() && b.is_callable() {
            (b.clone(), a.clone())
        } else {
            (a.clone(), b.clone())
        }
    }

    fn key_of(&mut self, by: &Value, item: &Value) -> Result<Value, AdarshError> {
        if matches!(by, Value::Null) {
            Ok(item.clone())
        } else {
            self.call_fn("by", by, vec![item.clone()])
        }
    }

    /// Stable merge sort by `<` on the keys, so mixed types give a clear error.
    fn sort_by_keys(&mut self, items: Vec<(Value, Value)>) -> Result<Vec<(Value, Value)>, AdarshError> {
        if items.len() <= 1 {
            return Ok(items);
        }
        let mut items = items;
        let right = items.split_off(items.len() / 2);
        let (l, r) = (self.sort_by_keys(items)?, self.sort_by_keys(right)?);
        let mut out = Vec::with_capacity(l.len() + r.len());
        let (mut i, mut j) = (0, 0);
        while i < l.len() && j < r.len() {
            let lt = self.binary_values(&r[j].0, BinOp::Lt, &l[i].0)?;
            if lt.truthy() {
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

    fn pick_extreme(
        &mut self,
        name: &str,
        items: Vec<Value>,
        by: &Value,
        want_max: bool,
    ) -> Result<Value, AdarshError> {
        let mut best: Option<(Value, Value)> = None;
        for item in items {
            let key = self.key_of(by, &item)?;
            let better = match &best {
                None => true,
                Some((bk, _)) => {
                    let op = if want_max { BinOp::Gt } else { BinOp::Lt };
                    self.binary_values(&key, op, bk)?.truthy()
                }
            };
            if better {
                best = Some((key, item));
            }
        }
        best.map(|(_, v)| v).ok_or_else(|| err(format!("{name}() ko khali list mili.")))
    }

    fn date_arg(&self, v: &Value) -> Result<i64, AdarshError> {
        if matches!(v, Value::Null) {
            let local = self.host.now() + self.host.tz_offset_minutes() as f64 * 60.0;
            return Ok((local / 86400.0).floor() as i64);
        }
        date_days(v)
    }

    fn dispatch(&mut self, name: &str, v: Vec<Value>, provided: &dyn Fn(usize) -> bool) -> Result<Value, AdarshError> {
        let a = &v;
        Ok(match name {
            // ---------------------------------------------------- collections
            "length" => Value::Int(len_of(&a[0])? as i64),
            "push" => match &a[0] {
                Value::List(l) => {
                    l.borrow_mut().push(a[1].clone());
                    a[0].clone()
                }
                Value::Set(s) => {
                    s.borrow_mut().insert(a[1].clone(), Value::Null)?;
                    a[0].clone()
                }
                other => return Err(wants(name, "pehla", "list ya set", other)),
            },
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
            "insert" => {
                let Value::List(l) = &a[0] else { return Err(wants(name, "pehla", "list", &a[0])) };
                let mut list = l.borrow_mut();
                let n = list.len() as i64;
                let i = normalize_index(&a[1])?.unwrap_or(n);
                let pos = if i < 0 { (i + n).max(0) } else { i.min(n) } as usize;
                list.insert(pos, a[2].clone());
                drop(list);
                a[0].clone()
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
                    Value::List(_) | Value::Str(_) | Value::Tuple(_) => {
                        let items = iter_items(&a[0]).unwrap();
                        let idx = normalize_index(&a[1])?;
                        match resolve(idx, items.len()) {
                            Some(i) => items[i].clone(),
                            None if has_default => a[2].clone(),
                            None => return Err(out_of_range(&a[1], items.len(), "List")),
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
                Value::Set(s) => {
                    let removed = s.borrow_mut().remove(&a[1])?;
                    removed.map(|_| a[1].clone()).ok_or_else(|| err(format!("{} set mein nahi hai.", repr(&a[1]))))?
                }
                Value::List(l) => {
                    let idx = normalize_index(&a[1])?;
                    let mut list = l.borrow_mut();
                    match resolve(idx, list.len()) {
                        Some(i) => list.remove(i),
                        None => return Err(out_of_range(&a[1], list.len(), "List")),
                    }
                }
                other => return Err(wants(name, "pehla", "dict, list ya set", other)),
            },
            "keys" | "values" => {
                let Value::Dict(d) = &a[0] else { return Err(wants(name, "pehla", "dict", &a[0])) };
                let d = d.borrow();
                Value::list(
                    d.entries().iter().map(|(k, v)| if name == "keys" { k.clone() } else { v.clone() }).collect(),
                )
            }
            "contains" => Value::Bool(crate::interp::contains(&a[0], &a[1]).or_else(|_| match &a[0] {
                Value::Str(s) => Ok(s.contains(display(&a[1]).as_str())),
                other => Err(wants(name, "pehla", "list, string ya dict", other)),
            })?),
            "find" => match &a[0] {
                Value::Str(s) => {
                    let needle = display(&a[1]);
                    Value::Int(match s.find(needle.as_str()) {
                        Some(byte) => s[..byte].chars().count() as i64,
                        None => -1,
                    })
                }
                other => {
                    let items = items_of(name, "pehla", other)?;
                    Value::Int(items.iter().position(|x| values_eq(x, &a[1])).map(|p| p as i64).unwrap_or(-1))
                }
            },
            "count" => match &a[0] {
                Value::Str(s) => {
                    let needle = display(&a[1]);
                    Value::Int(if needle.is_empty() { 0 } else { s.matches(needle.as_str()).count() as i64 })
                }
                other => {
                    let items = items_of(name, "pehla", other)?;
                    Value::Int(items.iter().filter(|x| values_eq(x, &a[1])).count() as i64)
                }
            },
            "sort" => {
                let items = match &a[0] {
                    Value::List(l) => l.borrow().clone(),
                    other => items_of(name, "pehla", other)?,
                };
                let mut keyed = Vec::with_capacity(items.len());
                for item in items {
                    keyed.push((self.key_of(&a[1], &item)?, item));
                }
                let mut sorted: Vec<Value> = self.sort_by_keys(keyed)?.into_iter().map(|(_, v)| v).collect();
                if a[2].truthy() {
                    sorted.reverse();
                }
                Value::list(sorted)
            }
            "reverse" => match &a[0] {
                Value::Str(s) => Value::Str(s.chars().rev().collect::<String>().into()),
                Value::Tuple(t) => Value::Tuple(Rc::new(t.iter().rev().cloned().collect())),
                other => Value::list(items_of(name, "pehla", other)?.into_iter().rev().collect()),
            },
            "slice" => {
                let start = Some(to_i(name, &a[1])?);
                let end = match &a[2] {
                    Value::Null => None,
                    x => Some(to_i(name, x)?),
                };
                match &a[0] {
                    Value::Str(st) => {
                        let chars: Vec<char> = st.chars().collect();
                        Value::Str(
                            slice_positions(chars.len(), start, end, 1)
                                .into_iter()
                                .map(|i| chars[i])
                                .collect::<String>()
                                .into(),
                        )
                    }
                    other => {
                        let items = items_of(name, "pehla", other)?;
                        Value::list(
                            slice_positions(items.len(), start, end, 1).into_iter().map(|i| items[i].clone()).collect(),
                        )
                    }
                }
            }
            "range" => {
                let (start, end) = if !matches!(a[1], Value::Null) {
                    (to_i(name, &a[0])?, to_i(name, &a[1])?)
                } else {
                    (0, to_i(name, &a[0])?)
                };
                let step = to_i(name, &a[2])?;
                if step == 0 {
                    return Err(err("range() ka step 0 nahi ho sakta."));
                }
                let count: i128 = if step > 0 {
                    if end > start {
                        (end as i128 - start as i128 + step as i128 - 1) / step as i128
                    } else {
                        0
                    }
                } else if start > end {
                    (start as i128 - end as i128 - step as i128 - 1) / (-(step as i128))
                } else {
                    0
                };
                if count > 100_000_000 {
                    return Err(err("range() bahut badi list banayega (10 crore se zyada items)."));
                }
                Value::list((0..count as i64).map(|k| Value::Int(start + k * step)).collect())
            }
            "join" => {
                let items = items_of(name, "pehla", &a[0])?;
                let parts: Vec<String> = items.iter().map(|x| self.show(x)).collect();
                Value::Str(parts.join(&display(&a[1])).into())
            }
            "map" | "filter" => {
                let (f, items) = Self::fn_and_items(&a[0], &a[1]);
                if !is_iterable(&items) {
                    return Err(wants(name, "doosra", "list, string ya dict", &items));
                }
                let mut out = Vec::new();
                let mut i = 0;
                while let Some(item) = self.nth_item(&items, i) {
                    i += 1;
                    let r = self.call_fn(name, &f, vec![item.clone()])?;
                    if name == "map" {
                        out.push(r);
                    } else if r.truthy() {
                        out.push(item);
                    }
                }
                Value::list(out)
            }
            "reduce" => {
                let (f, items) = Self::fn_and_items(&a[0], &a[1]);
                if !is_iterable(&items) {
                    return Err(wants(name, "doosra", "list, string ya dict", &items));
                }
                let mut i = 0;
                let mut acc = if provided(2) {
                    a[2].clone()
                } else {
                    i = 1;
                    self.nth_item(&items, 0).ok_or_else(|| {
                        err("reduce() ko khali list mili aur shuruaati value (teesra argument) nahi di.")
                    })?
                };
                while let Some(item) = self.nth_item(&items, i) {
                    i += 1;
                    acc = self.call_fn(name, &f, vec![acc, item])?;
                }
                acc
            }
            "sum" => {
                let mut acc = a[1].clone();
                for item in items_of(name, "pehla", &a[0])? {
                    acc = self.binary_values(&acc, BinOp::Add, &item)?;
                }
                acc
            }
            "min" | "max" => {
                let items = match &a[0] {
                    x if is_iterable(x) => items_of(name, "pehla", x)?,
                    _ => {
                        return Err(err(format!("{name}() ko list chahiye. Do values ke liye {name}_val(a, b) likho.")))
                    }
                };
                self.pick_extreme(name, items, &a[1], name == "max")?
            }
            "any" | "all" => {
                let mut result = name == "all";
                for item in items_of(name, "pehla", &a[0])? {
                    let t = self.key_of(&a[1], &item)?.truthy();
                    if name == "any" && t {
                        result = true;
                        break;
                    }
                    if name == "all" && !t {
                        result = false;
                        break;
                    }
                }
                Value::Bool(result)
            }
            "zip" => {
                let (x, y) = (items_of(name, "pehla", &a[0])?, items_of(name, "doosra", &a[1])?);
                Value::list(x.into_iter().zip(y).map(|(p, q)| Value::Tuple(Rc::new(vec![p, q]))).collect())
            }
            "unique" => {
                let mut seen = Dict::new();
                let mut out = Vec::new();
                for item in items_of(name, "pehla", &a[0])? {
                    if !seen.contains(&item)? {
                        seen.insert(item.clone(), Value::Null)?;
                        out.push(item);
                    }
                }
                Value::list(out)
            }
            "flatten" => {
                let mut out = Vec::new();
                for item in items_of(name, "pehla", &a[0])? {
                    match &item {
                        Value::List(_) | Value::Tuple(_) => out.extend(iter_items(&item).unwrap()),
                        _ => out.push(item),
                    }
                }
                Value::list(out)
            }
            "group_by" => {
                let (f, items) = Self::fn_and_items(&a[1], &a[0]);
                let mut groups = Dict::new();
                for item in items_of(name, "pehla", &items)? {
                    let key = self.call_fn(name, &f, vec![item.clone()])?;
                    match groups.get(&key)? {
                        Some(Value::List(l)) => l.borrow_mut().push(item),
                        _ => groups.insert(key, Value::list(vec![item]))?,
                    }
                }
                Value::dict(groups)
            }
            "list" => match &a[0] {
                Value::Null => Value::list(vec![]),
                x => Value::list(items_of(name, "pehla", x)?),
            },
            "tuple" => match &a[0] {
                Value::Null => Value::Tuple(Rc::new(vec![])),
                x => Value::Tuple(Rc::new(items_of(name, "pehla", x)?)),
            },
            "set" => match &a[0] {
                Value::Null => new_set(vec![])?,
                x => new_set(items_of(name, "pehla", x)?)?,
            },
            "union" | "intersection" | "difference" => {
                let (x, y) = (items_of(name, "pehla", &a[0])?, items_of(name, "doosra", &a[1])?);
                let ys = match new_set(y.clone())? {
                    Value::Set(s) => s,
                    _ => unreachable!(),
                };
                let items: Vec<Value> = match name {
                    "union" => x.into_iter().chain(y).collect(),
                    "intersection" => x.into_iter().filter(|i| ys.borrow().contains(i).unwrap_or(false)).collect(),
                    _ => x.into_iter().filter(|i| !ys.borrow().contains(i).unwrap_or(false)).collect(),
                };
                new_set(items)?
            }
            // ---------------------------------------------------- numbers
            "abs" => abs_value(&a[0])?,
            "floor" => round_value(name, &a[0], f64::floor)?,
            "ceil" => round_value(name, &a[0], f64::ceil)?,
            "round_val" => round_value(name, &a[0], f64::round_ties_even)?,
            "round" => {
                let digits = to_i(name, &a[1])?;
                if digits == 0 {
                    round_value(name, &a[0], f64::round_ties_even)?
                } else {
                    let x = to_f(name, &a[0])?;
                    let m = 10f64.powi(digits.clamp(-300, 300) as i32);
                    Value::Float((x * m).round_ties_even() / m)
                }
            }
            "min_val" => {
                if self.binary_values(&a[1], BinOp::Lt, &a[0])?.truthy() {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "max_val" => {
                if self.binary_values(&a[1], BinOp::Gt, &a[0])?.truthy() {
                    a[1].clone()
                } else {
                    a[0].clone()
                }
            }
            "sqrt" => {
                let x = to_f(name, &a[0])?;
                if x < 0.0 {
                    return Err(err("Negative number ka sqrt real number nahi hota."));
                }
                Value::Float(x.sqrt())
            }
            "sin" => float_result(name, to_f(name, &a[0])?.sin())?,
            "cos" => float_result(name, to_f(name, &a[0])?.cos())?,
            "tan" => float_result(name, to_f(name, &a[0])?.tan())?,
            "asin" => float_result(name, to_f(name, &a[0])?.asin())?,
            "acos" => float_result(name, to_f(name, &a[0])?.acos())?,
            "atan" => float_result(name, to_f(name, &a[0])?.atan())?,
            "exp" => float_result(name, to_f(name, &a[0])?.exp())?,
            "log" => {
                let x = to_f(name, &a[0])?;
                if x <= 0.0 {
                    return Err(err("log() sirf positive number ka hota hai."));
                }
                match &a[1] {
                    Value::Null => Value::Float(x.ln()),
                    b => Value::Float(x.ln() / to_f(name, b)?.ln()),
                }
            }
            "gcd" | "lcm" => {
                let to_big = |v: &Value| match as_num(v) {
                    Some(Num::I(i)) => Ok(num_bigint::BigInt::from(i)),
                    Some(Num::B(b)) => Ok(b),
                    _ => Err(err(format!("{name}() ko poore numbers chahiye."))),
                };
                let (x, y) = (to_big(&a[0])?, to_big(&a[1])?);
                Value::from_big(if name == "gcd" { x.gcd(&y) } else { x.lcm(&y) })
            }
            "is_prime" => {
                let n = to_i(name, &a[0])?;
                Value::Bool(n >= 2 && (2..).take_while(|d: &i64| d.saturating_mul(*d) <= n).all(|d| n % d != 0))
            }
            "random_number" => {
                let (lo, hi) = (to_f(name, &a[0])?, to_f(name, &a[1])?);
                let (lo, hi) = if lo > hi { (hi, lo) } else { (lo, hi) };
                Value::Float(lo + (hi - lo) * self.next_random())
            }
            "random_int" => {
                let (lo, hi) = (to_i(name, &a[0])?, to_i(name, &a[1])?);
                let (lo, hi) = if lo > hi { (hi, lo) } else { (lo, hi) };
                let span = (hi as i128 - lo as i128 + 1) as f64;
                Value::Int(lo + (self.next_random() * span).floor() as i64)
            }
            "random_choice" => {
                let items = items_of(name, "pehla", &a[0])?;
                if items.is_empty() {
                    return Err(err("random_choice() ko khali list mili."));
                }
                let i = (self.next_random() * items.len() as f64) as usize;
                items[i.min(items.len() - 1)].clone()
            }
            "shuffle" => {
                let Value::List(l) = &a[0] else { return Err(wants(name, "pehla", "list", &a[0])) };
                let n = l.borrow().len();
                for i in (1..n).rev() {
                    let j = ((self.next_random() * (i + 1) as f64) as usize).min(i);
                    l.borrow_mut().swap(i, j);
                }
                a[0].clone()
            }
            // ---------------------------------------------------- strings
            "upper" => Value::Str(self.show(&a[0]).to_uppercase().into()),
            "lower" => Value::Str(self.show(&a[0]).to_lowercase().into()),
            "title" => {
                let s = self.show(&a[0]);
                let mut out = String::new();
                let mut start = true;
                for c in s.chars() {
                    if start && c.is_alphabetic() {
                        out.extend(c.to_uppercase());
                    } else {
                        out.extend(c.to_lowercase());
                    }
                    start = !c.is_alphanumeric();
                }
                Value::Str(out.into())
            }
            "trim" => Value::Str(self.show(&a[0]).trim().into()),
            "split" => {
                let (s, sep) = (self.show(&a[0]), display(&a[1]));
                if sep.is_empty() {
                    return Err(err("split() ka separator khali string nahi ho sakta."));
                }
                Value::list(s.split(sep.as_str()).map(Value::str).collect())
            }
            "replace" => Value::Str(self.show(&a[0]).replace(&display(&a[1]), &display(&a[2])).into()),
            "starts_with" => Value::Bool(self.show(&a[0]).starts_with(display(&a[1]).as_str())),
            "ends_with" => Value::Bool(self.show(&a[0]).ends_with(display(&a[1]).as_str())),
            "pad_left" | "pad_right" => {
                let s = self.show(&a[0]);
                let width = to_i(name, &a[1])?.clamp(0, 1_000_000) as usize;
                let fill = display(&a[2]).chars().next().unwrap_or(' ');
                let len = s.chars().count();
                let pad: String = std::iter::repeat_n(fill, width.saturating_sub(len)).collect();
                Value::Str(if name == "pad_left" { format!("{pad}{s}") } else { format!("{s}{pad}") }.into())
            }
            "repeat" => arith(BinOp::Mul, &Value::Str(self.show(&a[0]).into()), &a[1])?,
            "char_code" => {
                let s = display(&a[0]);
                let mut cs = s.chars();
                match (cs.next(), cs.next()) {
                    (Some(c), None) => Value::Int(c as i64),
                    _ => return Err(err("char_code() ko ek akshar (1 character ki string) chahiye.")),
                }
            }
            "from_code" => {
                let n = to_i(name, &a[0])?;
                let c = u32::try_from(n)
                    .ok()
                    .and_then(char::from_u32)
                    .ok_or_else(|| err(format!("{n} koi akshar nahi hai.")))?;
                Value::Str(c.to_string().into())
            }
            "format" => {
                let text = self.show(&a[0]);
                Value::Str(format_spec(&a[0], &text, &display(&a[1]))?.into())
            }
            "lakh_format" => {
                let text = match as_num(&a[0]) {
                    Some(Num::F(f)) => float_repr(f),
                    Some(_) => display(&a[0]),
                    None => return Err(need_number(name, &a[0])),
                };
                Value::Str(group_digits(&text, 3, 2, ',').into())
            }
            "rupaye" => {
                let x = to_f(name, &a[0])?;
                let text = format!("{:.2}", x.abs());
                let sign = if x < 0.0 { "-" } else { "" };
                Value::Str(format!("{sign}₹{}", group_digits(&text, 3, 2, ',')).into())
            }
            "shabdon_mein" => {
                let n = to_i(name, &a[0])?;
                if n.unsigned_abs() >= 1_000_000_000_000_000 {
                    return Err(err("shabdon_mein() itna bada number nahi bol sakta."));
                }
                Value::Str(hindi_words(n).into())
            }
            // ---------------------------------------------------- types
            "prakar" => Value::str(a[0].prakar()),
            "shabdme" => Value::Str(self.show(&a[0]).into()),
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
            // ---------------------------------------------------- input / output / system
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
            "padho_number" => {
                let prompt = display(&a[0]);
                loop {
                    if !prompt.is_empty() {
                        self.write_str(&prompt);
                        self.flush();
                    }
                    let Some(line) = (self.input)() else { break Value::Null };
                    let t = line.trim();
                    if let Some(v) = parse_int(t) {
                        break v;
                    }
                    if let Some(f) = parse_float(t) {
                        break Value::Float(f);
                    }
                    self.write_str(&format!("'{t}' number nahi hai, dobara likho.\n"));
                }
            }
            "rang_mein" => {
                let colour = display(&a[1]);
                let code = ansi_colour(&colour).ok_or_else(|| {
                    err(format!("Rang '{colour}' nahi pata. Chuno: laal, hara, peela, neela, baingani, aasmani, safed, kala, mota."))
                })?;
                Value::Str(format!("\x1b[{code}m{}\x1b[0m", self.show(&a[0])).into())
            }
            "current_time" => Value::Float((self.clock)()),
            "ruko" => {
                let secs = to_f(name, &a[0])?;
                self.flush();
                self.host.sleep(secs);
                Value::Null
            }
            "band_karo" => {
                let code = to_i(name, &a[0])?.clamp(0, 255) as i32;
                return Err(AdarshError::new(ErrorKind::Exit(code), format!("band_karo({code})")));
            }
            "jaancho" => {
                if !a[0].truthy() {
                    let msg = display(&a[1]);
                    let detail = if msg.is_empty() { String::new() } else { format!(": {msg}") };
                    return Err(err(format!("Jaanch fail hui{detail}")).with_code("E012"));
                }
                Value::Null
            }
            // ---------------------------------------------------- dates
            "aaj" => make_date(self.date_arg(&Value::Null)?),
            "abhi" => make_time(self.host.now(), self.host.tz_offset_minutes()),
            "tareekh" => checked_date(to_i(name, &a[0])?, to_i(name, &a[1])?, to_i(name, &a[2])?)?,
            "din_ka_naam" => Value::str(weekday_name(self.date_arg(&a[0])?)),
            "tareekh_jodo" => make_date(date_days(&a[0])? + to_i(name, &a[1])?),
            "din_ka_fark" => Value::Int(date_days(&a[1])? - date_days(&a[0])?),
            // ---------------------------------------------------- data / files / internet
            "json_banao" => {
                let mut out = String::new();
                to_json(&a[0], a[1].truthy(), 0, &mut out)?;
                Value::Str(out.into())
            }
            "json_padho" => parse_json(&display(&a[0]))?,
            "file_padho" => {
                let p = display(&a[0]);
                Value::Str(self.host.read_file(&p).map_err(|e| err(format!("File '{p}' nahi padh paaye: {e}")))?.into())
            }
            "file_likho" | "file_jodo" => {
                let p = display(&a[0]);
                let text = self.show(&a[1]);
                self.host
                    .write_file(&p, &text, name == "file_jodo")
                    .map_err(|e| err(format!("File '{p}' mein nahi likh paaye: {e}")))?;
                Value::Null
            }
            "file_hai" => Value::Bool(self.host.file_exists(&display(&a[0]))),
            "internet_se" => {
                let url = display(&a[0]);
                Value::Str(self.host.fetch(&url).map_err(|e| err(format!("'{url}' se data nahi mila: {e}")))?.into())
            }
            // ---------------------------------------------------- turtle
            "kachhua_aage" => {
                let d = to_f(name, &a[0])?;
                self.turtle.forward(d)?;
                Value::Null
            }
            "kachhua_peeche" => {
                let d = to_f(name, &a[0])?;
                self.turtle.forward(-d)?;
                Value::Null
            }
            "kachhua_daayen" => {
                self.turtle.heading = (self.turtle.heading + to_f(name, &a[0])?).rem_euclid(360.0);
                Value::Null
            }
            "kachhua_baayen" => {
                self.turtle.heading = (self.turtle.heading - to_f(name, &a[0])?).rem_euclid(360.0);
                Value::Null
            }
            "kachhua_kalam_upar" => {
                self.turtle.pen_down = false;
                Value::Null
            }
            "kachhua_kalam_neeche" => {
                self.turtle.pen_down = true;
                Value::Null
            }
            "kachhua_rang" => {
                self.turtle.colour = Turtle::colour_value(&display(&a[0]));
                Value::Null
            }
            "kachhua_mota" => {
                self.turtle.width = to_f(name, &a[0])?.clamp(0.1, 100.0);
                Value::Null
            }
            "kachhua_jaao" => {
                let (x, y) = (to_f(name, &a[0])?, to_f(name, &a[1])?);
                if self.turtle.pen_down {
                    let t = &mut self.turtle;
                    t.lines.push((t.x, t.y, x, y, t.colour.clone(), t.width));
                }
                self.turtle.x = x;
                self.turtle.y = y;
                Value::Null
            }
            "kachhua_ghar" => {
                let t = &mut self.turtle;
                t.x = 0.0;
                t.y = 0.0;
                t.heading = 0.0;
                Value::Null
            }
            _ => unreachable!("unknown builtin {name}"),
        })
    }

    /// Item `i` of an iterable. Lists are read live, so items pushed during a
    /// loop are visited too.
    pub fn nth_item(&self, v: &Value, i: usize) -> Option<Value> {
        match v {
            Value::List(l) => l.borrow().get(i).cloned(),
            Value::Tuple(t) => t.get(i).cloned(),
            Value::Str(s) => s.chars().nth(i).map(|c| Value::Str(c.to_string().into())),
            Value::Dict(d) | Value::Set(d) => d.borrow().entries().get(i).map(|(k, _)| k.clone()),
            _ => None,
        }
    }
}
