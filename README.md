## just-lsp

[![release](https://img.shields.io/github/release/terror/just-lsp.svg?label=release&style=flat&labelColor=282c34&logo=github)](https://github.com/terror/just-lsp/releases/latest)
[![crates.io](https://shields.io/crates/v/just-lsp.svg)](https://crates.io/crates/just-lsp)
[![CI](https://github.com/terror/just-lsp/actions/workflows/ci.yaml/badge.svg)](https://github.com/terror/just-lsp/actions/workflows/ci.yaml)
[![downloads](https://img.shields.io/github/downloads/terror/just-lsp/total.svg)](https://github.com/terror/just-lsp/releases)
[![dependency status](https://deps.rs/repo/github/terror/just-lsp/status.svg)](https://deps.rs/repo/github/terror/just-lsp)

**just-lsp** is a server implementation of the [language server protocol](https://microsoft.github.io/language-server-protocol/) for [just](https://github.com/casey/just), the command runner.

<img width="1667" alt="demo" src="https://github.com/user-attachments/assets/77ea8cff-3de4-46ee-b7fc-52b2b6805e5d" />

## Installation

`just-lsp` should run on any system, including Linux, MacOS, and the BSDs.

The easiest way to install it is by using [cargo](https://doc.rust-lang.org/cargo/index.html),
the Rust package manager:

```bash
cargo install just-lsp
```

Otherwise, see below for the complete package list:

#### Cross-platform

<table>
  <thead>
    <tr>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href=https://www.rust-lang.org>Cargo</a></td>
      <td><a href=https://crates.io/crates/just-lsp>just-lsp</a></td>
      <td><code>cargo install just-lsp</code></td>
    </tr>
  </tbody>
</table>

#### Linux

<table>
  <thead>
    <tr>
      <th>Operating System</th>
      <th>Package Manager</th>
      <th>Package</th>
      <th>Command</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td><a href=https://www.archlinux.org>Arch</a></td>
      <td><a href=https://wiki.archlinux.org/title/Pacman>pacman</a></td>
      <td><a href=https://archlinux.org/packages/extra/x86_64/just-lsp/>just-lsp</a></td>
      <td><code>pacman -S just-lsp</code></td>
    </tr>
  </tbody>
</table>

![just-lsp package version table](https://repology.org/badge/vertical-allrepos/just-lsp.svg)

### Mason

You can also install the server via [mason](https://github.com/williamboman/mason.nvim),
the Neovim plugin that allows you to easily manage external editor tooling such as LSP servers,
DAP servers, etc.

Simply invoke `:Mason` in your editor, and find `just-lsp` in the dropdown to
install it.

### Pre-built binaries

Pre-built binaries for Linux, MacOS, and Windows can be found on [the releases
page](https://github.com/terror/just-lsp/releases).

## Usage

`just-lsp` can be used with any LSP client, this section documents integration
with some of the more popular ones.

### Neovim

You can use the release build of `just-lsp` by setting up the `just` server on
[`lspconfig`](https://github.com/neovim/nvim-lspconfig), so somewhere in your config:

```lua
local lsp = require('lspconfig')

lsp.just.setup({
  -- ...
})
```

This assumes `just-lsp` is installed on your system and is in your `$PATH`.

### Zed

A third-party [**zed**](https://zed.dev/) extension is maintained over at https://github.com/sectore/zed-just-ls,
written by [@sectore](https://github.com/sectore). Follow the instructions in that
repository to get it setup on your system.

## Features

The server implements a decent amount of the
language server protocol [specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/).
This section aims to document some of them.

### `textDocument/codeAction`

We provide a code action for each recipe. These code actions run the selected
recipe using `just`, populating a buffer with its output (stderr + stdout).

### `textDocument/completion`

Completions are provided to you as you type. We currently show recipes, built-in
functions, and constants.

### `textDocument/definition`

You're able to go to a recipe, parameter or assignment definition from an identifier.

### `textDocument/documentHighlight`

Like references, but highlights them inside the document.

### `textDocument/foldingRange`

Code folding for recipes.

### `textDocument/formatting`

You're able to format your justfile. This calls `just --fmt --unstable` and
writes the result to your buffer.

### `textDocument/hover`

You can request hover information for syntactic elements like recipes, built-in
functions, constants, etc. and see information about them.

### `textDocument/publishDiagnostics`

We try to publish useful diagnostics. Some of them include checks for non-existent
aliases, dependencies, and syntax errors.

### `textDocument/references`

All references to an identifier can be shown. This includes aliases,
dependencies, recipes, and more.

### `textDocument/rename`

Workspace-wide symbol renaming is supported.

## Development

I use [Neovim](https://neovim.io/) to work on this project, and I load the
development build of this server to test out my changes instantly. This section
describes a development setup using Neovim as the LSP client, for other clients
you would need to look up their respective documentation.

First, clone the repository and build the project:

```
git clone https://github.com/terror/just-lsp
cd just-lsp
cargo build
```

Add this to your editor configuration:

```lua
local lsp = require('lspconfig')

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

local on_attach = function(client)
  -- Add your implementation here
end

local capabilities = {} -- Define what functionality the LSP client is able to handle

lsp.just_lsp.setup({
  on_attach = on_attach,
  capabilities = capabilities,
})
```

**n.b.** You'll need to replace `/path/to/just-lsp/target/debug/just-lsp` with
the actual absolute path to the binary.

`on_attach` is a function that gets called after an LSP client gets attached
to a buffer, [mine](https://github.com/terror/dotfiles/blob/0cc595de761d27d99367ad0ea98920b7718be4fb/etc/nvim/lua/config.lua#L207) just sets up a few mappings:

```lua
local on_attach = function(client)
  -- ...
  map('n', '<leader>ar', '<cmd>lua vim.lsp.buf.rename()<CR>')
  map('n', '<leader>s', '<cmd>lua vim.lsp.buf.format({ async = true })<CR>')
  -- ...
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

### Extending the parser

`just-lsp` vendors the [`tree-sitter-just`](https://github.com/justfile/tree-sitter-just) grammar in
`vendor/tree-sitter-just`. After changing the grammar or query files, rebuild and
test the parser with the following commands:

```bash
`cd vendor/tree-sitter-just && npx tree-sitter generate`
`cd vendor/tree-sitter-just && npx tree-sitter test`
`cargo test`
```

**n.b.** `just update-parser` will run all of the above for you.

The generate step updates the parser artifacts under `vendor/tree-sitter-just/src/`. Commit
those files together with any updated corpora in `vendor/tree-sitter-just/test/corpus` so
downstream tooling sees your changes.

## Prior Art

Check out [just](https://github.com/casey/just), the command runner.
