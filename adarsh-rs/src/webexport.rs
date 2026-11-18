//! `adarsh web file.aak`: one self-contained HTML page that runs the program in
//! any browser, offline. The WebAssembly interpreter, the program and the
//! modules it imports are all embedded in the file.

/// The playground's WebAssembly build, embedded at compile time.
/// (Rebuilt by `scripts/build-playground.sh`.)
pub static WASM: &[u8] = include_bytes!("../../playground/adarshlang.wasm");

fn base64(bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let b = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
        let n = ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | b[2] as u32;
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if chunk.len() > 1 { T[(n >> 6) as usize & 63] as char } else { '=' });
        out.push(if chunk.len() > 2 { T[n as usize & 63] as char } else { '=' });
    }
    out
}

/// A JSON string literal that is also safe inside a <script> element.
fn js_string(s: &str) -> String {
    let mut out = String::new();
    let _ = crate::stdlib::to_json(&crate::Value::Str(s.into()), false, 0, &mut out);
    out.replace("</", "<\\/").replace('\u{2028}', "\\u2028").replace('\u{2029}', "\\u2029")
}

fn esc(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

/// `files[0]` is the main program.
pub fn html(title: &str, files: &[(String, String)]) -> String {
    let files_js: Vec<String> =
        files.iter().map(|(n, c)| format!("{{\"name\":{},\"code\":{}}}", js_string(n), js_string(c))).collect();
    let code_html = esc(&files[0].1);
    format!(
        r##"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title_esc}</title>
<style>
  :root {{ --bg: #f6f4ef; --surface: #fff; --ink: #14161a; --ink-2: #4a4f57; --rule: #dcd8cf; --accent: #0f6b4f; --err: #b3261e; }}
  @media (prefers-color-scheme: dark) {{ :root {{ --bg: #0e1013; --surface: #15181d; --ink: #e9e6df; --ink-2: #a9adb4; --rule: #2a2e35; --accent: #3ddc97; --err: #ff6b6b; }} }}
  * {{ box-sizing: border-box; }}
  body {{ margin: 0; background: var(--bg); color: var(--ink); font: 16px/1.6 system-ui, -apple-system, "Segoe UI", sans-serif; }}
  main {{ max-width: 860px; margin: 0 auto; padding: 32px 16px; }}
  h1 {{ font-family: Georgia, serif; font-weight: 400; font-size: 2rem; margin: 0 0 4px; }}
  .sub {{ color: var(--ink-2); font-size: .9rem; margin: 0 0 20px; }}
  .sub a {{ color: var(--accent); }}
  .panel {{ background: var(--surface); border: 1px solid var(--rule); border-radius: 2px; margin-bottom: 16px; }}
  .head {{ display: flex; justify-content: space-between; align-items: center; gap: 8px; padding: 8px 12px; border-bottom: 1px solid var(--rule);
    font: 500 12px ui-monospace, Consolas, monospace; text-transform: uppercase; letter-spacing: .06em; color: var(--ink-2); }}
  pre {{ margin: 0; padding: 14px; font: 14px/1.6 ui-monospace, Consolas, monospace; white-space: pre-wrap; word-break: break-word; overflow: auto; }}
  #out.err {{ color: var(--err); }}
  details summary {{ cursor: pointer; padding: 10px 12px; font: 13px ui-monospace, monospace; color: var(--ink-2); }}
  textarea {{ width: 100%; min-height: 60px; border: 0; border-top: 1px solid var(--rule); padding: 10px 12px; background: transparent; color: var(--ink);
    font: 14px ui-monospace, monospace; resize: vertical; }}
  button {{ font: 600 13px system-ui, sans-serif; padding: 7px 14px; border: 0; border-radius: 2px; background: var(--ink); color: var(--bg); cursor: pointer; }}
  canvas {{ display: none; width: 100%; height: 320px; border-top: 1px solid var(--rule); }}
</style>
</head>
<body>
<main>
  <h1>{title_esc}</h1>
  <p class="sub">Made with <a href="https://github.com/anonymousAAK/adarshLang">AdarshLang</a>. Runs entirely in this page, even offline.</p>
  <div class="panel">
    <div class="head"><span id="status">Chal raha hai…</span><button id="again" type="button">Dobara chalao</button></div>
    <pre id="out"></pre>
    <canvas id="turtle"></canvas>
  </div>
  <div class="panel"><details><summary>Input for padho() (one value per line)</summary><textarea id="input"></textarea></details></div>
  <div class="panel"><details><summary>Code dekho</summary><pre>{code_html}</pre></details></div>
</main>
<script>
var FILES = [{files}];
var WASM_B64 = "{wasm}";
var WORKER = `
let wasm = null; const enc = new TextEncoder(), dec = new TextDecoder();
const env = {{
  host_now: () => Date.now(), host_tz: () => -new Date().getTimezoneOffset(),
  host_sleep: (ms) => {{ const t = Date.now() + Math.min(ms, 10000); while (Date.now() < t) {{}} }},
  host_write: (p, n) => postMessage({{ chunk: dec.decode(new Uint8Array(wasm.memory.buffer, p, n).slice()) }}),
  host_read_line: () => -1,
}};
function put(t) {{ const b = enc.encode(t); const p = wasm.alloc(b.length || 1); new Uint8Array(wasm.memory.buffer, p, b.length).set(b); return [p, b.length]; }}
onmessage = async (e) => {{
  const bin = atob(e.data.wasm); const bytes = new Uint8Array(bin.length);
  for (let i = 0; i < bin.length; i++) bytes[i] = bin.charCodeAt(i);
  wasm = (await WebAssembly.instantiate(bytes, {{ env }})).instance.exports;
  wasm.clear_files();
  for (const f of e.data.files.slice(1)) {{ const n = put(f.name), s = put(f.code); wasm.set_file(n[0], n[1], s[0], s[1]); }}
  const s = put(e.data.files[0].code), i = put(e.data.input);
  wasm.run(s[0], s[1], i[0], i[1], 0);
  const parts = dec.decode(new Uint8Array(wasm.memory.buffer, wasm.result_ptr(), wasm.result_len()).slice()).split('\u0000');
  postMessage({{ done: true, output: parts[0], error: parts[1] || null, turtle: JSON.parse(parts[2] || '[]') }});
}};`;
function draw(canvas, lines) {{
  canvas.style.display = 'block';
  var ctx = canvas.getContext('2d'), dpr = devicePixelRatio || 1, W = canvas.clientWidth, H = canvas.clientHeight;
  canvas.width = W * dpr; canvas.height = H * dpr; ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
  var x0 = 0, y0 = 0, x1 = 0, y1 = 0;
  lines.forEach(function (l) {{ x0 = Math.min(x0, l[0], l[2]); x1 = Math.max(x1, l[0], l[2]); y0 = Math.min(y0, l[1], l[3]); y1 = Math.max(y1, l[1], l[3]); }});
  var s = Math.min((W - 32) / Math.max(x1 - x0, 1), (H - 32) / Math.max(y1 - y0, 1), 4);
  var ox = (W - (x1 - x0) * s) / 2 - x0 * s, oy = (H - (y1 - y0) * s) / 2 + y1 * s;
  ctx.lineCap = 'round';
  lines.forEach(function (l) {{ ctx.strokeStyle = l[4]; ctx.lineWidth = Math.max(1, l[5]); ctx.beginPath(); ctx.moveTo(ox + l[0] * s, oy - l[1] * s); ctx.lineTo(ox + l[2] * s, oy - l[3] * s); ctx.stroke(); }});
}}
function run() {{
  var out = document.getElementById('out'), status = document.getElementById('status');
  out.textContent = ''; out.className = ''; status.textContent = 'Chal raha hai…';
  var w = new Worker(URL.createObjectURL(new Blob([WORKER], {{ type: 'text/javascript' }})));
  var timer = setTimeout(function () {{ w.terminate(); out.className = 'err'; out.textContent += '\nProgram 10 second se zyada chala, isliye roka gaya.'; status.textContent = 'Roka gaya'; }}, 10000);
  w.onmessage = function (e) {{
    if (e.data.chunk !== undefined) {{ out.textContent += e.data.chunk; return; }}
    clearTimeout(timer); w.terminate();
    out.textContent = e.data.output + (e.data.error ? (e.data.output ? '\n' : '') + e.data.error : '');
    if (e.data.error) out.className = 'err';
    status.textContent = e.data.error ? 'Galti' : 'Output';
    if (e.data.turtle.length) draw(document.getElementById('turtle'), e.data.turtle);
  }};
  w.postMessage({{ wasm: WASM_B64, files: FILES, input: document.getElementById('input').value }});
}}
document.getElementById('again').onclick = run;
run();
</script>
</body>
</html>
"##,
        title_esc = esc(title),
        code_html = code_html,
        files = files_js.join(","),
        wasm = base64(WASM),
    )
}

#[cfg(test)]
mod tests {
    #[test]
    fn base64_matches_known_values() {
        assert_eq!(super::base64(b""), "");
        assert_eq!(super::base64(b"f"), "Zg==");
        assert_eq!(super::base64(b"fo"), "Zm8=");
        assert_eq!(super::base64(b"foo"), "Zm9v");
        assert_eq!(super::base64(b"foobar"), "Zm9vYmFy");
    }
}
