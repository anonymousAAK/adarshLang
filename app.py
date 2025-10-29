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
  <!-- Bootstrap via CDN -->
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
    .nav-tabs {
        margin-bottom: 1rem;
    }
  </style>
</head>
<body>

<div class="container">
  <h1 class="header-text text-center">AdarshLang Online Runner</h1>

  <ul class="nav nav-tabs" id="myTab" role="tablist">
    <li class="nav-item">
      <a class="nav-link active" id="editor-tab" data-toggle="tab" href="#editor" role="tab" aria-controls="editor" aria-selected="true">Code Editor</a>
    </li>
    <li class="nav-item">
      <a class="nav-link" id="docs-tab" data-toggle="tab" href="#docs" role="tab" aria-controls="docs" aria-selected="false">Documentation</a>
    </li>
    <li class="nav-item">
      <a class="nav-link" id="examples-tab" data-toggle="tab" href="#examples" role="tab" aria-controls="examples" aria-selected="false">Examples</a>
    </li>
  </ul>

  <div class="tab-content" id="myTabContent">
    <div class="tab-pane fade show active" id="editor" role="tabpanel" aria-labelledby="editor-tab">
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
    </div>
    <div class="tab-pane fade" id="docs" role="tabpanel" aria-labelledby="docs-tab">
        <div class="card shadow-sm">
            <div class="card-body">
                <h2>AdarshLang Documentation</h2>
                <p>AdarshLang is a Hinglish-inspired programming language built for fun. Below is a summary of its features:</p>
                <table class="table table-bordered">
                  <thead>
                    <tr>
                      <th>Feature</th>
                      <th>Syntax</th>
                      <th>Notes</th>
                    </tr>
                  </thead>
                  <tbody>
                    <tr>
                      <td>Variables</td>
                      <td><code>badlo x = 42;</code></td>
                      <td>Implicit declarations with <code>badlo</code> and later assignments.</td>
                    </tr>
                    <tr>
                      <td>Arithmetic & logic</td>
                      <td><code>+ - * /</code>, comparisons, <code>aur</code>/<code>ya</code>/<code>nahin</code></td>
                      <td>Uses standard Python semantics.</td>
                    </tr>
                    <tr>
                      <td>Booleans</td>
                      <td><code>sahi_hai_be</code>, <code>jhuth</code></td>
                      <td>Aliases of <code>True</code>/<code>False</code>.</td>
                    </tr>
                    <tr>
                      <td>Printing</td>
                      <td><code>dikhao(expr);</code></td>
                      <td>Sends values to stdout.</td>
                    </tr>
                    <tr>
                      <td>Conditionals</td>
                      <td><code>agar (...) { ... } warna { ... }</code></td>
                      <td>Supports chained <code>warna agar</code>.</td>
                    </tr>
                    <tr>
                      <td>While loops</td>
                      <td><code>jabtak (condition) { ... }</code></td>
                      <td>Break with <code>bas;</code>, continue with <code>aage_badho;</code>.</td>
                    </tr>
                    <tr>
                      <td>Counted loops</td>
                      <td><code>ginnati (init; condition; update) { ... }</code></td>
                      <td>Traditional C-style loop.</td>
                    </tr>
                    <tr>
                      <td>Foreach loops</td>
                      <td><code>ke_liye (badlo value in items) { ... }</code></td>
                      <td>Accepts either <code>in</code> or legacy colon separator.</td>
                    </tr>
                    <tr>
                      <td>Switch</td>
                      <td><code>chuno (expr) { case ... warna_case ... }</code></td>
                      <td>Multiple matches per case allowed.</td>
                    </tr>
                    <tr>
                      <td>Functions</td>
                      <td><code>kaam naam(params) { ... }</code></td>
                      <td>Default args, <code>baaki</code> varargs, anonymous <code>kaam (...)</code> expressions.</td>
                    </tr>
                    <tr>
                      <td>Returns</td>
                      <td><code>wapas expr;</code></td>
                      <td><code>wapas;</code> is valid for early exit without a value.</td>
                    </tr>
                    <tr>
                      <td>Lists</td>
                      <td><code>[1, 2, 3]</code></td>
                      <td>Supports indexing, assignment, <code>push</code>, <code>pop</code>, <code>length</code>.</td>
                    </tr>
                    <tr>
                      <td>Dictionaries</td>
                      <td><code>{ "key": value }</code></td>
                      <td>Works with <code>rakho</code>, <code>nikalo</code>, and attribute access (<code>obj.field</code>).</td>
                    </tr>
                    <tr>
                      <td>Dhacha records</td>
                      <td><code>dhacha Vyakti { naam, umar };</code></td>
                      <td>Construct with <code>Vyakti { naam: "Adarsh", umar: 24 }</code>.</td>
                    </tr>
                    <tr>
                      <td>Exceptions</td>
                      <td><code>pakdo { ... } chhoddo (err) { ... }</code></td>
                      <td>Throw via <code>chhoddo(expr);</code>.</td>
                    </tr>
                    <tr>
                      <td>Modules</td>
                      <td><code>lao "extras.aak";</code></td>
                      <td>Loads and executes other source files once.</td>
                    </tr>
                    <tr>
                      <td>Builtins</td>
                      <td><code>abs</code>, <code>floor</code>, <code>ceil</code>, <code>upper</code>, <code>lower</code>, <code>join</code>, _
                      <code>split</code>, <code>map</code>, <code>filter</code>, <code>reduce</code>, _
                      <code>random_number</code>, <code>current_time</code></td>
                      <td>Register automatically in every runtime.</td>
                    </tr>
                  </tbody>
                </table>
            </div>
        </div>
    </div>
    <div class="tab-pane fade" id="examples" role="tabpanel" aria-labelledby="examples-tab">
        <div class="card shadow-sm">
            <div class="card-body">
                <h2>AdarshLang Examples</h2>
                <p>Coming soon...</p>
            </div>
        </div>
    </div>
  </div>

  <div class="footer">
    <p>AdarshLang</p>
  </div>
</div>

<script src="https://code.jquery.com/jquery-3.5.1.slim.min.js"></script>
<script src="https://cdn.jsdelivr.net/npm/@popperjs/core@2.5.4/dist/umd/popper.min.js"></script>
<script src="https://stackpath.bootstrapcdn.com/bootstrap/4.5.2/js/bootstrap.min.js"></script>
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

    except (AdarshRuntimeError, AdarshUserException, AdarshSemanticError) as e:
        output = f"Error: {e}"
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return render_template_string(RESULT_PAGE_TEMPLATE, output=output)

if __name__ == "__main__":
    app.run(debug=True)
