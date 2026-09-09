---
title: Neovim
order: 10
---

[Install `just-lsp`](#installation) and make sure it is on your `PATH`. With
Neovim 0.11.3+ and
[nvim-lspconfig](https://github.com/neovim/nvim-lspconfig) installed, add this
to your `init.lua`:

```lua
vim.lsp.enable('just')
```

Open a justfile to attach the server. The
[`just` configuration](https://github.com/neovim/nvim-lspconfig/blob/master/lsp/just.lua)
provided by nvim-lspconfig supplies the command and filetype.

To customize [formatting and diagnostic rules](#configuration), pass
`init_options` through `vim.lsp.config` before enabling the server:

```lua
vim.lsp.config('just', {
  init_options = {
    formatting = {
      indentation = '\t',
    },
    rules = {
      ['unused-variables'] = 'off',
      ['unused-recipe-parameters'] = { level = 'warning' },
    },
  },
})

vim.lsp.enable('just')
```

If the binary is not on your `PATH`, add
`cmd = { '/absolute/path/to/just-lsp' }` to the same `vim.lsp.config` call.
Restart the language server after changing initialization options.
