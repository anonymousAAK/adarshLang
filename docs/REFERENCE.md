# AdarshLang reference (v0.3)

Everything the language can do, with examples. Every example on this page runs as written.

- [Basics](#basics)
- [Operators](#operators)
- [Strings](#strings)
- [Control flow](#control-flow)
- [Functions](#functions)
- [Data](#data)
- [Records (dhacha)](#records-dhacha)
- [Enums (vikalp)](#enums-vikalp)
- [Errors](#errors)
- [Types](#types)
- [Modules](#modules)
- [Builtins](#builtins)
- [Turtle graphics (kachhua)](#turtle-graphics-kachhua)
- [Tests](#tests)
- [Command line](#command-line)

## Basics

```aak
badlo naam = "Adarsh";        # variable
badlo a = 1, b = 2;           # several at once
pakka PI_ROUGH = 3.14;        # constant: changing it is an error
dikhao("Namaste {naam}!");    # print
```

| Value | Examples |
| --- | --- |
| Numbers | `42`, `-7`, `3.14`, `.5`, `1e6`, `2.5e-3`, `1_000_000`. Integers never overflow: `2 ** 100` is exact |
| Strings | `"text"`, `"""multi-line"""`, raw `r"C:\path"` and `r"""{"json": 1}"""` |
| Booleans | `sahi_hai_be`, `jhuth` |
| Nothing | `khali` |
| Lists | `[1, 2, 3]` |
| Dicts | `{"naam": "A", "umar": 9}` |
| Tuples | `(28.6, 77.2)`, `(1,)`, `()` (cannot be changed) |
| Sets | `set([1, 2, 2])` (unique values) |

Comments: `# line`, `#[ block ]#`, and `## doc comment` just above a `kaam`.

Keywords also work in Devanagari: `बदलो`, `दिखाओ`, `अगर`, `वरना`, `जबतक`, `काम`, `वापस`, `सही`, `झूठ`, `खाली`, `और`, `या`, `नहीं` and more.

```aak
बदलो नाम = "दुनिया";
दिखाओ("नमस्ते, {नाम}!");
```

## Operators

| | |
| --- | --- |
| Arithmetic | `+ - * / % **`, and `//` floor division. `/` always gives a decimal; `%` follows the divisor's sign |
| Comparison | `== != < > <= >=`; chains work: `0 < x < 10` |
| Logic | `aur`, `ya`, `nahin` (or `&&`, `\|\|`, `!`), short-circuit |
| Membership | `"chai" in menu`, `"x" in "xyz"`, `key in dict` |
| Type check | `x hai number`, `p hai Vyakti` |
| Range | `1..5` is `[1, 2, 3, 4, 5]`; `5..1` counts down |
| Default | `naam ?? "anjaan"`: right side only when the left is `khali` |
| Safe field | `user?.naam`: `khali` instead of an error when `user` is `khali` |
| Compound | `+= -= *= /= %=` on variables, `list[i]` and `obj.field` |
| Inline if | `agar (umar >= 18) "adult" warna "bachcha"` |

## Strings

```aak
badlo daam = 49.5;
dikhao("Daam: {daam:.2}");          # Daam: 49.50
dikhao("{7:03} | {42:>6} | {0.256:.1%}");
dikhao("Literal brace: \{ \}");
badlo kavita = """
Pehli line
Doosri line
""";
dikhao(r"Raw: \n {yeh text hi hai}");
```

Format specs: `.2` (decimals), `>8` / `<8` / `^8` (align in a width), `03` (zero pad), `,` (thousands), `%` (percent).

Indexing and slicing work on strings, lists and tuples:

```aak
badlo s = "namaste";
dikhao(s[0]);      # n
dikhao(s[-1]);     # e
dikhao(s[0:4]);    # nama
dikhao(s[::-1]);   # etsaman
```

## Control flow

```aak
badlo marks = 82;
agar (marks >= 90) { dikhao("A"); } warna agar (marks >= 75) { dikhao("B"); } warna { dikhao("C"); }

badlo n = 3;
jabtak (n > 0) { n -= 1; }                      # while
karo { n += 1; } jabtak (n < 3);                # do-while: runs at least once
ginnati (badlo i = 0; i < 3; i += 1) { }        # for
ke_liye (badlo x in [1, 2, 3]) { }              # for-each (lists, strings, dicts, tuples, sets)
ke_liye (badlo i, phal in ["aam", "kela"]) { dikhao("{i}: {phal}"); }   # index + value
ke_liye (badlo k, v in {"a": 1}) { dikhao("{k}={v}"); }                # key + value
ke_liye (badlo i in 1..5) { }                   # count with a range

bahar: ginnati (badlo i = 0; i < 3; i += 1) {   # labelled loop
    ke_liye (badlo j in 0..2) {
        agar (j == 1) { aage_badho bahar; }     # continue the outer loop
        agar (i == 2) { bas bahar; }            # break out of both
    }
}
```

`chuno` matches values, ranges, types and list shapes, with optional `agar` guards:

```aak
kaam batao(v) {
    chuno (v) {
        case 0: wapas "zero";
        case 1, 2, 3: wapas "chhota";
        case 90..100: wapas "bahut bada";
        case x agar x < 0: wapas "negative";
        case []: wapas "khali list";
        case [pehla, ...rest]: wapas "list jo {pehla} se shuru";
        case string: wapas "string";
        warna_case: wapas "kuch aur";
    }
}
dikhao(batao(95));
```

## Functions

```aak
kaam jodo(a, b = 10) { wapas a + b; }                  # default argument
kaam total(baaki nums) { wapas sum(nums); }            # any number of arguments
badlo double = (x) => x * 2;                           # short lambda
badlo sq = x => x * x;
badlo f = kaam (x) { wapas x + 1; };                   # anonymous function

dikhao(jodo(5));                  # 15
dikhao(jodo(b: 1, a: 2));         # named arguments
badlo nums = [1, 2, 3];
dikhao(total(...nums));           # spread a list into arguments
dikhao([0, ...nums, 4]);          # spread inside a list
```

- Functions can be called before the line that defines them, can be nested, and capture variables (closures).
- A function without `wapas` returns `khali`.
- Any builtin or function can be called with a dot on its first argument: `naam.upper()`, `xs.sort()`, `xs.map(x => x * 2)`.

**Generators.** A function that uses `dete_raho` returns the list of values it produced:

```aak
kaam squares(n) {
    ke_liye (badlo i in 1..n) { dete_raho i * i; }
}
dikhao(squares(4));    # [1, 4, 9, 16]
```

## Data

```aak
badlo xs = [3, 1, 2];
xs[0] = 5;  push(xs, 4);  dikhao(xs[-1]);
badlo d = {"naam": "Adarsh", "umar": 24};
d["shahar"] = "Indore";  hatao(d, "umar");
dikhao(d.naam);                       # dict keys can be read as fields

badlo [a, b, ...baaki_items] = [1, 2, 3, 4];   # destructuring
badlo { lambai, chaudai } = {"lambai": 4, "chaudai": 5};

badlo s = set([1, 2, 2]);
push(s, 3);
dikhao(2 in s);
dikhao(union(s, [9]));
```

## Records (dhacha)

```aak
dhacha Vyakti { naam, umar = 0 };          # fields, with optional defaults
dhacha Student : Vyakti { roll };          # inheritance: Student has naam, umar, roll

kaam Vyakti.parichay() {                   # method; `yeh` is the record
    wapas "Main {yeh.naam}, umar {yeh.umar}";
}
kaam Student.shabd() {                     # custom text for dikhao / interpolation
    wapas "<Student {yeh.naam} #{yeh.roll}>";
}

badlo s = Student { naam: "Riya", roll: 7 };
dikhao(s);               # <Student Riya #7>
dikhao(s.parichay());    # inherited method
dikhao(s hai Vyakti);    # sahi_hai_be
```

**Operator methods.** Define `jodo` (+), `ghatao` (-), `guna` (*), `bhaag` (/), `barabar` (== and !=) or `chhota` (< > <= >=):

```aak
dhacha Paisa { rupaye };
kaam Paisa.jodo(other) { wapas Paisa { rupaye: yeh.rupaye + other.rupaye }; }
dikhao(Paisa { rupaye: 120 } + Paisa { rupaye: 80 });
```

## Enums (vikalp)

```aak
vikalp Rang { Laal, Hara, Neela };
badlo r = Rang.Hara;
dikhao(r);          # Rang.Hara
dikhao(r.naam);     # Hara
dikhao(r.kram);     # 1 (position)
chuno (r) { case Rang.Hara: dikhao("chalo"); warna_case: dikhao("ruko"); }
```

## Errors

```aak
kaam bhaag(a, b) {
    pakdo {
        wapas a / b;
    } chhoddo (e) {
        dikhao("Pakda: {e}");
        wapas khali;
    } aakhir {
        dikhao("yeh hamesha chalta hai");
    }
}
dikhao(bhaag(1, 0));
```

- `chhoddo(value);` throws any value. `pakdo` also catches runtime errors (as their message).
- Several `chhoddo` clauses can catch by type: `chhoddo (e: BalanceKam) { }`, and `chhoddo (e: Galti) { }` for runtime errors.
- `jaancho(sharat, "sandesh")` fails when the condition is false (for tests).
- `band_karo(code)` stops the program.

Uncaught errors show the label, a code, the line and column, the call chain, and suggestions for typos:

```text
Galti [E011]: Index 3 bahar hai: List ki length 3 hai, isliye index 0 se 2 tak (ya -1 se -3).
  --> main.aak:2:10
   |
 2 | dikhao(xs[3]);
   |          ^
  Aur jaankari: adarsh samjhao E011
```

The labels mean:
- `Samjho`: syntax.
- `Soch`: found by the checks before running.
- `Galti`: while running.
- `Pakda nahi gaya`: an uncaught `chhoddo`.

Warnings (`Dhyan do`) point out unused variables, code after `wapas` that never runs, and variables that hide a builtin.

## Types

Annotations are optional and checked while running:

```aak
badlo umar: number = 24;
kaam double(x: number) -> number { wapas x * 2; }
```

Type names:
- Built in: `number`, `string`, `boolean`, `list`, `dict`, `tuple`, `set`, `kaam`, `khali`, `koi` (anything), `dhacha` (any record), `vikalp` (any enum value).
- Your own `dhacha` and `vikalp` names also work as types.

## Modules

```aak
lao "utils";            # runs utils.aak once; its definitions become available
lao "maths" as m;       # namespaced: m.square(4)
```

Paths are relative to the importing file. `lao` works in the CLI, not the browser playground.

## Builtins

Every builtin also works with a dot (`xs.length()`) and named arguments (`sort(xs, by: f)`).

| Area | Functions |
| --- | --- |
| Collections | `length push pop insert rakho nikalo hatao keys values contains find count sort(by:, ulta:) reverse slice range join map filter reduce` |
| Lists | `sum min(by:) max(by:) any all zip unique flatten group_by list tuple set union intersection difference` |
| Numbers | `abs floor ceil round(x, ank) round_val min_val max_val sqrt sin cos tan asin acos atan log(x, base) exp gcd lcm is_prime`, constants `PI` and `E` |
| Random | `random_number random_int random_choice shuffle` |
| Strings | `upper lower title trim split replace starts_with ends_with pad_left pad_right repeat char_code from_code format` |
| Indian style | `lakh_format(1500000)` → `15,00,000`; `rupaye(99.5)` → `₹99.50`; `shabdon_mein(125)` → `ek sau pachchees` |
| Types | `prakar` (type name), `shabdme` (to string), `sankhya` (to number) |
| Input / output | `padho(sawaal)`, `padho_number(sawaal)` (asks again on bad input), `rang_mein(text, "hara")` (terminal colours) |
| Dates | `aaj()`, `abhi()`, `tareekh(2025, 8, 15)`, `din_ka_naam(t)`, `tareekh_jodo(t, din)`, `din_ka_fark(a, b)` |
| JSON | `json_banao(value, sundar: sahi_hai_be)`, `json_padho(text)` |
| Files (CLI) | `file_padho file_likho file_jodo file_hai` |
| Internet (CLI) | `internet_se(url)` (uses the system's `curl`) |
| System | `ruko(second)`, `band_karo(code)`, `current_time()`, `jaancho(sharat, sandesh)` |

## Turtle graphics (kachhua)

```aak
kachhua.rang("laal");
ginnati (badlo i = 0; i < 5; i += 1) {
    kachhua.aage(100);
    kachhua.daayen(144);
}
```

The commands are:
- Moving: `aage`, `peeche`, `daayen`, `baayen`, `jaao(x, y)` and `ghar()`.
- Pen and style: `kalam_upar`, `kalam_neeche`, `rang` and `mota`.

The CLI saves the drawing as `<file>.svg`; the playground draws it on a canvas.

## Tests

Write `test_*` functions in files ending `_test.aak` and run `adarsh test`:

```aak
kaam jodo(a, b) { wapas a + b; }
kaam test_jodo() { jaancho(jodo(2, 2) == 4, "2 + 2 = 4"); }
```

```text
math_test.aak
  ✓ test_jodo

1 pass, 0 fail
```

## Command line

```text
adarsh <file.aak>              run a program
adarsh check <file.aak>        check for errors without running
adarsh test [files/folders]    run *_test.aak tests
adarsh samjhao [E001]          explain an error code
adarsh python <file.aak>       show the same program in Python
adarsh build <file.aak> [-o x] make a standalone executable
adarsh                         REPL
adarsh --english ...           add English titles to errors
```
