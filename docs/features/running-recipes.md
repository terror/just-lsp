---
title: Running Recipes
order: 130
---

Run a recipe from your editor using the `Run` code lens above its declaration or
the source code action named after the recipe. Both invoke the
`just-lsp.run_recipe` command.

Install `just` and make sure it is on the language server's `PATH`. Save the
justfile before running a recipe: execution reads files from disk, while code
actions and lenses are collected from the open document.

## Example

```just
foo:
  echo bar

baz qux='quux':
  echo {{qux}}
```

The editor can offer `Run` above both recipes, or source actions named `foo` and
`baz`. Running `baz` uses the default value `'quux'` for its parameter.

The server runs `just` from the directory of the project's root justfile,
passing that justfile with `--justfile`. Recipes in imported files run by name,
and recipes in module files run by their module path, such as `foo::bar`, so
they run in the same context as when invoked from the root. Execution captures standard output and standard error and sends them
to a `just-recipe:` output document. Displaying this output requires the editor
to support opening that URI and applying edits to it. A non-zero exit status
also produces a warning.

## Parameters

Recipes can run when they have no parameters or every parameter has a default
value. `*` variadic parameters can be empty and do not need a default. If any
other parameter lacks a default, the server shows a warning and does not
start the recipe. Argument prompting and overriding parameter defaults are not
currently supported.

The available actions and lenses cover recipes declared in the current document.
Their visibility depends on your editor's code action and code lens support.
Source actions are only returned when the editor's code action request allows
the `source` kind.

LSP requests: `textDocument/codeAction`, `textDocument/codeLens`, and
`workspace/executeCommand`.
