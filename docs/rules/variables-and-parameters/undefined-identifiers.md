---
title: Undefined identifiers
severity: error
order: 500
---

Reports a variable or parameter reference that cannot be resolved in its expression scope.

- Checks include expressions in assignments, recipe interpolations, parameter defaults, and function bodies.
- Variables from imports and parameters visible in the current scope can resolve a reference. A parameter from another recipe or function cannot.
- Shell text is not a just expression: `$FOO` and `{{FOO}}` have different meanings. Misspelled function calls are handled by `unknown-function`.

## How to fix it

Correct the identifier, define or import the variable, or add the parameter to the declaration that uses it. Quote text that should be a string literal.

**Editor quick fix:** Replaces the unresolved identifier with a suggested name from its scope when a close match is found.

## Examples

```just reported
foo:
  echo {{bar}}
```

```just corrected
bar := 'bar'

foo:
  echo {{bar}}
```
