//! `adarsh lsp`: a Language Server Protocol server over stdio, for editors.
//!
//! Supports: diagnostics (errors + warnings as you type), hover (builtins,
//! keywords, your functions with their `##` docs), completion, go to
//! definition, document formatting (`adarsh fmt`) and document symbols.

use crate::ast::*;
use crate::error::Pos;
use crate::stdlib::{parse_json, to_json};
use crate::value::{display, Value};
use std::collections::HashMap;
use std::io::{BufRead, Read, Write};
use std::rc::Rc;

fn jstr(s: &str) -> String {
    let mut out = String::new();
    let _ = to_json(&Value::Str(s.into()), false, 0, &mut out);
    out
}

fn get(v: &Value, path: &[&str]) -> Option<Value> {
    let mut cur = v.clone();
    for k in path {
        let Value::Dict(d) = &cur else { return None };
        let next = d.borrow().get_str(k)?;
        cur = next;
    }
    Some(cur)
}

fn get_str(v: &Value, path: &[&str]) -> Option<String> {
    match get(v, path)? {
        Value::Str(s) => Some(s.to_string()),
        _ => None,
    }
}

fn get_int(v: &Value, path: &[&str]) -> Option<i64> {
    match get(v, path)? {
        Value::Int(i) => Some(i),
        _ => None,
    }
}

fn uri_to_path(uri: &str) -> String {
    let p = uri.strip_prefix("file://").unwrap_or(uri);
    let mut out = String::new();
    let b = p.as_bytes();
    let mut i = 0;
    let mut bytes = Vec::new();
    while i < b.len() {
        if b[i] == b'%' && i + 2 < b.len() {
            if let Ok(x) = u8::from_str_radix(&p[i + 1..i + 3], 16) {
                bytes.push(x);
                i += 3;
                continue;
            }
        }
        bytes.push(b[i]);
        i += 1;
    }
    out.push_str(&String::from_utf8_lossy(&bytes));
    // file:///C:/x on Windows
    if out.len() > 2 && out.as_bytes()[0] == b'/' && out.as_bytes()[2] == b':' {
        out.remove(0);
    }
    out
}

const KEYWORD_DOCS: &[(&str, &str)] = &[
    ("badlo", "Naya variable: `badlo x = 5;`"),
    ("pakka", "Constant (badal nahi sakta): `pakka PI2 = 6.28;`"),
    ("dikhao", "Screen par dikhao: `dikhao(\"Namaste {naam}\");`"),
    ("agar", "Shart: `agar (x > 5) { } warna { }`; value ke roop mein: `agar (c) a warna b`"),
    ("warna", "Agar ki shart jhuth ho to yeh chalta hai."),
    ("jabtak", "While loop: `jabtak (shart) { }`"),
    ("karo", "Do-while: `karo { } jabtak (shart);` (kam se kam ek baar chalta hai)"),
    ("ginnati", "For loop: `ginnati (badlo i = 0; i < 5; i += 1) { }`"),
    ("ke_liye", "Har item ke liye: `ke_liye (badlo x in xs) { }`, ya `(badlo i, x in xs)`"),
    ("bas", "Loop se bahar niklo (break). Label ke saath: `bas bahar;`"),
    ("aage_badho", "Loop ka agla round (continue)."),
    ("kaam", "Function: `kaam naam(a, b = 2) { wapas a + b; }`"),
    ("wapas", "Function se value lautao (return)."),
    ("chuno", "Switch / pattern matching: `chuno (x) { case 1..5: ... warna_case: ... }`"),
    ("pakdo", "Try: `pakdo { } chhoddo (e) { } aakhir { }`"),
    ("chhoddo", "Error phenko: `chhoddo(\"sandesh\");`, ya pakdo ke baad catch: `chhoddo (e) { }`"),
    ("aakhir", "Finally: pakdo ke baad hamesha chalta hai."),
    ("dhacha", "Record type: `dhacha Vyakti { naam, umar = 0 };`"),
    ("vikalp", "Enum: `vikalp Rang { Laal, Hara };` then `Rang.Laal`"),
    ("lao", "Doosri file lao: `lao \"utils\";` ya `lao \"utils\" as u;`"),
    ("dete_raho", "Generator: function jo values deta rahe; call karne par list milti hai."),
    ("hai", "Type check: `x hai number`"),
    ("khali", "Kuch nahi (null)."),
    ("sahi_hai_be", "Boolean true."),
    ("jhuth", "Boolean false."),
    ("aur", "Logical and (dono sahi)."),
    ("ya", "Logical or (koi ek sahi)."),
    ("nahin", "Logical not."),
    ("yeh", "Method ke andar record khud (this/self)."),
];

struct Doc {
    text: String,
    program: Option<Vec<Node>>,
}

/// Every named definition in the program, with its position.
fn definitions(nodes: &[Node], out: &mut Vec<(String, Pos, String)>) {
    for n in nodes {
        match &n.kind {
            NodeKind::KaamDef(d) => {
                out.push((d.name.as_deref().unwrap_or("").to_string(), d.pos, "kaam".into()));
                definitions(&d.body, out);
            }
            NodeKind::DhachaDef(d) => out.push((d.name.to_string(), d.pos, "dhacha".into())),
            NodeKind::EnumDef(name, _) => out.push((name.to_string(), n.pos, "vikalp".into())),
            NodeKind::Badlo { name, .. } => out.push((name.to_string(), n.pos, "badlo".into())),
            NodeKind::Seq(ns) => definitions(ns, out),
            NodeKind::Block(b) | NodeKind::While { body: b, .. } | NodeKind::DoWhile { body: b, .. } => {
                definitions(b, out)
            }
            NodeKind::If { then, otherwise, .. } => {
                definitions(then, out);
                if let Some(o) = otherwise {
                    definitions(std::slice::from_ref(o), out);
                }
            }
            NodeKind::For { body, .. } | NodeKind::ForEach { body, .. } => definitions(body, out),
            NodeKind::TryCatch { body, catches, finally } => {
                definitions(body, out);
                for c in catches {
                    definitions(&c.body, out);
                }
                if let Some(f) = finally {
                    definitions(f, out);
                }
            }
            _ => {}
        }
    }
}

fn find_func<'a>(nodes: &'a [Node], name: &str) -> Option<&'a FuncDef> {
    nodes.iter().find_map(|n| match &n.kind {
        NodeKind::KaamDef(d) if d.name.as_deref() == Some(name) => Some(&**d),
        _ => None,
    })
}

fn word_at(text: &str, line: usize, ch: usize) -> Option<String> {
    let l: Vec<char> = text.lines().nth(line)?.chars().collect();
    let is_w = |c: char| c.is_alphanumeric() || c == '_';
    let mut s = ch.min(l.len());
    while s > 0 && is_w(l[s - 1]) {
        s -= 1;
    }
    let mut e = ch.min(l.len());
    while e < l.len() && is_w(l[e]) {
        e += 1;
    }
    (s < e).then(|| l[s..e].iter().collect())
}

pub struct Server {
    docs: HashMap<String, Doc>,
    out: std::io::Stdout,
}

impl Server {
    fn send(&mut self, body: &str) {
        let mut o = self.out.lock();
        let _ = write!(o, "Content-Length: {}\r\n\r\n{}", body.len(), body);
        let _ = o.flush();
    }

    fn reply(&mut self, id: &Value, result: &str) {
        let id = match id {
            Value::Str(s) => jstr(s),
            other => display(other),
        };
        self.send(&format!("{{\"jsonrpc\":\"2.0\",\"id\":{id},\"result\":{result}}}"));
    }

    fn publish(&mut self, uri: &str) {
        let Some(doc) = self.docs.get_mut(uri) else { return };
        let path = uri_to_path(uri);
        let base = std::path::Path::new(&path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let file = crate::error::register_source(&path, &doc.text);
        let mut diags = Vec::new();
        let mut push = |pos: Option<Pos>, msg: &str, severity: u8, code: Option<&str>| {
            let (line, col) = match pos {
                Some(p) if p.file == file => (p.line.saturating_sub(1), p.col.saturating_sub(1)),
                _ => (0, 0),
            };
            let code = code.map(|c| format!(",\"code\":{}", jstr(c))).unwrap_or_default();
            diags.push(format!(
                "{{\"range\":{{\"start\":{{\"line\":{line},\"character\":{col}}},\"end\":{{\"line\":{line},\"character\":{}}}}},\"severity\":{severity},\"source\":\"adarsh\",\"message\":{}{code}}}",
                col + 1,
                jstr(msg)
            ));
        };
        match crate::check_with_warnings(&path, &doc.text, Rc::new(crate::FsLoader::new(base))) {
            Ok((program, warnings)) => {
                for w in &warnings {
                    push(w.pos, &w.message, 2, w.code);
                }
                doc.program = Some(program);
            }
            Err(e) => {
                push(e.pos, &e.message, 1, e.code);
                // keep the last good program for hover/definition
            }
        }
        let body = format!(
            "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{{\"uri\":{},\"diagnostics\":[{}]}}}}",
            jstr(uri),
            diags.join(",")
        );
        self.send(&body);
    }

    fn handle(&mut self, msg: &Value) -> bool {
        let method = get_str(msg, &["method"]).unwrap_or_default();
        let id = get(msg, &["id"]);
        let uri = get_str(msg, &["params", "textDocument", "uri"]).unwrap_or_default();
        match method.as_str() {
            "initialize" => {
                let caps = r#"{"capabilities":{"textDocumentSync":1,"hoverProvider":true,"definitionProvider":true,"documentFormattingProvider":true,"documentSymbolProvider":true,"completionProvider":{"triggerCharacters":["."]}},"serverInfo":{"name":"adarsh","version":""#;
                let result = format!("{caps}{}\"}}}}", crate::VERSION);
                if let Some(id) = id {
                    self.reply(&id, &result);
                }
            }
            "textDocument/didOpen" => {
                let text = get_str(msg, &["params", "textDocument", "text"]).unwrap_or_default();
                self.docs.insert(uri.clone(), Doc { text, program: None });
                self.publish(&uri);
            }
            "textDocument/didChange" => {
                let text = match get(msg, &["params", "contentChanges"]) {
                    Some(Value::List(l)) => l.borrow().last().and_then(|c| get_str(c, &["text"])),
                    _ => None,
                };
                if let Some(text) = text {
                    let doc = self.docs.entry(uri.clone()).or_insert(Doc { text: String::new(), program: None });
                    doc.text = text;
                    self.publish(&uri);
                }
            }
            "textDocument/didClose" => {
                self.docs.remove(&uri);
                self.send(&format!(
                    "{{\"jsonrpc\":\"2.0\",\"method\":\"textDocument/publishDiagnostics\",\"params\":{{\"uri\":{},\"diagnostics\":[]}}}}",
                    jstr(&uri)
                ));
            }
            "textDocument/hover" => {
                let line = get_int(msg, &["params", "position", "line"]).unwrap_or(0) as usize;
                let ch = get_int(msg, &["params", "position", "character"]).unwrap_or(0) as usize;
                let result = self
                    .docs
                    .get(&uri)
                    .and_then(|d| {
                        let w = word_at(&d.text, line, ch)?;
                        if let Some(b) = crate::builtins::builtin_spec(&w) {
                            return Some(format!("**{}**({})\n\n{}", b.name, b.params.join(", "), b.doc));
                        }
                        if let Some((_, doc)) = KEYWORD_DOCS.iter().find(|(k, _)| *k == w) {
                            return Some(format!("**{w}**\n\n{doc}"));
                        }
                        let f = d.program.as_deref().and_then(|p| find_func(p, &w))?;
                        let params: Vec<String> = f.params.iter().map(|p| p.name.to_string()).collect();
                        Some(format!(
                            "```aak\nkaam {}({})\n```\n{}",
                            f.display_name(),
                            params.join(", "),
                            f.doc.as_deref().unwrap_or("")
                        ))
                    })
                    .map(|md| format!("{{\"contents\":{{\"kind\":\"markdown\",\"value\":{}}}}}", jstr(&md)))
                    .unwrap_or_else(|| "null".into());
                if let Some(id) = id {
                    self.reply(&id, &result);
                }
            }
            "textDocument/completion" => {
                let mut items: Vec<String> = Vec::new();
                for k in crate::semantics::KEYWORDS {
                    items.push(format!("{{\"label\":{},\"kind\":14}}", jstr(k)));
                }
                for b in crate::builtins::BUILTINS {
                    items.push(format!(
                        "{{\"label\":{},\"kind\":3,\"detail\":{},\"documentation\":{}}}",
                        jstr(b.name),
                        jstr(&format!("{}({})", b.name, b.params.join(", "))),
                        jstr(b.doc)
                    ));
                }
                if let Some(p) = self.docs.get(&uri).and_then(|d| d.program.as_deref()) {
                    let mut defs = Vec::new();
                    definitions(p, &mut defs);
                    for (name, _, kind) in defs {
                        let k = match kind.as_str() {
                            "kaam" => 3,
                            "dhacha" => 22,
                            "vikalp" => 13,
                            _ => 6,
                        };
                        items.push(format!("{{\"label\":{},\"kind\":{k}}}", jstr(&name)));
                    }
                }
                if let Some(id) = id {
                    self.reply(&id, &format!("[{}]", items.join(",")));
                }
            }
            "textDocument/definition" => {
                let line = get_int(msg, &["params", "position", "line"]).unwrap_or(0) as usize;
                let ch = get_int(msg, &["params", "position", "character"]).unwrap_or(0) as usize;
                let result = self
                    .docs
                    .get(&uri)
                    .and_then(|d| {
                        let w = word_at(&d.text, line, ch)?;
                        let mut defs = Vec::new();
                        definitions(d.program.as_deref()?, &mut defs);
                        let (_, pos, _) = defs.into_iter().find(|(n, _, _)| *n == w)?;
                        let (l, c) = (pos.line.saturating_sub(1), pos.col.saturating_sub(1));
                        Some(format!(
                            "{{\"uri\":{},\"range\":{{\"start\":{{\"line\":{l},\"character\":{c}}},\"end\":{{\"line\":{l},\"character\":{c}}}}}}}",
                            jstr(&uri)
                        ))
                    })
                    .unwrap_or_else(|| "null".into());
                if let Some(id) = id {
                    self.reply(&id, &result);
                }
            }
            "textDocument/formatting" => {
                let result = self
                    .docs
                    .get(&uri)
                    .map(|d| {
                        let lines = d.text.lines().count() + 1;
                        format!(
                            "[{{\"range\":{{\"start\":{{\"line\":0,\"character\":0}},\"end\":{{\"line\":{lines},\"character\":0}}}},\"newText\":{}}}]",
                            jstr(&crate::fmt::format_source(&d.text))
                        )
                    })
                    .unwrap_or_else(|| "[]".into());
                if let Some(id) = id {
                    self.reply(&id, &result);
                }
            }
            "textDocument/documentSymbol" => {
                let mut syms = Vec::new();
                if let Some(p) = self.docs.get(&uri).and_then(|d| d.program.as_deref()) {
                    for n in p {
                        let (name, kind, pos) = match &n.kind {
                            NodeKind::KaamDef(d) => {
                                (d.display_name(), if d.method_of.is_some() { 6 } else { 12 }, d.pos)
                            }
                            NodeKind::DhachaDef(d) => (d.name.to_string(), 23, d.pos),
                            NodeKind::EnumDef(name, _) => (name.to_string(), 10, n.pos),
                            _ => continue,
                        };
                        let (l, c) = (pos.line.saturating_sub(1), pos.col.saturating_sub(1));
                        let range = format!(
                            "{{\"start\":{{\"line\":{l},\"character\":{c}}},\"end\":{{\"line\":{l},\"character\":{}}}}}",
                            c + 1
                        );
                        syms.push(format!(
                            "{{\"name\":{},\"kind\":{kind},\"range\":{range},\"selectionRange\":{range}}}",
                            jstr(&name)
                        ));
                    }
                }
                if let Some(id) = id {
                    self.reply(&id, &format!("[{}]", syms.join(",")));
                }
            }
            "shutdown" => {
                if let Some(id) = id {
                    self.reply(&id, "null");
                }
            }
            "exit" => return false,
            _ => {
                // unknown request: answer so the editor doesn't wait
                if let Some(id) = id {
                    let id_text = match &id {
                        Value::Str(s) => jstr(s),
                        other => display(other),
                    };
                    self.send(&format!(
                        "{{\"jsonrpc\":\"2.0\",\"id\":{id_text},\"error\":{{\"code\":-32601,\"message\":\"method not found\"}}}}"
                    ));
                }
            }
        }
        true
    }
}

/// Run the language server on stdin/stdout until the editor says `exit`.
pub fn serve() {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut server = Server { docs: HashMap::new(), out: std::io::stdout() };
    loop {
        let mut len = 0usize;
        loop {
            let mut header = String::new();
            if input.read_line(&mut header).unwrap_or(0) == 0 {
                return;
            }
            let h = header.trim();
            if h.is_empty() {
                break;
            }
            if let Some(v) = h.strip_prefix("Content-Length:") {
                len = v.trim().parse().unwrap_or(0);
            }
        }
        let mut body = vec![0u8; len];
        if input.read_exact(&mut body).is_err() {
            return;
        }
        let Ok(msg) = parse_json(&String::from_utf8_lossy(&body)) else { continue };
        if !server.handle(&msg) {
            return;
        }
    }
}
