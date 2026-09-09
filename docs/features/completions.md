---
title: Completions
order: 20
---

Use your editor's completion menu to insert recipe names, variables, and
user-defined functions from the current justfile, along with builtin attributes,
constants, functions, and settings.

Recipe and variable suggestions include their source definitions. User-defined
function suggestions also show their parameter names, but insert only the
function name. Builtin suggestions include reference documentation, and
deprecated functions and settings are marked as deprecated.

Builtin functions, including their alternate names, insert call snippets with
argument placeholders. Your editor or completion plugin must support LSP
snippets to expand these placeholders.

## Example

```just
foo := 'bar'

baz:
  echo {{foo}}
```

The completion list includes `foo` and `baz`, together with builtins such as
`env` and `os`. Selecting `foo` inserts its name; selecting a builtin function
inserts a call snippet.

Completions currently use declarations from the open document. Imported
declarations, alias names, and local parameters are not added to the list. The
server returns the same list regardless of cursor position, leaving filtering
and presentation to the editor.

LSP request: `textDocument/completion`.
