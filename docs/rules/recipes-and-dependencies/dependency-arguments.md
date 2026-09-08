---
title: Dependency arguments
severity: error
order: 270
---

Reports a dependency invocation with an argument count that does not match the target recipe’s parameters.

- Required parameters and `+` variadic parameters without defaults each need an argument. Defaulted parameters and `*` variadic parameters can be omitted.
- Without list mode, a variadic parameter permits additional arguments. With `set lists`, extra arguments beyond the number of parameters are rejected.
- The check uses the resolved recipe signature, including imported recipes. A missing target is handled by `missing-dependencies`.

## How to fix it

Supply the required arguments in a parenthesized dependency invocation, remove extra arguments, or adjust the target’s parameter list.

## Examples

```just reported
foo: bar
  echo foo

bar baz:
  echo {{baz}}
```

```just corrected
foo: (bar 'baz')
  echo foo

bar baz:
  echo {{baz}}
```
