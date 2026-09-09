---
severity: error
order: 10
---

Reports a recipe body line whose indentation width differs from the first nonblank body line.

- The comparison applies to lines using the same kind of whitespace. Switching between tabs and spaces is handled by `mixed-recipe-indentation`.
- Blank lines are ignored. A line following a backslash continuation may use additional indentation.
- Script recipes are exempt, including shebang recipes, recipes with `[script]`, and recipes using `set default-script` unless `[shell]` overrides it.

## How to fix it

Use the same leading whitespace for ordinary lines in a recipe. Keep indentation that belongs to a script inside a recipe that runs in script mode.

## Examples

The second command has four leading spaces; the first has two.

```just reported
foo:
  echo foo
    echo bar
```

```just corrected
foo:
  echo foo
  echo bar
```
