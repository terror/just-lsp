---
title: Document Highlights
order: 90
---

When your editor requests document highlights for the identifier under the
cursor, `just-lsp` marks its declaration and matching uses in the current
justfile. This helps you see where a symbol is used while editing.

The highlights follow symbol scope, covering recipes, variables, recipe
parameters, user-defined functions, and function parameters. Recipe dependencies
and alias targets count as uses of a recipe.

## Example

```just
foo := 'bar'

baz foo='qux':
  echo {{foo}}

quux:
  echo {{foo}}
```

With the cursor on the parameter `foo` in `baz`, the parameter declaration and
its interpolation are highlighted. With the cursor on the global `foo`, the
assignment and the interpolation in `quux` are highlighted instead.

Highlights are limited to the requested document and do not include literal
strings or shell text. All returned ranges use the LSP `text` highlight kind;
the server does not distinguish reads from writes. Your editor controls when
highlights appear and how they look.

LSP request: `textDocument/documentHighlight`.
