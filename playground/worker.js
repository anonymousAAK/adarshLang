// Runs AdarshLang programs off the main thread, so an infinite loop can be
// stopped by terminating this worker.
let wasm = null;
const ready = fetch('adarshlang.wasm')
  .then((r) => r.arrayBuffer())
  .then((bytes) => WebAssembly.instantiate(bytes, {}))
  .then(({ instance }) => { wasm = instance.exports; });

const enc = new TextEncoder();
const dec = new TextDecoder();

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
    const text = dec.decode(res.slice());
    const cut = text.indexOf('\u0000');
    self.postMessage({ output: text.slice(0, cut), error: text.slice(cut + 1) || null, ms });
  } catch (err) {
    self.postMessage({ output: '', error: 'Playground error: ' + err.message, ms: 0 });
  }
};
