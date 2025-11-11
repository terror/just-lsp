import * as path from 'path';
import * as vscode from 'vscode';
import {
  LanguageClient,
  LanguageClientOptions,
  ServerOptions,
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
  const serverExecutable = path.join(context.extensionPath, 'bin', 'just-lsp'); // compiled Rust binary

  const serverOptions: ServerOptions = {
    command: serverExecutable,
    args: [],
    options: {}
  };

  const clientOptions: LanguageClientOptions = {
    documentSelector: [
      { scheme: 'file', language: 'just' },
    ],
    synchronize: {
      fileEvents: vscode.workspace.createFileSystemWatcher('**/*.just')
    }
  };

  client = new LanguageClient('scryrLSP', 'Scryr Language Server', serverOptions, clientOptions);
  client.start();
}

export function deactivate(): Thenable<void> | undefined {
  return client ? client.stop() : undefined;
}