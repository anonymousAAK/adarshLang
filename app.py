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
      background: #f4f6f8;
    }
    .hero {
      background: linear-gradient(120deg, #fdfbfb 0%, #ebedee 100%);
      border-radius: 1rem;
      padding: 2.5rem;
      margin-bottom: 2rem;
      box-shadow: 0 20px 25px -15px rgba(0, 0, 0, 0.25);
    }
    textarea {
      width: 100%;
      font-family: "Fira Code", monospace;
      min-height: 220px;
      resize: vertical;
    }
    pre {
      background: #212529;
      color: #f8f9fa;
      padding: 1rem 1.2rem;
      border-radius: 0.75rem;
      white-space: pre-wrap;
      word-wrap: break-word;
      font-size: 0.95rem;
    }
    .cheatsheet-table th,
    .cheatsheet-table td {
      vertical-align: middle;
    }
    .section-title {
      margin-top: 2.5rem;
      margin-bottom: 1rem;
    }
    .footer {
      text-align: center;
      margin: 3rem 0 1rem;
      color: #6c757d;
      font-size: 0.9rem;
    }
  </style>
</head>
<body>

<div class="container py-4">
  <nav class="navbar navbar-expand-lg navbar-light bg-white shadow-sm rounded mb-4">
    <a class="navbar-brand font-weight-bold" href="#">AdarshLang</a>
    <div class="ml-auto">
      <a class="btn btn-outline-primary btn-sm" href="https://github.com/adarshk-raj/adarshLang" target="_blank">View on GitHub</a>
    </div>
  </nav>

  <div class="hero">
    <div class="row align-items-center">
      <div class="col-lg-7">
        <h1 class="display-5 mb-3">AdarshLang Online Runner</h1>
        <p class="lead mb-3">Experiment with the Hinglish-flavoured programming language right in your browser. Type some code, hit <strong>Run</strong>, and see the output instantly.</p>
        <ul class="list-unstyled mb-4">
          <li>✅ High-level, expressive syntax with Hinglish keywords</li>
          <li>✅ Batteries-included standard library with functional helpers</li>
          <li>✅ Great for learning, tinkering, and having fun with code</li>
        </ul>
        <a class="btn btn-primary btn-lg" href="#quick-start">Start coding</a>
        <a class="btn btn-link btn-lg" href="#language-tour">Read the language tour</a>
      </div>
      <div class="col-lg-5 mt-4 mt-lg-0">
        <div class="bg-dark text-white rounded-lg p-4 shadow-sm">
<pre class="mb-0">badlo naam = "Adarsh";
dikhao("Namaste " + naam + "!");</pre>
        </div>
      </div>
    </div>
  </div>

  <div class="card shadow-sm" id="quick-start">
    <div class="card-body">
      <h2 class="h4 mb-3">Try it now</h2>
      <form action="/run" method="post">
        <div class="form-group">
          <label for="source_code"><strong>Enter your AdarshLang code:</strong></label>
          <textarea id="source_code" name="source_code" rows="12"
                    placeholder="badlo x = 10;
agar (x > 5) {
    dikhao("Badi value!");
} warna {
    dikhao("Chhoti value!");
}"></textarea>
        </div>
        <button type="submit" class="btn btn-primary btn-block">Run Code</button>
      </form>
    </div>
  </div>

  <section id="language-tour">
    <h2 class="section-title">Language tour</h2>
    <p class="text-muted">AdarshLang mixes expressive Hinglish keywords with familiar programming concepts. Use this quick guide as you explore.</p>

    <div class="row">
      <div class="col-lg-6 mb-4">
        <div class="card h-100 shadow-sm">
          <div class="card-body">
            <h3 class="h5">Essentials cheat sheet</h3>
            <table class="table table-sm cheatsheet-table">
              <thead class="thead-light">
                <tr>
                  <th>Concept</th>
                  <th>How it looks</th>
                </tr>
              </thead>
              <tbody>
                <tr>
                  <td>Variables</td>
                  <td><code>badlo score = 42;</code></td>
                </tr>
                <tr>
                  <td>Printing</td>
                  <td><code>dikhao(score);</code></td>
                </tr>
                <tr>
                  <td>Booleans</td>
                  <td><code>sahi_hai_be</code>, <code>jhuth</code></td>
                </tr>
                <tr>
                  <td>Conditions</td>
                  <td><code>agar (score > 40) { ... } warna { ... }</code></td>
                </tr>
                <tr>
                  <td>Loops</td>
                  <td><code>jabtak</code>, <code>ginnati</code>, <code>ke_liye</code></td>
                </tr>
                <tr>
                  <td>Functions</td>
                  <td><code>kaam add(a, b = 0) { wapas a + b; }</code></td>
                </tr>
                <tr>
                  <td>Collections</td>
                  <td><code>badlo nums = [1, 2, 3];</code></td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
      </div>
      <div class="col-lg-6 mb-4">
        <div class="card h-100 shadow-sm">
          <div class="card-body">
            <h3 class="h5">Quick start checklist</h3>
            <ol class="pl-3">
              <li>Declare values with <code>badlo</code> and modify them freely.</li>
              <li>Use <code>agar</code>/<code>warna</code> for branching logic and <code>warna agar</code> for chains.</li>
              <li>Pick a loop: <code>jabtak</code> (while), <code>ginnati</code> (classic for), or <code>ke_liye</code> (foreach).</li>
              <li>Create reusable functions via <code>kaam</code> and return with <code>wapas</code>.</li>
              <li>Wrap risky code inside <code>pakdo { ... } chhoddo (err) { ... }</code> blocks.</li>
              <li>Break programs into files and load them using <code>lao "utils.aak";</code>.</li>
            </ol>
          </div>
        </div>
      </div>
    </div>

    <div class="card shadow-sm mb-4">
      <div class="card-body">
        <h3 class="h5">Example: mini todo tracker</h3>
<pre>
dhacha Todo { title, done }

kaam naya_todo(what) {
    wapas Todo { title: what, done: jhuth };
}

badlo todos = [];
push(todos, naya_todo("Write AdarshLang docs"));
push(todos, naya_todo("Drink chai"));

ke_liye (badlo item in todos) {
    agar (item.done) {
        dikhao("✔️  " + item.title);
    } warna {
        dikhao("⬜  " + item.title);
    }
}
        </pre>
        <p class="mb-0">Paste the snippet above into the editor to get a feel for structs, loops, and string concatenation.</p>
      </div>
    </div>

    <div class="card shadow-sm">
      <div class="card-body">
        <h3 class="h5">Standard library highlights</h3>
        <div class="row">
          <div class="col-md-6">
            <ul class="mb-0">
              <li><code>map</code>, <code>filter</code>, and <code>reduce</code> for data pipelines</li>
              <li><code>push</code>, <code>pop</code>, <code>length</code>, <code>join</code>, and <code>split</code> for collections</li>
              <li><code>random_number(min, max)</code> and <code>current_time()</code></li>
            </ul>
          </div>
          <div class="col-md-6">
            <ul class="mb-0">
              <li><code>rakho</code> / <code>nikalo</code> helpers for <code>dhacha</code> records</li>
              <li>Exception helpers: <code>chhoddo(expr);</code> to throw custom errors</li>
              <li>Interop: <code>lao "math.aak";</code> to reuse other AdarshLang files</li>
            </ul>
          </div>
        </div>
      </div>
    </div>
  </section>

  <section class="section-title" id="further-reading">
    <h2 class="h4">Further reading</h2>
    <p class="mb-2">Ready to dive deeper? Check out the resources below:</p>
    <ul>
      <li><a href="https://github.com/adarshk-raj/adarshLang#readme" target="_blank">Project README</a> — full language reference, CLI usage, and roadmap.</li>
      <li><code>extras.aak</code> &amp; <code>test_*.aak</code> — curated samples showcasing features and edge cases.</li>
    </ul>
  </section>

  <div class="footer">
    Built with ❤ for the AdarshLang community. Try something silly, then share it with friends!
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
    
    # Capture stdout
    old_stdout = sys.stdout
    mystdout = StringIO()
    sys.stdout = mystdout

    try:
        # Compile and run the AdarshLang code
        adarshlang_compile_and_run(source_code)

        # DEMO fallback block:
        print("Your code was:\n")
        print(source_code)

    except (AdarshRuntimeError, AdarshUserException, AdarshSemanticError) as e:
        output = f"Error: {e}"
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return render_template_string(RESULT_PAGE_TEMPLATE, output=output)

if __name__ == "__main__":
    app.run(debug=True)
