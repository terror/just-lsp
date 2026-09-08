---
title: Configuration
order: 30
---

`just-lsp` accepts configuration through the LSP `initializationOptions` object,
sent by your editor when the server starts. Configuration is optional; omitted
keys keep their default behavior. Restart the language server after changing
these options.

```json
{
  "formatting": {
    "indentation": "\t"
  },
  "rules": {
    "unused-variables": "off",
    "unused-parameters": { "level": "warning" }
  }
}
```
