# AdarshLang for VS Code

Syntax highlighting, bracket matching, and snippets for **AdarshLang** (`.aak` files), a Hinglish toy language.

## Features

- Highlighting for comments, strings (with escapes and `{ ... }` interpolation), numbers (`1_000`, `.5`, `2.5e-3`), keywords, operators, constants, builtins, function and record names.
- Language configuration: `#` line comments, bracket and quote auto-closing, surrounding pairs, and indentation after `{`.
- Snippets for `agar`, `agarwarna`, `jabtak`, `ginnati`, `keliye`, `chuno`, `kaam`, `dhacha`, and `pakdo`.

A sample program that uses every highlighted construct is in `examples/demo.aak`.

## Install locally

The extension has no code and no dependencies. It is only a grammar, language configuration, and snippets.

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

This creates `adarshlang-0.2.0.vsix`. Then in VS Code open the Extensions view, choose the `...` menu, select **Install from VSIX...**, and pick the file.

## Development

Edit `syntaxes/adarshlang.tmLanguage.json`, then run **Developer: Reload Window** to see the change. Use **Developer: Inspect Editor Tokens and Scopes** to check the scope of a token under the cursor.
