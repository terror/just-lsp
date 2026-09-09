---
title: Document Symbols
order: 70
---

Use your editor's outline or go-to-symbol command to navigate declarations in
the current justfile.

The outline includes recipes, aliases, variables, user-defined functions, and
settings, ordered by their position in the file. Alias entries identify the
recipe they target, function entries include their parameter names, and setting
entries include the setting kind.

## Example

```just
set shell := ['sh', '-cu']

foo := 'bar'

baz:
  echo {{foo}}

alias qux := baz
```

The outline lists `shell`, `foo`, `baz`, and `qux` in that order. Selecting an
entry navigates to its declaration, and `qux` includes the detail
`alias for baz`.

Symbols are returned as a flat list for the requested document. Imported
declarations, modules, and local parameters are not included. This is a document
outline; the server does not provide a workspace-wide symbol search.

LSP request: `textDocument/documentSymbol`.
