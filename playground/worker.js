// Runs AdarshLang programs off the main thread, so an infinite loop can be
// stopped by terminating this worker. See adarsh-rs/src/wasm.rs for the protocol.
let wasm = null;
let sab = null; // shared with the page for live padho() input
const enc = new TextEncoder();
const dec = new TextDecoder();

const env = {
  host_now: () => Date.now(),
  host_tz: () => -new Date().getTimezoneOffset(),
  host_sleep: (ms) => {
    // ruko(): block this worker (never the page) for up to 10 seconds
    const until = Date.now() + Math.min(ms, 10000);
    if (typeof SharedArrayBuffer !== 'undefined' && typeof Atomics !== 'undefined') {
      try {
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0, Math.max(0, until - Date.now()));
        return;
      } catch (e) { /* fall back to spinning */ }
    }
    while (Date.now() < until) { /* spin */ }
  },
  host_write: (ptr, len) => {
    self.postMessage({ chunk: dec.decode(new Uint8Array(wasm.memory.buffer, ptr, len).slice()) });
  },
  host_read_line: (ptr, cap) => {
    if (!sab) return -1;
    const flag = new Int32Array(sab, 0, 2);
    Atomics.store(flag, 0, 0);
    self.postMessage({ needInput: true });
    Atomics.wait(flag, 0, 0); // until the page answers
    const len = Atomics.load(flag, 1);
    if (len < 0) return -1;
    const n = Math.min(len, cap);
    new Uint8Array(wasm.memory.buffer, ptr, n).set(new Uint8Array(sab, 8, n));
    return n;
  },
};

const ready = fetch('adarshlang.wasm')
  .then((r) => r.arrayBuffer())
  .then((bytes) => WebAssembly.instantiate(bytes, { env }))
  .then(({ instance }) => { wasm = instance.exports; });

function put(text) {
  const bytes = enc.encode(text);
  const ptr = wasm.alloc(bytes.length || 1);
  new Uint8Array(wasm.memory.buffer, ptr, bytes.length).set(bytes);
  return [ptr, bytes.length];
}

function free([ptr, len]) {
  wasm.dealloc(ptr, len || 1);
}

function result() {
  return dec.decode(new Uint8Array(wasm.memory.buffer, wasm.result_ptr(), wasm.result_len()).slice()).split('\u0000');
}

function setFiles(files) {
  wasm.clear_files();
  for (const f of files || []) {
    const n = put(f.name), s = put(f.code);
    wasm.set_file(n[0], n[1], s[0], s[1]);
    free(n); free(s);
  }
}

function parse(text, fallback) {
  try { return JSON.parse(text || ''); } catch (e) { return fallback; }
}

self.onmessage = async (e) => {
  try {
    await ready;
    const d = e.data;
    if (d.warmup) {
      sab = d.sab || null;
      self.postMessage({ ready: true });
      return;
    }
    if (d.python) {
      setFiles(d.python.files);
      const src = put(d.python.code);
      wasm.to_python(src[0], src[1]);
      free(src);
      const [code, error] = result();
      self.postMessage({ python: { code, error: error || null } });
      return;
    }
    if (d.run) {
      const r = d.run;
      setFiles(r.files);
      const src = put(r.code), inp = put(r.input || '');
      const mode = (r.steps ? 1 : 0) | (r.live && sab ? 2 : 0);
      const started = performance.now();
      wasm.run(src[0], src[1], inp[0], inp[1], mode);
      const ms = performance.now() - started;
      free(src); free(inp);
      const parts = result();
      self.postMessage({
        done: true,
        output: parts[0],
        error: parts[1] || null,
        turtle: parse(parts[2], []),
        steps: parse(parts[3], []),
        warnings: parse(parts[4], []),
        ms,
      });
    }
  } catch (err) {
    self.postMessage({ done: true, output: '', error: 'Playground error: ' + err.message, turtle: [], steps: [], warnings: [], ms: 0 });
  }
};
