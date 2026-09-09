---
title: Hover
order: 30
---

Hover over an identifier, or invoke your editor's hover command, to inspect its
definition without leaving the current location.

- Recipes show their header and body, including when referenced as a dependency
  or as the target of an alias.
- Variables show their assignment, and recipe parameters show their declaration
  and any default value.
- User-defined functions show their definition; their parameters show the
  parameter name.
- Builtin attributes, constants, functions, and settings show their reference
  documentation.

## Example

```just
foo := 'bar'

baz foo='qux':
  echo {{foo}}

quux:
  echo {{foo}}
```

Hovering over `foo` in `baz` shows `foo='qux'`. Hovering over `foo` in `quux`
shows the global assignment `foo := 'bar'`. The result follows the identifier's
scope, including references in parameter defaults and function bodies.

Hover also resolves recipes, variables, and user-defined functions from imports.
Open imported files use their unsaved editor contents; closed files use their
contents on disk. Unresolved names have no hover result.

LSP request: `textDocument/hover`.
