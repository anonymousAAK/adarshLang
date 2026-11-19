// Shared playground code: running programs in a Web Worker, share links,
// syntax highlighting and turtle drawing. Used by index.html and embed.html.
(function (global) {
  'use strict';

  var TIME_LIMIT_MS = 5000;

  // ------------------------------------------------------------ runner

  // Runs AdarshLang in a Web Worker (the Rust interpreter compiled to WebAssembly).
  // A program that runs too long is stopped by terminating the worker.
  function Runner() {
    this.worker = null;
    this.ready = false;
    this.onReady = null;
    // live keyboard input for padho() needs SharedArrayBuffer (cross-origin isolated pages)
    this.live = typeof SharedArrayBuffer !== 'undefined' && global.crossOriginIsolated === true;
    this.sab = this.live ? new SharedArrayBuffer(8 + 8192) : null;
  }

  Runner.prototype.start = function () {
    if (this.worker) return this.worker;
    var self = this;
    this.ready = false;
    this.worker = new Worker('worker.js');
    this.worker.addEventListener('message', function (e) {
      if (e.data.ready) {
        self.ready = true;
        if (self.onReady) self.onReady();
      }
    });
    this.worker.postMessage({ warmup: true, sab: this.sab });
    return this.worker;
  };

  Runner.prototype.kill = function () {
    if (this.worker) this.worker.terminate();
    this.worker = null;
  };

  // opts: { code, input, files: [{name, code}], steps, onChunk(text), onNeedInput(answer) }
  // answer(text) supplies a line; answer(null) means end of input.
  Runner.prototype.run = function (opts) {
    var self = this;
    return new Promise(function (resolve, reject) {
      var w = self.start();
      var timer = null;
      var arm = function () {
        clearTimeout(timer);
        timer = setTimeout(function () {
          self.kill();
          reject(new Error('Galti: Program ' + (TIME_LIMIT_MS / 1000) + ' second se zyada chala, isliye roka gaya. Shayad koi infinite loop hai?'));
        }, TIME_LIMIT_MS);
      };
      arm();
      w.onmessage = function (e) {
        var d = e.data;
        if (d.ready) return;
        if (d.chunk !== undefined) { if (opts.onChunk) opts.onChunk(d.chunk); return; }
        if (d.needInput) {
          clearTimeout(timer);   // waiting for a person doesn't count
          var answer = function (text) { self.answer(text); arm(); };
          if (opts.onNeedInput) opts.onNeedInput(answer); else answer(null);
          return;
        }
        if (d.done) { clearTimeout(timer); resolve(d); }
      };
      w.onerror = function (e) { clearTimeout(timer); self.kill(); reject(new Error('Playground error: ' + e.message)); };
      w.postMessage({
        run: { code: opts.code, input: opts.input || '', files: opts.files || [], steps: !!opts.steps, live: self.live && !!opts.onNeedInput },
      });
    });
  };

  Runner.prototype.answer = function (text) {
    var flag = new Int32Array(this.sab, 0, 2);
    if (text === null) {
      Atomics.store(flag, 1, -1);
    } else {
      var bytes = new TextEncoder().encode(text).slice(0, 8192);
      new Uint8Array(this.sab, 8, 8192).set(bytes);
      Atomics.store(flag, 1, bytes.length);
    }
    Atomics.store(flag, 0, 1);
    Atomics.notify(flag, 0);
  };

  Runner.prototype.python = function (code, files) {
    var self = this;
    return new Promise(function (resolve, reject) {
      var w = self.start();
      w.onmessage = function (e) { if (e.data.python) resolve(e.data.python); };
      w.onerror = function (e) { reject(new Error(e.message)); };
      w.postMessage({ python: { code: code, files: files || [] } });
    });
  };

  // ------------------------------------------------------------ share links

  function toBase64Url(bytes) {
    var s = '';
    for (var i = 0; i < bytes.length; i++) s += String.fromCharCode(bytes[i]);
    return btoa(s).replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
  }
  function fromBase64Url(text) {
    var b = atob(text.replace(/-/g, '+').replace(/_/g, '/'));
    var out = new Uint8Array(b.length);
    for (var i = 0; i < b.length; i++) out[i] = b.charCodeAt(i);
    return out;
  }
  async function pipe(bytes, stream) {
    var res = new Response(new Blob([bytes]).stream().pipeThrough(stream));
    return new Uint8Array(await res.arrayBuffer());
  }
  // files: [{name, code}] → compact URL-safe text
  async function encodeFiles(files) {
    var json = JSON.stringify(files.map(function (f) { return [f.name, f.code]; }));
    var bytes = new TextEncoder().encode(json);
    if (typeof CompressionStream !== 'undefined') {
      return 'z' + toBase64Url(await pipe(bytes, new CompressionStream('deflate-raw')));
    }
    return 'j' + toBase64Url(bytes);
  }
  async function decodeFiles(text) {
    var kind = text[0], bytes = fromBase64Url(text.slice(1));
    if (kind === 'z') bytes = await pipe(bytes, new DecompressionStream('deflate-raw'));
    var arr = JSON.parse(new TextDecoder().decode(bytes));
    return arr.map(function (p) { return { name: String(p[0]), code: String(p[1]) }; });
  }

  // ------------------------------------------------------------ highlighting

  var KEYWORDS = ('badlo pakka kaam wapas agar warna jabtak karo dikhao bas aage_badho ginnati ke_liye in chuno case ' +
    'warna_case pakdo chhoddo aakhir lao as dhacha vikalp baaki aur ya nahin hai dete_raho yeh').split(' ');
  var CONSTANTS = 'sahi_hai_be jhuth khali PI E'.split(' ');
  var DEVANAGARI_KW = 'बदलो काम वापस अगर वरना जबतक दिखाओ बस सही झूठ गिनती के_लिए चुनो पकड़ो छोड़ो लाओ खाली और या नहीं पक्का करो है आखिर'.split(' ');
  var BUILTINS = [];

  function esc(s) {
    return s.replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;');
  }

  // Returns HTML with <span class="t-..."> around tokens.
  function highlight(src) {
    var out = '', i = 0, n = src.length;
    var span = function (cls, text) { out += '<span class="t-' + cls + '">' + esc(text) + '</span>'; };
    while (i < n) {
      var c = src[i], rest;
      if (c === '#' && src[i + 1] === '[') {
        var end = src.indexOf(']#', i + 2);
        end = end < 0 ? n : end + 2;
        span('c', src.slice(i, end)); i = end; continue;
      }
      if (c === '#') {
        var e2 = src.indexOf('\n', i); if (e2 < 0) e2 = n;
        span(src[i + 1] === '#' ? 'doc' : 'c', src.slice(i, e2)); i = e2; continue;
      }
      if (c === 'r' && src[i + 1] === '"' && !/[\wऀ-ॿ]/.test(src[i - 1] || '')) {
        var triple = src.substr(i + 1, 3) === '"""';
        var close = triple ? src.indexOf('"""', i + 4) : src.indexOf('"', i + 2);
        var stop = close < 0 ? n : close + (triple ? 3 : 1);
        span('s', src.slice(i, stop)); i = stop; continue;
      }
      if (c === '"') {
        var tq = src.substr(i, 3) === '"""';
        var j = i + (tq ? 3 : 1), buf = tq ? '"""' : '"';
        out += '<span class="t-s">';
        while (j < n) {
          if (src[j] === '\\') { buf += src.substr(j, 2); j += 2; continue; }
          if (tq ? src.substr(j, 3) === '"""' : src[j] === '"') { buf += tq ? '"""' : '"'; j += tq ? 3 : 1; break; }
          if (src[j] === '{') {
            var depth = 1, k = j + 1;
            while (k < n && depth > 0) { if (src[k] === '{') depth++; else if (src[k] === '}') depth--; if (depth > 0) k++; }
            out += esc(buf); buf = '';
            out += '</span><span class="t-i">{</span>' + highlight(src.slice(j + 1, k)) + '<span class="t-i">}</span><span class="t-s">';
            j = Math.min(k + 1, n); continue;
          }
          buf += src[j]; j++;
        }
        out += esc(buf) + '</span>'; i = j; continue;
      }
      if (/[0-9]/.test(c) || (c === '.' && /[0-9]/.test(src[i + 1] || '') && !/[\w)\]]/.test(src[i - 1] || ''))) {
        var m = /^(?:\d[\d_]*)?(?:\.\d[\d_]*)?(?:[eE][+-]?\d+)?/.exec(src.slice(i));
        var num = m && m[0] ? m[0] : c;
        span('n', num); i += num.length; continue;
      }
      rest = /^[A-Za-z_ऀ-ॿ][\wऀ-ॿ]*/.exec(src.slice(i));
      if (rest) {
        var w = rest[0];
        if (KEYWORDS.indexOf(w) >= 0 || DEVANAGARI_KW.indexOf(w) >= 0) span('k', w);
        else if (CONSTANTS.indexOf(w) >= 0) span('b', w);
        else if (BUILTINS.indexOf(w) >= 0 || w === 'kachhua') span('f', w);
        else if (/^[A-Z]/.test(w)) span('t', w);
        else out += esc(w);
        i += w.length; continue;
      }
      out += esc(c); i++;
    }
    return out;
  }

  // ------------------------------------------------------------ turtle

  function drawTurtle(canvas, lines) {
    var ctx = canvas.getContext('2d');
    var dpr = global.devicePixelRatio || 1;
    var cssW = canvas.clientWidth || 400, cssH = canvas.clientHeight || 300;
    canvas.width = cssW * dpr; canvas.height = cssH * dpr;
    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, cssW, cssH);
    if (!lines || !lines.length) return;
    var minx = 0, miny = 0, maxx = 0, maxy = 0;
    lines.forEach(function (l) {
      minx = Math.min(minx, l[0], l[2]); maxx = Math.max(maxx, l[0], l[2]);
      miny = Math.min(miny, l[1], l[3]); maxy = Math.max(maxy, l[1], l[3]);
    });
    var pad = 16, w = Math.max(maxx - minx, 1), h = Math.max(maxy - miny, 1);
    var scale = Math.min((cssW - 2 * pad) / w, (cssH - 2 * pad) / h, 4);
    var ox = (cssW - w * scale) / 2 - minx * scale, oy = (cssH - h * scale) / 2 + maxy * scale;
    ctx.lineCap = 'round';
    lines.forEach(function (l) {
      ctx.strokeStyle = l[4]; ctx.lineWidth = Math.max(1, l[5] * Math.min(scale, 2));
      ctx.beginPath();
      ctx.moveTo(ox + l[0] * scale, oy - l[1] * scale);
      ctx.lineTo(ox + l[2] * scale, oy - l[3] * scale);
      ctx.stroke();
    });
  }

  // ANSI colours from rang_mein() → spans
  function ansiToHtml(text) {
    var map = { 30: '#14161a', 31: '#c23a48', 32: '#0f6b4f', 33: '#9a6200', 34: '#3456c8', 35: '#7a3fb0', 36: '#1f7a8c', 37: '#888' };
    return esc(text).replace(/\x1b\[(\d+)m([\s\S]*?)\x1b\[0m/g, function (_, code, body) {
      if (code === '1') return '<b>' + body + '</b>';
      return '<span style="color:' + (map[code] || 'inherit') + '">' + body + '</span>';
    });
  }

  global.Adarsh = {
    Runner: Runner,
    encodeFiles: encodeFiles,
    decodeFiles: decodeFiles,
    highlight: highlight,
    setBuiltins: function (list) { BUILTINS = list; },
    keywords: KEYWORDS.concat(CONSTANTS),
    drawTurtle: drawTurtle,
    ansiToHtml: ansiToHtml,
    esc: esc,
  };
})(this);
