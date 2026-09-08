---
title: Recipe parameters
severity: error
order: 230
---

Reports duplicate recipe parameter names and invalid ordering of required, defaulted, or variadic parameters.

- Each parameter name must be unique within a recipe.
- A required nonvariadic parameter cannot follow a parameter with a default value.
- A variadic parameter, introduced by `*` or `+`, must be last. Multiple variadic parameters therefore also violate the ordering requirement. Malformed variadic lists can be reported as `syntax-errors` before this rule runs.

## How to fix it

Give parameters distinct names, put required parameters before defaulted parameters, and place a single variadic parameter at the end.

## Examples

Parameter names must be distinct.

```just reported
foo bar bar:
  echo {{bar}}
```

```just corrected
foo bar baz:
  echo {{bar}} {{baz}}
```

Required parameters come before parameters with defaults.

```just reported
foo bar='bar' baz:
  echo {{bar}} {{baz}}
```

```just corrected
foo baz bar='bar':
  echo {{bar}} {{baz}}
```
