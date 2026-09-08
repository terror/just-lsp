---
title: Duplicate function
severity: error
order: 340
---

Reports user-defined functions that share a name and have overlapping platform constraints.

- Changing the parameter list does not make the name unique; this rule does not permit overloads by argument count.
- Functions on disjoint platforms may share a name. Defining a function with a builtin’s name is not a duplicate declaration by itself.

## How to fix it

Keep a single definition, rename one function and its calls, or use disjoint platform attributes for platform-specific definitions.

## Examples

```just reported
set unstable

foo() := 'foo'
foo() := 'bar'

bar:
  echo {{foo()}}
```

```just corrected
set unstable

foo() := 'bar'

bar:
  echo {{foo()}}
```
