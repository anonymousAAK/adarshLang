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
  adarsh build <file.aak> [-o out]
                                 standalone executable banao (no install needed to run it)
  adarsh repl                    interactive REPL
  adarsh --version
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
    let out = BufWriter::with_capacity(64 * 1024, std::io::stdout());
    let result = adarshlang::check(name, source, loader.clone()).and_then(|program| {
        let mut interp = adarshlang::Interpreter::new(loader, Box::new(out));
        interp.input = stdin_input();
        let r = interp.run(&program).map(|_| ());
        interp.flush();
        r
    });
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let _ = std::io::stdout().flush();
            eprintln!("{}", e.render());
            ExitCode::from(1)
        }
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

// ---------------------------------------------------------------- main

fn real_main(args: Vec<String>) -> ExitCode {
    if let Some(files) = embedded_bundle() {
        return run_bundle(files);
    }
    let a: Vec<&str> = args.iter().skip(1).map(String::as_str).collect();
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
