---
severity: warning
order: 10
---

Reports a user-defined function parameter that is never referenced in the function body.

- The check looks for identifier references in that function’s body. A similarly named variable used elsewhere does not count.
- Names beginning with `_` are exempt, which allows a parameter to remain part of a signature even when its value is intentionally ignored.

## How to fix it

Use the parameter in the body, remove it and update callers, or prefix its name with `_` to mark it intentionally unused.

## Examples

```just reported
set unstable

foo(bar) := 'foo'

baz:
  echo {{foo('bar')}}
```

```just corrected
set unstable

foo(_bar) := 'foo'

baz:
  echo {{foo('bar')}}
```
