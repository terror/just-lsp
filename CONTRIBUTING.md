## Contributing

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you shall be licensed as in
LICENSE, without any additional terms or conditions.

## Development

First, clone the repository and build the project:

```
git clone https://github.com/terror/just-lsp
cd just-lsp
cargo build
```

### Neovim

To use the development build as Neovim's language server, add this to your
editor configuration:

```lua
local dev_cmd = '/path/to/just-lsp/target/debug/just-lsp'

local on_attach = function(client, bufnr)
  -- Add your implementation here
end

local capabilities = require('cmp_nvim_lsp').default_capabilities()

vim.lsp.config('just_dev', {
  cmd = { dev_cmd },
  filetypes = { 'just' },
  root_dir = function(fname)
    return vim.fs.root(fname, { '.git', 'justfile' })
  end,
  on_attach = on_attach,
  capabilities = capabilities,
})

vim.lsp.enable('just_dev')
```

This uses a separate config name (`just_dev`) so you can switch between the
local development build and the stock `just` config. Replace `dev_cmd` with the
absolute path to your freshly built binary.

`on_attach` is a function that gets called after an LSP client attaches to a
buffer,
[mine](https://github.com/terror/dotfiles/blob/0cc595de761d27d99367ad0ea98920b7718be4fb/etc/nvim/lua/config.lua#L207)
just sets up a few mappings:

```lua
local on_attach = function(client, bufnr)
  -- ...
  map('n', '<leader>ar', '<cmd>lua vim.lsp.buf.rename()<CR>')
  map('n', '<leader>s', '<cmd>lua vim.lsp.buf.format({ async = true })<CR>')
  -- ...
end
```

As in the basic example above, we use `cmp_nvim_lsp.default_capabilities()` so
that the dev build inherits completion-related capabilities from `nvim-cmp`.
Swap in your own table if you use a different completion plugin.

**n.b.** This setup requires the
[nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) plugin (and
optionally [cmp-nvim-lsp](https://github.com/hrsh7th/cmp-nvim-lsp) for the
capabilities helper).

### Zed

After installing the
[Zed Justfile extension](https://github.com/jackTabsCode/zed-just), point its
`just-lsp` adapter at the development binary in your Zed `settings.json`:

```json
{
  "lsp": {
    "just-lsp": {
      "binary": {
        "path": "/absolute/path/to/just-lsp/target/debug/just-lsp"
      }
    }
  }
}
```

The binary path must be absolute. After rebuilding the server, run
`editor: restart language server` from the command palette. You can optionally
add a shortcut for this action to `keymap.json`:

```json
[
  {
    "context": "Editor",
    "bindings": {
      "ctrl-r l": "editor::RestartLanguageServer"
    }
  }
]
```

### Extending the parser

`just-lsp` vendors the
[`tree-sitter-just`](https://github.com/terror/just-lsp/tree/master/vendor/tree-sitter-just)
parser in `vendor/tree-sitter-just`. After changing the grammar or query files,
rebuild and test the parser with the following commands:

```bash
just -f vendor/tree-sitter-just/justfile gen
cd vendor/tree-sitter-just && tree-sitter test
cargo test
```

**n.b.** `just update-parser` will run all of the above for you.

The generate step updates the parser artifacts under
`vendor/tree-sitter-just/src/`. Commit those files together with any updated
corpora in `vendor/tree-sitter-just/test/corpus` so downstream tooling sees your
changes.
