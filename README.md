## just-lsp

[![CI](https://github.com/terror/just-lsp/actions/workflows/ci.yml/badge.svg)](https://github.com/terror/just-lsp/actions/workflows/ci.yml)
[![crates.io](https://shields.io/crates/v/just-lsp.svg)](https://crates.io/crates/just-lsp)
[![dependency status](https://deps.rs/repo/github/terror/just-lsp/status.svg)](https://deps.rs/repo/github/terror/just-lsp)

**just-lsp** is a server implementation of the [language server protocol](https://microsoft.github.io/language-server-protocol/) for [just](https://github.com/casey/just), the command runner.

<img width="1667" alt="Screenshot 2025-03-19 at 2 19 48 PM" src="https://github.com/user-attachments/assets/f10f3eb7-1a62-4449-aa09-2891b7f91187" />

## Installation

You can install the server using the [cargo](https://doc.rust-lang.org/cargo/index.html)
package manager:

```bash
cargo install just-lsp
```

## Features

The server implements a decent amount of the
language server protocol [specifiction](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
This section aims to document some of them.

### `textDocument/completion`

Completions are provided to you as you type. We currently show recipes, built-in
functions, and constants.

### `textDocument/definition`

You're able to go to a recipe definition from an identifier.

### `textDocument/documentHighlight`

Like references, but highlights them inside the document.

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

## Development

I use [Neovim](https://neovim.io/) to work on this project, and I load the
development build of this server to test out my changes instantly. This section
describes a development setup using Neovim as the LSP client, for other clients
you would need to look up their respective documentation.

First, clone the repository:

```
git clone https://github.com/terror/just-lsp
```

Build the project:

```
cd just-lsp
cargo build
```

Add this to your editor configuration:

```lua
local configs = require('lspconfig.configs')

if not configs.just_lsp then
  configs.just_lsp = {
    default_config = {
      cmd = { '/path/to/just-lsp/target/debug/just-lsp' },
      filetypes = { 'just' },
      root_dir = function(fname)
        return lsp.util.find_git_ancestor(fname)
      end,
      settings = {},
    },
  }
end

local lsp = require('lspconfig')

lsp.just_lsp.setup({
  on_attach = on_attach,
  capabilities = capabilities,
})
```

**n.b.** You'll need to replace `/path/to/just-lsp/target/debug/just-lsp` with
the actual absolute path to the binary.

`on_attach` is a function that gets called after an LSP client gets attached
to a buffer, mine just sets up a few mappings:

```lua
local on_attach = function(client)
  client.server_capabilities.semanticTokensProvider = nil

  map('n', '<leader>ar', '<cmd>lua vim.lsp.buf.rename()<CR>')
  map('n', '<leader>s', '<cmd>lua vim.lsp.buf.format({ async = true })<CR>')
  map('n', 'K', '<cmd>lua vim.lsp.buf.hover()<CR>')
  map('n', 'gd', '<cmd>lua vim.lsp.buf.definition()<CR>')
  map('n', 'gi', '<cmd>lua vim.lsp.buf.implementation()<CR>')
  map('n', 'gr', '<cmd>lua vim.lsp.buf.references()<CR>')
end
```

...and `capabilities` is a table defining what functionality the LSP client is able
to handle, I use
[default capabilities](https://github.com/hrsh7th/cmp-nvim-lsp/blob/99290b3ec1322070bcfb9e846450a46f6efa50f0/lua/cmp_nvim_lsp/init.lua#L37)
provided by [cmp-nvim-lsp](https://github.com/hrsh7th/cmp-nvim-lsp):

```lua
local capabilities = require('cmp_nvim_lsp').default_capabilities()
```

**n.b.** This will require you to have the [nvim-lspconfig](https://github.com/neovim/nvim-lspconfig)
(and optionally [cmp-nvim-lsp](https://github.com/hrsh7th/cmp-nvim-lsp)) plugin installed.

## Prior Art

Check out [just](https://github.com/casey/just), the command runner.
