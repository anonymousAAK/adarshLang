from flask import Flask, request, render_template_string, jsonify
import sys
from io import StringIO
from adarsh_lang_compiler import (
    adarshlang_compile_and_run,
    AdarshRuntimeError,
    AdarshUserException,
    AdarshSemanticError,
    AdarshParserError,
)

# Language-level errors we surface to the user as friendly output.
ADARSH_ERRORS = (
    AdarshRuntimeError,
    AdarshUserException,
    AdarshSemanticError,
    AdarshParserError,
)

app = Flask(__name__)

# --------------------------------------------------------------------
# MAIN PAGE TEMPLATE (Bootstrap + Tutorial Section)
# --------------------------------------------------------------------
HOME_PAGE_TEMPLATE = """
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>AdarshLang</title>
  <link rel="preconnect" href="https://fonts.googleapis.com">
  <style>
    :root {
      --bg: #ffffff;
      --bg-alt: #fbfbfd;
      --section: #f5f5f7;
      --text: #1d1d1f;
      --text-dim: #86868b;
      --accent: #0071e3;
      --link: #0066cc;
      --border: #d2d2d7;
      --hairline: rgba(0, 0, 0, 0.08);
      --surface: #ffffff;
      --editor-bg: #fbfbfd;
      --editor-text: #1d1d1f;
      --output-bg: #1d1d1f;
      --output-text: #f5f5f7;
      --nav-bg: rgba(255, 255, 255, 0.72);
      --error: #e30000;
      --success: #1d8a2b;
      --mono: ui-monospace, "SF Mono", SFMono-Regular, Menlo, Monaco, "Cascadia Mono", monospace;
    }
    [data-theme="dark"] {
      --bg: #000000;
      --bg-alt: #0a0a0a;
      --section: #101012;
      --text: #f5f5f7;
      --text-dim: #86868b;
      --accent: #2997ff;
      --link: #2997ff;
      --border: #2a2a2c;
      --hairline: rgba(255, 255, 255, 0.10);
      --surface: #1c1c1e;
      --editor-bg: #161617;
      --editor-text: #f5f5f7;
      --output-bg: #161617;
      --output-text: #f5f5f7;
      --nav-bg: rgba(22, 22, 23, 0.72);
      --error: #ff6961;
      --success: #30d158;
    }

    * { box-sizing: border-box; }
    html { scroll-behavior: smooth; -webkit-text-size-adjust: 100%; }
    body {
      margin: 0;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "SF Pro Display", "Helvetica Neue", Helvetica, Arial, sans-serif;
      color: var(--text);
      background: var(--bg);
      -webkit-font-smoothing: antialiased;
      -moz-osx-font-smoothing: grayscale;
      line-height: 1.47059;
      letter-spacing: -0.01em;
      font-size: 17px;
      transition: background-color .5s ease, color .5s ease;
    }
    a { color: var(--link); text-decoration: none; }
    a:hover { text-decoration: underline; }
    code {
      font-family: var(--mono);
      font-size: 0.84em;
      background: var(--section);
      color: var(--text);
      padding: 1px 6px;
      border-radius: 5px;
    }

    /* Nav — apple.com style: thin, centered, translucent */
    .nav {
      position: sticky; top: 0; z-index: 50;
      backdrop-filter: saturate(180%) blur(20px);
      -webkit-backdrop-filter: saturate(180%) blur(20px);
      background: var(--nav-bg);
      border-bottom: 1px solid var(--hairline);
    }
    .nav-inner {
      max-width: 1024px; margin: 0 auto; height: 48px;
      display: flex; align-items: center; justify-content: space-between;
      padding: 0 22px;
    }
    .nav-brand { font-weight: 600; font-size: 19px; letter-spacing: -0.02em; color: var(--text); }
    .nav-links { display: flex; align-items: center; gap: 4px; }
    .nav-link { color: var(--text); opacity: .8; font-size: 13px; padding: 6px 11px; border-radius: 6px; transition: opacity .2s; }
    .nav-link:hover { opacity: 1; text-decoration: none; }
    .theme-toggle {
      width: 30px; height: 30px; border-radius: 50%; border: none;
      background: transparent; color: var(--text); cursor: pointer; font-size: 15px;
      display: grid; place-items: center; opacity: .8; transition: opacity .2s;
    }
    .theme-toggle:hover { opacity: 1; }

    /* Hero */
    .hero { text-align: center; padding: clamp(70px, 13vw, 150px) 22px clamp(40px, 7vw, 80px); max-width: 900px; margin: 0 auto; }
    .eyebrow { font-size: clamp(19px, 3vw, 24px); font-weight: 600; color: var(--accent); letter-spacing: -0.01em; margin: 0 0 6px; }
    .hero h1 {
      font-size: clamp(48px, 9vw, 96px); line-height: 1.03; letter-spacing: -0.025em;
      font-weight: 700; margin: 0 0 22px; color: var(--text);
    }
    .hero p {
      font-size: clamp(19px, 2.6vw, 26px); line-height: 1.38; font-weight: 400;
      color: var(--text-dim); max-width: 680px; margin: 0 auto;
    }
    .hero .cta { margin-top: 30px; display: flex; gap: 16px; justify-content: center; flex-wrap: wrap; }

    .section { max-width: 1024px; margin: 0 auto; padding: 0 22px; }
    .section-head { text-align: center; max-width: 720px; margin: clamp(50px, 9vw, 100px) auto clamp(28px, 4vw, 44px); }
    .section-head h2 { font-size: clamp(32px, 5vw, 48px); line-height: 1.08; letter-spacing: -0.02em; font-weight: 700; margin: 0 0 10px; }
    .section-head p { font-size: clamp(17px, 2.2vw, 21px); color: var(--text-dim); margin: 0; }

    /* Editor card */
    .panel {
      background: var(--surface);
      border: 1px solid var(--border);
      border-radius: 18px;
      overflow: hidden;
    }
    .editor-bar {
      display: flex; align-items: center; justify-content: space-between;
      padding: 13px 18px; border-bottom: 1px solid var(--hairline);
      background: var(--bg-alt);
    }
    .editor-file { font-family: var(--mono); font-size: 13px; color: var(--text-dim); }
    .editor-tag { font-size: 12px; color: var(--text-dim); }
    textarea#source_code {
      width: 100%; min-height: 340px; resize: vertical; border: 0; outline: none; display: block;
      padding: 22px 24px; background: var(--editor-bg); color: var(--editor-text);
      font-family: var(--mono); font-size: 14px; line-height: 1.7; tab-size: 4;
      letter-spacing: 0;
    }
    textarea#source_code::placeholder { color: var(--text-dim); }

    .toolbar { display: flex; gap: 12px; align-items: center; margin-top: 20px; flex-wrap: wrap; }
    .btn {
      font: inherit; font-size: 16px; font-weight: 400; cursor: pointer; border: 1px solid transparent;
      border-radius: 980px; padding: 9px 22px; transition: background .25s, color .25s, border-color .25s, opacity .2s;
      display: inline-flex; align-items: center; gap: 7px; line-height: 1.2; letter-spacing: -0.01em;
    }
    .btn-primary { background: var(--accent); color: #fff; }
    .btn-primary:hover { opacity: .88; }
    .btn-primary:disabled { opacity: .5; cursor: progress; }
    .btn-secondary { background: transparent; color: var(--link); border-color: transparent; padding: 9px 14px; }
    .btn-secondary:hover { text-decoration: underline; }
    .btn-outline { background: transparent; color: var(--text); border-color: var(--border); }
    .btn-outline:hover { background: var(--section); }
    .spin { display: inline-block; animation: spin 0.7s linear infinite; }
    @keyframes spin { to { transform: rotate(360deg); } }
    .kbd-hint { color: var(--text-dim); font-size: 13px; margin-left: auto; }
    .kbd { font-family: var(--mono); font-size: 12px; background: var(--section); border: 1px solid var(--hairline); border-radius: 5px; padding: 2px 6px; }

    /* Output */
    .output-wrap { margin-top: 20px; display: none; }
    .output-head { display: flex; align-items: center; justify-content: space-between; margin-bottom: 10px; }
    .output-label { display: flex; align-items: center; gap: 9px; font-weight: 600; font-size: 15px; }
    .status-dot { width: 8px; height: 8px; border-radius: 50%; background: var(--success); }
    .status-dot.err { background: var(--error); }
    pre#output-pre {
      margin: 0; background: var(--output-bg); color: var(--output-text);
      border: 1px solid var(--border); border-radius: 14px; padding: 20px 22px;
      min-height: 64px; max-height: 460px; overflow-y: auto;
      white-space: pre-wrap; word-wrap: break-word;
      font-family: var(--mono); font-size: 13px; line-height: 1.65;
    }
    pre#output-pre.error-output { color: var(--error); }

    /* Examples — quiet chips */
    .chips { display: flex; flex-wrap: wrap; gap: 10px; justify-content: center; }
    .chip {
      font: inherit; font-size: 14px; font-weight: 400; cursor: pointer;
      background: var(--section); color: var(--text); border: 1px solid transparent;
      border-radius: 980px; padding: 8px 17px; transition: background .2s, color .2s;
      letter-spacing: -0.01em;
    }
    .chip:hover { background: var(--border); }
    .chip.featured { background: var(--text); color: var(--bg); }
    .chip.featured:hover { opacity: .85; }

    /* Feature grid — clean typographic columns */
    .feature-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1px; background: var(--hairline); border: 1px solid var(--hairline); border-radius: 18px; overflow: hidden; }
    .feature { background: var(--bg); padding: 26px 28px; }
    .feature h3 { margin: 0 0 14px; font-size: 19px; letter-spacing: -0.015em; font-weight: 600; }
    .feature ul { margin: 0; padding: 0; list-style: none; }
    .feature li { padding: 5px 0; font-size: 15px; color: var(--text-dim); line-height: 1.45; }
    .feature li strong { color: var(--text); font-weight: 600; }
    .feature code { font-size: 13px; }

    .footer { border-top: 1px solid var(--hairline); margin-top: clamp(60px, 10vw, 110px); }
    .footer-inner { max-width: 1024px; margin: 0 auto; padding: 24px 22px; color: var(--text-dim); font-size: 12px; }

    ::-webkit-scrollbar { width: 10px; height: 10px; }
    ::-webkit-scrollbar-thumb { background: rgba(140,140,150,.4); border-radius: 8px; border: 3px solid transparent; background-clip: content-box; }
    ::-webkit-scrollbar-thumb:hover { background: rgba(140,140,150,.6); background-clip: content-box; }

    @media (max-width: 600px) {
      .nav-link { padding: 6px 8px; }
      .kbd-hint { display: none; }
    }
  </style>
</head>
<body>

<nav class="nav">
  <div class="nav-inner">
    <span class="nav-brand">AdarshLang</span>
    <div class="nav-links">
      <a class="nav-link" href="#playground">Playground</a>
      <a class="nav-link" href="#examples">Examples</a>
      <a class="nav-link" href="#docs">Docs</a>
      <button class="theme-toggle" id="theme-toggle" onclick="toggleTheme()" title="Toggle appearance" aria-label="Toggle appearance">&#9789;</button>
    </div>
  </div>
</nav>

<header class="hero">
  <p class="eyebrow">AdarshLang</p>
  <h1>Your language.<br>Your code.</h1>
  <p>A Hinglish programming language with a complete compiler. Write <code>badlo</code>, <code>dikhao</code>, <code>agar</code> — and run it instantly, right here in your browser.</p>
  <div class="cta">
    <a class="btn btn-primary" href="#playground">Open the Playground</a>
    <a class="btn btn-secondary" href="#docs">Learn the language &rsaquo;</a>
  </div>
</header>

<main>

  <!-- Code Editor -->
  <section class="section" id="playground">
    <div class="section-head">
      <h2>The Playground</h2>
      <p>Write AdarshLang and run it in real time.</p>
    </div>

    <form id="codeForm" onsubmit="runCode(event)">
      <div class="panel">
        <div class="editor-bar">
          <span class="editor-file">playground.aak</span>
          <span class="editor-tag">AdarshLang</span>
        </div>
        <textarea id="source_code" name="source_code" spellcheck="false"
                  placeholder="badlo naam = &quot;Adarsh&quot;;&#10;dikhao(&quot;Namaste, &quot; + naam + &quot;!&quot;);"></textarea>
      </div>
      <div class="toolbar">
        <button type="submit" id="run-btn" class="btn btn-primary">Run</button>
        <button type="button" class="btn btn-outline" onclick="copyCode()">Copy</button>
        <button type="button" class="btn btn-outline" onclick="resetEditor()">Reset</button>
        <span class="kbd-hint"><span class="kbd">&#8984;</span> + <span class="kbd">&crarr;</span> to run</span>
      </div>
    </form>

    <!-- Inline Output -->
    <div id="output-card" class="output-wrap">
      <div class="output-head">
        <div class="output-label"><span class="status-dot" id="status-dot"></span><span id="output-label">Output</span></div>
        <button class="btn btn-secondary" style="padding:4px 10px;font-size:13px;" onclick="clearOutput()">Clear</button>
      </div>
      <pre id="output-pre"></pre>
    </div>
  </section>

  <!-- One-click snippet buttons -->
  <section class="section" id="examples">
    <div class="section-head">
      <h2>Start from an example</h2>
      <p>Tap any example to load it into the playground, then Run.</p>
    </div>
    <div class="chips">
      <button class="chip" onclick="loadSnippet('hello')">Hello World</button>
      <button class="chip" onclick="loadSnippet('variables')">Variables &amp; Types</button>
      <button class="chip" onclick="loadSnippet('ifelse')">If / Else</button>
      <button class="chip" onclick="loadSnippet('loops')">Loops</button>
      <button class="chip" onclick="loadSnippet('functions')">Functions</button>
      <button class="chip" onclick="loadSnippet('lists')">Lists</button>
      <button class="chip" onclick="loadSnippet('dicts')">Dictionaries</button>
      <button class="chip" onclick="loadSnippet('structs')">Structs (dhacha)</button>
      <button class="chip" onclick="loadSnippet('switch')">Switch (chuno)</button>
      <button class="chip" onclick="loadSnippet('trycatch')">Try / Catch</button>
      <button class="chip" onclick="loadSnippet('math')">Math Builtins</button>
      <button class="chip" onclick="loadSnippet('strings')">String Builtins</button>
      <button class="chip" onclick="loadSnippet('functional')">Map / Filter / Reduce</button>
      <button class="chip" onclick="loadSnippet('compound')">Compound Assignment</button>
      <button class="chip" onclick="loadSnippet('power_mod')">Power &amp; Modulo</button>
      <button class="chip" onclick="loadSnippet('null')">Null (khali)</button>
      <button class="chip" onclick="loadSnippet('typecheck')">Type Checking</button>
      <button class="chip" onclick="loadSnippet('range')">Range &amp; Iteration</button>
      <button class="chip" onclick="loadSnippet('collections')">Collection Helpers</button>
      <button class="chip" onclick="loadSnippet('fizzbuzz')">FizzBuzz</button>
      <button class="chip featured" onclick="loadSnippet('all_features')">All Features Demo</button>
    </div>
  </section>

  <!-- Tutorial Section -->
  <section class="section" id="docs">
    <div class="section-head">
      <h2>Everything it can do</h2>
      <p>A Hinglish language with the depth of a real one.</p>
    </div>

    <div class="feature-grid">
      <div class="feature">
        <h3>Basics</h3>
        <ul>
          <li><strong>Variables:</strong> <code>badlo x = 10;</code></li>
          <li><strong>Printing:</strong> <code>dikhao(x);</code></li>
          <li><strong>Booleans:</strong> <code>sahi_hai_be</code> / <code>jhuth</code></li>
          <li><strong>Null:</strong> <code>khali</code></li>
          <li><strong>Comments:</strong> <code># yeh comment hai</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Operators</h3>
        <ul>
          <li><strong>Arithmetic:</strong> <code>+ - * /</code> <code>%</code> <code>**</code></li>
          <li><strong>Comparison:</strong> <code>== != &lt; &gt; &lt;= &gt;=</code></li>
          <li><strong>Logical:</strong> <code>&amp;&amp;</code> <code>||</code> <code>!</code></li>
          <li><strong>Compound:</strong> <code>+= -= *= /= %=</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Control Flow</h3>
        <ul>
          <li><strong>If/Else:</strong> <code>agar (...) { } warna { }</code></li>
          <li><strong>While:</strong> <code>jabtak (y &gt; 0) { }</code></li>
          <li><strong>For:</strong> <code>ginnati (...; ...; ...) { }</code></li>
          <li><strong>For-each:</strong> <code>ke_liye (badlo i in list)</code></li>
          <li><strong>Switch:</strong> <code>chuno (expr) { case ... }</code></li>
          <li><strong>Break/Continue:</strong> <code>bas;</code> / <code>aage_badho;</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Functions</h3>
        <ul>
          <li><strong>Define:</strong> <code>kaam add(a, b) { wapas a+b; }</code></li>
          <li><strong>Defaults:</strong> <code>kaam g(n, m = "hi") { }</code></li>
          <li><strong>Varargs:</strong> <code>kaam total(baaki nums) { }</code></li>
          <li><strong>Anonymous:</strong> <code>kaam (x) { wapas x*2; }</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Collections</h3>
        <ul>
          <li><strong>Lists:</strong> <code>[1,2,3]</code> + push, pop, sort, slice</li>
          <li><strong>Dicts:</strong> <code>{"key": "value"}</code> + keys, values</li>
          <li><strong>Structs:</strong> <code>dhacha Vyakti { naam, umar };</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Built-in Functions</h3>
        <ul>
          <li><strong>Math:</strong> <code>abs floor ceil round_val min_val max_val</code></li>
          <li><strong>Strings:</strong> <code>upper lower join split trim replace find</code></li>
          <li><strong>Type:</strong> <code>prakar shabdme sankhya</code></li>
          <li><strong>Iteration:</strong> <code>range contains</code></li>
          <li><strong>Functional:</strong> <code>map filter reduce</code></li>
        </ul>
      </div>

      <div class="feature">
        <h3>Errors &amp; Modules</h3>
        <ul>
          <li><strong>Try/Catch:</strong> <code>pakdo { } chhoddo (err) { }</code></li>
          <li><strong>Throw:</strong> <code>chhoddo("error message");</code></li>
          <li><strong>Import:</strong> <code>lao "module.aak";</code></li>
        </ul>
      </div>
    </div>
  </section>
</main>

<footer class="footer">
  <div class="footer-inner">
    AdarshLang &mdash; a Hinglish programming language. Apni bhasha, apna code.
  </div>
</footer>

<script>
var snippets = {

hello: `# Hello World in AdarshLang!
dikhao("Namaste Duniya!");
dikhao("AdarshLang mein aapka swagat hai!");`,

variables: `# Variables & Types
badlo naam = "Adarsh";
badlo umar = 24;
badlo pi = 3.14;
badlo active = sahi_hai_be;
badlo kuch_nahi = khali;

dikhao("Naam: ");
dikhao(naam);
dikhao("Umar: ");
dikhao(umar);
dikhao("Pi: ");
dikhao(pi);
dikhao("Active: ");
dikhao(active);
dikhao("Kuch nahi: ");
dikhao(kuch_nahi);`,

ifelse: `# If / Else (agar / warna)
badlo marks = 75;

agar (marks >= 90) {
    dikhao("Grade: A+");
} warna agar (marks >= 80) {
    dikhao("Grade: A");
} warna agar (marks >= 70) {
    dikhao("Grade: B");
} warna agar (marks >= 60) {
    dikhao("Grade: C");
} warna {
    dikhao("Grade: F - padhai karo!");
}

# Logical operators
badlo age = 20;
badlo hasID = sahi_hai_be;
agar (age >= 18 && hasID) {
    dikhao("Entry allowed!");
} warna {
    dikhao("Entry denied!");
}`,

loops: `# While loop (jabtak)
badlo countdown = 5;
dikhao("Countdown shuru:");
jabtak (countdown > 0) {
    dikhao(countdown);
    countdown -= 1;
}
dikhao("Blast off!");

# For loop (ginnati) with +=
dikhao("\\nSquares:");
ginnati (badlo i = 1; i <= 5; i += 1) {
    dikhao(i ** 2);
}

# For-each (ke_liye)
dikhao("\\nFruits:");
badlo fruits = ["Seb", "Kela", "Amrud", "Aam"];
ke_liye (badlo fruit in fruits) {
    dikhao(fruit);
}

# Break (bas) and Continue (aage_badho)
dikhao("\\nSkip 3, stop at 7:");
ginnati (badlo i = 1; i <= 10; i += 1) {
    agar (i == 3) { aage_badho; }
    agar (i == 7) { bas; }
    dikhao(i);
}`,

functions: `# Functions (kaam)
kaam namaste(naam) {
    dikhao("Namaste, " + naam + "!");
}
namaste("Adarsh");
namaste("Duniya");

# Return values (wapas)
kaam add(a, b) {
    wapas a + b;
}
dikhao("3 + 7 = ");
dikhao(add(3, 7));

# Default parameters
kaam greet(naam, msg = "Kaise ho?") {
    dikhao(naam + " - " + msg);
}
greet("Rahul");
greet("Priya", "Bahut acche!");

# Varargs (baaki)
kaam totalKar(baaki nums) {
    badlo sum = 0;
    ke_liye (badlo n in nums) {
        sum += n;
    }
    wapas sum;
}
dikhao("Total: ");
dikhao(totalKar(10, 20, 30, 40));

# Anonymous functions
badlo square = kaam (x) { wapas x ** 2; };
dikhao("5 squared = ");
dikhao(square(5));`,

lists: `# Lists
badlo numbers = [10, 20, 30, 40, 50];
dikhao("Original: ");
dikhao(numbers);

# Length
dikhao("Length: ");
dikhao(length(numbers));

# Push & Pop
push(numbers, 60);
dikhao("After push(60): ");
dikhao(numbers);

badlo removed = pop(numbers);
dikhao("Popped: ");
dikhao(removed);

# Index access & update
dikhao("Element [2]: ");
dikhao(numbers[2]);
numbers[0] = 99;
dikhao("After numbers[0]=99: ");
dikhao(numbers);

# Sort & Reverse
badlo messy = [5, 1, 4, 2, 3];
dikhao("Sorted: ");
dikhao(sort(messy));
dikhao("Reversed: ");
dikhao(reverse(messy));

# Slice
dikhao("Slice [1:4]: ");
dikhao(slice(numbers, 1, 4));

# Contains & Find
dikhao("Contains 30? ");
dikhao(contains(numbers, 30));
dikhao("Index of 30: ");
dikhao(find(numbers, 30));`,

dicts: `# Dictionaries
badlo student = {"naam": "Adarsh", "umar": 24, "shahar": "Indore"};
dikhao("Student: ");
dikhao(student);

# Access
dikhao("Naam: ");
dikhao(student["naam"]);

# Add with rakho
rakho(student, "marks", 95);
dikhao("After adding marks: ");
dikhao(student);

# Safe access with nikalo
dikhao("Shahar: ");
dikhao(nikalo(student, "shahar"));
dikhao("Phone (default): ");
dikhao(nikalo(student, "phone", "not available"));

# Keys & Values
dikhao("Keys: ");
dikhao(keys(student));
dikhao("Values: ");
dikhao(values(student));

# Contains check
dikhao("Has naam? ");
dikhao(contains(student, "naam"));
dikhao("Has email? ");
dikhao(contains(student, "email"));`,

structs: `# Structs (dhacha)
dhacha Car { brand, model, year, color };

badlo myCar = Car { brand: "Maruti", model: "Swift", year: 2023, color: "Red" };
dikhao("My car:");
dikhao(myCar.brand + " " + myCar.model);
dikhao("Year: ");
dikhao(myCar.year);
dikhao("Color: ");
dikhao(myCar.color);

# Modify attribute
myCar.color = "Blue";
dikhao("Repainted: ");
dikhao(myCar.color);

# Function that creates structs
dhacha Point { x, y };

kaam makePoint(x, y) {
    wapas Point { x: x, y: y };
}

kaam distance(p1, p2) {
    badlo dx = p1.x - p2.x;
    badlo dy = p1.y - p2.y;
    wapas (dx ** 2 + dy ** 2) ** 0.5;
}

badlo a = makePoint(0, 0);
badlo b = makePoint(3, 4);
dikhao("Distance: ");
dikhao(distance(a, b));`,

switch: `# Switch (chuno)
badlo day = "somvar";

chuno (day) {
    case "somvar":
        dikhao("Monday - Kaam shuru!");
    case "shukravar":
        dikhao("Friday - Party time!");
    case "ravivar":
        dikhao("Sunday - Aaraam!");
    warna_case:
        dikhao("Koi aur din hai");
}

# Multi-value case
badlo grade = "B";
chuno (grade) {
    case "A", "A+":
        dikhao("Excellent!");
    case "B", "B+":
        dikhao("Good job!");
    case "C":
        dikhao("Average");
    warna_case:
        dikhao("Need improvement");
}`,

trycatch: `# Try/Catch (pakdo/chhoddo)
pakdo {
    dikhao("Trying something risky...");
    chhoddo("Oops! Galti ho gayi!");
    dikhao("This won't print");
} chhoddo (err) {
    dikhao("Caught error: ");
    dikhao(err);
}

# Custom error handling
kaam divide(a, b) {
    agar (b == 0) {
        chhoddo("Cannot divide by zero!");
    }
    wapas a / b;
}

pakdo {
    dikhao(divide(10, 2));
    dikhao(divide(10, 0));
} chhoddo (err) {
    dikhao("Error pakda: ");
    dikhao(err);
}`,

math: `# Math Built-in Functions
dikhao("abs(-42) = ");
dikhao(abs(-42));

dikhao("floor(4.7) = ");
dikhao(floor(4.7));

dikhao("ceil(4.2) = ");
dikhao(ceil(4.2));

dikhao("round_val(4.6) = ");
dikhao(round_val(4.6));

dikhao("round_val(4.4) = ");
dikhao(round_val(4.4));

dikhao("min_val(10, 20) = ");
dikhao(min_val(10, 20));

dikhao("max_val(10, 20) = ");
dikhao(max_val(10, 20));

dikhao("random_number(1, 100) = ");
dikhao(random_number(1, 100));

# Power & Modulo
dikhao("2 ** 10 = ");
dikhao(2 ** 10);
dikhao("17 % 5 = ");
dikhao(17 % 5);`,

strings: `# String Built-in Functions
badlo text = "  Namaste Duniya  ";

dikhao("Original: '" + text + "'");
dikhao("Trimmed: '" + trim(text) + "'");
dikhao("Upper: " + upper(trim(text)));
dikhao("Lower: " + lower(trim(text)));

# Split & Join
badlo words = split("seb,kela,amrud,aam", ",");
dikhao("Split: ");
dikhao(words);
dikhao("Joined: " + join(words, " | "));

# Replace
dikhao("Replace: " + replace("Hello World", "World", "Duniya"));

# Find
dikhao("Find 'Duniya' in 'Namaste Duniya': ");
dikhao(find("Namaste Duniya", "Duniya"));

# Reverse string
dikhao("Reverse 'namaste': " + reverse("namaste"));

# Escape sequences
dikhao("Tab:\\there");
dikhao("Newline:\\nhere");
dikhao("Quote: \\"hello\\"");`,

functional: `# Functional Programming: map, filter, reduce

badlo numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

# Map - square every number
badlo squares = map(kaam (x) { wapas x ** 2; }, numbers);
dikhao("Squares: ");
dikhao(squares);

# Filter - keep only even numbers
badlo evens = filter(kaam (x) { wapas x % 2 == 0; }, numbers);
dikhao("Evens: ");
dikhao(evens);

# Reduce - sum all numbers
badlo total = reduce(kaam (acc, x) { wapas acc + x; }, numbers, 0);
dikhao("Sum: ");
dikhao(total);

# Chain them: sum of squares of even numbers
badlo result = reduce(
    kaam (acc, x) { wapas acc + x; },
    map(
        kaam (x) { wapas x ** 2; },
        filter(kaam (x) { wapas x % 2 == 0; }, numbers)
    ),
    0
);
dikhao("Sum of squares of evens: ");
dikhao(result);`,

compound: `# Compound Assignment Operators (+=, -=, *=, /=, %=)
badlo x = 10;
dikhao("Start: ");
dikhao(x);

x += 5;
dikhao("x += 5 => ");
dikhao(x);

x -= 3;
dikhao("x -= 3 => ");
dikhao(x);

x *= 2;
dikhao("x *= 2 => ");
dikhao(x);

x /= 6;
dikhao("x /= 6 => ");
dikhao(x);

x %= 3;
dikhao("x %= 3 => ");
dikhao(x);

# Works on list elements too
badlo scores = [10, 20, 30];
scores[1] += 100;
dikhao("scores after scores[1] += 100:");
dikhao(scores);

# Ginnati loop with +=
badlo sum = 0;
ginnati (badlo i = 1; i <= 100; i += 1) {
    sum += i;
}
dikhao("Sum 1 to 100: ");
dikhao(sum);`,

power_mod: `# Power (**) and Modulo (%) Operators
dikhao("=== Power ===");
dikhao("2 ** 8 = ");
dikhao(2 ** 8);
dikhao("5 ** 3 = ");
dikhao(5 ** 3);
dikhao("9 ** 0.5 = ");
dikhao(9 ** 0.5);

dikhao("\\n=== Modulo ===");
dikhao("10 % 3 = ");
dikhao(10 % 3);
dikhao("25 % 7 = ");
dikhao(25 % 7);

# Even/Odd checker
dikhao("\\nEven or Odd:");
ginnati (badlo i = 1; i <= 10; i += 1) {
    agar (i % 2 == 0) {
        dikhao(shabdme(i) + " is even");
    } warna {
        dikhao(shabdme(i) + " is odd");
    }
}`,

null: `# Null (khali)
badlo x = khali;
dikhao("x = ");
dikhao(x);
dikhao("prakar(khali) = ");
dikhao(prakar(x));

# Null comparison
agar (x == khali) {
    dikhao("x is khali (null)!");
}

# Assign a value
x = 42;
dikhao("Now x = ");
dikhao(x);
agar (x != khali) {
    dikhao("x is no longer khali!");
}

# Use in functions
kaam findUser(id) {
    agar (id == 1) {
        wapas "Adarsh";
    }
    wapas khali;
}

badlo user1 = findUser(1);
badlo user2 = findUser(99);
dikhao("User 1: ");
dikhao(user1);
dikhao("User 99: ");
dikhao(user2);`,

typecheck: `# Type Checking with prakar()
dikhao("prakar(42) = " + prakar(42));
dikhao("prakar(3.14) = " + prakar(3.14));
dikhao("prakar(\\"hello\\") = " + prakar("hello"));
dikhao("prakar(sahi_hai_be) = " + prakar(sahi_hai_be));
dikhao("prakar([1,2]) = " + prakar([1, 2]));
dikhao("prakar({}) = " + prakar({"a": 1}));
dikhao("prakar(khali) = " + prakar(khali));

# Type conversion
badlo numStr = "123";
dikhao("\\nBefore: " + prakar(numStr) + " => " + numStr);
badlo num = sankhya(numStr);
dikhao("After sankhya: " + prakar(num));
dikhao(num + 1);

badlo val = 456;
badlo str = shabdme(val);
dikhao("\\nshabdme(456): " + prakar(str) + " => " + str);
dikhao("shabdme(sahi_hai_be) => " + shabdme(sahi_hai_be));
dikhao("shabdme(khali) => " + shabdme(khali));`,

range: `# Range & Iteration
dikhao("range(5):");
dikhao(range(5));

dikhao("\\nrange(3, 8):");
dikhao(range(3, 8));

dikhao("\\nrange(0, 50, 10):");
dikhao(range(0, 50, 10));

# Sum using range
badlo total = 0;
ke_liye (badlo n in range(1, 11)) {
    total += n;
}
dikhao("\\nSum 1-10: ");
dikhao(total);

# Multiplication table
dikhao("\\n5 ka table:");
ke_liye (badlo i in range(1, 11)) {
    dikhao("5 x " + shabdme(i) + " = " + shabdme(5 * i));
}`,

collections: `# Collection Helper Functions

# sort
badlo nums = [42, 7, 15, 3, 99, 28];
dikhao("Original: ");
dikhao(nums);
dikhao("Sorted: ");
dikhao(sort(nums));

# reverse
dikhao("Reversed: ");
dikhao(reverse(nums));
dikhao("Reverse string: " + reverse("AdarshLang"));

# slice
dikhao("Slice [1:4]: ");
dikhao(slice(nums, 1, 4));
dikhao("Slice string: " + slice("AdarshLang", 0, 6));

# contains
dikhao("Contains 42? ");
dikhao(contains(nums, 42));
dikhao("Contains 100? ");
dikhao(contains(nums, 100));

# find
dikhao("Find 99: index ");
dikhao(find(nums, 99));
dikhao("Find 'Lang' in 'AdarshLang': index ");
dikhao(find("AdarshLang", "Lang"));

# keys, values
badlo info = {"naam": "Adarsh", "lang": "AdarshLang", "version": 2};
dikhao("Keys: ");
dikhao(keys(info));
dikhao("Values: ");
dikhao(values(info));

# min_val, max_val
dikhao("min(3, 7) = ");
dikhao(min_val(3, 7));
dikhao("max(3, 7) = ");
dikhao(max_val(3, 7));`,

fizzbuzz: `# FizzBuzz - Classic programming challenge!
ginnati (badlo i = 1; i <= 30; i += 1) {
    agar (i % 15 == 0) {
        dikhao("FizzBuzz");
    } warna agar (i % 3 == 0) {
        dikhao("Fizz");
    } warna agar (i % 5 == 0) {
        dikhao("Buzz");
    } warna {
        dikhao(i);
    }
}`,

all_features: `# === AdarshLang - ALL Features Demo ===

# 1. Variables & Types
badlo naam = "Adarsh";
badlo umar = 24;
badlo active = sahi_hai_be;
badlo nothing = khali;
dikhao("--- Variables ---");
dikhao(naam);
dikhao(umar);
dikhao(active);
dikhao(nothing);

# 2. Arithmetic with %, **, +=
badlo x = 10;
x += 5;
dikhao("\\n--- Operators ---");
dikhao("10 + 5 = " + shabdme(x));
dikhao("17 % 5 = " + shabdme(17 % 5));
dikhao("2 ** 10 = " + shabdme(2 ** 10));

# 3. If/Else
dikhao("\\n--- If/Else ---");
agar (umar >= 18) {
    dikhao(naam + " is an adult");
} warna {
    dikhao(naam + " is a minor");
}

# 4. Loops
dikhao("\\n--- Loops ---");
ginnati (badlo i = 1; i <= 5; i += 1) {
    dikhao("Square of " + shabdme(i) + " = " + shabdme(i ** 2));
}

# 5. Functions
dikhao("\\n--- Functions ---");
kaam factorial(n) {
    agar (n <= 1) { wapas 1; }
    wapas n * factorial(n - 1);
}
dikhao("5! = " + shabdme(factorial(5)));

# 6. Lists & Helpers
dikhao("\\n--- Lists ---");
badlo nums = [5, 2, 8, 1, 9];
dikhao("Original: ");
dikhao(nums);
dikhao("Sorted: ");
dikhao(sort(nums));
dikhao("Reversed: ");
dikhao(reverse(nums));

# 7. Dicts
dikhao("\\n--- Dictionary ---");
badlo person = {"naam": "Adarsh", "lang": "AdarshLang"};
dikhao("Keys: ");
dikhao(keys(person));
dikhao("Contains naam? ");
dikhao(contains(person, "naam"));

# 8. Struct (dhacha)
dikhao("\\n--- Struct ---");
dhacha Point { x, y };
badlo p = Point { x: 3, y: 4 };
dikhao("Point: (" + shabdme(p.x) + ", " + shabdme(p.y) + ")");

# 9. Functional
dikhao("\\n--- Map/Filter/Reduce ---");
badlo data = range(1, 11);
badlo evens = filter(kaam (x) { wapas x % 2 == 0; }, data);
badlo squares = map(kaam (x) { wapas x ** 2; }, evens);
badlo total = reduce(kaam (a, b) { wapas a + b; }, squares, 0);
dikhao("Sum of squares of evens 1-10: ");
dikhao(total);

# 10. Type checking
dikhao("\\n--- Type Checking ---");
dikhao("prakar(42) = " + prakar(42));
dikhao("prakar(\\"hi\\") = " + prakar("hi"));
dikhao("prakar(khali) = " + prakar(khali));

# 11. String helpers
dikhao("\\n--- Strings ---");
dikhao(upper("namaste"));
dikhao(replace("Hello World", "World", "Duniya"));
dikhao(trim("  spaces  "));

# 12. Try/Catch
dikhao("\\n--- Try/Catch ---");
pakdo {
    chhoddo("Test error!");
} chhoddo (err) {
    dikhao("Caught: " + err);
}

# 13. Switch
dikhao("\\n--- Switch ---");
badlo mood = "khush";
chuno (mood) {
    case "udaas":
        dikhao("Cheer up!");
    case "khush":
        dikhao("Great mood!");
    warna_case:
        dikhao("Unknown mood");
}

# 14. Range
dikhao("\\n--- Range ---");
dikhao(range(0, 20, 5));

# 15. Math
dikhao("\\n--- Math ---");
dikhao("floor(4.7) = " + shabdme(floor(4.7)));
dikhao("ceil(4.2) = " + shabdme(ceil(4.2)));
dikhao("abs(-99) = " + shabdme(abs(-99)));
dikhao("min(3,7) = " + shabdme(min_val(3, 7)));
dikhao("max(3,7) = " + shabdme(max_val(3, 7)));

dikhao("\\n=== Sab features kaam kar rahe hain! ===");`

};

var DEFAULT_CODE = snippets.hello;

function loadSnippet(name) {
    document.getElementById('source_code').value = snippets[name];
    clearOutput();
    var ta = document.getElementById('source_code');
    ta.focus();
    document.getElementById('playground').scrollIntoView({behavior: 'smooth', block: 'start'});
}

async function runCode(event) {
    if (event) event.preventDefault();
    var code = document.getElementById('source_code').value;
    var btn = document.getElementById('run-btn');
    var card = document.getElementById('output-card');
    var pre = document.getElementById('output-pre');
    var label = document.getElementById('output-label');
    var dot = document.getElementById('status-dot');

    btn.innerHTML = '<span class="spin">\\u25CC</span>&nbsp;Running';
    btn.disabled = true;
    pre.className = '';
    pre.textContent = '';
    card.style.display = 'block';
    label.textContent = 'Output';
    dot.className = 'status-dot';

    try {
        var resp = await fetch('/run_json', {
            method: 'POST',
            headers: {'Content-Type': 'application/json'},
            body: JSON.stringify({source_code: code})
        });
        var data = await resp.json();
        if (data.error) {
            pre.className = 'error-output';
            label.textContent = 'Error';
            dot.className = 'status-dot err';
            pre.textContent = data.output;
        } else {
            label.textContent = 'Output';
            dot.className = 'status-dot';
            pre.textContent = data.output || '(no output)';
        }
    } catch (err) {
        pre.className = 'error-output';
        label.textContent = 'Error';
        dot.className = 'status-dot err';
        pre.textContent = 'Network error: ' + err.message;
    } finally {
        btn.innerHTML = 'Run';
        btn.disabled = false;
        pre.scrollIntoView({behavior: 'smooth', block: 'nearest'});
    }
}

function clearOutput() {
    var card = document.getElementById('output-card');
    card.style.display = 'none';
    document.getElementById('output-pre').textContent = '';
}

function copyCode() {
    var code = document.getElementById('source_code').value;
    navigator.clipboard.writeText(code).then(function () { flashButton(event.target, 'Copied!'); });
}

function resetEditor() {
    document.getElementById('source_code').value = DEFAULT_CODE;
    clearOutput();
}

function flashButton(btn, text) {
    if (!btn) return;
    var orig = btn.innerHTML;
    btn.innerHTML = text;
    setTimeout(function () { btn.innerHTML = orig; }, 1200);
}

function toggleTheme() {
    var root = document.documentElement;
    var next = root.getAttribute('data-theme') === 'dark' ? 'light' : 'dark';
    root.setAttribute('data-theme', next);
    localStorage.setItem('aak-theme', next);
    document.getElementById('theme-toggle').textContent = next === 'dark' ? '\\u2600\\uFE0F' : '\\uD83C\\uDF19';
}

(function initTheme() {
    var saved = localStorage.getItem('aak-theme');
    if (!saved) {
        saved = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }
    document.documentElement.setAttribute('data-theme', saved);
    document.addEventListener('DOMContentLoaded', function () {
        document.getElementById('theme-toggle').textContent = saved === 'dark' ? '\\u2600\\uFE0F' : '\\uD83C\\uDF19';
        var ta = document.getElementById('source_code');
        if (!ta.value) ta.value = DEFAULT_CODE;
    });
})();

// Cmd/Ctrl + Enter to run
document.addEventListener('keydown', function (e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') {
        e.preventDefault();
        runCode();
    }
});
</script>

</body>
</html>
"""

# --------------------------------------------------------------------
# RESULT PAGE TEMPLATE
# --------------------------------------------------------------------
RESULT_PAGE_TEMPLATE = """
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>AdarshLang — Output</title>
  <style>
    * { box-sizing: border-box; }
    body {
      margin: 0; min-height: 100vh;
      font-family: -apple-system, BlinkMacSystemFont, "SF Pro Text", "Helvetica Neue", Arial, sans-serif;
      color: #1d1d1f; background: #f5f5f7;
      background-image: radial-gradient(1200px 600px at 15% -10%, #e7f0ff, transparent 60%),
                        radial-gradient(1000px 500px at 100% 0%, #fbe9ff, transparent 55%);
      display: flex; align-items: center; justify-content: center; padding: 40px 20px;
    }
    .card {
      background: rgba(255,255,255,0.72); backdrop-filter: saturate(180%) blur(20px);
      border: 1px solid rgba(0,0,0,0.08); border-radius: 22px;
      box-shadow: 0 12px 40px rgba(0,0,0,0.10); padding: 32px; max-width: 760px; width: 100%;
    }
    h1 { font-size: 28px; letter-spacing: -0.02em; margin: 0 0 18px; }
    pre {
      background: #1d1d1f; color: #e8e8ed; border-radius: 16px; padding: 18px 20px; margin: 0;
      white-space: pre-wrap; word-wrap: break-word;
      font-family: "JetBrains Mono", ui-monospace, Menlo, monospace; font-size: 13.5px; line-height: 1.6;
    }
    a.btn {
      display: inline-block; margin-top: 20px; text-decoration: none; color: #fff; background: #0071e3;
      padding: 12px 26px; border-radius: 980px; font-weight: 500; box-shadow: 0 6px 20px rgba(0,113,227,0.35);
    }
  </style>
</head>
<body>
  <div class="card">
    <h1>Execution Output</h1>
    <pre>{{ output }}</pre>
    <a class="btn" href="/">&larr; Back to Playground</a>
  </div>
</body>
</html>
"""

@app.route("/", methods=["GET"])
def home():
    return render_template_string(HOME_PAGE_TEMPLATE)

@app.route("/run", methods=["POST"])
def run_code():
    source_code = request.form.get("source_code", "")

    old_stdout = sys.stdout
    mystdout = StringIO()
    sys.stdout = mystdout

    try:
        adarshlang_compile_and_run(source_code)
    except ADARSH_ERRORS as e:
        output = f"Error: {e}"
    except Exception as e:  # noqa: BLE001 - playground should never 500
        output = f"Error: {e}"
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return render_template_string(RESULT_PAGE_TEMPLATE, output=output)

@app.route("/run_json", methods=["POST"])
def run_code_json():
    data = request.get_json(force=True)
    source_code = data.get("source_code", "")

    old_stdout = sys.stdout
    mystdout = StringIO()
    sys.stdout = mystdout

    error = False
    try:
        adarshlang_compile_and_run(source_code)
    except ADARSH_ERRORS as e:
        error = True
        output = str(e)
    except Exception as e:  # noqa: BLE001 - playground should never 500
        error = True
        output = str(e)
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return jsonify({"output": output, "error": error})


if __name__ == "__main__":
    app.run(debug=True)
