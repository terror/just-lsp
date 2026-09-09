---
title: Rename
order: 60
---

Invoke your editor's rename command on a recipe, variable, recipe parameter,
user-defined function, or function parameter to update its declaration and all
matching identifier references in the current justfile.

Renaming follows symbol scope. A recipe parameter can be renamed without
changing a global variable with the same name, and a user-defined function can
be renamed independently of a same-named recipe.

## Example

```just
foo := 'bar'

baz foo='qux':
  echo {{foo}}

quux:
  echo {{foo}}
```

Rename the parameter `foo` in `baz` to `quuz`. The parameter declaration and its
interpolation change; the global assignment and the use in `quux` keep their
original name. Renaming the global variable instead updates that assignment and
the reference in `quux`.

Renaming a recipe also updates dependency names and alias targets that refer to
it. Alias names themselves cannot be renamed as independent symbols. Literal
strings and shell text are not rewritten.

Edits are limited to the current document; references in other files are not
updated. Builtins and unresolved identifiers cannot be renamed. The server
prepares a valid target before renaming, but does not validate the replacement
name for syntax or conflicts with other declarations.

LSP requests: `textDocument/prepareRename` and `textDocument/rename`.
