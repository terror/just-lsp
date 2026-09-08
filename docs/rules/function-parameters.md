---
title: Function parameters
category: Functions
severity: error
order: 350
---

Reports repeated parameter names within a user-defined function.

- Each occurrence after the first is reported at the duplicate parameter.
- The check is local to a function’s parameter list. Different functions may use the same parameter names.

## How to fix it

Give every parameter a distinct name and update the function body to reference the intended parameters.

## Examples

```just reported
set unstable

foo(bar, bar) := bar

baz:
  echo {{foo('foo', 'bar')}}
```

```just corrected
set unstable

foo(bar, baz) := bar + baz

qux:
  echo {{foo('foo', 'bar')}}
```
