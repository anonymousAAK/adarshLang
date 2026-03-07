from flask import Flask, request, render_template_string, jsonify
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
    .snippet-btn {
      margin: 4px;
      font-size: 0.85rem;
    }
    .snippet-section {
      margin-top: 1rem;
    }
    .error-output {
      background: #fdecea !important;
      color: #c0392b;
    }
  </style>
</head>
<body>

<div class="container">
  <h1 class="header-text text-center">AdarshLang Online Runner</h1>

  <!-- Code Editor Card -->
  <div class="card shadow-sm">
    <div class="card-body">
      <form id="codeForm" onsubmit="runCode(event)">
        <div class="form-group">
          <label for="source_code"><strong>Enter your AdarshLang code:</strong></label>
          <textarea id="source_code" name="source_code" rows="14"
                    placeholder="badlo x = 10;&#10;dikhao(x);"></textarea>
        </div>
        <button type="submit" id="run-btn" class="btn btn-primary btn-block">&#9654;&nbsp; Run Code</button>
      </form>

      <!-- Inline Output -->
      <div id="output-card" class="mt-3" style="display:none;">
        <div class="d-flex justify-content-between align-items-center mb-1">
          <strong id="output-label">Output</strong>
          <button class="btn btn-sm btn-outline-secondary" onclick="clearOutput()">&#10005; Clear</button>
        </div>
        <pre id="output-pre" style="min-height:60px;max-height:420px;overflow-y:auto;"></pre>
      </div>

      <!-- One-click snippet buttons -->
      <div class="snippet-section">
        <label><strong>Try a snippet (click to load, then Run):</strong></label>
        <div>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('hello')">Hello World</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('variables')">Variables &amp; Types</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('ifelse')">If / Else</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('loops')">Loops</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('functions')">Functions</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('lists')">Lists</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('dicts')">Dictionaries</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('structs')">Structs (dhacha)</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('switch')">Switch (chuno)</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('trycatch')">Try / Catch</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('math')">Math Builtins</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('strings')">String Builtins</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('functional')">Map / Filter / Reduce</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('compound')">Compound Assignment</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('power_mod')">Power &amp; Modulo</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('null')">Null (khali)</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('typecheck')">Type Checking</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('range')">Range &amp; Iteration</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('collections')">Collection Helpers</button>
          <button class="btn btn-outline-secondary snippet-btn" onclick="loadSnippet('fizzbuzz')">FizzBuzz</button>
          <button class="btn btn-outline-info snippet-btn" onclick="loadSnippet('all_features')">ALL Features Demo</button>
        </div>
      </div>
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
    </div>
  </div>

  <div class="footer">
    <p>AdarshLang</p>
  </div>
</div>

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

function loadSnippet(name) {
    document.getElementById('source_code').value = snippets[name];
    clearOutput();
    document.getElementById('source_code').focus();
}

async function runCode(event) {
    event.preventDefault();
    var code = document.getElementById('source_code').value;
    var btn = document.getElementById('run-btn');
    var card = document.getElementById('output-card');
    var pre = document.getElementById('output-pre');
    var label = document.getElementById('output-label');

    btn.textContent = 'Running...';
    btn.disabled = true;
    pre.className = '';
    pre.textContent = '';
    card.style.display = 'block';
    label.textContent = 'Output';

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
            pre.textContent = data.output;
        } else {
            pre.textContent = data.output || '(no output)';
        }
    } catch (err) {
        pre.className = 'error-output';
        label.textContent = 'Error';
        pre.textContent = 'Network error: ' + err.message;
    } finally {
        btn.textContent = '\u25b6\u00a0 Run Code';
        btn.disabled = false;
        pre.scrollIntoView({behavior: 'smooth', block: 'nearest'});
    }
}

function clearOutput() {
    var card = document.getElementById('output-card');
    card.style.display = 'none';
    document.getElementById('output-pre').textContent = '';
}
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
    except (AdarshRuntimeError, AdarshUserException, AdarshSemanticError) as e:
        error = True
        output = str(e)
    else:
        output = mystdout.getvalue()
    finally:
        sys.stdout = old_stdout

    return jsonify({"output": output, "error": error})


if __name__ == "__main__":
    app.run(debug=True)
