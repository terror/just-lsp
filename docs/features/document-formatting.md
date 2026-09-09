---
title: Document Formatting
order: 120
---

Invoke your editor's format-document command to format the current justfile.
Formatting uses the current editor contents, including unsaved changes, and
returns an edit for the editor to apply.

Install `just` and make sure it is on the language server's `PATH`. The server
runs `just --fmt --unstable --quiet` on a temporary copy of the document. For a
file on disk, this copy is created beside the original so relative paths can
resolve from the same directory.

## Indentation

By default, `just` chooses the indentation. To use a custom indentation string,
set `formatting.indentation` in the language server's initialization options:

```json
{
  "formatting": {
    "indentation": "  "
  }
}
```

See [formatting configuration](#formatting) for details. The server uses this
configuration rather than the formatting request's `tabSize` or `insertSpaces`
options. Format-on-save can be enabled through your editor if it supports
calling the document formatter on save.

If formatting fails, the server shows an error and returns no edits. If the
document is already formatted, it returns an empty edit list. Formatting covers
the whole document; range formatting and on-type formatting are not supported.

LSP request: `textDocument/formatting`.
