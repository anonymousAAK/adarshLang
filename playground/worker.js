// Runs AdarshLang programs off the main thread, so an infinite loop can be
// stopped by terminating this worker.
let wasm = null;
const enc = new TextEncoder();
const dec = new TextDecoder();

// Functions the interpreter calls back into (see adarsh-rs/src/wasm.rs).
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
    // live output while the program runs
    const text = dec.decode(new Uint8Array(wasm.memory.buffer, ptr, len).slice());
    self.postMessage({ chunk: text });
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

self.onmessage = async (e) => {
  try {
    await ready;
    if (e.data.warmup) {
      self.postMessage({ ready: true });
      return;
    }
    const { code, input } = e.data;
    const [sp, sl] = put(code);
    const [ip, il] = put(input || '');
    const started = performance.now();
    wasm.run(sp, sl, ip, il);
    const ms = performance.now() - started;
    wasm.dealloc(sp, sl || 1);
    wasm.dealloc(ip, il || 1);
    const res = new Uint8Array(wasm.memory.buffer, wasm.result_ptr(), wasm.result_len());
    const parts = dec.decode(res.slice()).split('\u0000');
    let turtle = [];
    try { turtle = JSON.parse(parts[2] || '[]'); } catch (err) { turtle = []; }
    self.postMessage({ done: true, output: parts[0], error: parts[1] || null, turtle, ms });
  } catch (err) {
    self.postMessage({ done: true, output: '', error: 'Playground error: ' + err.message, turtle: [], ms: 0 });
  }
};
