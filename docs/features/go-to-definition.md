---
title: Go to Definition
order: 40
---

Place the cursor on an identifier and invoke your editor's go-to-definition
command to jump to the declaration it refers to. Supported targets include
recipes, variables, recipe parameters, user-defined functions, and function
parameters.

Recipe dependencies and the recipe name on the right-hand side of an alias
resolve to the recipe definition. Variables and parameters follow their scope,
so a local parameter takes precedence over a global variable with the same name.

## Example

```just
foo := 'bar'

baz foo='qux':
  echo {{foo}}

quux: baz
  echo {{foo}}

alias quuz := baz
```

Go to definition on `foo` in `baz` jumps to its parameter declaration. On `foo`
in `quux`, it jumps to the global assignment. On the dependency `baz` or the
target of `quuz`, it jumps to the `baz` recipe.

Definitions in imported justfiles can be opened even when those files are not
already open in the editor. Resolution uses unsaved contents for open imported
files. Invoking go to definition on a resolved import path opens the imported
file at its start.

Builtin identifiers have no source declaration in the justfile, so their
definition location is the identifier itself. Unresolved names have no
definition result. Alias names are not independent definition targets.

LSP request: `textDocument/definition`.
