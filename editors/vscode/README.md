# AdarshLang for VS Code

Support for **AdarshLang** (`.aak` files), the Hinglish programming language.

## Features

- **Language server** (needs the `adarsh` command installed): errors and warnings as you type, hover docs for builtins, keywords and your own functions (`##` comments), completion, go to definition, an outline of functions and records, and **Format Document** (`adarsh fmt`).
- **Run file**: the ▶ button in the editor title (or `Ctrl+F5` / `Cmd+F5`) runs the file in a terminal.

- Highlighting for comments (`#` line, `#[ ... ]#` block, `##` doc), strings (escapes and `{ ... }` interpolation with `{x:.2}` format specs, raw `r"..."` and `r"""..."""` strings, multi-line `"""..."""` strings), numbers (`1_000`, `.5`, `2.5e-3`), keywords (including Devanagari spellings), operators (`..`, `...`, `?.`, `??`, `//`, `=>`), constants, builtins, function, record and enum names.
- Language 0.3: `pakka`, `karo ... jabtak`, `hai`, `vikalp`, `dete_raho`, `aakhir`, `lao ... as`, loop labels (`bahar: jabtak`, `bas bahar;`, `aage_badho bahar;`), type annotations (`badlo x: number`, `-> string`), inheritance (`dhacha A : B`), typed catch (`chhoddo (e: Galti)`), and `kachhua` turtle methods.
- Language configuration: `#` line comments and `#[ ]#` block comments, bracket and quote auto-closing, surrounding pairs, and indentation after `{`.
- Snippets for `agar`, `agarwarna`, `jabtak`, `ginnati`, `keliye`, `chuno`, `kaam`, `dhacha`, `pakdo`, `pakdoaakhir`, `karo`, `vikalp`, `lambda`, `test`, and `kachhua`.

A sample program that uses every highlighted construct is in `examples/demo.aak`. It runs with the interpreter (`examples/ganit.aak` is the module it loads with `lao ... as`).

## Install locally

1. Install `adarsh` (see the main README) so `adarsh lsp` works in a terminal. If it isn't on your PATH, set **Settings → AdarshLang: Path** to the executable.
2. In this folder run `npm install` once (it fetches `vscode-languageclient`). Without it, highlighting and snippets still work, but the language server features don't.
3. Install the folder with one of the options below.

### Option 1: Copy or symlink the folder

Linux and macOS:

```sh
ln -s /path/to/adarshLang/editors/vscode ~/.vscode/extensions/adarshlang
```

Windows (PowerShell, run as your user):

```powershell
New-Item -ItemType SymbolicLink -Path "$env:USERPROFILE\.vscode\extensions\adarshlang" -Target "C:\path\to\adarshLang\editors\vscode"
```

Or copy the folder instead of linking it:

```sh
cp -r /path/to/adarshLang/editors/vscode ~/.vscode/extensions/adarshlang
```

Restart VS Code (or run **Developer: Reload Window**). Open any `.aak` file and check that the language mode in the status bar shows **AdarshLang**.

### Option 2: Package a VSIX and install it

```sh
cd /path/to/adarshLang/editors/vscode
npx @vscode/vsce package
```

This creates `adarshlang-0.3.0.vsix`. Then in VS Code open the Extensions view, choose the `...` menu, select **Install from VSIX...**, and pick the file.

## Settings

- `adarshlang.path` (default `adarsh`): the executable used for the language server and **Run file**.
- `adarshlang.languageServer` (default on): turn the language server off to keep only highlighting.

## Development

Edit `syntaxes/adarshlang.tmLanguage.json`, then run **Developer: Reload Window** to see the change. Use **Developer: Inspect Editor Tokens and Scopes** to check the scope of a token under the cursor.
