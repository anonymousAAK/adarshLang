// AdarshLang for VS Code: starts `adarsh lsp` (errors as you type, hover,
// completion, go to definition, formatting) and adds a "Run file" command.
const vscode = require('vscode');

let client = null;

function adarshPath() {
  return vscode.workspace.getConfiguration('adarshlang').get('path') || 'adarsh';
}

async function startServer(context) {
  if (!vscode.workspace.getConfiguration('adarshlang').get('languageServer')) return;
  let lc;
  try {
    lc = require('vscode-languageclient/node');
  } catch (e) {
    vscode.window.showWarningMessage('AdarshLang: language server ke liye "npm install" chalao (vscode-languageclient missing). Highlighting phir bhi chalegi.');
    return;
  }
  const server = { command: adarshPath(), args: ['lsp'] };
  client = new lc.LanguageClient(
    'adarshlang',
    'AdarshLang',
    { run: server, debug: server },
    { documentSelector: [{ scheme: 'file', language: 'adarshlang' }, { scheme: 'untitled', language: 'adarshlang' }] }
  );
  try {
    await client.start();
  } catch (e) {
    client = null;
    vscode.window.showWarningMessage(
      `AdarshLang: "${adarshPath()} lsp" shuru nahi hua. adarsh install karo ya Settings mein "adarshlang.path" set karo.`
    );
  }
}

function activate(context) {
  context.subscriptions.push(
    vscode.commands.registerCommand('adarshlang.run', async () => {
      const editor = vscode.window.activeTextEditor;
      if (!editor) return;
      await editor.document.save();
      const file = editor.document.fileName;
      let term = vscode.window.terminals.find((t) => t.name === 'AdarshLang');
      if (!term) term = vscode.window.createTerminal('AdarshLang');
      term.show(true);
      const quote = (s) => (process.platform === 'win32' ? `& "${s}"` : `"${s}"`);
      const exe = adarshPath() === 'adarsh' ? 'adarsh' : quote(adarshPath());
      term.sendText(`${exe} "${file}"`);
    }),
    vscode.commands.registerCommand('adarshlang.restartServer', async () => {
      if (client) await client.stop();
      client = null;
      await startServer(context);
    })
  );
  startServer(context);
}

function deactivate() {
  return client ? client.stop() : undefined;
}

module.exports = { activate, deactivate };
