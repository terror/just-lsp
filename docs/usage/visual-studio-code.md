---
title: Visual Studio Code
order: 20
---

Install the
[vscode-just extension](https://marketplace.visualstudio.com/items?itemName=nefrob.vscode-just-syntax)
and [install `just-lsp`](#installation) so it is on your `PATH`. Open a justfile
to start the language server.

If the binary is installed elsewhere, set `vscode-just.lspPath` in your
`settings.json`:

```json
{
  "vscode-just.lspPath": "/absolute/path/to/just-lsp"
}
```

Reload Visual Studio Code after changing the binary path.

The extension's
[language client](https://github.com/nefrob/vscode-just/blob/main/src/lsp.ts)
currently does not expose custom `initializationOptions`, so `just-lsp` uses its
default formatting and diagnostic rule configuration.
