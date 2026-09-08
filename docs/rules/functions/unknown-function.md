---
title: Unknown function
severity: error
order: 310
---

Reports a function call whose name is neither a known builtin nor a user-defined function.

- Builtin aliases and user-defined functions from the document and its imports are recognized.
- A user-defined function may be declared after its call. When a similar known function name exists, the diagnostic suggests it.

## How to fix it

Correct the function name, define the function, or import its definition. Calls to recipes belong in recipe dependencies, rather than expressions.

**Editor quick fix:** Replaces the unknown name with a suggested builtin or user-defined function name.

## Examples

```just reported
foo:
  echo {{uppercas('foo')}}
```

```just corrected
foo:
  echo {{uppercase('foo')}}
```
