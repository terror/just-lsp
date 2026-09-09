---
severity: warning
order: 10
---

Reports a non-exported global variable that is never referenced in the document or its imports.

- Variables whose names begin with `_` are exempt.
- Explicitly exported variables and variables covered by `set export` are exempt because shell commands can consume their environment values.
- References in just expressions count as usage. Shell text alone does not reference a non-exported just variable.

## How to fix it

Remove the unused assignment, use the variable in an expression, export it if it belongs in the environment, or prefix its name with `_` when it is intentionally unused.

## Examples

```just reported
foo := 'bar'
```

```just corrected
_foo := 'bar'
```
