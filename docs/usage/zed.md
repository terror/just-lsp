---
title: Zed
order: 30
---

Install the [Justfile extension](https://github.com/jackTabsCode/zed-just) from
Zed's Extensions view, then open a justfile. The extension uses `just-lsp` from
your `PATH` when available and downloads the server automatically otherwise.

To customize [formatting and diagnostic rules](#configuration), set
[`initialization_options`](https://zed.dev/docs/configuring-languages#configuring-language-servers)
under `lsp.just-lsp` in your Zed `settings.json`:

```json
{
  "lsp": {
    "just-lsp": {
      "initialization_options": {
        "formatting": {
          "indentation": "\t"
        },
        "rules": {
          "unused-variables": "off",
          "unused-parameters": { "level": "warning" }
        }
      }
    }
  }
}
```

To use a specific binary, set its absolute path under `binary.path`:

```json
{
  "lsp": {
    "just-lsp": {
      "binary": {
        "path": "/absolute/path/to/just-lsp"
      }
    }
  }
}
```

Run `editor: restart language server` from the command palette after changing
these options.
