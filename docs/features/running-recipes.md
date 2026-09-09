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

The server runs `just` with the recipe name in the directory containing the
document. It does not pass `--justfile`, so `just` uses its normal justfile
discovery. Execution captures standard output and standard error and sends them
to a `just-recipe:` output document. Displaying this output requires the editor
to support opening that URI and applying edits to it. A non-zero exit status
also produces a warning.

## Parameters

Recipes can run when they have no parameters or every parameter has a default
value. If any parameter lacks a default, the server shows a warning and does not
start the recipe. Argument prompting and overriding parameter defaults are not
currently supported.

The available actions and lenses cover recipes declared in the current document.
Their visibility depends on your editor's code action and code lens support.

LSP requests: `textDocument/codeAction`, `textDocument/codeLens`, and
`workspace/executeCommand`.
