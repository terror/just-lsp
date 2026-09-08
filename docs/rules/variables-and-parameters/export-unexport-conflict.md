---
title: Export and unexport conflict
severity: error
order: 490
---

Reports a name that is both assigned and unexported for overlapping platforms.

- The check applies to ordinary assignments as well as explicit `export` assignments.
- It does not depend on declaration order. The diagnostic is attached to the conflicting assignment’s name.
- Assignments and unexports on disjoint platforms do not conflict.

## How to fix it

Choose whether to assign the name or remove it from the environment. Remove the conflicting declaration, or use a different variable name for an unrelated value.

## Examples

```just reported
export FOO := 'bar'
unexport FOO
```

```just corrected
export FOO := 'bar'
```
