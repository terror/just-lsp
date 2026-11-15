---
title: Configuration
order: 30
---

`just-lsp` accepts configuration from `just-lsp.toml` (since 0.11.0) or through
the LSP `initializationOptions` object sent by your editor when the server
starts. Configuration is optional.

To disable unused recipe parameter diagnostics in any editor, create
`just-lsp.toml` beside your root justfile:

```toml
[rules]
unused-recipe-parameter = "off"
```

The server searches the root justfile's directory and then its parent
directories, using the first `just-lsp.toml` it finds. It does not merge
multiple configuration files. File settings override initialization options for
the same setting or rule; other options keep their editor-provided values or
defaults.

For configuration sent through `initializationOptions`, use JSON:

```json
{
  "formatting": {
    "indentation": "\t"
  },
  "rules": {
    "unused-variable": "off",
    "unused-recipe-parameter": "off"
  }
}
```

See the [Neovim example](#neovim) for passing `init_options` through
`vim.lsp.config`, or the [Visual Studio Code instructions](#visual-studio-code)
for configuring the server with `just-lsp.toml`. Restart the language server
after changing configuration.
