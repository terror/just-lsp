---
severity: error
order: 10
---

Reports a variable whose value depends on itself.

- A variable that refers to itself is reported, as are longer cycles such as `foo := bar` and `bar := foo`. The diagnostic lists the variables in the cycle.
- Calls to user-defined functions are followed, so a variable that calls a function whose body refers back to that variable is also a cycle.
- Variables from imports are included. Each variable in the cycle is reported in the document that declares it.

## How to fix it

Define one of the variables in the cycle without referring back to the others.

```just reported
foo := bar
bar := foo

baz:
  echo {{foo}}
```

```just corrected
foo := bar
bar := 'bar'

baz:
  echo {{foo}}
```
