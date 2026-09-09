---
title: Find References
order: 50
---

Use your editor's find-references command on a recipe, variable, parameter, or
user-defined function to find the identifiers that refer to it in the current
document. Results include the declaration as well as its uses.

References follow symbol scope. A parameter and a global variable with the same
name have separate reference lists. Recipe names and function names also remain
distinct when they share a spelling.

## Example

```just
foo:
  echo foo

bar: foo

alias baz := foo
```

Finding references to the `foo` recipe returns its declaration, the dependency
in `bar`, and the alias target. The shell text `echo foo` is not an identifier
reference.

Variable references include just expressions such as `{{foo}}`, parameter
defaults, and function bodies when they resolve to that variable. Literal text
in strings and shell commands is not included.

Reference searches currently resolve and search within the requested document;
they do not search imported files or the rest of the workspace. Alias targets
are recipe references, but alias names are not resolved as separate symbols.

LSP request: `textDocument/references`.
