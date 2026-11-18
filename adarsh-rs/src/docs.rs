//! `adarsh docs`: Markdown documentation from `##` doc comments.

use crate::ast::*;
use std::fmt::Write as _;

fn params(def: &FuncDef) -> String {
    def.params
        .iter()
        .map(|p| {
            let mut s = String::new();
            if p.is_varargs {
                s.push_str("baaki ");
            }
            s.push_str(&p.name);
            if let Some(t) = &p.ty {
                let _ = write!(s, ": {}", t.name);
            }
            if p.default.is_some() {
                s.push_str(" = …");
            }
            s
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn signature(def: &FuncDef) -> String {
    let ret = def.ret.as_ref().map(|t| format!(" -> {}", t.name)).unwrap_or_default();
    format!("kaam {}({}){ret}", def.display_name(), params(def))
}

/// Markdown for one source file.
pub fn markdown(title: &str, program: &[Node]) -> String {
    let mut out = format!("# {title}\n\n");
    let funcs: Vec<&FuncDef> = program
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::KaamDef(d) if d.method_of.is_none() && !d.name.as_deref().unwrap_or("").starts_with('_') => {
                Some(&**d)
            }
            _ => None,
        })
        .collect();
    let types: Vec<&DhachaDef> = program
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::DhachaDef(d) => Some(&**d),
            _ => None,
        })
        .collect();
    let enums: Vec<(&str, &Vec<std::rc::Rc<str>>)> = program
        .iter()
        .filter_map(|n| match &n.kind {
            NodeKind::EnumDef(name, vs) => Some((&**name, &**vs)),
            _ => None,
        })
        .collect();
    if !funcs.is_empty() {
        out.push_str("## Functions\n\n");
        for f in &funcs {
            let _ = writeln!(out, "### `{}`\n\n```aak\n{}\n```\n", f.display_name(), signature(f));
            if let Some(doc) = &f.doc {
                let _ = writeln!(out, "{doc}\n");
            }
        }
    }
    if !types.is_empty() {
        out.push_str("## Records (dhacha)\n\n");
        for t in &types {
            let parent = t.parent.as_ref().map(|p| format!(" : {p}")).unwrap_or_default();
            let fields: Vec<String> = t
                .fields
                .iter()
                .map(|f| if f.default.is_some() { format!("{} = …", f.name) } else { f.name.to_string() })
                .collect();
            let _ = writeln!(
                out,
                "### `{}`\n\n```aak\ndhacha {}{parent} {{ {} }};\n```\n",
                t.name,
                t.name,
                fields.join(", ")
            );
            let methods: Vec<&FuncDef> = program
                .iter()
                .filter_map(|n| match &n.kind {
                    NodeKind::KaamDef(d) if d.method_of.as_deref() == Some(&*t.name) => Some(&**d),
                    _ => None,
                })
                .collect();
            for m in methods {
                let _ = writeln!(
                    out,
                    "- `{}`{}",
                    signature(m),
                    m.doc.as_ref().map(|d| format!(": {}", d.replace('\n', " "))).unwrap_or_default()
                );
            }
            out.push('\n');
        }
    }
    if !enums.is_empty() {
        out.push_str("## Enums (vikalp)\n\n");
        for (name, vs) in enums {
            let names: Vec<&str> = vs.iter().map(|v| &**v).collect();
            let _ = writeln!(out, "- `{name}`: {}", names.join(", "));
        }
        out.push('\n');
    }
    if funcs.is_empty() && types.is_empty() {
        out.push_str("_Is file mein koi function ya dhacha nahi hai._\n");
    }
    out
}
