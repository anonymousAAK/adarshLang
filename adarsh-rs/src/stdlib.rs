//! Helpers behind the standard library: host access (files, time, internet),
//! JSON, dates, number formatting, Hindi number words and turtle graphics.

use crate::error::AdarshError;
use crate::value::*;
use num_traits::ToPrimitive;
use std::fmt::Write as _;
use std::rc::Rc;

fn err(msg: impl Into<String>) -> AdarshError {
    AdarshError::runtime(msg)
}

// ---------------------------------------------------------------- host

/// Things that work differently on a computer and in the browser.
pub trait Host {
    fn read_file(&self, path: &str) -> Result<String, String>;
    fn write_file(&self, path: &str, text: &str, append: bool) -> Result<(), String>;
    fn file_exists(&self, path: &str) -> bool;
    fn sleep(&self, secs: f64);
    fn fetch(&self, url: &str) -> Result<String, String>;
    /// Seconds since 1970-01-01 UTC.
    fn now(&self) -> f64;
    /// Local time zone offset from UTC, in minutes.
    fn tz_offset_minutes(&self) -> i32;
}

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeHost;

#[cfg(not(target_arch = "wasm32"))]
impl Host for NativeHost {
    fn read_file(&self, path: &str) -> Result<String, String> {
        std::fs::read_to_string(path).map_err(|e| e.to_string())
    }
    fn write_file(&self, path: &str, text: &str, append: bool) -> Result<(), String> {
        use std::io::Write;
        let mut f = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .append(append)
            .truncate(!append)
            .open(path)
            .map_err(|e| e.to_string())?;
        f.write_all(text.as_bytes()).map_err(|e| e.to_string())
    }
    fn file_exists(&self, path: &str) -> bool {
        std::path::Path::new(path).exists()
    }
    fn sleep(&self, secs: f64) {
        if secs > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(secs.min(3600.0)));
        }
    }
    fn fetch(&self, url: &str) -> Result<String, String> {
        if !(url.starts_with("http://") || url.starts_with("https://")) {
            return Err("URL 'http://' ya 'https://' se shuru hona chahiye".into());
        }
        // `curl` ships with Windows 10+, macOS and most Linux systems
        let out = std::process::Command::new("curl")
            .args(["-sSL", "--max-time", "30", "-A", "adarshlang", url])
            .output()
            .map_err(|_| "is computer par 'curl' nahi mila".to_string())?;
        if !out.status.success() {
            return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
        }
        Ok(String::from_utf8_lossy(&out.stdout).into_owned())
    }
    fn now(&self) -> f64 {
        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs_f64()).unwrap_or(0.0)
    }
    fn tz_offset_minutes(&self) -> i32 {
        chrono::Local::now().offset().local_minus_utc() / 60
    }
}

#[cfg(target_arch = "wasm32")]
pub struct BrowserHost;

#[cfg(target_arch = "wasm32")]
impl Host for BrowserHost {
    fn read_file(&self, _path: &str) -> Result<String, String> {
        Err("browser playground mein files nahi padh sakte; yeh CLI (adarsh) mein chalta hai".into())
    }
    fn write_file(&self, _: &str, _: &str, _: bool) -> Result<(), String> {
        Err("browser playground mein files nahi likh sakte; yeh CLI (adarsh) mein chalta hai".into())
    }
    fn file_exists(&self, _: &str) -> bool {
        false
    }
    fn sleep(&self, secs: f64) {
        crate::wasm::sleep(secs);
    }
    fn fetch(&self, _: &str) -> Result<String, String> {
        Err("browser playground se internet_se() nahi chalta; yeh CLI (adarsh) mein chalta hai".into())
    }
    fn now(&self) -> f64 {
        crate::wasm::now()
    }
    fn tz_offset_minutes(&self) -> i32 {
        crate::wasm::tz_offset_minutes()
    }
}

pub fn default_host() -> Rc<dyn Host> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        Rc::new(NativeHost)
    }
    #[cfg(target_arch = "wasm32")]
    {
        Rc::new(BrowserHost)
    }
}

// ---------------------------------------------------------------- formatting

/// Apply a format spec such as `.2`, `>8`, `<10`, `^9`, `08.3`, `,` or `.1%`.
pub fn format_spec(v: &Value, text: &str, spec: &str) -> Result<String, AdarshError> {
    let mut chars = spec.chars().peekable();
    let mut align = None;
    if let Some(&c) = chars.peek() {
        if matches!(c, '<' | '>' | '^') {
            align = Some(c);
            chars.next();
        }
    }
    let mut zero = false;
    if chars.peek() == Some(&'0') {
        zero = true;
        chars.next();
    }
    let mut width = String::new();
    while let Some(&c) = chars.peek() {
        if c.is_ascii_digit() {
            width.push(c);
            chars.next();
        } else {
            break;
        }
    }
    let mut comma = false;
    if chars.peek() == Some(&',') {
        comma = true;
        chars.next();
    }
    let mut precision = None;
    if chars.peek() == Some(&'.') {
        chars.next();
        let mut p = String::new();
        while let Some(&c) = chars.peek() {
            if c.is_ascii_digit() {
                p.push(c);
                chars.next();
            } else {
                break;
            }
        }
        precision = p.parse::<usize>().ok();
    }
    let percent = chars.peek() == Some(&'%');
    if percent {
        chars.next();
    }
    if chars.next().is_some() {
        return Err(err(format!("Format '{spec}' samajh nahi aaya. Jaise '.2', '>8', ',' ya '.1%' likho.")));
    }
    let number = as_num(v).is_some();
    let mut body = if number && (precision.is_some() || comma || percent) {
        let mut f = match as_num(v).unwrap() {
            Num::I(i) => i as f64,
            Num::B(b) => b.to_f64().unwrap_or(f64::INFINITY),
            Num::F(f) => f,
        };
        if percent {
            f *= 100.0;
        }
        let s = match precision {
            Some(p) => format!("{f:.p$}"),
            None if matches!(v, Value::Float(_)) || percent => float_repr(f),
            None => text.to_string(),
        };
        let s = if comma { group_digits(&s, 3, 3, ',') } else { s };
        if percent {
            format!("{s}%")
        } else {
            s
        }
    } else if let (Some(p), false) = (precision, number) {
        text.chars().take(p).collect()
    } else {
        text.to_string()
    };
    if let Ok(w) = width.parse::<usize>() {
        let len = body.chars().count();
        if len < w {
            let pad = w - len;
            let fill = if zero && number { '0' } else { ' ' };
            let align = align.unwrap_or(if number { '>' } else { '<' });
            body = match align {
                '<' => format!("{body}{}", " ".repeat(pad)),
                '^' => format!("{}{body}{}", " ".repeat(pad / 2), " ".repeat(pad - pad / 2)),
                _ if fill == '0' && body.starts_with('-') => format!("-{}{}", "0".repeat(pad), &body[1..]),
                _ => format!("{}{body}", fill.to_string().repeat(pad)),
            };
        }
    }
    Ok(body)
}

/// Insert separators into the integer part: first group `first` digits, then `rest`
/// (3,3 gives 1,234,567; 3,2 gives the Indian 12,34,567).
pub fn group_digits(s: &str, first: usize, rest: usize, sep: char) -> String {
    let (sign, s) = if let Some(x) = s.strip_prefix('-') { ("-", x) } else { ("", s) };
    let (int, frac) = match s.find('.') {
        Some(i) => (&s[..i], &s[i..]),
        None => (s, ""),
    };
    if !int.chars().all(|c| c.is_ascii_digit()) {
        return format!("{sign}{s}");
    }
    let digits: Vec<char> = int.chars().collect();
    let mut groups: Vec<String> = Vec::new();
    let mut end = digits.len();
    let mut size = first;
    while end > 0 {
        let start = end.saturating_sub(size);
        groups.push(digits[start..end].iter().collect());
        end = start;
        size = rest;
    }
    groups.reverse();
    format!("{sign}{}{frac}", groups.join(&sep.to_string()))
}

const ONES: [&str; 100] = [
    "shunya",
    "ek",
    "do",
    "teen",
    "chaar",
    "paanch",
    "chhah",
    "saat",
    "aath",
    "nau",
    "das",
    "gyarah",
    "barah",
    "terah",
    "chaudah",
    "pandrah",
    "solah",
    "satrah",
    "atharah",
    "unnees",
    "bees",
    "ikkees",
    "baees",
    "teees",
    "chaubees",
    "pachchees",
    "chhabbees",
    "sattaees",
    "atthaees",
    "untees",
    "tees",
    "ikattees",
    "battees",
    "taintees",
    "chauntees",
    "paintees",
    "chhattees",
    "saintees",
    "adtees",
    "untaalees",
    "chaalees",
    "iktaalees",
    "bayaalees",
    "taintaalees",
    "chauvaalees",
    "paintaalees",
    "chhiyaalees",
    "saintaalees",
    "adtaalees",
    "unchaas",
    "pachaas",
    "ikyaavan",
    "baavan",
    "tirpan",
    "chauvan",
    "pachpan",
    "chhappan",
    "sattaavan",
    "atthaavan",
    "unsath",
    "saath",
    "iksath",
    "baasath",
    "tirsath",
    "chaunsath",
    "painsath",
    "chhiyaasath",
    "sadsath",
    "adsath",
    "unhattar",
    "sattar",
    "ikhattar",
    "bahattar",
    "tihattar",
    "chauhattar",
    "pachhattar",
    "chhihattar",
    "sathattar",
    "athattar",
    "unaasi",
    "assi",
    "ikyaasi",
    "bayaasi",
    "tiraasi",
    "chauraasi",
    "pachaasi",
    "chhiyaasi",
    "sattaasi",
    "atthaasi",
    "navaasi",
    "nabbe",
    "ikyaanve",
    "baanve",
    "tiraanve",
    "chauraanve",
    "pachaanve",
    "chhiyaanve",
    "sattaanve",
    "atthaanve",
    "ninyaanve",
];

/// 125 → "ek sau pachchees"; uses the Indian system (hazaar, lakh, crore).
pub fn hindi_words(n: i64) -> String {
    if n == 0 {
        return ONES[0].into();
    }
    if n < 0 {
        return format!("minus {}", hindi_words(n.unsigned_abs() as i64));
    }
    let mut parts = Vec::new();
    let mut n = n as u64;
    let crore = n / 10_000_000;
    n %= 10_000_000;
    if crore > 0 {
        parts.push(format!("{} crore", hindi_words(crore as i64)));
    }
    for (unit, size) in [("lakh", 100_000u64), ("hazaar", 1000), ("sau", 100)] {
        let q = n / size;
        n %= size;
        if q > 0 {
            parts.push(format!("{} {unit}", ONES[q as usize]));
        }
    }
    if n > 0 {
        parts.push(ONES[n as usize].to_string());
    }
    parts.join(" ")
}

// ---------------------------------------------------------------- dates

const DIN: [&str; 7] = ["Somvaar", "Mangalvaar", "Budhvaar", "Guruvaar", "Shukravaar", "Shanivaar", "Ravivaar"];
const MAHINE: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];

/// Days since 1970-01-01 for a civil date (proleptic Gregorian).
pub fn days_from_civil(y: i64, m: i64, d: i64) -> i64 {
    let y = if m <= 2 { y - 1 } else { y };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = (m + 9) % 12;
    let doy = (153 * mp + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

pub fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (if m <= 2 { y + 1 } else { y }, m, d)
}

fn days_in_month(y: i64, m: i64) -> i64 {
    days_from_civil(if m == 12 { y + 1 } else { y }, if m == 12 { 1 } else { m + 1 }, 1) - days_from_civil(y, m, 1)
}

/// A `Tareekh { saal, mahina, din }` record.
pub fn make_date(days: i64) -> Value {
    let (y, m, d) = civil_from_days(days);
    let mut r = Dict::record(Rc::from("Tareekh"));
    r.insert_str("saal", Value::Int(y));
    r.insert_str("mahina", Value::Int(m));
    r.insert_str("din", Value::Int(d));
    Value::dict(r)
}

pub fn date_days(v: &Value) -> Result<i64, AdarshError> {
    let bad = || err(format!("Yahan Tareekh chahiye (jaise aaj() ya tareekh(2025, 8, 15)), {} mila.", v.type_desc()));
    let Value::Dict(d) = v else { return Err(bad()) };
    let d = d.borrow();
    let get = |k: &str| match d.get_str(k) {
        Some(Value::Int(i)) => Ok(i),
        _ => Err(bad()),
    };
    Ok(days_from_civil(get("saal")?, get("mahina")?, get("din")?))
}

pub fn checked_date(y: i64, m: i64, d: i64) -> Result<Value, AdarshError> {
    if !(1..=12).contains(&m) {
        return Err(err(format!("Mahina 1 se 12 ke beech hona chahiye, {m} mila.")));
    }
    if d < 1 || d > days_in_month(y, m) {
        return Err(err(format!("{} {y} mein {d} taareekh nahi hoti.", MAHINE[(m - 1) as usize])));
    }
    Ok(make_date(days_from_civil(y, m, d)))
}

pub fn weekday_name(days: i64) -> &'static str {
    // 1970-01-01 was a Thursday (Guruvaar)
    DIN[((days + 3).rem_euclid(7)) as usize]
}

/// `Samay { saal, mahina, din, ghanta, minute, second }` in local time.
pub fn make_time(unix: f64, offset_min: i32) -> Value {
    let local = unix + offset_min as f64 * 60.0;
    let days = (local / 86400.0).floor() as i64;
    let secs = local - days as f64 * 86400.0;
    let (y, m, d) = civil_from_days(days);
    let mut r = Dict::record(Rc::from("Samay"));
    r.insert_str("saal", Value::Int(y));
    r.insert_str("mahina", Value::Int(m));
    r.insert_str("din", Value::Int(d));
    r.insert_str("ghanta", Value::Int((secs / 3600.0) as i64));
    r.insert_str("minute", Value::Int(((secs % 3600.0) / 60.0) as i64));
    r.insert_str("second", Value::Int((secs % 60.0) as i64));
    Value::dict(r)
}

// ---------------------------------------------------------------- JSON

pub fn to_json(v: &Value, pretty: bool, indent: usize, out: &mut String) -> Result<(), AdarshError> {
    let pad = |n: usize| "  ".repeat(n);
    match v {
        Value::Null => out.push_str("null"),
        Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Value::Int(_) | Value::Big(_) => out.push_str(&display(v)),
        Value::Float(f) => {
            if !f.is_finite() {
                return Err(err("JSON mein nan ya inf nahi likh sakte."));
            }
            out.push_str(&float_repr(*f));
        }
        Value::Str(s) => {
            out.push('"');
            for c in s.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\r' => out.push_str("\\r"),
                    '\t' => out.push_str("\\t"),
                    c if (c as u32) < 0x20 => {
                        let _ = write!(out, "\\u{:04x}", c as u32);
                    }
                    c => out.push(c),
                }
            }
            out.push('"');
        }
        Value::List(_) | Value::Tuple(_) | Value::Set(_) => {
            let items = crate::builtins::iter_items(v).unwrap_or_default();
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    out.push_str(&pad(indent + 1));
                }
                to_json(item, pretty, indent + 1, out)?;
            }
            if pretty && !items.is_empty() {
                out.push('\n');
                out.push_str(&pad(indent));
            }
            out.push(']');
        }
        Value::Dict(d) => {
            out.push('{');
            let d = d.borrow();
            for (i, (k, val)) in d.entries().iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    out.push_str(&pad(indent + 1));
                }
                to_json(&Value::Str(display(k).into()), false, 0, out)?;
                out.push(':');
                if pretty {
                    out.push(' ');
                }
                to_json(val, pretty, indent + 1, out)?;
            }
            if pretty && !d.is_empty() {
                out.push('\n');
                out.push_str(&pad(indent));
            }
            out.push('}');
        }
        Value::Enum(e) => to_json(&Value::Str(e.name.clone()), pretty, indent, out)?,
        other => return Err(err(format!("{} ko JSON mein nahi badal sakte.", other.type_desc()))),
    }
    Ok(())
}

pub fn parse_json(text: &str) -> Result<Value, AdarshError> {
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    let v = json_value(&chars, &mut i)?;
    skip_ws(&chars, &mut i);
    if i < chars.len() {
        return Err(json_err(&chars, i));
    }
    Ok(v)
}

fn skip_ws(c: &[char], i: &mut usize) {
    while *i < c.len() && c[*i].is_whitespace() {
        *i += 1;
    }
}

fn json_err(c: &[char], i: usize) -> AdarshError {
    let line = c[..i.min(c.len())].iter().filter(|&&x| x == '\n').count() + 1;
    err(format!("JSON galat hai (line {line} ke paas, position {i})."))
}

fn json_value(c: &[char], i: &mut usize) -> Result<Value, AdarshError> {
    skip_ws(c, i);
    let Some(&ch) = c.get(*i) else { return Err(json_err(c, *i)) };
    match ch {
        '{' => {
            *i += 1;
            let mut d = Dict::new();
            skip_ws(c, i);
            if c.get(*i) == Some(&'}') {
                *i += 1;
                return Ok(Value::dict(d));
            }
            loop {
                skip_ws(c, i);
                let Value::Str(k) = json_value(c, i)? else { return Err(json_err(c, *i)) };
                skip_ws(c, i);
                if c.get(*i) != Some(&':') {
                    return Err(json_err(c, *i));
                }
                *i += 1;
                let v = json_value(c, i)?;
                d.insert(Value::Str(k), v)?;
                skip_ws(c, i);
                match c.get(*i) {
                    Some(',') => *i += 1,
                    Some('}') => {
                        *i += 1;
                        return Ok(Value::dict(d));
                    }
                    _ => return Err(json_err(c, *i)),
                }
            }
        }
        '[' => {
            *i += 1;
            let mut items = Vec::new();
            skip_ws(c, i);
            if c.get(*i) == Some(&']') {
                *i += 1;
                return Ok(Value::list(items));
            }
            loop {
                items.push(json_value(c, i)?);
                skip_ws(c, i);
                match c.get(*i) {
                    Some(',') => *i += 1,
                    Some(']') => {
                        *i += 1;
                        return Ok(Value::list(items));
                    }
                    _ => return Err(json_err(c, *i)),
                }
            }
        }
        '"' => {
            *i += 1;
            let mut s = String::new();
            loop {
                let Some(&ch) = c.get(*i) else { return Err(json_err(c, *i)) };
                *i += 1;
                match ch {
                    '"' => return Ok(Value::Str(s.into())),
                    '\\' => {
                        let Some(&e) = c.get(*i) else { return Err(json_err(c, *i)) };
                        *i += 1;
                        match e {
                            'n' => s.push('\n'),
                            't' => s.push('\t'),
                            'r' => s.push('\r'),
                            'b' => s.push('\u{8}'),
                            'f' => s.push('\u{c}'),
                            'u' => {
                                let hex: String = c.get(*i..*i + 4).map(|x| x.iter().collect()).unwrap_or_default();
                                let code = u32::from_str_radix(&hex, 16).map_err(|_| json_err(c, *i))?;
                                *i += 4;
                                s.push(char::from_u32(code).unwrap_or('\u{FFFD}'));
                            }
                            other => s.push(other),
                        }
                    }
                    other => s.push(other),
                }
            }
        }
        't' | 'f' | 'n' => {
            for (word, v) in [("true", Value::Bool(true)), ("false", Value::Bool(false)), ("null", Value::Null)] {
                let w: Vec<char> = word.chars().collect();
                if c.get(*i..*i + w.len()) == Some(&w[..]) {
                    *i += w.len();
                    return Ok(v);
                }
            }
            Err(json_err(c, *i))
        }
        _ => {
            let start = *i;
            while *i < c.len() && matches!(c[*i], '0'..='9' | '-' | '+' | '.' | 'e' | 'E') {
                *i += 1;
            }
            let text: String = c[start..*i].iter().collect();
            if let Some(v) = parse_int(&text) {
                return Ok(v);
            }
            text.parse::<f64>().map(Value::Float).map_err(|_| json_err(c, start))
        }
    }
}

// ---------------------------------------------------------------- terminal colours

pub fn ansi_colour(name: &str) -> Option<&'static str> {
    Some(match name.to_lowercase().as_str() {
        "laal" | "red" => "31",
        "hara" | "green" => "32",
        "peela" | "yellow" => "33",
        "neela" | "blue" => "34",
        "baingani" | "jamuni" | "purple" | "magenta" => "35",
        "aasmani" | "cyan" => "36",
        "safed" | "white" => "37",
        "kala" | "black" => "30",
        "mota" | "bold" => "1",
        _ => return None,
    })
}

// ---------------------------------------------------------------- turtle graphics

/// Drawing made with `kachhua.aage(100)` etc. Coordinates start at (0, 0) facing up.
#[derive(Clone, Debug)]
pub struct Turtle {
    pub x: f64,
    pub y: f64,
    /// degrees, 0 = up, clockwise
    pub heading: f64,
    pub pen_down: bool,
    pub colour: String,
    pub width: f64,
    /// (x1, y1, x2, y2, colour, width)
    pub lines: Vec<(f64, f64, f64, f64, String, f64)>,
}

impl Default for Turtle {
    fn default() -> Self {
        Turtle { x: 0.0, y: 0.0, heading: 0.0, pen_down: true, colour: "#0f6b4f".into(), width: 2.0, lines: Vec::new() }
    }
}

impl Turtle {
    pub fn forward(&mut self, d: f64) -> Result<(), AdarshError> {
        if self.lines.len() > 200_000 {
            return Err(err("Kachhua ne 2 lakh se zyada lines bana di; drawing bahut badi hai."));
        }
        let rad = self.heading.to_radians();
        let (nx, ny) = (self.x + d * rad.sin(), self.y + d * rad.cos());
        if self.pen_down {
            self.lines.push((self.x, self.y, nx, ny, self.colour.clone(), self.width));
        }
        self.x = nx;
        self.y = ny;
        Ok(())
    }

    pub fn colour_value(name: &str) -> String {
        let n = name.to_lowercase();
        match n.as_str() {
            "laal" => "#c23a48",
            "hara" => "#0f6b4f",
            "peela" => "#d4a017",
            "neela" => "#3456c8",
            "baingani" | "jamuni" => "#7a3fb0",
            "aasmani" => "#1f7a8c",
            "kala" => "#14161a",
            "safed" => "#ffffff",
            "narangi" => "#e8710a",
            "gulabi" => "#d63384",
            _ => return name.to_string(),
        }
        .to_string()
    }

    /// An SVG picture of the drawing (y axis flipped so up is up).
    pub fn svg(&self) -> String {
        let (mut minx, mut miny, mut maxx, mut maxy) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for (x1, y1, x2, y2, _, _) in &self.lines {
            minx = minx.min(*x1).min(*x2);
            maxx = maxx.max(*x1).max(*x2);
            miny = miny.min(*y1).min(*y2);
            maxy = maxy.max(*y1).max(*y2);
        }
        let pad = 20.0;
        let (w, h) = ((maxx - minx) + 2.0 * pad, (maxy - miny) + 2.0 * pad);
        let mut out = format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{:.1} {:.1} {w:.1} {h:.1}\" width=\"{w:.0}\" height=\"{h:.0}\">\n<rect x=\"{:.1}\" y=\"{:.1}\" width=\"{w:.1}\" height=\"{h:.1}\" fill=\"#f6f4ef\"/>\n",
            minx - pad,
            -maxy - pad,
            minx - pad,
            -maxy - pad
        );
        for (x1, y1, x2, y2, c, wd) in &self.lines {
            let _ = writeln!(
                out,
                "<line x1=\"{x1:.2}\" y1=\"{:.2}\" x2=\"{x2:.2}\" y2=\"{:.2}\" stroke=\"{c}\" stroke-width=\"{wd}\" stroke-linecap=\"round\"/>",
                -y1, -y2
            );
        }
        out.push_str("</svg>\n");
        out
    }

    /// JSON list of line segments for the playground canvas.
    pub fn json(&self) -> String {
        let mut out = String::from("[");
        for (i, (x1, y1, x2, y2, c, w)) in self.lines.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let _ = write!(out, "[{x1:.2},{y1:.2},{x2:.2},{y2:.2},\"{c}\",{w}]");
        }
        out.push(']');
        out
    }
}
