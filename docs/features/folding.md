---
title: Folding
order: 110
---

Use your editor's fold controls to collapse recipes while navigating a long
justfile. `just-lsp` supplies a folding range for each recipe in the current
document.

## Example

```just
foo:
  echo bar
  echo baz

qux:
  echo quux
```

The recipes `foo` and `qux` have separate folding ranges, so the editor can
collapse one recipe while leaving the other expanded. Ranges cover the recipe
declaration and body, including any attributes attached to the recipe.

Folding currently covers recipes only. The server does not supply separate folds
for comment blocks, assignments, functions, or modules. The editor controls the
folding interface and whether a range is collapsed.

LSP request: `textDocument/foldingRange`.
