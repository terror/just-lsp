---
title: Find References
order: 50
---

Use your editor's find-references command on a recipe, variable, parameter, or
user-defined function to find the identifiers that refer to it across the
project, including imported files and modules. Results include the declaration
when the editor requests it.

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

Recipes, variables, and functions are found in every file of the project that
can refer to them, such as a dependency on `foo::bar` in the root justfile that
refers to a recipe in module `foo`. Parameters are searched only within their
document. Builtins return every use of the builtin. Alias targets are recipe
references, but alias names are not resolved as separate symbols. Finding
references from a file imported by several justfiles searches every project
that imports it.

LSP request: `textDocument/references`.
