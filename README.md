## just-lsp

**just-lsp** is a server implementation of the [language server protocol](https://microsoft.github.io/language-server-protocol/) for [just](https://github.com/casey/just), the command runner.

<img width="1667" alt="Screenshot 2025-03-18 at 2 16 05 PM" src="https://github.com/user-attachments/assets/ff50b110-1b92-4904-b34a-93d5e2abd194" />

## Features

The server implements a decent amount of the
language server protocol [specifiction](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
This section aims to document some of them.

### `textDocument/completion`

Completions are provided to you as you type. We currently show recipes, built-in
functions, and constants.

### `textDocument/definition`

You're able to go to a recipe definition from an identifier.

### `textDocument/hover`

You can request hover information for recipes, built-in functions, constants
and see information about them.

### `textDocument/publishDiagnostics`

We try to publish useful diagnostics. Some of them include checks for non-existent
aliases, dependencies, and syntax errors.

### `textDocument/references`

All references to an identifier can be shown. This includes aliases,
dependencies, and recipes.

### `textDocument/rename`

Workspace-wide symbol renaming is supported.

## Prior Art

Check out [just](https://github.com/casey/just), the command runner.
