//! `adarsh` command-line tool.

use adarshlang::{value::repr, FsLoader, MemLoader, ModuleLoader, Session, Value, VERSION};
use std::io::{BufRead, BufWriter, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::rc::Rc;

/// Deep AdarshLang recursion needs a deep native stack.
const STACK_SIZE: usize = 512 * 1024 * 1024;
/// Marks a program bundled into an executable by `adarsh build`.
const BUNDLE_MAGIC: &[u8; 8] = b"ADARSH01";

const HELP: &str = "\
AdarshLang — Hinglish programming language

Usage:
  adarsh <file.aak>              program chalao (run a program)
  adarsh run <file.aak>          same as above
  adarsh check <file.aak>        sirf galtiyan dhundo, chalao mat (check only)
  adarsh test [files/folders]    *_test.aak files mein test_* functions chalao
  adarsh samjhao [E001]          error code ka matlab aur example
  adarsh python <file.aak>       wahi program Python mein dekho
  adarsh fmt [files] [--check]   code ko saaf format karo
  adarsh docs <file.aak> [-o x]  ## comments se Markdown docs banao
  adarsh debug <file.aak>        line by line chalao, variables dekho
  adarsh web <file.aak> [-o x]   akeli HTML file banao jo browser mein chale
  adarsh add <user/repo> [as n]  package jodo (aak_modules/ mein)
  adarsh remove <naam>           package hatao
  adarsh install                 adarsh.json ke saare packages lao
  adarsh lsp                     editors ke liye language server
  adarsh build <file.aak> [-o out]
                                 standalone executable banao (no install needed to run it)
  adarsh repl                    interactive REPL
  adarsh --version

Options (kisi bhi command ke saath):
  --english                      errors ke saath English title bhi dikhao
";

fn stdin_input() -> Box<dyn FnMut() -> Option<String>> {
    Box::new(|| {
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(line),
        }
    })
}

fn run_source(name: &str, source: &str, loader: Rc<dyn ModuleLoader>) -> ExitCode {
    let (program, warnings) = match adarshlang::check_with_warnings(name, source, loader.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e.render());
            return ExitCode::from(1);
        }
    };
    if std::env::var_os("ADARSH_NO_WARNINGS").is_none() {
        for w in &warnings {
            eprintln!("{}", w.render());
        }
    }
    let out = BufWriter::with_capacity(64 * 1024, std::io::stdout());
    let mut interp = adarshlang::Interpreter::new(loader, Box::new(out));
    interp.input = stdin_input();
    let r = interp.run(&program);
    interp.flush();
    if let Some(svg) = (!interp.turtle.lines.is_empty()).then(|| interp.turtle.svg()) {
        let path = Path::new(name).with_extension("svg");
        match std::fs::write(&path, svg) {
            Ok(()) => eprintln!("Kachhua ki drawing save hui: {}", path.display()),
            Err(e) => eprintln!("Kachhua ki drawing save nahi hui: {e}"),
        }
    }
    match r {
        Ok(_) => ExitCode::SUCCESS,
        Err(e) => match e.kind {
            adarshlang::ErrorKind::Exit(code) => ExitCode::from(code as u8),
            _ => {
                let _ = std::io::stdout().flush();
                eprintln!("{}", e.render());
                ExitCode::from(1)
            }
        },
    }
}

fn read_program(path: &str) -> Result<String, ExitCode> {
    std::fs::read_to_string(path).map_err(|e| {
        eprintln!("File '{path}' nahi padh paaye: {e}");
        ExitCode::from(2)
    })
}

fn base_dir(path: &str) -> PathBuf {
    Path::new(path).parent().map(Path::to_path_buf).filter(|p| !p.as_os_str().is_empty()).unwrap_or_else(|| ".".into())
}

fn cmd_run(path: &str) -> ExitCode {
    match read_program(path) {
        Ok(src) => run_source(path, &src, Rc::new(FsLoader::new(base_dir(path)))),
        Err(code) => code,
    }
}

fn cmd_check(path: &str) -> ExitCode {
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match adarshlang::check(path, &src, Rc::new(FsLoader::new(base_dir(path)))) {
        Ok(_) => {
            println!("Sab theek hai: {path}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}", e.render());
            ExitCode::from(1)
        }
    }
}

// ---------------------------------------------------------------- bundling

/// Collect the main file plus every module it imports with a literal `lao "..."`.
fn collect_modules(main: &str, base: &Path, out: &mut Vec<(String, String)>) {
    let mut pending = vec![main.to_string()];
    while let Some(src) = pending.pop() {
        for line in src.lines() {
            let t = line.trim_start();
            let Some(rest) = t.strip_prefix("lao") else { continue };
            let rest = rest.trim_start();
            let Some(rest) = rest.strip_prefix('"') else { continue };
            let Some(end) = rest.find('"') else { continue };
            let path = adarshlang::loader::module_path(&rest[..end]);
            if out.iter().any(|(p, _)| *p == path) {
                continue;
            }
            if let Ok(text) = std::fs::read_to_string(base.join(&path)) {
                out.push((path, text.clone()));
                pending.push(text);
            }
        }
    }
}

fn encode_bundle(files: &[(String, String)]) -> Vec<u8> {
    let mut payload = Vec::new();
    for (name, text) in files {
        for part in [name.as_bytes(), text.as_bytes()] {
            payload.extend_from_slice(&(part.len() as u64).to_le_bytes());
            payload.extend_from_slice(part);
        }
    }
    payload.extend_from_slice(&(payload.len() as u64).to_le_bytes());
    payload.extend_from_slice(BUNDLE_MAGIC);
    payload
}

fn decode_bundle(exe: &[u8]) -> Option<Vec<(String, String)>> {
    let n = exe.len();
    if n < 16 || &exe[n - 8..] != BUNDLE_MAGIC {
        return None;
    }
    let len = u64::from_le_bytes(exe[n - 16..n - 8].try_into().ok()?) as usize;
    let mut data = exe.get(n.checked_sub(16 + len)?..n - 16)?;
    let mut files = Vec::new();
    let take = |data: &mut &[u8]| -> Option<String> {
        let l = u64::from_le_bytes(data.get(..8)?.try_into().ok()?) as usize;
        let s = String::from_utf8(data.get(8..8 + l)?.to_vec()).ok()?;
        *data = &data[8 + l..];
        Some(s)
    };
    while !data.is_empty() {
        let name = take(&mut data)?;
        let text = take(&mut data)?;
        files.push((name, text));
    }
    Some(files)
}

fn cmd_build(path: &str, output: Option<&str>) -> ExitCode {
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let base = base_dir(path);
    if let Err(e) = adarshlang::check(path, &src, Rc::new(FsLoader::new(&base))) {
        eprintln!("{}", e.render());
        return ExitCode::from(1);
    }
    let main_name = Path::new(path).file_name().unwrap().to_string_lossy().to_string();
    let mut files = vec![(main_name, src.clone())];
    collect_modules(&src, &base, &mut files);

    let exe = match std::env::current_exe().and_then(std::fs::read) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("adarsh executable nahi padh paaye: {e}");
            return ExitCode::from(2);
        }
    };
    // if this adarsh is itself a bundle, strip the old payload
    let runtime = match decode_bundle(&exe) {
        Some(old) => &exe[..exe.len() - encode_bundle(&old).len()],
        None => &exe[..],
    };
    let default_out = {
        let stem = Path::new(path).file_stem().unwrap().to_string_lossy().to_string();
        if cfg!(windows) {
            format!("{stem}.exe")
        } else {
            stem
        }
    };
    let out_path = output.map(str::to_string).unwrap_or(default_out);
    let mut bytes = runtime.to_vec();
    bytes.extend_from_slice(&encode_bundle(&files));
    if let Err(e) = std::fs::write(&out_path, &bytes) {
        eprintln!("'{out_path}' nahi likh paaye: {e}");
        return ExitCode::from(2);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = std::fs::set_permissions(&out_path, std::fs::Permissions::from_mode(0o755));
    }
    println!(
        "Ban gaya: {out_path} ({} file{}, {:.1} MB). Ise kahin bhi chalao, AdarshLang install karne ki zaroorat nahi.",
        files.len(),
        if files.len() == 1 { "" } else { "s" },
        bytes.len() as f64 / 1_048_576.0
    );
    ExitCode::SUCCESS
}

/// Read only the executable's last bytes first; load it fully only if it is a bundle.
fn embedded_bundle() -> Option<Vec<(String, String)>> {
    use std::io::{Read, Seek, SeekFrom};
    let path = std::env::current_exe().ok()?;
    let mut f = std::fs::File::open(&path).ok()?;
    f.seek(SeekFrom::End(-8)).ok()?;
    let mut tail = [0u8; 8];
    f.read_exact(&mut tail).ok()?;
    if &tail != BUNDLE_MAGIC {
        return None;
    }
    decode_bundle(&std::fs::read(path).ok()?)
}

fn run_bundle(files: Vec<(String, String)>) -> ExitCode {
    let (main_name, main_src) = files[0].clone();
    let loader = MemLoader { files: files.into_iter().collect() };
    run_source(&main_name, &main_src, Rc::new(loader))
}

// ---------------------------------------------------------------- REPL

/// True while braces/brackets/parens or a string are still open.
fn incomplete(src: &str) -> bool {
    let (mut depth, mut in_str, mut esc, mut comment) = (0i32, false, false, false);
    for c in src.chars() {
        if comment {
            if c == '\n' {
                comment = false;
            }
            continue;
        }
        if in_str {
            match (esc, c) {
                (true, _) => esc = false,
                (false, '\\') => esc = true,
                (false, '"') => in_str = false,
                _ => {}
            }
            continue;
        }
        match c {
            '"' => in_str = true,
            '#' => comment = true,
            '{' | '(' | '[' => depth += 1,
            '}' | ')' | ']' => depth -= 1,
            _ => {}
        }
    }
    depth > 0 || in_str
}

fn history_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE"))?;
    Some(PathBuf::from(home).join(".adarsh_history"))
}

fn cmd_repl() -> ExitCode {
    println!("AdarshLang {VERSION} REPL. Code likho aur Enter dabao; 'exit' se bahar niklo.");
    let mut session = Session::new(Rc::new(FsLoader::cwd()), Box::new(std::io::stdout()));
    session.interp.input = stdin_input();
    // line editing + history when typing in a terminal; plain reads when input is piped
    let mut editor =
        if std::io::IsTerminal::is_terminal(&std::io::stdin()) { rustyline::DefaultEditor::new().ok() } else { None };
    let history = history_path();
    if let (Some(ed), Some(h)) = (editor.as_mut(), &history) {
        let _ = ed.load_history(h);
    }
    let mut buffer = String::new();
    loop {
        let prompt = if buffer.is_empty() { "adarsh> " } else { "   ... " };
        let line = match editor.as_mut() {
            Some(ed) => match ed.readline(prompt) {
                Ok(l) => l,
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    buffer.clear();
                    continue;
                }
                Err(_) => break,
            },
            None => {
                let mut l = String::new();
                match std::io::stdin().lock().read_line(&mut l) {
                    Ok(0) | Err(_) => break,
                    Ok(_) => l.trim_end_matches(['\n', '\r']).to_string(),
                }
            }
        };
        if buffer.is_empty() && matches!(line.trim(), "exit" | "quit" | "bahar") {
            break;
        }
        buffer.push_str(&line);
        buffer.push('\n');
        // keep reading while a block is open, unless the user enters a blank line
        if incomplete(&buffer) && !line.trim().is_empty() {
            continue;
        }
        let mut source = std::mem::take(&mut buffer);
        if source.trim().is_empty() {
            continue;
        }
        if let Some(ed) = editor.as_mut() {
            let _ = ed.add_history_entry(source.trim_end());
        }
        // allow a missing final ';' on a single expression: `2 + 3`
        let trimmed = source.trim_end();
        if !trimmed.ends_with(';') && !trimmed.ends_with('}') {
            source = format!("{trimmed};");
        }
        match session.execute(&source) {
            Ok(adarshlang::Flow::Normal(v) | adarshlang::Flow::Return(v)) if !matches!(v, Value::Null) => {
                println!("{}", repr(&v))
            }
            Ok(_) => {}
            Err(e) => println!("{}", e.render()),
        }
        let _ = std::io::stdout().flush();
    }
    if let (Some(ed), Some(h)) = (editor.as_mut(), &history) {
        let _ = ed.save_history(h);
    }
    ExitCode::SUCCESS
}

fn cmd_samjhao(code: Option<&str>) -> ExitCode {
    match code {
        None => {
            println!("Error codes (adarsh samjhao <code>):\n");
            for e in adarshlang::explain::EXPLANATIONS {
                println!("  {}  {} ({})", e.code, e.title_hi, e.title_en);
            }
            ExitCode::SUCCESS
        }
        Some(c) => match adarshlang::explain::find(c) {
            Some(e) => {
                print!("{}", adarshlang::explain::render(e));
                ExitCode::SUCCESS
            }
            None => {
                eprintln!("'{c}' naam ka error code nahi mila. Saare codes: adarsh samjhao");
                ExitCode::from(2)
            }
        },
    }
}

fn collect_test_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else { return };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let hidden = p.file_name().is_some_and(|n| n.to_string_lossy().starts_with('.'));
            if p.is_dir() && !hidden && !p.ends_with("target") {
                collect_test_files(&p, out);
            } else if p.to_string_lossy().ends_with("_test.aak") {
                out.push(p);
            }
        }
    } else {
        out.push(path.to_path_buf());
    }
}

fn cmd_test(paths: &[&str]) -> ExitCode {
    let mut files = Vec::new();
    if paths.is_empty() {
        collect_test_files(Path::new("."), &mut files);
    } else {
        for p in paths {
            collect_test_files(Path::new(p), &mut files);
        }
    }
    if files.is_empty() {
        eprintln!(
            "Koi test file nahi mili. Test files ka naam '_test.aak' par khatam hona chahiye (jaise math_test.aak)."
        );
        return ExitCode::from(2);
    }
    let (mut pass, mut fail) = (0, 0);
    for f in files {
        let name = f.to_string_lossy().to_string();
        let src = match std::fs::read_to_string(&f) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("{name}: nahi padh paaye: {e}");
                fail += 1;
                continue;
            }
        };
        println!("{name}");
        for t in adarshlang::run_tests(&name, &src, Rc::new(FsLoader::new(base_dir(&name)))) {
            match t.error {
                None => {
                    pass += 1;
                    println!("  ✓ {}", t.name);
                }
                Some(e) => {
                    fail += 1;
                    println!("  ✗ {}", t.name);
                    for line in e.lines() {
                        println!("      {line}");
                    }
                    if !t.output.is_empty() {
                        println!("      output:");
                        for line in t.output.lines() {
                            println!("        {line}");
                        }
                    }
                }
            }
        }
    }
    println!("\n{pass} pass, {fail} fail");
    if fail > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn cmd_python(path: &str) -> ExitCode {
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    match adarshlang::check(path, &src, Rc::new(FsLoader::new(base_dir(path)))) {
        Ok(program) => {
            print!("{}", adarshlang::topython::translate(&program));
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("{}", e.render());
            ExitCode::from(1)
        }
    }
}

// ---------------------------------------------------------------- fmt / docs / web

/// `.aak` files under `path` (skipping hidden folders, target/ and aak_modules/).
fn aak_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = std::fs::read_dir(path) else { return };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            let name = p.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
            if p.is_dir() {
                if !name.starts_with('.') && name != "target" && name != "aak_modules" {
                    aak_files(&p, out);
                }
            } else if name.ends_with(".aak") {
                out.push(p);
            }
        }
    } else {
        out.push(path.to_path_buf());
    }
}

fn cmd_fmt(args: &[&str]) -> ExitCode {
    let check = args.contains(&"--check");
    let paths: Vec<&str> = args.iter().copied().filter(|a| *a != "--check").collect();
    let mut files = Vec::new();
    if paths.is_empty() {
        aak_files(Path::new("."), &mut files);
    } else {
        for p in paths {
            aak_files(Path::new(p), &mut files);
        }
    }
    let (mut changed, mut failed) = (0, 0);
    for f in &files {
        let name = f.to_string_lossy().to_string();
        let Ok(src) = std::fs::read_to_string(f) else {
            eprintln!("{name}: padh nahi paaye");
            failed += 1;
            continue;
        };
        let out = adarshlang::fmt::format_source(&src);
        if out == src {
            continue;
        }
        // never write something that no longer parses
        let before_ok = adarshlang::parser::parse(&src, adarshlang::error::register_source(&name, &src)).is_ok();
        let after_ok = adarshlang::parser::parse(&out, adarshlang::error::register_source(&name, &out)).is_ok();
        if before_ok && !after_ok {
            eprintln!("{name}: formatter ne galti kar di, file nahi badli (yeh bug report karo)");
            failed += 1;
            continue;
        }
        changed += 1;
        if check {
            println!("format chahiye: {name}");
        } else if let Err(e) = std::fs::write(f, out) {
            eprintln!("{name}: likh nahi paaye: {e}");
            failed += 1;
        } else {
            println!("formatted: {name}");
        }
    }
    if check {
        println!("{} files dekhi, {changed} ko format chahiye", files.len());
        return if changed > 0 || failed > 0 { ExitCode::from(1) } else { ExitCode::SUCCESS };
    }
    println!("{} files dekhi, {changed} formatted", files.len());
    if failed > 0 {
        ExitCode::from(1)
    } else {
        ExitCode::SUCCESS
    }
}

fn output_flag<'a>(args: &[&'a str]) -> (Vec<&'a str>, Option<&'a str>) {
    let mut rest = Vec::new();
    let mut out = None;
    let mut i = 0;
    while i < args.len() {
        if args[i] == "-o" && i + 1 < args.len() {
            out = Some(args[i + 1]);
            i += 2;
        } else {
            rest.push(args[i]);
            i += 1;
        }
    }
    (rest, out)
}

fn cmd_docs(args: &[&str]) -> ExitCode {
    let (rest, out) = output_flag(args);
    let Some(path) = rest.first() else {
        eprintln!("Usage: adarsh docs <file.aak> [-o docs.md]");
        return ExitCode::from(2);
    };
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let program = match adarshlang::check(path, &src, Rc::new(FsLoader::new(base_dir(path)))) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e.render());
            return ExitCode::from(1);
        }
    };
    let title = Path::new(path).file_name().unwrap().to_string_lossy().to_string();
    let md = adarshlang::docs::markdown(&title, &program);
    match out {
        Some(o) => match std::fs::write(o, md) {
            Ok(()) => {
                println!("Docs likhe: {o}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("'{o}' nahi likh paaye: {e}");
                ExitCode::from(2)
            }
        },
        None => {
            print!("{md}");
            ExitCode::SUCCESS
        }
    }
}

fn cmd_web(args: &[&str]) -> ExitCode {
    let (rest, out) = output_flag(args);
    let Some(path) = rest.first() else {
        eprintln!("Usage: adarsh web <file.aak> [-o page.html]");
        return ExitCode::from(2);
    };
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let base = base_dir(path);
    if let Err(e) = adarshlang::check(path, &src, Rc::new(FsLoader::new(&base))) {
        eprintln!("{}", e.render());
        return ExitCode::from(1);
    }
    let main_name = Path::new(path).file_name().unwrap().to_string_lossy().to_string();
    let mut files = vec![(main_name.clone(), src.clone())];
    collect_modules(&src, &base, &mut files);
    let html = adarshlang::webexport::html(&main_name, &files);
    let out_path =
        out.map(str::to_string).unwrap_or_else(|| Path::new(path).with_extension("html").to_string_lossy().to_string());
    match std::fs::write(&out_path, html) {
        Ok(()) => {
            println!("Web page ban gaya: {out_path}. Kisi bhi browser mein kholo (internet ki zaroorat nahi).");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("'{out_path}' nahi likh paaye: {e}");
            ExitCode::from(2)
        }
    }
}

// ---------------------------------------------------------------- debugger

fn cmd_debug(path: &str) -> ExitCode {
    use std::cell::RefCell;
    use std::collections::HashSet;
    let src = match read_program(path) {
        Ok(s) => s,
        Err(code) => return code,
    };
    let loader: Rc<dyn ModuleLoader> = Rc::new(FsLoader::new(base_dir(path)));
    let program = match adarshlang::check(path, &src, loader.clone()) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{}", e.render());
            return ExitCode::from(1);
        }
    };
    println!("adarsh debug: {path}");
    println!("Commands: Enter/n = agli line, c = chalate raho, b 12 = line 12 par ruko, p naam = variable dekho, v = saare variables, q = band");
    let lines: Vec<String> = src.lines().map(String::from).collect();
    #[derive(Default)]
    struct State {
        stepping: bool,
        breaks: HashSet<u32>,
    }
    let state = Rc::new(RefCell::new(State { stepping: true, ..Default::default() }));
    let mut interp = adarshlang::Interpreter::new(loader, Box::new(std::io::stdout()));
    interp.input = stdin_input();
    let st = state.clone();
    interp.on_step = Some(Box::new(move |pos, scope, depth| {
        let mut s = st.borrow_mut();
        if !(s.stepping || s.breaks.contains(&pos.line)) {
            return true;
        }
        let text = lines.get(pos.line.saturating_sub(1) as usize).map(String::as_str).unwrap_or("");
        let _ = std::io::stdout().flush();
        eprintln!("\n{}{:>4} | {}", "  ".repeat(depth.min(10)), pos.line, text.trim());
        loop {
            eprint!("(debug) ");
            let _ = std::io::stderr().flush();
            let mut cmd = String::new();
            if std::io::stdin().lock().read_line(&mut cmd).unwrap_or(0) == 0 {
                return false;
            }
            let cmd = cmd.trim();
            match cmd.split_once(' ').unwrap_or((cmd, "")) {
                ("" | "n", _) => {
                    s.stepping = true;
                    return true;
                }
                ("c", _) => {
                    s.stepping = false;
                    return true;
                }
                ("q", _) => return false,
                ("b", n) => match n.trim().parse::<u32>() {
                    Ok(l) => {
                        s.breaks.insert(l);
                        eprintln!("breakpoint: line {l}");
                    }
                    Err(_) => eprintln!("line number likho: b 12"),
                },
                ("p", name) => {
                    let vars = adarshlang::interp::visible_vars(scope);
                    match vars.iter().find(|(k, _)| k == name.trim()) {
                        Some((k, v)) => eprintln!("{k} = {v}"),
                        None => eprintln!("'{}' yahan nahi dikhta", name.trim()),
                    }
                }
                ("v", _) => {
                    let vars = adarshlang::interp::visible_vars(scope);
                    if vars.is_empty() {
                        eprintln!("(abhi koi variable nahi)");
                    }
                    for (k, v) in vars {
                        eprintln!("{k} = {v}");
                    }
                }
                _ => eprintln!("Commands: n, c, b <line>, p <naam>, v, q"),
            }
        }
    }));
    let r = interp.run(&program);
    interp.flush();
    match r {
        Ok(_) => {
            eprintln!("\nProgram poora hua.");
            ExitCode::SUCCESS
        }
        Err(e) => match e.kind {
            adarshlang::ErrorKind::Exit(0) => {
                eprintln!("\nDebug band kiya.");
                ExitCode::SUCCESS
            }
            adarshlang::ErrorKind::Exit(code) => ExitCode::from(code as u8),
            _ => {
                eprintln!("{}", e.render());
                ExitCode::from(1)
            }
        },
    }
}

// ---------------------------------------------------------------- packages

const MANIFEST: &str = "adarsh.json";

fn read_manifest() -> Vec<(String, String)> {
    let Ok(text) = std::fs::read_to_string(MANIFEST) else { return Vec::new() };
    let Ok(Value::Dict(d)) = adarshlang::stdlib::parse_json(&text) else { return Vec::new() };
    let Some(Value::Dict(pkgs)) = d.borrow().get_str("packages") else { return Vec::new() };
    let pkgs = pkgs.borrow();
    pkgs.entries().iter().map(|(k, v)| (adarshlang::value::display(k), adarshlang::value::display(v))).collect()
}

fn write_manifest(pkgs: &[(String, String)]) -> std::io::Result<()> {
    let mut d = adarshlang::value::Dict::new();
    for (k, v) in pkgs {
        d.insert_str(k, Value::str(v));
    }
    let mut root = adarshlang::value::Dict::new();
    root.insert_str("packages", Value::dict(d));
    let mut out = String::new();
    let _ = adarshlang::stdlib::to_json(&Value::dict(root), true, 0, &mut out);
    std::fs::write(MANIFEST, out + "\n")
}

fn package_url(spec: &str) -> String {
    // full URLs, ssh remotes and local folders are used as they are
    if spec.contains("://") || spec.starts_with("git@") || Path::new(spec).is_dir() {
        spec.to_string()
    } else {
        format!("https://github.com/{}.git", spec.trim_end_matches(".git"))
    }
}

fn package_name(spec: &str) -> String {
    let last = spec.trim_end_matches('/').rsplit(['/', ':']).next().unwrap_or(spec);
    last.trim_end_matches(".git").to_lowercase().replace('-', "_")
}

fn clone_package(name: &str, url: &str) -> bool {
    let dest = Path::new("aak_modules").join(name);
    if dest.exists() {
        println!("  {name}: pehle se hai (aak_modules/{name})");
        return true;
    }
    let _ = std::fs::create_dir_all("aak_modules");
    let ok = std::process::Command::new("git")
        .args(["clone", "--depth", "1", "--quiet", url])
        .arg(&dest)
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if ok {
        println!("  {name}: install hua ({url})");
    } else {
        eprintln!("  {name}: '{url}' clone nahi ho paaya (git installed hai? URL sahi hai?)");
    }
    ok
}

fn cmd_add(args: &[&str]) -> ExitCode {
    let (spec, name) = match args {
        [spec] => (*spec, package_name(spec)),
        [spec, "as", name] => (*spec, name.to_string()),
        _ => {
            eprintln!("Usage: adarsh add <github-user/repo | git URL> [as naam]");
            return ExitCode::from(2);
        }
    };
    let url = package_url(spec);
    if !clone_package(&name, &url) {
        return ExitCode::from(1);
    }
    let mut pkgs: Vec<(String, String)> = read_manifest().into_iter().filter(|(n, _)| *n != name).collect();
    pkgs.push((name.clone(), url));
    if let Err(e) = write_manifest(&pkgs) {
        eprintln!("{MANIFEST} nahi likh paaye: {e}");
        return ExitCode::from(1);
    }
    println!("Use karo: lao \"{name}\";   (ya lao \"{name}/file\";)");
    ExitCode::SUCCESS
}

fn cmd_remove(name: &str) -> ExitCode {
    let pkgs = read_manifest();
    if !pkgs.iter().any(|(n, _)| n == name) {
        eprintln!("'{name}' {MANIFEST} mein nahi hai.");
        return ExitCode::from(1);
    }
    let rest: Vec<(String, String)> = pkgs.into_iter().filter(|(n, _)| n != name).collect();
    if let Err(e) = write_manifest(&rest) {
        eprintln!("{MANIFEST} nahi likh paaye: {e}");
        return ExitCode::from(1);
    }
    println!("'{name}' {MANIFEST} se hataya. Files hatane ke liye aak_modules/{name} folder delete karo.");
    ExitCode::SUCCESS
}

fn cmd_install() -> ExitCode {
    let pkgs = read_manifest();
    if pkgs.is_empty() {
        println!("{MANIFEST} mein koi package nahi. Jodne ke liye: adarsh add user/repo");
        return ExitCode::SUCCESS;
    }
    let ok = pkgs.iter().all(|(n, u)| clone_package(n, u));
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::from(1)
    }
}

// ---------------------------------------------------------------- Jupyter kernel backend

/// JSON-lines protocol used by the Jupyter kernel in editors/jupyter:
/// each input line is {"code": "..."}; each output line is
/// {"output": "...", "error": null|"...", "value": null|"...", "svg": null|"..."}.
fn cmd_kernel_repl() -> ExitCode {
    use adarshlang::stdlib::{parse_json, to_json};
    let buf = adarshlang::SharedBuf::default();
    let mut session = Session::new(Rc::new(FsLoader::cwd()), Box::new(buf.clone()));
    let jstr = |s: &str| {
        let mut o = String::new();
        let _ = to_json(&Value::str(s), false, 0, &mut o);
        o
    };
    let opt = |s: Option<String>| s.map(|x| jstr(&x)).unwrap_or_else(|| "null".into());
    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        let Ok(line) = line else { break };
        let code = match parse_json(&line) {
            Ok(Value::Dict(d)) => {
                d.borrow().get_str("code").map(|v| adarshlang::value::display(&v)).unwrap_or_default()
            }
            _ => continue,
        };
        session.interp.turtle = Default::default();
        let mut source = code.trim_end().to_string();
        if !source.is_empty() && !source.ends_with(';') && !source.ends_with('}') {
            source.push(';');
        }
        let result = session.execute(&source);
        session.interp.flush();
        let output = buf.take_string();
        let (value, error) = match result {
            Ok(adarshlang::Flow::Normal(v) | adarshlang::Flow::Return(v)) if !matches!(v, Value::Null) => {
                (Some(session.interp.show_repr(&v)), None)
            }
            Ok(_) => (None, None),
            Err(e) if matches!(e.kind, adarshlang::ErrorKind::Exit(_)) => (None, None),
            Err(e) => (None, Some(e.render())),
        };
        let svg = (!session.interp.turtle.lines.is_empty()).then(|| session.interp.turtle.svg());
        println!(
            "{{\"output\":{},\"error\":{},\"value\":{},\"svg\":{}}}",
            jstr(&output),
            opt(error),
            opt(value),
            opt(svg)
        );
        let _ = std::io::stdout().flush();
    }
    ExitCode::SUCCESS
}

// ---------------------------------------------------------------- main

fn real_main(args: Vec<String>) -> ExitCode {
    if let Some(files) = embedded_bundle() {
        return run_bundle(files);
    }
    let mut a: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
    if let Some(i) = a.iter().position(|x| *x == "--english") {
        a.remove(i);
        adarshlang::error::set_english(true);
    }
    match a.as_slice() {
        [] | ["repl"] | ["--repl"] => cmd_repl(),
        ["--version" | "-V" | "version"] => {
            println!("adarsh {VERSION}");
            ExitCode::SUCCESS
        }
        ["--help" | "-h" | "help"] => {
            print!("{HELP}");
            ExitCode::SUCCESS
        }
        ["run", file] => cmd_run(file),
        ["samjhao"] => cmd_samjhao(None),
        ["samjhao", code] => cmd_samjhao(Some(code)),
        ["test", rest @ ..] => cmd_test(rest),
        ["python", file] => cmd_python(file),
        ["fmt", rest @ ..] => cmd_fmt(rest),
        ["docs", rest @ ..] => cmd_docs(rest),
        ["web", rest @ ..] => cmd_web(rest),
        ["debug", file] => cmd_debug(file),
        ["add", rest @ ..] => cmd_add(rest),
        ["remove", name] => cmd_remove(name),
        ["install"] => cmd_install(),
        ["lsp"] => {
            adarshlang::lsp::serve();
            ExitCode::SUCCESS
        }
        ["kernel-repl"] => cmd_kernel_repl(),
        ["check", file] => cmd_check(file),
        ["build", file] => cmd_build(file, None),
        ["build", file, "-o", out] | ["build", "-o", out, file] => cmd_build(file, Some(out)),
        [file] if !file.starts_with('-') => cmd_run(file),
        _ => {
            eprint!("Yeh command samajh nahi aaya.\n\n{HELP}");
            ExitCode::from(2)
        }
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().collect();
    std::thread::Builder::new()
        .stack_size(STACK_SIZE)
        .spawn(move || real_main(args))
        .expect("spawn interpreter thread")
        .join()
        .unwrap_or(ExitCode::FAILURE)
}
