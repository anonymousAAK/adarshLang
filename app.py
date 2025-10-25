from flask import Flask, request, render_template_string
import sys
from io import StringIO
from adarsh_lang import (
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
      <p>AdarshLang is a Hinglish-inspired language with a comedic twist. Below is a summary of its features:</p>
      <ul>
        <li><strong>Variables:</strong> <code>badlo x = 10;</code> aur expressions</li>
        <li><strong>Printing:</strong> <code>dikhao(x);</code></li>
        <li><strong>Booleans:</strong> <code>sahi_hai_be</code>, <code>jhuth</code></li>
        <li><strong>Conditionals:</strong> <code>agar ... warna</code> with chained <code>warna agar</code></li>
        <li><strong>Loops:</strong> <code>jabtak</code> (while), <code>ginnati</code> (for), <code>ke_liye (value in items)</code> (foreach)</li>
        <li><strong>Switching:</strong> <code>chuno (expr) { case ... warna_case ... }</code></li>
        <li><strong>Functions:</strong> defaults, <code>baaki</code> varargs, anonymous <code>kaam (...) { ... }</code></li>
        <li><strong>Collections:</strong> lists + helpers (<code>push</code>, <code>pop</code>, <code>length</code>) aur dictionaries/`dhacha` records with <code>rakho</code>/<code>nikalo</code></li>
        <li><strong>Exceptions:</strong> <code>pakdo { ... } chhoddo (err) { ... }</code> and <code>chhoddo(expr);</code> throws</li>
        <li><strong>Modules:</strong> <code>lao "mera_module.aak";</code> for reuse</li>
        <li><strong>Built-ins:</strong> math/string helpers (<code>abs</code>, <code>floor</code>, <code>join</code>, <code>split</code>), functional (<code>map</code>, <code>filter</code>, <code>reduce</code>), <code>random_number</code>, <code>current_time</code> aur zyada</li>
      </ul>
      <p>Here’s an example using all features:</p>
      <pre>
dhacha Vyakti { naam, umar }

kaam banakar(naam, umar = 18) {
    wapas Vyakti { naam: naam, umar: umar };
}

badlo hero = banakar("Adarsh", 24);
badlo nums = [1, 2, 3];

ginnati (badlo i = 0; i < length(nums); i = i + 1) {
    dikhao(nums[i]);
}

ke_liye (badlo value in nums) {
    dikhao(value);
}

pakdo {
    chhoddo("demo error");
} chhoddo (err) {
    dikhao(err);
}

badlo doubled = map(kaam (value) { wapas value * 2; }, nums);
dikhao(doubled);
      </pre>
      <p>Try pasting the snippet above into the editor, then click "Run Code".</p>
    </div>
  </div>

  <div class="footer">
    <p>AdarshLang </p>
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
