from flask import Flask, request, render_template_string
import sys
from io import StringIO
from adarsh_lang_compiler import (
    adarshlang_compile_and_run,
    AdarshRuntimeError,
    AdarshUserException,
    AdarshSemanticError,
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
  <title>AdarshLang Online</title>
  <link rel="stylesheet"
    href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css">
  <style>
    body {
      background: #f8f9fa;
      margin: 20px;
    }
    .header-text {
      margin-bottom: 1rem;
    }
    textarea {
      width: 100%;
      font-family: monospace;
      min-height: 200px;
      resize: vertical;
    }
    .card {
      margin-top: 2rem;
    }
    pre {
      background: #eee;
      padding: 1rem;
      white-space: pre-wrap;
      word-wrap: break-word;
    }
    .footer {
      text-align: center;
      margin-top: 3rem;
      color: #999;
    }
  </style>
</head>
<body>

<div class="container">
  <h1 class="header-text text-center">AdarshLang Online Runner</h1>

  <!-- Code Editor Card -->
  <div class="card shadow-sm">
    <div class="card-body">
      <form action="/run" method="post">
        <div class="form-group">
          <label for="source_code"><strong>Enter your AdarshLang code:</strong></label>
          <textarea id="source_code" name="source_code" rows="10"
                    placeholder="badlo x = 10;&#10;dikhao(x);"></textarea>
        </div>
        <button type="submit" class="btn btn-primary btn-block">Run Code</button>
      </form>
    </div>
  </div>

  <!-- Tutorial Section -->
  <div class="card shadow-sm mt-4">
    <div class="card-body">
      <h2>AdarshLang Quick Tutorial</h2>
      <p>AdarshLang is a Hinglish-inspired programming language. Below is a summary of its features:</p>

      <h4>Basics</h4>
      <ul>
        <li><strong>Variables:</strong> <code>badlo x = 10;</code></li>
        <li><strong>Printing:</strong> <code>dikhao(x);</code></li>
        <li><strong>Booleans:</strong> <code>sahi_hai_be</code> (true), <code>jhuth</code> (false)</li>
        <li><strong>Null:</strong> <code>khali</code></li>
        <li><strong>Comments:</strong> <code># yeh comment hai</code></li>
      </ul>

      <h4>Operators</h4>
      <ul>
        <li><strong>Arithmetic:</strong> <code>+</code>, <code>-</code>, <code>*</code>, <code>/</code>, <code>%</code> (modulo), <code>**</code> (power)</li>
        <li><strong>Comparison:</strong> <code>==</code>, <code>!=</code>, <code>&lt;</code>, <code>&gt;</code>, <code>&lt;=</code>, <code>&gt;=</code></li>
        <li><strong>Logical:</strong> <code>&amp;&amp;</code> (aur), <code>||</code> (ya), <code>!</code> (nahin)</li>
        <li><strong>Compound Assignment:</strong> <code>+=</code>, <code>-=</code>, <code>*=</code>, <code>/=</code>, <code>%=</code></li>
      </ul>

      <h4>Control Flow</h4>
      <ul>
        <li><strong>If/Else:</strong> <code>agar (x &lt; 20) { ... } warna { ... }</code> with chained <code>warna agar</code></li>
        <li><strong>While:</strong> <code>jabtak (y &gt; 0) { ... }</code></li>
        <li><strong>For:</strong> <code>ginnati (badlo i = 0; i &lt; 10; i += 1) { ... }</code></li>
        <li><strong>For-each:</strong> <code>ke_liye (badlo item in list) { ... }</code></li>
        <li><strong>Switch:</strong> <code>chuno (expr) { case val: ... warna_case: ... }</code></li>
        <li><strong>Break/Continue:</strong> <code>bas;</code> / <code>aage_badho;</code></li>
      </ul>

      <h4>Functions</h4>
      <ul>
        <li><strong>Define:</strong> <code>kaam add(a, b) { wapas a + b; }</code></li>
        <li><strong>Default params:</strong> <code>kaam greet(naam, msg = "namaste") { ... }</code></li>
        <li><strong>Varargs:</strong> <code>kaam total(baaki nums) { ... }</code></li>
        <li><strong>Anonymous:</strong> <code>badlo fn = kaam (x) { wapas x * 2; };</code></li>
      </ul>

      <h4>Collections</h4>
      <ul>
        <li><strong>Lists:</strong> <code>[1, 2, 3]</code> with <code>push</code>, <code>pop</code>, <code>length</code>, <code>sort</code>, <code>reverse</code>, <code>slice</code></li>
        <li><strong>Dicts:</strong> <code>{"key": "value"}</code> with <code>rakho</code>, <code>nikalo</code>, <code>keys</code>, <code>values</code></li>
        <li><strong>Structs:</strong> <code>dhacha Vyakti { naam, umar };</code></li>
      </ul>

      <h4>Built-in Functions</h4>
      <ul>
        <li><strong>Math:</strong> <code>abs</code>, <code>floor</code>, <code>ceil</code>, <code>round_val</code>, <code>min_val</code>, <code>max_val</code>, <code>random_number</code></li>
        <li><strong>Strings:</strong> <code>upper</code>, <code>lower</code>, <code>join</code>, <code>split</code>, <code>trim</code>, <code>replace</code>, <code>find</code></li>
        <li><strong>Type:</strong> <code>prakar(x)</code> (type), <code>shabdme(42)</code> (toString), <code>sankhya("42")</code> (toNumber)</li>
        <li><strong>Iteration:</strong> <code>range(start, end, step)</code>, <code>contains(list, val)</code></li>
        <li><strong>Functional:</strong> <code>map</code>, <code>filter</code>, <code>reduce</code></li>
        <li><strong>Other:</strong> <code>current_time()</code></li>
      </ul>

      <h4>Error Handling &amp; Modules</h4>
      <ul>
        <li><strong>Try/Catch:</strong> <code>pakdo { ... } chhoddo (err) { ... }</code></li>
        <li><strong>Throw:</strong> <code>chhoddo("error message");</code></li>
        <li><strong>Import:</strong> <code>lao "module.aak";</code></li>
      </ul>

      <h4>Example: FizzBuzz</h4>
      <pre>
ginnati (badlo i = 1; i &lt;= 15; i += 1) {
    agar (i % 15 == 0) {
        dikhao("FizzBuzz");
    } warna agar (i % 3 == 0) {
        dikhao("Fizz");
    } warna agar (i % 5 == 0) {
        dikhao("Buzz");
    } warna {
        dikhao(i);
    }
}
      </pre>

      <h4>Example: Collections &amp; Functions</h4>
      <pre>
badlo nums = range(1, 6);
badlo doubled = map(kaam (x) { wapas x ** 2; }, nums);
dikhao(doubled);
dikhao(sort(reverse(nums)));

badlo person = {"naam": "Adarsh", "umar": 24};
dikhao(keys(person));
dikhao(contains(person, "naam"));
      </pre>
      <p>Try pasting a snippet above into the editor, then click "Run Code".</p>
    </div>
  </div>

  <div class="footer">
    <p>AdarshLang</p>
  </div>
</div>

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
  <title>AdarshLang Online - Results</title>
  <link rel="stylesheet"
    href="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/css/bootstrap.min.css">
  <style>
    body {
      background: #f8f9fa;
      margin: 20px;
    }
    .card {
      margin-top: 2rem;
    }
    pre {
      background: #eee;
      padding: 1rem;
      white-space: pre-wrap;
      word-wrap: break-word;
    }
    .footer {
      text-align: center;
      margin-top: 3rem;
      color: #999;
    }
  </style>
</head>
<body>
<div class="container">
  <h1 class="text-center">AdarshLang Execution Output</h1>
  <div class="card shadow-sm">
    <div class="card-body">
      <pre>{{ output }}</pre>
    </div>
  </div>
  <div class="text-center mt-3">
    <a href="/" class="btn btn-secondary">Back to Editor</a>
  </div>
  <div class="footer">
    <p>AdarshLang</p>
  </div>
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
    except (AdarshRuntimeError, AdarshUserException, AdarshSemanticError) as e:
        output = f"Error: {e}"
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return render_template_string(RESULT_PAGE_TEMPLATE, output=output)

if __name__ == "__main__":
    app.run(debug=True)
